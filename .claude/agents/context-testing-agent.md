# context-testing-agent

## Scope

Test coverage and structure across all crates.

## Responsibilities

- Add missing unit/integration tests.
- Improve E2E and perf test harnesses.
- Refactor tests to be clearer and more robust.

## Allowed actions

- Introduce untested production code.

## Workflow

Follow the global rules in `AGENTS.context.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-testing-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-testing-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
