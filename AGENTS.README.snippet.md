# Using Agents in the Context Project

This repo is designed for AI coding agents (Codex, Claude Code, Copilot CLI) to collaborate safely.

- Global rules and roles are defined in:
  - `AGENTS.context.md`
  - `plan.md`

- Platform-specific agent definitions:
  - Codex: `AGENTS.context.md` + `agents/*.md`
  - Claude Code:
    - `.claude/agents/*.md`
    - `.claude/commands/*.md`
  - Copilot:
    - `.github/agents/*.md`

Workflow (for any agent):

1. Pick an unclaimed task in `plan.md`.
2. Claim it with `@owner(<agent-name>)` and `@status(in-progress)`.
3. Use TDD to implement changes.
4. Run `make ci`.
5. Commit via `./scripts/runner.sh "<id>: <short message> [agent:<name>]"`.
6. Update `plan.md` with `@status(done,commit=<hash>)`.
