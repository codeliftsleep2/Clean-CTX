# CBM live verification

This package contains repeatable operator investigations for behavior owned by
the installed `codebase-memory-mcp` process and observed through Clean-CTX's
registered MCP boundary.

The scripts are tracked verification assets, not Rust regression tests or CI
gates. Generated evidence belongs under `target/`.

## Duplicate bare-name trace verification

On Windows, using the current Clean-CTX and installed CBM executables, run:

```powershell
pwsh -NoProfile -File ./verification/cbm/scripts/Investigate-DuplicateTraceNames.ps1 `
  -BinaryPath ./target/release/clean-ctx.exe `
  -ProviderPath "$env:LOCALAPPDATA\Programs\codebase-memory-mcp\codebase-memory-mcp.exe"
```

The script creates and indexes a temporary Rust repository containing two
functions with the same bare name. It verifies that both `graph_trace` and
`cbm_proxy(trace_path)` reject the bare identity with `-32602` and the same two
canonical candidates, while canonical inputs retain their direct trace paths.
Generated response evidence is written under `target/`; this field harness does
not replace the tracked Rust regression. The harness runs one Clean-CTX MCP
session, records the provider hash as evidence, and tolerates only the exact
temporary auto-index conflict before its explicit fast index.

## Windows provider admission and lifecycle acceptance

[A3 prepared live cases](A3_ACCEPTANCE.md) describe the bounded artifact,
startup, shared-session, recovery and refusal checks. Historical Linux results
and their declared limits remain recorded in the
[A5 acceptance report](../../docs/plans/CBM_A5_ENVIRONMENT_RECOVERY_AND_ACCEPTANCE.md).

The tracked PowerShell harness uses the existing stdio-session pattern and
Windows parent-process observations. It requires the installed Windows provider
and a current Clean-CTX binary. Executable hashes are captured as evidence; the
operator does not supply or approve a hash. Obtain execution authorization under
AGENTS.md before running:

```powershell
pwsh -NoProfile -File ./verification/cbm/scripts/Invoke-A5BootstrapAcceptance.ps1 `
  -BinaryPath ./target/debug/clean-ctx.exe `
  -ProviderPath "$env:LOCALAPPDATA\Programs\codebase-memory-mcp\codebase-memory-mcp.exe" `
  -ContinueMatrix
```

Without `-ContinueMatrix` it captures only bootstrap/native availability. With
the switch it adds compatible frontends, registered project routing, retirement,
one safe recovery and controlled admission exit. Process snapshots, MCP frames,
stderr and provenance are stored under `target/cbm-a5/`. Mutation uncertainty
and incompatible cohort stimulation are intentionally absent.

## Phase B typed/raw live acceptance

The [B1–B4 coverage matrix](PHASE_B_ACCEPTANCE.md) distinguishes unit/module
contracts from the live model-facing boundary. The Windows PowerShell driver
creates owned TypeScript/C# fixtures and primary/additional Git roots, then
exposes real registered tools through one Clean-CTX stdin/stdout MCP session.
Raw provider-shaped observations use registered `cbm_proxy` calls through that
same session; the driver never launches an independent CBM frontend.
It does not invoke Codex, build binaries or run test suites.

Windows is the primary Phase B acceptance platform. The provider may begin
auto-indexing the process working directory before the driver's explicit fast
index request, which is especially observable on Windows. The driver waits
within the existing per-case deadline only for the provider's exact
different-options conflict, then retries the explicit request. Other provider
errors still fail immediately and every attempt remains in the evidence.

Use a current all-feature Clean-CTX binary and the provider executable intended
for Windows. The harness calculates and records both executable hashes as run
evidence; the operator never supplies a hash.

```powershell
pwsh -NoProfile -File ./verification/cbm/scripts/Invoke-PhaseBAcceptance.ps1 -BinaryPath ./target/debug/clean-ctx.exe -ProviderPath 'C:\path\to\codebase-memory-mcp.exe'
```

Evidence, requests, responses, stderr, fixture manifests, provenance and
per-phase verdicts are saved
under `target/cbm-phase-b/`. There is no private CBM cache override or daemon kill.
Only generated fixtures are indexed. Live execution requires the repository's
runtime authorization; the standing focused-check authorization does not itself
claim a successful live run. Scripts are prepared acceptance assets until run.
