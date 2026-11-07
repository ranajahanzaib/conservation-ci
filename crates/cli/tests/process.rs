use std::path::PathBuf;
use std::process::Command;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn check(id: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_conservationci"))
        .current_dir(root())
        .args([
            "check",
            "--input",
            &format!("examples/fixtures/{id}.csv"),
            "--config",
            &format!("examples/fixtures/{id}.toml"),
        ])
        .output()
        .unwrap()
}
#[test]
fn real_exit_contract() {
    for (id, code) in [
        ("analytic", 0),
        ("euler", 1),
        ("driven", 0),
        ("energy-corruption", 1),
        ("momentum-corruption", 1),
        ("damped", 0),
    ] {
        let result = check(id);
        assert_eq!(
            result.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let bad = Command::new(env!("CARGO_BIN_EXE_conservationci"))
        .args([
            "check",
            "--input",
            "/does-not-exist.csv",
            "--config",
            "/does-not-exist.toml",
        ])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
}
#[test]
fn validation_ignores_physical_failures() {
    let r = Command::new(env!("CARGO_BIN_EXE_conservationci"))
        .current_dir(root())
        .args([
            "validate",
            "--input",
            "examples/fixtures/euler.csv",
            "--config",
            "examples/models/conservative.toml",
        ])
        .output()
        .unwrap();
    assert!(r.status.success());
}
#[test]
fn reports_written_on_failure_and_annotations() {
    let dir = std::env::temp_dir().join(format!("conservationci-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let json = dir.join("report.json");
    let md = dir.join("report.md");
    let csv = dir.join("samples.csv");
    let out = Command::new(env!("CARGO_BIN_EXE_conservationci"))
        .current_dir(root())
        .args([
            "check",
            "--input",
            "examples/fixtures/energy-corruption.csv",
            "--config",
            "examples/models/conservative.toml",
            "--github-annotations",
            "--report-json",
        ])
        .arg(&json)
        .arg("--report-md")
        .arg(&md)
        .arg("--samples-csv")
        .arg(&csv)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("line=252"));
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(json).unwrap()).unwrap();
    assert_eq!(report["status"], "FAIL");
    assert!(md.exists() && csv.exists());
    std::fs::remove_dir_all(dir).unwrap();
}
