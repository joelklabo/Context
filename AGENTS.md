# Context Project Agents (Codex CLI)

This file defines the main agents for the `context` project.

## Global Rules

- Follow **TDD** for every task (red → green → refactor).
- Only work on tasks defined in `plan.md`.
- Claim a task by setting `@owner(<agent-name>)` and `@status(in-progress)` on that line.
- Only one in-progress task per agent at a time.
- Always run `make ci` before committing.
- Never call `git commit` or `git push` directly; use `./scripts/runner.sh "<msg>"`.
- Do not edit another agent's in-progress task.

## Agents

- `context-core-agent` – owns `context-core/` storage, schema, search.
- `context-cli-agent` – owns `context-cli/` command behavior and JSON output.
- `context-web-agent` – owns `context-web/` server and `web-ui/` frontend.
- `context-devops-agent` – owns Makefile, CI, workflows, release.
- `context-debug-agent` – owns logging, tracing, debug bundles, Dev Web UI.
- `context-testing-agent` – owns tests and coverage across crates.
- `context-git-agent` – owns repo hygiene (.gitignore, .gitattributes, .vscode, hooks).
- `context-docs-agent` – owns README, AGENTS, CLAUDE/agent docs.
- `context-research-agent` – researches features and writes briefs stored in Context.
- `context-planner-agent` – turns research into plan.md-style implementation plans.

Each agent has a dedicated profile in `agents/*.md`.

## Generated agent-doc snippet

Keep this section in sync by running:

```bash
cargo run -p context-cli -- agent-doc --format markdown > docs/agent-doc.md
```

Current snippet (copied from `docs/agent-doc.md`):

> # context - Agent Usage
>
> This guide is for automation/LLM agents that call `context` to read and write project knowledge.
>
> ## Quickstart
> - Store: `context put --project <id> --key <key> --tag t1 --tag t2 --json` (body from stdin).
> - Fetch content-only: `context cat --project <id> --key <key>`.
> - Fetch with metadata: `context get --project <id> --key <key> --json`.
> - Search: `context find --project <id> "<query>" --json`.
> - Remove: `context rm --project <id> --key <key>` (soft delete).
> - Cleanup: `context gc --project <id>` (vacuum tombstones).
>
> ## Conventions for agents
> - Always pass `--json` when parsing output programmatically.
> - Provide `--project` for every command until `context project` subcommands land.
> - Include `--scenario` or `CONTEXT_SCENARIO` when running scripted sessions.
> - Prefer stable keys like `notes/<topic>`; add `--tag` for filtering.
> - Use `context cat` when you only need body text (no JSON framing).
>
> ## Command cheatsheet
> - `context put [--project <id>] [--key <key>] [--tag <tag>...] [--json]` — reads stdin or `--file`, creates/updates a document.
> - `context get [--project <id>] (--key <key> | --id <id>) [--json]` — returns metadata + body (default markdown).
> - `context cat [--project <id>] (--key <key> | --id <id>)` — body only, no framing.
> - `context find [--project <id>] <query> [--limit N] [--all-projects] [--json]` — ranked search results.
> - `context ls [--project <id>] [--json]` — list documents for a project.
> - `context rm [--project <id>] (--key <key> | --id <id>) [--force] [--json]` — soft delete; use `gc` to purge.
> - `context gc [--project <id>] [--dry-run] [--json]` — vacuum/purge tombstones.
> - `context web|web-dev [--port <p>] [--json]` — launch server wrappers.
> - `context debug-bundle [--scenario <id>] [--out <path>] [--json]` — collect logs/traces.
> - `context agent-config --target <all|codex|claude|copilot>` — emit agent configs.
> - `context agent-doc --format markdown` — emit this guide; redirect to `docs/agent-doc.md` to sync.
>
> ## Tips
> - STDIN vs `--file`: prefer piping for generated content; use `--file` for saved notes.
> - Tags: pass multiple `--tag` flags or comma-separated values.
> - Errors: non-zero exit codes indicate failure; stderr carries user-facing messages.
>
> ## Keeping docs in sync
> Run: `cargo run -p context-cli -- agent-doc --format markdown > docs/agent-doc.md`.
