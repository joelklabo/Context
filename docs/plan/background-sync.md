# Background Sync - Implementation Plan (MVP)

Based on `docs/research/background-sync.md`. Goal: ship a reliable, offline-testable MVP that moves a local SQLite store between machines via a filesystem "remote" with `context sync push|pull|status`. Default policy is last-writer-wins with divergence warning and `--force`.

## Scope and assumptions
- Single-user, same-trust machines. No auth/encryption.
- Storage lives under `CONTEXT_HOME` (default `~/.context`): `db.sqlite`, `sync-meta.json`, `logs/`.
- Remote is a filesystem path (abs or relative). Future remotes (s3/ssh) are pluggable but stubbed.
- CLI must exit before sync; sync code ensures WAL checkpoint + exclusive copy to avoid corruption.

## Data + metadata
- Local DB: `CONTEXT_HOME/db.sqlite` (ensure WAL on).
- Local meta: `CONTEXT_HOME/sync-meta.json`.
- Remote payload: `db.sqlite` + `sync-meta.json` in remote dir.
- Meta fields: `generation` (u64 increment on push), `db_hash` (sha256), `db_bytes`, `last_synced_at` (UTC RFC3339), `machine` (hostname), `schema_version` (for future), `project` (default or current).

## Core implementation (context-core)
1) New module `sync`:
   - `pub struct SyncConfig { local_db: PathBuf, local_meta: PathBuf, remote: PathBuf }`.
   - `pub async fn status(cfg) -> Result<SyncStatus>`: load local/remote meta, compare generation/hash, compute `ahead`/`behind`/`diverged`, include timestamps and sizes.
   - `pub async fn push(cfg, force: bool) -> Result<SyncResult>`:
     - ensure DB closed: acquire advisory lock file `CONTEXT_HOME/.lock` or fail with helpful message.
     - checkpoint WAL: `PRAGMA wal_checkpoint(TRUNCATE); PRAGMA vacuum;` (or copy after checkpoint).
     - back up local DB to `db.sqlite.bak` (fs copy + fsync).
     - copy DB to remote (fs::copy + fsync), write updated meta (generation+1, hash/bytes/ts/machine).
   - `pub async fn pull(cfg, force: bool) -> Result<SyncResult>`:
     - load remote meta; if diverged and !force -> error with guidance.
     - back up local DB to `db.sqlite.before-pull`.
     - copy remote DB down, fsync, replace local DB atomically, write local meta to match remote.
   - Hash helper: sha256 of DB file (streamed).
   - Meta serde struct + read/write helpers (fsync).
   - Tests: use tempdirs for local/remote, seed DB via sqlx to include docs + TTL/tombstones, assert hash/gen changes, conflict warning, force overwrite, TTL/deleted rows preserved.
2) Expose `SyncStatus`/`SyncResult` structs with JSON-serializable fields (status, generation, hash, bytes, last_synced_at, diverged bool).

## CLI surface (context-cli)
1) Add `context sync` with subcommands:
   - `status [--remote <path>] [--json]`
   - `push [--remote <path>] [--force] [--json]`
   - `pull [--remote <path>] [--force] [--json]`
   - `--remote` default: `$CONTEXT_SYNC_REMOTE` or `$CONTEXT_HOME/sync-remote` (create if missing).
2) Wire to core sync functions; handle errors with actionable messages (e.g., "database busy; close other context processes").
3) JSON output: include status/result structs; human output: concise summary with generations and hashes.
4) Telemetry: add spans `cli.sync.status|push|pull` with project/scenario; log remote path.

## Locking and safety
- Implement simple file lock in `CONTEXT_HOME/sync.lock` (advisory) before push/pull; fail fast if held.
- Checkpoint/snapshot: open sqlx pool to local DB, run `PRAGMA wal_checkpoint(TRUNCATE)`, close pool before file copy. Copies use `std::fs::copy` + `File::sync_all`.
- Atomic replace on pull: write to temp file in same dir, fsync, rename over `db.sqlite`.
- Keep backups: latest local backup kept as `db.sqlite.bak` (push) or `db.sqlite.before-pull`.

## Tests (integration)
- Location: `context-core/tests/sync.rs` (core) and `context-cli/tests/sync_command.rs`.
- Cases:
  - Happy path push/pull: docs, TTL, deleted_at preserved; hash/generation updated.
  - Status shows ahead/behind/equal/diverged traits.
  - Divergence error without `--force`; succeeds with `--force`.
  - Lock contention: second push fails while lock held.
  - Remote missing -> clear error; remote created on push.
  - CLI JSON output shape matches structs; human output mentions paths and gens.
  - WAL checkpoint invoked (e.g., create WAL file, ensure truncated after push).

## Config and paths
- Introduce `CONTEXT_HOME` env defaulting to `~/.context` for CLI/core; ensure existing config readers migrate to use it.
- Default DB filename `db.sqlite`; meta `sync-meta.json`; remote default `${CONTEXT_HOME}/sync-remote`.
- Document these in README and agent docs once implemented.

## Follow-ups (post-MVP, do not block)
- Remote providers (s3/ssh) behind a trait.
- Compression of DB snapshot.
- Include log bundle with sync result.
- Stronger multi-writer detection (sqlite `user_version` stamping, PID ownership).
- Optional encryption for remote payload.
