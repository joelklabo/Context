# context – Agent Implementation Plan (v0)

> This file is for humans **and** AI agents. It is the single source of truth for
> what to build next, in what order, and under which rules.

---

## 0. Rules of engagement (MANDATORY)

1. **Test‑Driven Development only.**
   - Every task must follow the **red → green → refactor** loop.
   - Write or update tests **first**, watch them fail, then implement code until they pass.
   - Refactor only when tests are green.

2. **One task → one commit (or small cluster) → one agent.**
   - Each checklist item below represents a *small*, well‑scoped step.
   - One agent “owns” a task while it’s in progress.
   - Each completed task must be traceable to at least one commit that mentions its ID.

3. **Full CI before every commit.**
   - Locally, `make ci` **must succeed** before `git commit`.
   - The `pre-commit` hook enforces this; agents must treat a failing hook as a hard stop.

4. **Runner is the only way to commit for agents.**
   - Agents **never** call `git commit` or `git push` directly.
   - They call `./scripts/runner.sh "msg"` which handles:
     - acquiring a repo lock,
     - `git fetch && git rebase`,
     - `make ci`,
     - `git add/commit/push`.

5. **Multi‑agent safety.**
   - Only one agent may own a given task at a time.
   - Runner lock prevents two agents from committing simultaneously.
   - If `runner.sh` reports “Another agent is committing”, back off and retry later.

6. **PLAN.md is authoritative for tasks.**
   - Do not invent tasks outside this file.
   - If you need new work, add a new checklist item and keep it small.

7. **No production code without tests.**
   - If you can’t express the requirement in tests, the task is not ready.
   - Agents should refine or split the task rather than coding without tests.

8. **Commit early, commit often.**
   - Prefer multiple small commits that each keep tests green.
   - Avoid huge, multi‑feature commits that are hard to revert.

---

## 1. Task line format

Each task is a single checklist item with metadata tags.

Canonical shape:

```md
- [ ] <id>: <short description> @area(<area>) @owner(<agent>|unassigned) @status(unclaimed) @scenario(<optional>) @notes(<optional>)
```

When **in progress**:

```md
- [ ] cli-002: implement `context put` command (stub + tests + impl)
      @area(cli) @owner(context-cli-agent) @status(in-progress) @scenario(2025-11-21T04:10:00Z-xyz)
```

When **complete** (after CI + commit):

```md
- [x] cli-002: implement `context put` command (stub + tests + impl)
      @area(cli) @owner(context-cli-agent) @status(done,commit=<bootstrap>) @scenario(2025-11-21T04:10:00Z-xyz)
```

Conventions:

- `id`: stable identifier (`boot-001`, `core-003`, `cli-007`, etc).
- `@area(...)`: one of `boot`, `core`, `cli`, `web`, `obs`, `agents`, `docs`, `infra`.
- `@owner(...)`: agent name (e.g. `context-core-agent`, `context-web-agent`); `unassigned` if nobody owns it yet.
- `@status(...)`: `unclaimed`, `in-progress`, or `done,commit=<hash>`.
- `@scenario(...)`: free‑form scenario ID (usually timestamp + random suffix).

---

## 2. Agent workflow for each task

**Every agent must follow this exact cycle:**

1. **Pick a task**
   - Choose a `- [ ]` task with `@status(unclaimed)`.
   - Do not pick more than **one** task at a time.

2. **Claim it**
   - Edit this file:
     - Set `@owner(<your-agent-name>)`.
     - Set `@status(in-progress)`.
     - Add `@scenario(<your-scenario-id>)` if not present.
   - Save `plan.md`.

3. **Design tests (red)**
   - Identify where tests should live:
     - Rust core → `context-core/tests/…` or module tests.
     - CLI → `context-cli/tests/…` or integration tests that shell out.
     - Web UI → `web-ui` tests.
   - Write/extend tests so they **fail** against current code.

4. **Implement (green)**
   - Modify code just enough to satisfy the tests.
   - Keep scope within the task; avoid sneaking in extra features.

5. **Refactor**
   - Clean up implementation while keeping tests green.
   - Update docs/README only if clearly in scope.

6. **Run full CI locally**
   - Run `make ci` from repo root.
   - If anything fails:
     - Fix the issue.
     - Re‑run `make ci` until it passes.

7. **Commit via runner**
   - Call `./scripts/runner.sh "<id>: <short message> [agent:<name>] [scenario:<id>]"`.
   - If it fails due to lock, wait and retry.

8. **Update PLAN.md**
   - Change `[ ]` → `[x]`.
   - Set `@status(done,commit=<actual-commit-hash>)`.
   - Keep `@owner` and `@scenario` unchanged.

---

## 3. Bootstrapping status (where we are now)

> Note: `commit=<bootstrap>` is a placeholder; once you create the first real
> commit in your own repo, you can either leave these as historical or update
> them with the true commit hash that introduced the scaffold.

### 3.1 Boot / infra

- [x] boot-001: scaffold Rust workspace with `context-core`, `context-cli`, `context-agent`, `context-web`, `context-plan`
      @area(boot) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] boot-002: add top-level Makefile with `build`, `test`, `lint`, `ci`, `web`, `web-dev`, `plan-check`
      @area(boot) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] boot-003: add `.github/workflows/ci.yml` that runs `make ci` on macOS
      @area(infra) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] boot-004: add `.vscode` with baseline extensions and formatter config
      @area(infra) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] boot-005: add `scripts/runner.sh` with git lock + `make ci` + commit + push
      @area(infra) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] boot-006: add `.githooks/pre-commit` that runs `make ci`
      @area(infra) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] plan-001: create PLAN.md with agent/TDD rules and initial tasks
      @area(docs) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] plan-002: add plan validator tool + `make plan-check` target
      @area(infra) @owner(bootstrap) @status(done,commit=<bootstrap>)

### 3.2 Core / CLI / web stubs

- [x] core-001: define `Document` and `Storage` trait in `context-core`
      @area(core) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] cli-001: scaffold `context-cli` with Clap + tracing and full stubbed commands
      @area(cli) @owner(bootstrap) @status(done,commit=<bootstrap>)

- [x] web-001: scaffold `context-web` Axum server with `/healthz` and `/agent-doc` routes
      @area(web) @owner(bootstrap) @status(done,commit=<bootstrap>)

---

## 4. Upcoming work – core & storage

### 4.1 Storage & schema

- [x] core-010: design SQLite schema and migrations for projects, documents, versions, FTS5 index
      @area(core) @owner(context-core-agent) @status(done,commit=3a6154d) @scenario(2025-11-20T20:27:16-08:00)

- [x] core-011: implement SQLite-backed `Storage` (put/get/search) with basic tests
      @area(core) @owner(context-core-agent) @status(done,commit=74ae9c3) @scenario(2025-11-20T20:49:43-08:00)

- [x] core-012: add TTL/soft-delete fields and logic; tests for expired/tombstoned docs
      @area(core) @owner(context-core-agent) @status(done,commit=bd29805) @scenario(2025-11-20T20:59:05-08:00)

- [x] core-013: wire up FTS5 search and ranking by recency and tags
      @area(core) @owner(context-core-agent) @status(done,commit=36e9ef5) @scenario(2025-11-20T21:03:29-08:00)

---

## 5. Upcoming work – CLI commands

### 5.1 Stubbed commands (shape already present in code)

The CLI already exposes stub variants for these commands; they currently just log/print “TODO”. The tasks below are about **tests + real behavior**.

- [x] cli-010: implement `context put` (stdin/file, project/key, tags) with tests
      @area(cli) @owner(context-cli-agent) @status(done,commit=de98b23) @scenario(2025-11-21T04:27:35Z-cli-010)

- [x] cli-011: implement `context get` (key/id, project, format) with tests
      @area(cli) @owner(context-cli-agent) @status(done,commit=0277f68) @scenario(2025-11-21T05:16:09Z-cli-011)

- [ ] cli-012: implement `context cat` (content-only output for agents) with tests
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-013: implement `context find` (search API, JSON output) with tests
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-014: implement `context ls` (list docs) with tests
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-015: implement `context rm` (soft delete) + `context gc` (hard delete/vacuum) with tests
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-016: implement `context web` wrapper around `context-web` server with tests (at least smoke)
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-017: implement `context web-dev` alias/flags for dev web UI
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-018: implement `context debug-bundle` to gather logs + traces into an archive
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-019: implement `context agent-config` to emit Codex/Claude/Copilot agent configs
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-020: flesh out `context agent-doc` to generate full agent usage docs
      @area(cli) @owner(unassigned) @status(unclaimed)

- [ ] cli-021: implement `context project` subcommands (`current`, `set`, `list`) with tests
      @area(cli) @owner(unassigned) @status(unclaimed)

---

## 6. Upcoming work – Web & Dev Web UI

- [ ] web-010: scaffold React + TS + Vite + shadcn web UI in `web-ui/`
      @area(web) @owner(unassigned) @status(unclaimed)

- [ ] web-011: serve compiled `web-ui` assets from `context-web` binary
      @area(web) @owner(unassigned) @status(unclaimed)

- [ ] web-012: implement search + doc viewer in user web UI with tests
      @area(web) @owner(unassigned) @status(unclaimed)

- [ ] web-013: implement TTL/aging view in user web UI
      @area(web) @owner(unassigned) @status(unclaimed)

- [ ] web-020: implement Dev Web UI (logs, spans, scenarios, debug bundles)
      @area(web) @owner(unassigned) @status(unclaimed)

---

## 7. Upcoming work – Observability

- [ ] obs-010: wire `tracing` with JSON logs to file + pretty TTY logs
      @area(obs) @owner(context-devops-agent) @status(in-progress) @scenario(2025-11-21T04:27:39Z-obs010)

- [ ] obs-011: add scenario_id + project + command fields to all CLI/web logs
      @area(obs) @owner(unassigned) @status(unclaimed)

- [ ] obs-012: implement basic span tracing for major operations (put/get/find/web)
      @area(obs) @owner(unassigned) @status(unclaimed)

- [ ] obs-013: implement `context debug-bundle` backend (log collection, trace export)
      @area(obs) @owner(unassigned) @status(unclaimed)

---

## 8. Upcoming work – Agents & docs

- [ ] agents-010: define Codex CLI agents in `AGENTS.context.md` using `context agent-doc`
      @area(agents) @owner(unassigned) @status(unclaimed)

- [ ] agents-011: define Claude Code subagents and commands under `.claude/`
      @area(agents) @owner(unassigned) @status(unclaimed)

- [ ] agents-012: define Copilot CLI agents under `.github/agents/`
      @area(agents) @owner(unassigned) @status(unclaimed)

- [ ] agents-013: add a “debug web” agent that explains how to use Dev Web UI + debug bundles
      @area(agents) @owner(unassigned) @status(unclaimed)

- [ ] agents-014: update agent-prompts.txt with push-capable multi-task instructions
      @area(agents) @owner(context-docs-agent) @status(in-progress) @scenario(2025-11-21T05:25:00Z-agent-prompts)

- [ ] docs-010: write detailed README with install, Quickstart, agents, TDD rules
      @area(docs) @owner(unassigned) @status(unclaimed)

- [ ] docs-011: add CLAUDE.md / AGENTS.md snippets and keep them in sync via `context agent-doc`
      @area(docs) @owner(unassigned) @status(unclaimed)
