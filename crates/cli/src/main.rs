use clap::{Parser, Subcommand};
use conservationci_core::*;
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    name = "conservationci",
    version,
    about = "Catch broken physics before it ships."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Check {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        report_json: Option<PathBuf>,
        #[arg(long)]
        report_md: Option<PathBuf>,
        #[arg(long)]
        samples_csv: Option<PathBuf>,
        #[arg(long)]
        github_annotations: bool,
    },
    Validate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        config: PathBuf,
    },
    Fixtures {
        #[command(subcommand)]
        command: Fixture,
    },
    Serve {
        #[arg(long, default_value_t = 8787)]
        port: u16,
        #[arg(long, default_value = "web/dist")]
        web: PathBuf,
    },
}
#[derive(Subcommand)]
enum Fixture {
    Generate {
        #[arg(long, default_value = "examples/fixtures")]
        output: PathBuf,
    },
}
fn load(input: &std::path::Path, config: &std::path::Path) -> Result<Analysis> {
    let cfg = Config::parse(&std::fs::read_to_string(config)?)?;
    let f = std::fs::File::open(input)?;
    use std::io::Read;
    let mut bytes = Vec::new();
    f.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    analyze(&bytes, &cfg)
}
fn escape(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(',', "%2C")
        .replace(':', "%3A")
}
async fn run(cli: Cli) -> Result<i32> {
    match cli.command {
        Command::Check {
            input,
            config,
            report_json,
            report_md,
            samples_csv: csv,
            github_annotations,
        } => {
            let a = load(&input, &config)?;
            if let Some(p) = report_json {
                std::fs::write(p, serde_json::to_vec_pretty(&a.report)?)?;
            }
            if let Some(p) = report_md {
                std::fs::write(p, a.report.markdown())?;
            }
            if let Some(p) = csv {
                std::fs::write(p, samples_csv(&a.samples)?)?;
            }
            print!("{}", a.report.markdown());
            if github_annotations {
                for (name, check) in &a.report.checks {
                    if let Some(i) = check.first_violation_index {
                        println!(
                            "::error file={},line={},title={} tolerance exceeded::First violation at t={} s; maximum deviation {:.9e} {}",
                            escape(&input.display().to_string()),
                            a.samples[i].state.row,
                            escape(name),
                            a.samples[i].state.t,
                            check.max_absolute_error,
                            check.unit
                        );
                    }
                }
            }
            Ok(if a.report.status == "FAIL" { 1 } else { 0 })
        }
        Command::Validate { input, config } => {
            let a = load(&input, &config)?;
            println!(
                "Valid: {} samples; ignored columns: {:?}",
                a.samples.len(),
                a.report.ignored_columns
            );
            Ok(0)
        }
        Command::Fixtures {
            command: Fixture::Generate { output },
        } => {
            fixtures::write_all(&output)?;
            println!(
                "Generated seven fixtures and configurations in {}",
                output.display()
            );
            Ok(0)
        }
        Command::Serve { port, web } => {
            conservationci_server::serve(port, web).await?;
            Ok(0)
        }
    }
}
#[tokio::main]
async fn main() {
    let result = run(Cli::parse()).await;
    let code = match result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", serde_json::json!({"error":e.to_string()}));
            match e {
                Error::Invalid(_) | Error::Csv(_) | Error::TooLarge(_) => 2,
                Error::Io(ref e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidData
                    ) =>
                {
                    2
                }
                _ => 3,
            }
        }
    };
    std::process::exit(code)
}
