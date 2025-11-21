use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use context_core::sync::{load_meta, pull, push, status, write_meta, SyncConfig, SyncMeta, SyncState};
use tempfile::tempdir;

fn write_file(path: &PathBuf, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn push_creates_remote_and_meta() -> Result<()> {
    let temp = tempdir()?;
    let local_db = temp.path().join("db.sqlite");
    let local_meta = temp.path().join("sync-meta.json");
    let remote = temp.path().join("remote");

    write_file(&local_db, b"hello-world");

    let cfg = SyncConfig {
        local_db: local_db.clone(),
        local_meta: local_meta.clone(),
        remote: remote.clone(),
    };

    let result = push(&cfg, false)?;
    assert_eq!(result.generation, 1);

    let remote_db = remote.join("db.sqlite");
    assert!(remote_db.exists());
    let remote_meta = remote.join("sync-meta.json");
    assert!(remote_meta.exists());

    let local_meta_loaded = load_meta(&local_meta)?.unwrap();
    let remote_meta_loaded = load_meta(&remote_meta)?.unwrap();
    assert_eq!(local_meta_loaded.generation, 1);
    assert_eq!(remote_meta_loaded.generation, 1);
    assert_eq!(local_meta_loaded.db_hash, remote_meta_loaded.db_hash);

    Ok(())
}

#[test]
fn pull_requires_force_when_diverged() -> Result<()> {
    let temp = tempdir()?;
    let local_db = temp.path().join("db.sqlite");
    let local_meta = temp.path().join("sync-meta.json");
    let remote = temp.path().join("remote");

    write_file(&local_db, b"alpha");
    let cfg = SyncConfig {
        local_db: local_db.clone(),
        local_meta: local_meta.clone(),
        remote: remote.clone(),
    };

    push(&cfg, false)?;

    write_file(&local_db, b"local-change");
    let mut local_meta_loaded = load_meta(&local_meta)?.unwrap();
    local_meta_loaded.generation = 2;
    local_meta_loaded.db_hash = "local-hash".to_string();
    local_meta_loaded.last_synced_at = Utc::now();
    local_meta_loaded.db_bytes = fs::metadata(&local_db)?.len();
    write_meta(&local_meta, &local_meta_loaded)?;

    let remote_db = remote.join("db.sqlite");
    write_file(&remote_db, b"remote-change");
    let mut remote_meta_loaded = load_meta(&remote.join("sync-meta.json"))?.unwrap();
    remote_meta_loaded.generation = 2;
    remote_meta_loaded.db_hash = "remote-hash".to_string();
    remote_meta_loaded.last_synced_at = Utc::now();
    remote_meta_loaded.db_bytes = fs::metadata(&remote_db)?.len();
    write_meta(&remote.join("sync-meta.json"), &remote_meta_loaded)?;

    let err = pull(&cfg, false).expect_err("expected divergence");
    assert!(err.to_string().contains("diverg"));

    let result = pull(&cfg, true)?;
    assert_eq!(result.generation, 2);
    let contents = fs::read(&local_db)?;
    assert_eq!(contents, b"remote-change");

    Ok(())
}

#[test]
fn status_reports_ahead_and_behind() -> Result<()> {
    let temp = tempdir()?;
    let local_db = temp.path().join("db.sqlite");
    let local_meta = temp.path().join("sync-meta.json");
    let remote = temp.path().join("remote");

    write_file(&local_db, b"alpha");
    let cfg = SyncConfig {
        local_db: local_db.clone(),
        local_meta: local_meta.clone(),
        remote: remote.clone(),
    };

    push(&cfg, false)?;

    let remote_db = remote.join("db.sqlite");
    write_file(&remote_db, b"beta");
    let mut remote_meta_loaded = load_meta(&remote.join("sync-meta.json"))?.unwrap();
    remote_meta_loaded.generation = 2;
    remote_meta_loaded.db_hash = "remote-new".to_string();
    remote_meta_loaded.db_bytes = fs::metadata(&remote_db)?.len();
    write_meta(&remote.join("sync-meta.json"), &remote_meta_loaded)?;

    let behind = status(&cfg)?;
    assert_eq!(behind.state, SyncState::Behind);

    let mut local_meta_loaded = load_meta(&local_meta)?.unwrap();
    local_meta_loaded.generation = 3;
    local_meta_loaded.db_hash = "local-new".to_string();
    write_meta(&local_meta, &local_meta_loaded)?;

    let ahead = status(&cfg)?;
    assert_eq!(ahead.state, SyncState::Ahead);

    Ok(())
}
