use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Result;

#[derive(Clone, Debug)]
pub struct SyncConfig {
    pub local_db: PathBuf,
    pub local_meta: PathBuf,
    pub remote: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMeta {
    pub generation: u64,
    pub db_hash: String,
    pub db_bytes: u64,
    pub last_synced_at: DateTime<Utc>,
    pub machine: String,
    pub schema_version: u32,
    pub project: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncState {
    InSync,
    Ahead,
    Behind,
    Diverged,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncStatus {
    pub state: SyncState,
    pub local: Option<SyncMeta>,
    pub remote: Option<SyncMeta>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncResult {
    pub generation: u64,
    pub db_hash: String,
    pub db_bytes: u64,
}

pub fn status(cfg: &SyncConfig) -> Result<SyncStatus> {
    let local_meta = load_meta(&cfg.local_meta)?;
    let remote_meta = load_meta(&cfg.remote.join("sync-meta.json"))?;

    let state = match (&local_meta, &remote_meta) {
        (Some(local), Some(remote)) => {
            if local.db_hash == remote.db_hash {
                SyncState::InSync
            } else if local.generation > remote.generation {
                SyncState::Ahead
            } else if local.generation < remote.generation {
                SyncState::Behind
            } else {
                SyncState::Diverged
            }
        }
        _ => SyncState::Unknown,
    };

    Ok(SyncStatus {
        state,
        local: local_meta,
        remote: remote_meta,
    })
}

pub fn push(cfg: &SyncConfig, force: bool) -> Result<SyncResult> {
    let _lock = acquire_lock(&cfg.local_db)?;

    if !cfg.local_db.exists() {
        return Err("local database not found".into());
    }

    fs::create_dir_all(&cfg.remote)?;
    let remote_db = cfg.remote.join("db.sqlite");
    let remote_meta_path = cfg.remote.join("sync-meta.json");

    let current_remote_meta = load_meta(&remote_meta_path)?;
    let local_meta = load_meta(&cfg.local_meta)?;

    if !force {
        if let (Some(local), Some(remote)) = (&local_meta, &current_remote_meta) {
            if local.generation != remote.generation && local.db_hash != remote.db_hash {
                return Err("remote diverged; use --force to overwrite".into());
            }
        }
    }

    let bak = cfg.local_db.with_extension("bak");
    fs::copy(&cfg.local_db, &bak)?;

    let meta = build_meta(&cfg.local_db, &local_meta)?;

    fs::copy(&cfg.local_db, &remote_db)?;
    write_meta(&cfg.local_meta, &meta)?;
    write_meta(&remote_meta_path, &meta)?;

    Ok(SyncResult {
        generation: meta.generation,
        db_hash: meta.db_hash.clone(),
        db_bytes: meta.db_bytes,
    })
}

pub fn pull(cfg: &SyncConfig, force: bool) -> Result<SyncResult> {
    let _lock = acquire_lock(&cfg.local_db)?;
    let remote_db = cfg.remote.join("db.sqlite");
    let remote_meta_path = cfg.remote.join("sync-meta.json");

    if !remote_db.exists() {
        return Err("remote database not found".into());
    }

    let remote_meta = load_meta(&remote_meta_path)?
        .ok_or_else(|| "remote metadata missing".to_string())?;
    let local_meta = load_meta(&cfg.local_meta)?;

    if !force {
        if let Some(local) = &local_meta {
            if local.generation != remote_meta.generation && local.db_hash != remote_meta.db_hash {
                return Err("local and remote have diverged; rerun with --force".into());
            }
        }
    }

    if cfg.local_db.exists() {
        let bak = cfg.local_db.with_file_name("db.sqlite.before-pull");
        fs::copy(&cfg.local_db, &bak)?;
    }

    fs::create_dir_all(cfg.local_db.parent().unwrap())?;
    let tmp = cfg.local_db.with_extension("tmp");
    fs::copy(&remote_db, &tmp)?;
    fs::rename(&tmp, &cfg.local_db)?;

    write_meta(&cfg.local_meta, &remote_meta)?;

    Ok(SyncResult {
        generation: remote_meta.generation,
        db_hash: remote_meta.db_hash.clone(),
        db_bytes: remote_meta.db_bytes,
    })
}

fn build_meta(local_db: &Path, existing: &Option<SyncMeta>) -> Result<SyncMeta> {
    let db_bytes = fs::metadata(local_db)?.len();
    let db_hash = compute_db_hash(local_db)?;
    let generation = existing.as_ref().map(|m| m.generation + 1).unwrap_or(1);
    let machine = hostname();
    let now = Utc::now();

    Ok(SyncMeta {
        generation,
        db_hash,
        db_bytes,
        last_synced_at: now,
        machine,
        schema_version: 1,
        project: None,
    })
}

pub fn load_meta(path: &Path) -> Result<Option<SyncMeta>> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read(path)?;
    let meta: SyncMeta = serde_json::from_slice(&data)?;
    Ok(Some(meta))
}

pub fn write_meta(path: &Path, meta: &SyncMeta) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec_pretty(meta)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn compute_db_hash(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_lock(local_db: &Path) -> Result<LockGuard> {
    let dir = local_db
        .parent()
        .ok_or_else(|| "local db path missing parent".to_string())?;
    let lock_path = dir.join("sync.lock");
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(&lock_path);
    match file {
        Ok(mut f) => {
            let pid = std::process::id();
            let _ = writeln!(f, "{pid}");
            Ok(LockGuard { path: lock_path })
        }
        Err(e) => Err(format!("could not obtain sync lock: {e}").into()),
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "unknown".to_string())
}
