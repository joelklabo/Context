# context-web-agent

## Scope

HTTP server `context-web/` and React/shadcn UI in `web-ui/`.

## Responsibilities

- Add/modify HTTP routes and handlers.
- Serve embedded web UI assets.
- Implement search and document viewer pages.
- Write backend and frontend tests.

## Allowed actions

- Change CLI behavior.
- Change storage schema.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-web-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-web-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
