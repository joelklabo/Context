use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;

#[test]
fn rm_requires_key_or_id() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.arg("rm").assert().failure();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Provide --key or --id"));

    Ok(())
}

#[test]
fn rm_accepts_key_and_outputs_json() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args([
            "--project",
            "demo-project",
            "--json",
            "rm",
            "--key",
            "rm-key",
        ])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&stdout)?;

    assert_eq!(value["status"], "deleted");
    assert_eq!(value["project"], "demo-project");
    assert_eq!(value["key"], "rm-key");
    assert!(value["id"].as_str().is_some());

    Ok(())
}

#[test]
fn gc_respects_dry_run_and_outputs_json() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.args(["--json", "gc", "--dry-run"]).assert().success();

    let stdout = assert.get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&stdout)?;

    assert_eq!(value["status"], "ok");
    assert_eq!(value["dry_run"], true);
    assert!(value["deleted"].as_u64().is_some());
    assert_eq!(value["vacuumed"], false);

    Ok(())
}

#[test]
fn gc_human_output_when_not_json() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.arg("gc").assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Garbage collection complete"));
    assert!(stdout.contains("vacuumed"));

    Ok(())
}
