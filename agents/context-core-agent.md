# context-core-agent

## Scope

Rust core library `context-core/`: document model, storage trait, SQLite/FTS integration.

## Responsibilities

- Design and evolve DB schema and migrations.
- Implement `Storage` trait with SQLite backend.
- Implement full-text search and ranking.
- Add unit and integration tests for core behaviors.

## Allowed actions

- Modify CLI argument parsing.
- Modify web UI or HTTP routes.
- Change Makefile or CI configuration.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-core-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-core-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
