use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn web_dev_prints_start_message() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["web-dev", "--port", "9190"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Starting context web-dev"));
    assert!(stdout.contains("9190"));

    Ok(())
}

#[test]
fn web_dev_outputs_json_when_requested() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["--json", "web-dev", "--port", "9191"])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&stdout)?;

    assert_eq!(value["status"], "starting");
    assert_eq!(value["port"], 9191);
    assert_eq!(value["host"], "127.0.0.1");
    assert_eq!(value["mode"], "dev");

    Ok(())
}
