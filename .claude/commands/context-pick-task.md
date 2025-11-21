# /context-pick-task

You are a coordinator. Use this command to help an agent pick a task from `plan.md`.

Steps:

1. Open `plan.md`.
2. List unclaimed tasks (`- [ ]` lines with `@status(unclaimed)`).
3. Ask the user which area/agent they want (core/cli/web/obs/agents/docs).
4. Suggest an appropriate task and agent.
5. Show the exact line to add or modify in `plan.md` with:
   - `@owner(<agent-name>)`
   - `@status(in-progress)`
   - `@scenario(<scenario-id>)`.
