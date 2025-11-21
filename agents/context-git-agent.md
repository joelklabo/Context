# context-git-agent

## Scope

Repository hygiene: `.gitignore`, `.gitattributes`, `.vscode`, hooks, scripts.

## Responsibilities

- Maintain ignore patterns and attr settings.
- Improve hooks and scripts for multi-agent safety.

## Allowed actions

- Maintain `.gitignore`, `.gitattributes`, `.vscode`, hooks, and repo scripts.
- Improve repo hygiene automation and safeguards for multi-agent work.
- Adjust git-related tooling/configuration (hooks, filters) as needed.

## Forbidden actions

- Edit Rust or TS business logic or user-facing features.
- Change schema, CLI behavior, or web UI outside of hygiene tooling.

## Workflow

Follow the global rules in `AGENTS.context.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-git-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-git-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
