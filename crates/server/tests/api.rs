use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use conservationci_core::*;
use http_body_util::BodyExt;
use tower::ServiceExt;
async fn call(path: &str, body: Option<String>) -> (StatusCode, Vec<u8>) {
    let mut req = Request::builder().uri(path);
    if body.is_some() {
        req = req
            .method("POST")
            .header("content-type", "application/json");
    }
    let r = conservationci_server::app(std::path::Path::new("../../web/dist"))
        .oneshot(req.body(Body::from(body.unwrap_or_default())).unwrap())
        .await
        .unwrap();
    (
        r.status(),
        r.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
}
fn payload(id: &str) -> String {
    serde_json::json!({"csv":String::from_utf8(fixtures::generate(id).unwrap()).unwrap(),"config":fixtures::config_text(id)}).to_string()
}
#[tokio::test]
async fn api_and_core_identical() {
    for id in fixtures::IDS {
        let (status, body) = call("/api/analyze", Some(payload(id))).await;
        assert_eq!(status, StatusCode::OK);
        let view: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let core = analyze(
            &fixtures::generate(id).unwrap(),
            &Config::parse(&fixtures::config_text(id)).unwrap(),
        )
        .unwrap();
        assert_eq!(view["report"], serde_json::to_value(core.report).unwrap());
    }
}
#[tokio::test]
async fn bundled_examples_and_health() {
    assert_eq!(call("/api/health", None).await.0, StatusCode::OK);
    for id in fixtures::IDS {
        let (status, body) = call(&format!("/api/examples/{id}"), None).await;
        assert_eq!(status, StatusCode::OK);
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let c = Config::parse(data["config"].as_str().unwrap()).unwrap();
        assert!(analyze(data["csv"].as_str().unwrap().as_bytes(), &c).is_ok());
    }
    assert_eq!(
        call("/api/examples/no-such-example", None).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
#[tokio::test]
async fn structured_errors_and_validate() {
    let (status, body) = call("/api/analyze", Some("{".into())).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"].is_string());
    assert_eq!(
        call("/api/validate", Some(payload("euler"))).await.0,
        StatusCode::OK
    );
}
#[tokio::test]
async fn all_exports_and_exact_sample() {
    for format in ["json", "markdown", "csv"] {
        let (status, body) = call(
            &format!("/api/reports/{format}"),
            Some(payload("energy-corruption")),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(!body.is_empty());
    }
    let (_, body) = call("/api/samples/250", Some(payload("energy-corruption"))).await;
    let s: Sample = serde_json::from_slice(&body).unwrap();
    assert_eq!(s.state.row, 252);
    assert!(s.energy_violation);
    assert_eq!(
        call("/api/samples/999999", Some(payload("analytic")))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
#[tokio::test]
async fn request_size_limit() {
    let (status, _) = call(
        "/api/analyze",
        Some(" ".repeat(MAX_BYTES + 1024 * 1024 + 1)),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}
