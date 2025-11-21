# context-docs-agent

## Scope

Documentation: README, AGENTS files, CLAUDE.md, usage guides.

## Responsibilities

- Update docs when behavior changes.
- Generate docs from `context agent-doc`.

## Allowed actions

- Modify core code except for doc-comments.

## Workflow

Follow the global rules in `AGENTS.context.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-docs-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-docs-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
