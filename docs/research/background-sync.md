# Background Sync Research Brief

## Purpose and scope

- **Goal:** Enable Background Sync so a user can keep their Context storage in sync across two machines (e.g., laptop ↔ desktop) with minimal manual steps.
- **Intended outcome of this research:** Provide constraints, options, and a recommended MVP approach that planners can turn into `plan.md` tasks, and implementers can execute.
- **Audience:** `context-planner-agent` (to draft the plan) and `context-core-agent` (to implement).

## Problem statement

Users want Context data available on multiple machines. Today, storage is local SQLite. We need a way to push/pull data safely, avoid corruption, and handle conflicts, ideally without a heavy always-on server.

## Goals

- Sync documents and metadata between two peers in a predictable, low-friction flow.
- Avoid corrupting the SQLite database; keep operations crash-safe.
- Allow unattended periodic sync (cron/systemd), plus manual `context sync`.
- Preserve history/versioning enough to recover from accidental overwrites.

## Non-goals (for MVP)

- Realtime multi-writer conflict-free replication (CRDTs, multi-leader).
- Zero-downtime bidirectional sync under active concurrent writes.
- End-to-end encryption and remote auth; assume trusted, same-user machines for MVP.
- Mobile clients.

## Constraints and assumptions

- Storage is SQLite with WAL enabled; we already record document versions and tombstones.
- Typical usage is single-writer at a time; background sync can assume “mostly one writer” but must not break if both write occasionally.
- We can require that the CLI exits cleanly before syncing (avoid open connections).
- Network: we can use HTTPS to an object store or SSH to a shared location; avoid bespoke daemon.
- File size: DB + history likely fits comfortably (<100MB). Large binary blobs are rare.

## User stories

1. As a user, I run `context sync push` on Machine A to upload my latest data somewhere safe.
2. On Machine B, I run `context sync pull` to fetch and apply updates.
3. I can run `context sync status` to see whether I’m behind/ahead and when last synced.
4. A cron job can run `context sync pull` daily without trashing my DB.

## Candidate approaches

### A) Whole-DB snapshot with WAL guard (simple, manual conflict policy)
- Steps: Ensure DB is not open, checkpoint WAL, copy SQLite file + metadata to remote, fetch on other machine and replace local after backup.
- Pros: Simple, predictable, low engineering effort; reuses existing schema.
- Cons: Heavyweight (copies entire DB); last-writer-wins; downtime during swap.

### B) Incremental change log export/import (document_versions)
- Steps: Export new rows from `document_versions` since last sync id, transmit, apply on peer.
- Pros: Smaller payloads; retains per-doc history; less downtime.
- Cons: Requires export/import tooling, ordering guarantees, and conflict policy; more code.

### C) Git-based sync (repo of exported docs + sqlite)
- Steps: Export docs to files, commit, push/pull, re-import.
- Pros: Reuses Git transport/conflict tools.
- Cons: Lots of glue, not great for binary SQLite; overkill for MVP.

## Recommended MVP approach

Start with **A) Whole-DB snapshot with WAL guard** plus a minimal status file:
- Implement `context sync push|pull|status` as CLI stubs that call into core.
- Use a simple remote abstraction: local path or `s3://` (optionally `file://`/`ssh://` later). For MVP, default to a local filesystem path to keep testable offline.
- Before push: ensure no active connections (spawn check), checkpoint WAL, copy DB + a small `sync-meta.json` containing `{db_hash, last_sync_ts, generation}`.
- Before pull: verify local backup (copy current DB aside), fetch remote DB + meta, replace local DB atomically, and vacuum.
- Conflict policy: last-writer-wins; detect divergent generations and warn (require `--force` to overwrite if local has unseen changes).
- Logging: include project and scenario_id in sync spans.

## Risks and mitigations

- **DB corruption if copied mid-write:** mitigate by checkpointing WAL, performing copies with fsync, and refusing if an exclusive lock can’t be acquired.
- **Divergent histories:** store a generation/id in meta; if local generation differs from remote and both advanced, require `--force`.
- **Slow copies on large DBs:** document size expectations; add `limit`/exclude tables later if needed.
- **Secrets in DB:** none today, but future auth tokens would need encryption—call out as a follow-up.

## Deliverables for planner

- Define tasks for CLI surface (`context sync push|pull|status`), core sync module (checkpoint + copy + meta), conflict detection, and tests (happy path, conflict warning, force).
- Specify MVP transport (local path) and how to stub other remotes for later.
- Decide where to store `sync-meta.json` remotely and locally.
- Ensure plan.md tasks include `make ci` expectation and coverage for TTL/deleted docs persistence across sync.

## Open questions

- Exact location of local DB (respect `CONTEXT_HOME`?); planner should pin this.
- How to detect “active connections” robustly (PID file vs. advisory lock).
- Should we bundle log export with sync for debugging?
- Do we need optional compression for the DB snapshot?

## Suggested Context storage keys (for CLI usage)

- Research doc key: `research/background-sync`
- Plan doc key (to be produced by planner): `plan/background-sync`
