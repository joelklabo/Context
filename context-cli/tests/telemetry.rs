use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn logs_include_scenario_project_and_command() {
    let temp = tempdir().unwrap();

    let mut cmd = cargo_bin_cmd!("context-cli");
    cmd.env("CONTEXT_LOG_DIR", temp.path())
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
}
