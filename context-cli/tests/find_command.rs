use anyhow::Result;
use assert_cmd::Command;
use context_core::Document;
use tempfile::tempdir;

#[test]
fn find_returns_json_hits() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args([
            "--project",
            "demo-project",
            "--json",
            "find",
            "rust search",
            "--limit",
            "2",
        ])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let documents: Vec<Document> = serde_json::from_slice(&stdout)?;

    assert_eq!(documents.len(), 2);
    assert!(documents.iter().all(|d| d.project == "demo-project"));
    assert!(documents
        .iter()
        .all(|d| d.body_markdown.contains("rust search")));

    Ok(())
}

#[test]
fn find_prints_human_readable_when_not_json() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["find", "hello world"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Found"));
    assert!(stdout.contains("hello world"));
    assert!(stdout.contains("project default"));

    Ok(())
}

#[test]
fn find_rejects_zero_limit() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["find", "hello", "--limit", "0"])
        .assert()
        .failure();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Limit must be greater than 0"));

    Ok(())
}
