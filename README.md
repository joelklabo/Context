# Context

Context is a Rust workspace for storing, searching, and serving project context to CLI and web clients. The repo is built for multi-agent collaboration (Codex, Claude Code, Copilot) with strict TDD and task tracking in `plan.md`.

## Workspace layout

- `context-core/` - document model, SQLite storage, migrations, FTS search, TTL/soft delete.
- `context-cli/` - `context` binary (put/get/find/ls/rm/gc, agent-doc, debug bundle stubs).
- `context-web/` - Axum server scaffold.
- `context-agent/` - agent-doc generation helpers.
- `context-telemetry/` - shared telemetry setup for CLI and web.
- `context-plan/` - plan validator (`make plan-check`).
- `scripts/runner.sh` - locked commit/push helper.

## Prerequisites

- Rust stable toolchain (see `rust-toolchain.toml`), with `rustfmt` and `clippy` components.
- Git and a working C toolchain (for building SQLite via `sqlx`).
- (macOS) `brew install sqlite` if system headers are missing.

## Install and setup

```bash
git clone <your-fork-url> context
cd context
rustup component add rustfmt clippy
make ci
```

`make ci` runs `fmt`, `clippy`, tests for all crates, and `plan-check`.

## Quickstart (CLI)

The CLI currently exercises stubbed behaviors that mirror the final shapes. JSON output is agent-friendly; human output is concise.

Store a note (stdin):

```bash
cargo run -p context-cli -- --project demo --json put --key note-1 --tag alpha --tag beta <<'NOTE'
Hello from stdin
NOTE
```

Fetch by key (human vs JSON):

```bash
cargo run -p context-cli -- --project demo get --key note-1
cargo run -p context-cli -- --project demo --json get --key note-1
```

Search and list:

```bash
cargo run -p context-cli -- --project demo find "demo query"
cargo run -p context-cli -- --project demo ls
```

Delete and GC (soft delete + cleanup stubs):

```bash
cargo run -p context-cli -- --project demo rm --key note-1
cargo run -p context-cli -- --project demo gc --dry-run
```

Agent documentation (Markdown):

```bash
cargo run -p context-cli -- agent-doc --format markdown
```

## Telemetry and debug bundles

- Set `CONTEXT_LOG_DIR` to capture JSON logs (default: current working directory). Logs write to `context-cli.jsonl` and include spans.
- Set `CONTEXT_SCENARIO` to tag logs with a scenario ID (use the same value you put in `plan.md` `@scenario(...)`).
- Generate a bundle that collects logs and metadata:

```bash
cargo run -p context-cli -- debug-bundle --scenario my-scn --out bundle.zip
```

## Agent workflow and TDD rules

- Tasks live only in `plan.md`. Claim a task by setting `@owner(<agent-name>)`, `@status(in-progress)`, and `@scenario(<id>)`.
- One in-progress task per agent.
- TDD always: write or extend tests to fail, implement to green, refactor while green.
- Always run `make ci` before committing (pre-commit hook enforces it).
- Commit and push via `./scripts/runner.sh "<id>: <message> [agent:<name>] [scenario:<id>]"` - never call `git commit` or `git push` directly.
- After a successful commit, flip `[ ]` to `[x]` in `plan.md` and set `@status(done,commit=<hash>)`.

## Contributing checklist

1. Read `plan.md`, `AGENTS.md`, and your `agents/<role>.md`.
2. Pick the earliest unclaimed task in your area; claim it with `@owner`, `@status(in-progress)`, and `@scenario`.
3. Write failing tests, implement, refactor.
4. Run `make ci` until green.
5. Commit with `scripts/runner.sh`, then update `plan.md` to mark the task done.

## Troubleshooting

- `make ci` fails formatting: run `cargo fmt --all`.
- `clippy` warnings: run `cargo clippy --all-targets --all-features`.
- Plan errors: run `make plan-check` to see validation output.
- SQLite build issues: ensure SQLite headers are installed (`brew install sqlite` on macOS).

## Status

Many commands are still stubs; see `plan.md` for prioritized work and ownership. Contributions should keep behavior aligned with the documented CLI shapes and plan tasks.
