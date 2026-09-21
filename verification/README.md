# Verification assets

This tree contains reusable, reviewable definitions for operator-run verification:
fixtures, scenario manifests, oracles, and driver scripts. It does not contain
generated evidence and does not replace tracked regression tests under
`src/tests/**`.

Generated captures, databases, temporary workspaces, compiled helpers, and other
runtime state belong under `target/`. A reported operator PASS is observational
evidence only; the repository's tracked tests and final verification gate remain
authoritative.

Current packages:

- `context-compression/` — CONTROL-PROD/CONTROL-FULL capture, measurement, and
  reasoning-verification definitions;
- `lifecycle/phase9/` — durable-context lifecycle field harness;
- `live-acceptance/` — focused live MCP acceptance drivers.
