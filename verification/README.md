# Verification assets

This tree contains reusable, reviewable definitions for operator-run verification:
fixtures, scenario manifests, oracles, and driver scripts. It does not contain
generated evidence and does not replace tracked regression tests under
`src/tests/**`.

Generated captures, databases, temporary workspaces, compiled helpers, and other
runtime state belong under `target/`. A reported operator PASS is observational
evidence only; the repository's tracked tests and final verification gate remain
authoritative.

Confirmed failures found during the current Windows gate sequence are recorded
in [`LIVE_GATE_REGRESSIONS.md`](LIVE_GATE_REGRESSIONS.md) until they are repaired
and protected by tracked tests.

Current packages:

- `invocation-evidence/` — live MCP checks for TypeScript/C# canonical calls,
  structural RxJS pipe extraction, fidelity/configuration and source freshness;

- `context-compression/` — historical presentation and retired-codec research
  records. These are not production live gates and must not be added to a
  Binary `0x04` or MCP acceptance sequence. CONTROL-FULL is only a regenerated
  semantic oracle; the A1/A3 implementations have been retired;
- `workspace-query/` — registered workspace graph-query capture and model-
  visibility diagnostics;
- `lifecycle/phase9/` — durable-context lifecycle field harness;
- `live-acceptance/` — focused live MCP acceptance drivers, including the comprehensive Angular template ownership/linkage pilot documented in `live-acceptance/ANGULAR_TEMPLATE_README.md`.

Reusable drivers in this tree are tracked verification assets. Generated
captures and runtime state are written beneath `target/` and remain untracked.
