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

Each agent has a dedicated profile in `agents/*.md`.

## Generated agent-doc snippet

Keep this section in sync by running:

```bash
cargo run -p context-cli -- agent-doc --format markdown > docs/agent-doc.md
```

Current snippet (copied from `docs/agent-doc.md`):

> # context - Agent Usage (stub)
>
> - Use `context find --project <id> --json` to search.
> - Use `context get --project <id> --key <key> --json` to fetch a doc.
> - Always prefer `--json` when parsing output as an AI agent.
>
> This is a stub; the real content will be generated later.
