# Binary0x04 production edge-case operator verification

This package drives a freshly built Clean-CTX binary over the tracked
TypeScript/Angular and C# fixtures. It is operator evidence, not a replacement
for the tracked tests under `src/tests/**`.

```powershell
cargo build --locked --all-features --bin clean-ctx -j 4
pwsh -NoProfile -File .\verification\context-compression\scripts\Build-MeasureHelper.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\edge-cases\scripts\Capture-EdgeCases.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\edge-cases\scripts\Verify-EdgeCases.ps1
```

Python 3 is required for the read-only SQLite exporter, following the existing
Phase 9 verification pattern. PowerShell discovers `python3`, `python`, or `py`;
pass `-PythonPath` if necessary. The helper build uses the isolated helper's
locked dependency graph and an offline cache; provision its dependencies with
`cargo fetch --locked --manifest-path verification/context-compression/measure-helper/Cargo.toml`
if that cache is empty. Both builder and capture support Linux and Windows
executable names. `-BinaryPath` and `-HelperPath` select explicit built artifacts.

Capture enables persistence in a fresh isolated runtime, requests Edit context
through production MCP, and reads the resulting physical `0x04` baseline and
semantic-edge snapshots from SQLite. The existing measurement helper uses the
production binary decoder to select the snapshot matching the IR version and
validates durable owner identity, source hash alignment, and canonical round-trip
equality. It regenerates normalized `CONTROL-FULL` as an operator semantic oracle
only. Reduced `result.ir`, stored pretty text, and model-visible text are not used
as reversible state.

The model-facing `content` is captured separately, checked for codec-boundary
violations, and checked against exact source when it declares raw passthrough.
A fresh server session restores the same durable context and semantic state.
The verifier retains the Angular arrow identity/nested-call/spread/DI and C#
overload/lambda/routing checks, with exact source-byte checks for C# body spans.

Generated evidence stays under `target/context-compression-verification/`.
A completed capture manifest hashes the response, restore response, binary,
snapshot, and regenerated oracle files. The verifier refuses incomplete or
changed captures rather than accepting stale evidence. Runtime directories are
retained for source-span and database inspection.

COMPACT-A1/A3 remain research codecs; their measurement scripts are not part of
this durable production verification. Java/Spring remains outside this focused
operator pass. No full suite runs here, and a harness result does not replace
tracked regression tests or the CI gate.
