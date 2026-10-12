# Angular template live MCP acceptance

This tracked operator harness exercises the completed Angular Template
Correctness and Linkage slice through the real Clean-CTX binary over MCP stdio.
It is field evidence, not a substitute for the Rust regressions under
`src/tests/**`.

## Coverage

The single consolidated run checks:

- decorator-owned arbitrary external template filenames;
- conventional Angular templates and unrelated generic HTML;
- Angular 15–16 legacy, Angular 17+ modern, and mixed control flow;
- complete modern else-if/let/track/empty/switch/defer companion delivery at
  Medium and High, accepting byte-exact fallback when it preserves source;
- Low, Medium, and verbatim fidelity truthfulness;
- canonical external and deterministic inline Template identity;
- forward and reverse `HasTemplate`;
- repeated selector occurrences, UTF-8 byte spans, and control ancestry;
- additive neutral `UsesSelector` and unique `UsesComponent`;
- unresolved, ambiguous, and missing-template diagnostics;
- exact capability versus source completeness;
- reverse-Component lower-bound coverage;
- refresh-driven replacement, selector change, deletion, and restoration;
- republish/query stability and duplicate prevention.

The harness creates an isolated temporary workspace and removes it after the MCP
process exits. Generated responses are written to:

`target/live-acceptance/angular-template-linkage/responses.json`

## Run

From the repository root, after the authoritative local test gate is GREEN:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/angular_template_linkage_live_acceptance.mjs
```

To select an explicit binary:

```powershell
node verification/live-acceptance/angular_template_linkage_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

The driver selects the newest debug/release binary by default and refuses to
report a clean pass when that binary predates any relevant Angular production
source marker. A successful run ends with:

`ALL ANGULAR TEMPLATE LIVE MCP CHECKS PASSED`

The temporary native fixture disables CBM explicitly, so template acceptance
does not silently admit an unrelated provider binary from the operator PATH.
