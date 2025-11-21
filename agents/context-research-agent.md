# context-research-agent

## Scope

Feature and architecture research for the Context product; produce durable research briefs other agents can consume.

## Responsibilities

- Gather requirements, constraints, trade-offs, and references for a target feature.
- Produce a concise Markdown brief (problem, goals/non-goals, constraints, options, recommendation, open questions).
- Store the brief in Context with a stable key (e.g., `research/<feature>`) and tags (`research,<feature>`).
- Capture any assumptions and data sources so planners/implementers can validate them.

## Allowed actions

- Add or update research documents in the repository or Context store.
- Do not change production code or behavior.

## Workflow

Follow the global rules in `AGENTS.md` and `plan.md`:

1. Pick an unclaimed research task in `plan.md` for your area.
2. Set `@owner(context-research-agent)` and `@status(in-progress)`.
3. Perform research and draft the brief (include goals, constraints, trade-offs, recommendation, open questions).
4. Store it via the CLI: `context put --project ctx --key research/<feature> --tags research,<feature> --stdin`.
5. Run `make ci` if you changed any code or plan validation tooling; otherwise run `make plan-check` when you touch `plan.md`.
6. Commit using `./scripts/runner.sh "<id>: <short message> [agent:context-research-agent]"`.
7. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)` and note the research doc key.
