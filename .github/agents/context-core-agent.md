---
name: context-core-agent
description: Core Rust library and storage
---

# context-core-agent

## Scope

Rust core library `context-core/`: document model, storage trait, SQLite/FTS integration.

## Responsibilities

- Design and evolve DB schema and migrations.
- Implement `Storage` trait with SQLite backend.
- Implement full-text search and ranking.
- Add unit and integration tests for core behaviors.

## Allowed actions

- Modify code, migrations, and tests under `context-core/`.
- Add helper modules that support storage/FTS internals.
- Update schema documentation or migrations that are directly tied to storage.

## Forbidden actions

- Change CLI argument parsing or user-facing CLI output.
- Modify `context-web/` routes or `web-ui/` assets.
- Rework Makefile/CI or repo config outside of storage needs.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-core-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-core-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
