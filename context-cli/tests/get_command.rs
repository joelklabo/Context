use anyhow::Result;
use assert_cmd::Command;
use context_core::Document;

#[test]
fn get_returns_json_by_key() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args([
            "--project",
            "demo-project",
            "--json",
            "get",
            "--key",
            "test-key",
        ])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let document: Document = serde_json::from_slice(&stdout)?;

    assert_eq!(document.project, "demo-project");
    assert_eq!(document.key.as_deref(), Some("test-key"));
    assert!(document.body_markdown.contains("test-key"));
    assert!(!document.id.0.is_empty());

    Ok(())
}

#[test]
fn get_prints_markdown_when_not_json() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.args(["get", "--id", "doc-123"]).assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Project: default"));
    assert!(stdout.contains("Document ID: doc-123"));
    assert!(stdout.contains("doc-123"));

    Ok(())
}

#[test]
fn get_requires_key_or_id() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.arg("get").assert().failure();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Provide --key or --id"));

    Ok(())
}
