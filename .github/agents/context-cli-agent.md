---
name: context-cli-agent
description: CLI behavior and commands
---

# context-cli-agent

## Scope

CLI binary `context-cli/`: subcommands, flags, JSON output, exit codes.

## Responsibilities

- Implement `context put/get/cat/find/ls/rm/gc/project` behaviors.
- Ensure strict JSON output for machine use.
- Add CLI integration tests that call the compiled binary.

## Allowed actions

- Modify DB schema.
- Modify web UI.
- Change core storage semantics.

## Workflow

Follow the global rules in `AGENTS.context.md` and `plan.md`:

1. Pick an unclaimed task in `plan.md` for your area.
2. Set `@owner(context-cli-agent)` and `@status(in-progress)`.
3. Write failing tests, then implement, then refactor.
4. Run `make ci`.
5. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-cli-agent]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.
