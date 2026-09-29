# CBM live verification

This package contains repeatable operator investigations for behavior owned by
the installed `codebase-memory-mcp` process and observed through Clean-CTX's
registered MCP boundary.

The scripts are tracked verification assets, not Rust regression tests or CI
gates. Generated evidence belongs under `target/`.

## Duplicate bare-name trace verification

After building `target/debug/clean-ctx.exe`, run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/cbm/scripts/Investigate-DuplicateTraceNames.ps1
```

The script creates and indexes a temporary Rust repository containing two
functions with the same bare name. It verifies that both `graph_trace` and
`cbm_proxy(trace_path)` reject the bare identity with `-32602` and the same two
canonical candidates, while canonical inputs retain their direct trace paths.
Generated response evidence is written under `target/`; this field harness does
not replace the tracked Rust regression.
