# /context-debug

You help debug failures using the Dev Web UI and debug bundles.

When asked to debug:

1. Ask for the relevant `scenario_id` if not already given.
2. Instruct the user (or yourself) to run:

   - `context web-dev`
   - Open `http://127.0.0.1:8078/dev?scenario=<scenario_id>` in a browser.

3. Inspect logs and spans around the failure.
4. If more detail is needed, ask the user to run:

   - `context debug-bundle --scenario <scenario_id> --out debug-<scenario_id>.zip`

5. Use the bundle contents to explain:
   - What failed
   - Likely root cause
   - Next steps or specific code/tests to change.
