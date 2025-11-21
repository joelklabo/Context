use anyhow::Result;
use assert_cmd::Command;
use context_core::Document;
use tempfile::tempdir;

#[test]
fn ls_outputs_json_list_for_project() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["--project", "demo-project", "--json", "ls"])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let documents: Vec<Document> = serde_json::from_slice(&stdout)?;

    assert_eq!(documents.len(), 3);
    assert!(documents.iter().all(|d| d.project == "demo-project"));
    assert!(documents
        .iter()
        .any(|d| d.body_markdown.contains("listed document")));

    Ok(())
}

#[test]
fn ls_prints_human_readable_output() -> Result<()> {
    let temp = tempdir()?;
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .args(["ls"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Documents in project default"));
    assert!(stdout.contains("doc-1"));
    assert!(stdout.contains("Key: doc-1"));

    Ok(())
}
