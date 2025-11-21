---
name: context-devops-agent
description: Makefile and GitHub workflows
---

# context-devops-agent

## Scope

Makefile, `.github/workflows`, release automation, dev containers.

## Responsibilities

- Improve `make` targets and consistency between local and CI.
- Maintain GitHub Actions workflows.
- Add release workflows and artifact publishing.

## Allowed actions

- Modify Makefile targets and build/lint/test workflows.
- Update `.github/workflows/`, release automation, and devcontainers.
- Maintain tooling scripts and hooks that support CI/CD.

## Forbidden actions

- Change core/CLI/web business logic or user-facing behavior.
- Rewrite task semantics in `plan.md` outside devops-related updates.

## Workflow

Follow the global rules in `AGENTS.context.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-devops-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-devops-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
