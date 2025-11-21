use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;

#[test]
fn web_prints_start_message() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.args(["web", "--port", "9090"]).assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Starting context web"));
    assert!(stdout.contains("9090"));

    Ok(())
}

#[test]
fn web_outputs_json_when_requested() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args(["--json", "web", "--port", "9091"])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&stdout)?;

    assert_eq!(value["status"], "starting");
    assert_eq!(value["port"], 9091);
    assert_eq!(value["host"], "127.0.0.1");

    Ok(())
}
