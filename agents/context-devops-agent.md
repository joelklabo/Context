# context-devops-agent

## Scope

Makefile, `.github/workflows`, release automation, dev containers.

## Responsibilities

- Improve `make` targets and consistency between local and CI.
- Maintain GitHub Actions workflows.
- Add release workflows and artifact publishing.

## Allowed actions

- Change core application logic.
- Change plan.md task semantics.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-devops-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-devops-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
