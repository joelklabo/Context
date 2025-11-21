use anyhow::Result;
use assert_cmd::Command;
use context_core::Document;

#[test]
fn cat_outputs_body_only_by_id() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.args(["cat", "--id", "doc-42"]).assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("doc-42"));
    assert!(!stdout.contains("Project:"));

    Ok(())
}

#[test]
fn cat_can_output_json_with_key() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args([
            "--project",
            "demo-project",
            "--json",
            "cat",
            "--key",
            "note-9",
        ])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let document: Document = serde_json::from_slice(&stdout)?;

    assert_eq!(document.project, "demo-project");
    assert_eq!(document.key.as_deref(), Some("note-9"));
    assert!(document.body_markdown.contains("note-9"));
    assert!(!document.id.0.is_empty());

    Ok(())
}

#[test]
fn cat_requires_key_or_id() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.arg("cat").assert().failure();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Provide --key or --id"));

    Ok(())
}
