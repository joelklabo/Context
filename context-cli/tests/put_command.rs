use std::io::Write;

use anyhow::Result;
use assert_cmd::Command;
use context_core::{Document, SourceType};
use tempfile::NamedTempFile;

#[test]
fn put_accepts_stdin_and_outputs_json() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args([
            "--project",
            "demo-project",
            "--json",
            "put",
            "--key",
            "note-1",
            "--tag",
            "alpha",
            "--tag",
            "beta",
        ])
        .write_stdin("hello from stdin")
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let document: Document = serde_json::from_slice(&stdout)?;

    assert_eq!(document.project, "demo-project");
    assert_eq!(document.key.as_deref(), Some("note-1"));
    assert_eq!(document.tags, vec!["alpha".to_string(), "beta".to_string()]);
    assert_eq!(document.body_markdown, "hello from stdin");
    assert_eq!(document.version, 1);
    assert!(document.deleted_at.is_none());
    assert!(document.ttl_seconds.is_none());
    assert!(matches!(document.source, SourceType::User));
    assert_eq!(document.created_at, document.updated_at);
    assert!(!document.id.0.is_empty());

    Ok(())
}

#[test]
fn put_supports_file_input_without_json() -> Result<()> {
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "file body")?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args([
            "put",
            "--file",
            temp_file.path().to_str().expect("temp file path"),
            "--key",
            "file-key",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("Stored document"));
    assert!(stdout.contains("project default"));
    assert!(stdout.contains("Key: file-key"));

    Ok(())
}

#[test]
fn put_fails_without_input() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd.arg("put").write_stdin("").assert().failure();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No input provided"));

    Ok(())
}
