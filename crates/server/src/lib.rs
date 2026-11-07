use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use conservationci_core::*;
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    csv: String,
    config: String,
}
#[derive(Serialize)]
struct ApiError {
    error: String,
}
struct Failure(Error);
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self(e)
    }
}
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (
            if matches!(self.0, Error::TooLarge(_)) {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            Json(ApiError {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}
fn decode(
    payload: std::result::Result<Json<Request>, JsonRejection>,
) -> std::result::Result<Request, Failure> {
    payload.map(|Json(r)| r).map_err(|e| {
        Failure(if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Error::TooLarge(e.body_text())
        } else {
            Error::Invalid(e.body_text())
        })
    })
}
async fn compute(req: Request) -> std::result::Result<Analysis, Failure> {
    tokio::task::spawn_blocking(move || {
        let c = Config::parse(&req.config)?;
        analyze(req.csv.as_bytes(), &c)
    })
    .await
    .map_err(|e| Failure(Error::Invalid(e.to_string())))?
    .map_err(Failure)
}
async fn sample(
    Path(index): Path<usize>,
    payload: std::result::Result<Json<Request>, JsonRejection>,
) -> std::result::Result<Json<Sample>, Failure> {
    let a = compute(decode(payload)?).await?;
    a.samples
        .get(index)
        .cloned()
        .map(Json)
        .ok_or_else(|| Failure(Error::Invalid("Sample index out of range".into())))
}
#[derive(Serialize)]
struct View {
    report: Report,
    samples: Vec<Sample>,
    indices: Vec<usize>,
    full_sample_count: usize,
    markdown: String,
}
async fn analyze_request(
    payload: std::result::Result<Json<Request>, JsonRejection>,
) -> std::result::Result<Json<View>, Failure> {
    let a = compute(decode(payload)?).await?;
    let indices = plot_indices(&a.samples);
    let samples = indices.iter().map(|&i| a.samples[i].clone()).collect();
    let markdown = a.report.markdown();
    Ok(Json(View {
        full_sample_count: a.samples.len(),
        report: a.report,
        samples,
        indices,
        markdown,
    }))
}
async fn validate(
    payload: std::result::Result<Json<Request>, JsonRejection>,
) -> std::result::Result<Json<serde_json::Value>, Failure> {
    let a = compute(decode(payload)?).await?;
    Ok(Json(
        serde_json::json!({"valid":true,"sample_count":a.samples.len(),"ignored_columns":a.report.ignored_columns}),
    ))
}
async fn example(Path(id): Path<String>) -> std::result::Result<Json<RequestOut>, Failure> {
    let (csv, config) = match id.as_str() {
        "analytic" => (
            include_str!("../../../examples/fixtures/analytic.csv"),
            include_str!("../../../examples/fixtures/analytic.toml"),
        ),
        "euler" => (
            include_str!("../../../examples/fixtures/euler.csv"),
            include_str!("../../../examples/fixtures/euler.toml"),
        ),
        "verlet" => (
            include_str!("../../../examples/fixtures/verlet.csv"),
            include_str!("../../../examples/fixtures/verlet.toml"),
        ),
        "energy-corruption" => (
            include_str!("../../../examples/fixtures/energy-corruption.csv"),
            include_str!("../../../examples/fixtures/energy-corruption.toml"),
        ),
        "momentum-corruption" => (
            include_str!("../../../examples/fixtures/momentum-corruption.csv"),
            include_str!("../../../examples/fixtures/momentum-corruption.toml"),
        ),
        "damped" => (
            include_str!("../../../examples/fixtures/damped.csv"),
            include_str!("../../../examples/fixtures/damped.toml"),
        ),
        "driven" => (
            include_str!("../../../examples/fixtures/driven.csv"),
            include_str!("../../../examples/fixtures/driven.toml"),
        ),
        _ => return Err(Failure(Error::Invalid("Unknown example identifier".into()))),
    };
    Ok(Json(RequestOut {
        csv: csv.into(),
        config: config.into(),
    }))
}
#[derive(Serialize)]
struct RequestOut {
    csv: String,
    config: String,
}
async fn export(
    Path(format): Path<String>,
    payload: std::result::Result<Json<Request>, JsonRejection>,
) -> std::result::Result<Response, Failure> {
    let a = compute(decode(payload)?).await?;
    let (mime, bytes) = match format.as_str() {
        "json" => (
            "application/json",
            serde_json::to_vec_pretty(&a.report).map_err(Error::from)?,
        ),
        "markdown" => ("text/markdown", a.report.markdown().into_bytes()),
        "csv" => ("text/csv", samples_csv(&a.samples)?),
        _ => return Err(Failure(Error::Invalid("Unknown report format".into()))),
    };
    Ok(([("content-type", mime)], bytes).into_response())
}
pub fn app(web: &std::path::Path) -> Router {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(2));
    Router::new()
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/examples", get(|| async { Json(fixtures::IDS) }))
        .route("/api/examples/{id}", get(example))
        .route("/api/analyze", post(analyze_request))
        .route("/api/samples/{index}", post(sample))
        .route("/api/validate", post(validate))
        .route("/api/reports/{format}", post(export))
        .layer(DefaultBodyLimit::max(MAX_BYTES + 1024 * 1024))
        .layer(tower_http::limit::RequestBodyLimitLayer::new(
            MAX_BYTES + 1024 * 1024,
        ))
        .fallback_service(tower_http::services::ServeDir::new(web))
        .layer(axum::middleware::from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let semaphore = semaphore.clone();
                async move {
                    let Ok(_permit) = semaphore.try_acquire_owned() else {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(ApiError {
                                error: "Two requests are already running; retry shortly".into(),
                            }),
                        )
                            .into_response();
                    };
                    next.run(req).await
                }
            },
        ))
}
pub async fn serve(port: u16, web: std::path::PathBuf) -> std::io::Result<()> {
    let _ = tracing_subscriber::fmt().try_init();
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    tracing::info!("ConservationCI: http://127.0.0.1:{port}");
    axum::serve(listener, app(&web))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}
