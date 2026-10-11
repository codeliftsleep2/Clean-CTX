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

## Pinned upgrade admission acceptance

[A3 prepared live cases](A3_ACCEPTANCE.md) describe the bounded source/artifact,
startup, shared-session and refusal checks. Linux results and declared limits are
recorded in the [A5 acceptance report](../../docs/plans/CBM_A5_ENVIRONMENT_RECOVERY_AND_ACCEPTANCE.md).

The tracked PowerShell harness uses the existing stdio-session pattern and Linux
strace to observe the actual provider admission pipes. It requires a writable
normal account cache, the frozen provider artifact, and a current Clean-CTX
binary. Obtain execution authorization under AGENTS.md before running:

```powershell
pwsh -NoProfile -File ./verification/cbm/scripts/Invoke-A5BootstrapAcceptance.ps1 -BinaryPath ./target/debug/clean-ctx -ContinueMatrix
```

Without `-ContinueMatrix` it captures only bootstrap/native availability. With
the switch it adds compatible frontends, registered project routing, retirement,
one safe recovery and controlled admission exit. Verdicts require inspecting the
captured traces; a successful script exit alone does not establish all A5 claims.
Mutation uncertainty and incompatible cohort stimulation are intentionally absent.

## Phase B typed/raw live acceptance

The [B1–B4 coverage matrix](PHASE_B_ACCEPTANCE.md) distinguishes unit/module
contracts from the live model-facing boundary. The portable PowerShell driver
creates owned TypeScript/C# fixtures and primary/additional Git roots, then
exposes real registered tools through stdin/stdout MCP sessions. It follows the
same ProcessStartInfo/stdio pattern as the existing verification scripts.
It does not invoke Codex, build binaries or run test suites.

Use a current all-feature Clean-CTX binary and the pinned provider artifact.
Pass the independently verified SHA256 for your platform; the frozen Linux A5
artifact hash is `6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0`.
A hash pins the artifact, not source provenance by itself; retain build evidence.

```powershell
pwsh -NoProfile -File ./verification/cbm/scripts/Invoke-PhaseBAcceptance.ps1 -BinaryPath ./target/debug/clean-ctx.exe -ProviderPath 'C:\path\to\codebase-memory-mcp.exe' -ExpectedProviderHash '<verified SHA256>'
```

On Linux use the corresponding binary paths without `.exe`. Evidence, requests,
responses, stderr, fixture manifests, provenance and per-phase verdicts are saved
under `target/cbm-phase-b/`. There is no private CBM cache override or daemon kill.
Only generated fixtures are indexed. Live execution requires the repository's
runtime authorization; the standing focused-check authorization does not itself
claim a successful live run. Scripts are prepared acceptance assets until run.
