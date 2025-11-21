# context - Agent Usage

This guide is for automation/LLM agents that call `context` to read and write project knowledge.

## Quickstart
- Store: `context put --project <id> --key <key> --tag t1 --tag t2 --json` (body from stdin).
- Fetch content-only: `context cat --project <id> --key <key>`.
- Fetch with metadata: `context get --project <id> --key <key> --json`.
- Search: `context find --project <id> "<query>" --json`.
- Remove: `context rm --project <id> --key <key>` (soft delete).
- Cleanup: `context gc --project <id>` (vacuum tombstones).

## Conventions for agents
- Always pass `--json` when parsing output programmatically.
- Provide `--project` for every command until `context project` subcommands land.
- Include `--scenario` or `CONTEXT_SCENARIO` when running scripted sessions.
- Prefer stable keys like `notes/<topic>`; add `--tag` for filtering.
- Use `context cat` when you only need body text (no JSON framing).

## Command cheatsheet
- `context put [--project <id>] [--key <key>] [--tag <tag>...] [--json]` — reads stdin or `--file`, creates/updates a document.
- `context get [--project <id>] (--key <key> | --id <id>) [--json]` — returns metadata + body (default markdown).
- `context cat [--project <id>] (--key <key> | --id <id>)` — body only, no framing.
- `context find [--project <id>] <query> [--limit N] [--all-projects] [--json]` — ranked search results.
- `context ls [--project <id>] [--json]` — list documents for a project.
- `context rm [--project <id>] (--key <key> | --id <id>) [--force] [--json]` — soft delete; use `gc` to purge.
- `context gc [--project <id>] [--dry-run] [--json]` — vacuum/purge tombstones.
- `context web|web-dev [--port <p>] [--json]` — launch server wrappers.
- `context debug-bundle [--scenario <id>] [--out <path>] [--json]` — collect logs/traces.
- `context agent-config --target <all|codex|claude|copilot>` — emit agent configs.
- `context agent-doc --format markdown` — emit this guide; redirect to `docs/agent-doc.md` to sync.

## Tips
- STDIN vs `--file`: prefer piping for generated content; use `--file` for saved notes.
- Tags: pass multiple `--tag` flags or comma-separated values.
- Errors: non-zero exit codes indicate failure; stderr carries user-facing messages.

## Keeping docs in sync
Run: `cargo run -p context-cli -- agent-doc --format markdown > docs/agent-doc.md`.
