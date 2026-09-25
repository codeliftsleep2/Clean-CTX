# Codec harness (A2/A3 reversible wire)

This subdirectory is the **codec regression harness**: it reconstructs the
CONTROL-FULL v2 semantic oracle from code-side `result.ir`, renders the A2/A3
reversible wire (`compact-a2.txt` / `compact-a3.txt`), measures token
economics, and scores **codec preservation/round-trip** reasoning (canonical
IDs, occurrence order, duplicates, unresolved callees, injection occurrences,
byte spans).

The model-visible SCHEMA-v5 presentation is scored separately by `schema-v5/`;
this harness feeds the codec, not the reader.

## Command order

```powershell
cargo build --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Reset.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Prepare-Fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/codec/scripts/Validate-ReasoningDefinitions.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Build-MeasureHelper.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Capture-Baselines.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/codec/scripts/Verify-Captures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/codec/scripts/Measure-Baselines.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/codec/scripts/Prepare-ReasoningWorksheet.ps1
```

## Layout

- `oracles.json` — codec-preservation reasoning oracles (file, workspace, and
  lifecycle lanes).
- `workspace-query-oracles.json` — the workspace_query ops the codec workspace
  lane drives.
- `transport-assertions.json` — edit-focus error transport assertions.
- `audit-v2.json`, `repairs-v2.json`, `run-lineage.json` — historical codec-era
  reasoning artifacts.
- `scripts/` — codec measurement, capture, verification, and reasoning runners.

Shared scripts (`McpSession.ps1`, `measure.rs`, `Build-MeasureHelper.ps1`,
`Prepare-Fixtures.ps1`, `Reset.ps1`, `Capture-Baselines.ps1`,
`Capture-WorkspaceQuery.ps1`) live in `../scripts/`; the shared scoring schema
is `../score.schema.json`.
