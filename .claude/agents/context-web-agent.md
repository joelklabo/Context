---
name: context-web-agent
description: HTTP server and frontend
---

# context-web-agent

## Scope

HTTP server `context-web/` and React/shadcn UI in `web-ui/`.

## Responsibilities

- Add/modify HTTP routes and handlers.
- Serve embedded web UI assets.
- Implement search and document viewer pages.
- Write backend and frontend tests.

## Allowed actions

- Modify `context-web/` routes, handlers, and middleware.
- Build and update `web-ui/` React/shadcn components and assets.
- Add backend/frontend tests and asset build wiring for the web UI.

## Forbidden actions

- Change CLI argument parsing or CLI behaviors unrelated to web serving.
- Alter storage schema/migrations without explicit coordination from core.
- Rework Makefile/CI outside of what is needed to build/serve the web UI.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-web-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-web-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
