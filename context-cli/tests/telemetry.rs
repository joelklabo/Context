use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;
use zip::read::ZipArchive;

#[test]
fn logs_include_scenario_project_and_command() {
    let temp = tempdir().unwrap();

    let mut cmd = cargo_bin_cmd!("context-cli");
    cmd.env("CONTEXT_HOME", temp.path())
        .env("CONTEXT_LOG_DIR", temp.path())
        .env("CONTEXT_SCENARIO", "scn-cli")
        .args(["--project", "proj-cli"])
        .arg("ls");

    cmd.assert().success();

    let log_path = temp.path().join("context-cli.jsonl");
    let contents = fs::read_to_string(log_path).unwrap();
    let first = contents.lines().next().unwrap();
    let json: Value = serde_json::from_str(first).unwrap();

    let fields = &json["fields"];
    assert_eq!(fields["scenario_id"], "scn-cli");
    assert_eq!(fields["project"], "proj-cli");
    assert_eq!(fields["command"], "ls");

    let spans = json["spans"].as_array().cloned().unwrap_or_default();
    assert!(
        spans.iter().any(|span| span["name"] == "cli.ls"),
        "expected cli.ls span in log entry"
    );
}

#[test]
fn debug_bundle_collects_logs() {
    let temp = tempdir().unwrap();
    let log_dir = temp.path().join("logs");

    // Produce some logs
    {
        let mut cmd = cargo_bin_cmd!("context-cli");
        cmd.env("CONTEXT_HOME", temp.path())
            .env("CONTEXT_LOG_DIR", &log_dir)
            .env("CONTEXT_SCENARIO", "bundle-scn")
            .arg("ls");
        cmd.output().unwrap();
    }

    let bundle_path = temp.path().join("bundle.zip");
    {
        let mut cmd = cargo_bin_cmd!("context-cli");
        cmd.env("CONTEXT_HOME", temp.path())
            .env("CONTEXT_LOG_DIR", &log_dir)
            .arg("debug-bundle")
            .arg("--scenario")
            .arg("bundle-scn")
            .arg("--out")
            .arg(&bundle_path);
        cmd.assert().success();
    }

    let file = fs::File::open(&bundle_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    // meta.json included
    let mut meta_contents = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("meta.json").unwrap(),
        &mut meta_contents,
    )
    .unwrap();
    let meta_json: Value = serde_json::from_str(&meta_contents).unwrap();
    assert_eq!(meta_json["scenario_id"], "bundle-scn");

    // logs copied
    let mut log_contents = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("logs/context-cli.jsonl").unwrap(),
        &mut log_contents,
    )
    .unwrap();
    assert!(
        log_contents.contains("bundle-scn"),
        "expected scenario id in log contents"
    );
}
