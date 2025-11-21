# /context-apply

Use this when you already know the change you want to make and need to implement it with TDD.

Steps:

1. Identify the relevant task ID in `plan.md`.
2. Write or update tests FIRST so they fail.
3. Implement code changes until tests pass.
4. Run `make ci`.
5. Call `./scripts/runner.sh "<id>: <short message> [agent:<name>]"`.
6. Update `plan.md` to mark the task done with `@status(done,commit=<hash>)`.

You must never commit without `make ci` passing.
