use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct CliSyncResult {
    generation: u64,
    #[serde(rename = "db_hash")]
    _db_hash: String,
    #[serde(rename = "db_bytes")]
    _db_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct CliSyncStatus {
    state: String,
}

#[test]
fn sync_push_and_status_via_cli() -> Result<()> {
    let temp = tempdir()?;
    let home = temp.path().join("home");
    let remote = temp.path().join("remote");
    fs::create_dir_all(&home)?;
    fs::write(home.join("db.sqlite"), b"cli-sync")?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", &home)
        .env("CONTEXT_SYNC_REMOTE", &remote)
        .args(["--json", "sync", "push"])
        .assert()
        .success();

    let result: CliSyncResult = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(result.generation, 1);
    assert!(remote.join("db.sqlite").exists());
    assert!(home.join("sync-meta.json").exists());
    assert!(remote.join("sync-meta.json").exists());

    let status_output = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .env("CONTEXT_HOME", &home)
        .env("CONTEXT_SYNC_REMOTE", &remote)
        .args(["--json", "sync", "status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let status: CliSyncStatus = serde_json::from_slice(&status_output)?;
    assert_eq!(status.state.to_lowercase(), "insync");

    Ok(())
}

#[test]
fn sync_pull_overwrites_when_force_enabled() -> Result<()> {
    let temp = tempdir()?;
    let home = temp.path().join("home");
    let remote = temp.path().join("remote");
    fs::create_dir_all(&home)?;
    fs::write(home.join("db.sqlite"), b"initial-local")?;

    Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .env("CONTEXT_HOME", &home)
        .env("CONTEXT_SYNC_REMOTE", &remote)
        .args(["--json", "sync", "push"])
        .assert()
        .success();

    fs::write(remote.join("db.sqlite"), b"remote-change")?;
    let mut remote_meta =
        context_core::sync::load_meta(&remote.join("sync-meta.json"))?.expect("remote meta");
    remote_meta.generation += 1;
    remote_meta.db_hash = context_core::sync::compute_db_hash(&remote.join("db.sqlite"))?;
    remote_meta.db_bytes = fs::metadata(remote.join("db.sqlite"))?.len();
    context_core::sync::write_meta(&remote.join("sync-meta.json"), &remote_meta)?;

    let err = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .env("CONTEXT_HOME", &home)
        .env("CONTEXT_SYNC_REMOTE", &remote)
        .args(["--json", "sync", "pull"])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&err.get_output().stderr);
    assert!(stderr.to_lowercase().contains("force"));

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .env("CONTEXT_HOME", &home)
        .env("CONTEXT_SYNC_REMOTE", &remote)
        .args(["--json", "sync", "pull", "--force"])
        .assert()
        .success();

    let result: CliSyncResult = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(result.generation, remote_meta.generation);

    let local_contents = fs::read(home.join("db.sqlite"))?;
    assert_eq!(local_contents, b"remote-change");

    Ok(())
}
