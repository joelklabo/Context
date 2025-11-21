# CLAUDE.md – Context Project

Claude Code is used as a multi-agent collaborator on this repo.

Key files:

- `plan.md` – master task list & workflow rules.
- `AGENTS.context.md` – global agent definitions and responsibilities.
- `.claude/agents/*.md` – per-agent profiles (core, cli, web, devops, debug, testing, git, docs).
- `.claude/commands/*.md` – reusable commands (`/context-pick-task`, `/context-debug`, `/context-apply`).

When working as a Claude agent:

1. Read your profile in `.claude/agents/<name>.md`.
2. Only pick tasks in your scope.
3. Always use TDD and `make ci`.
4. Only commit via `./scripts/runner.sh`.

Do not modify tasks owned by other agents or violate the global rules in `plan.md`.

## Generated agent-doc snippet

Regenerate from the CLI to keep in sync:

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
