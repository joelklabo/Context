use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;
use zip::read::ZipArchive;

#[test]
fn debug_bundle_outputs_json_when_requested() -> Result<()> {
    let temp = tempdir()?;
    let log_dir = temp.path().join("logs");
    fs::create_dir_all(&log_dir)?;

    // Create a small log file to include
    let log_file = log_dir.join("context-cli.jsonl");
    fs::write(&log_file, r#"{"message":"hello"}"#)?;

    let out_path = temp.path().join("bundle.zip");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", temp.path())
        .env("CONTEXT_LOG_DIR", &log_dir)
        .args([
            "--json",
            "debug-bundle",
            "--scenario",
            "cli-018",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&stdout)?;

    assert_eq!(value["status"], "ok");
    assert_eq!(value["scenario"], "cli-018");
    assert_eq!(
        value["path"].as_str(),
        Some(out_path.to_string_lossy().as_ref())
    );
    assert!(out_path.exists());

    // Ensure bundle contains meta and logs
    let file = fs::File::open(&out_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut meta_contents = String::new();
    std::io::Read::read_to_string(&mut archive.by_name("meta.json")?, &mut meta_contents)?;
    let meta_json: Value = serde_json::from_str(&meta_contents)?;
    assert_eq!(meta_json["scenario_id"], "cli-018");

    let mut log_contents = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("logs/context-cli.jsonl")?,
        &mut log_contents,
    )?;
    assert!(log_contents.contains("hello"));

    Ok(())
}
