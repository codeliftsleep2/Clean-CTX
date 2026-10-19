# C# Semantic Resolution R9 ASP.NET Base Convergence

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Phase:** Implementation complete; operator-run deterministic and live MCP verification GREEN

## Result

R9 migrates ASP.NET Core `Controller` and `ControllerBase` base-type authority
to the shared C# resolution boundary. Explicit, `global::`, ordinary-import,
namespace-alias, type-alias, partial, and admitted-source indirect ancestry
forms converge. Local shadows, competing imports, cycles, missing targets, and
other non-success dispositions fail closed.

Resolved ancestry now publishes the durable `dotnet/Controller` role and
authorizes existing route/action payload extraction. Generation reconciliation
rebuilds those facts from the current admitted-source universe, so upstream
base removal/restoration and owner deletion retract or restore the complete
controller projection.

## Production changes

- Added neutral known-external identities for
  `Microsoft.AspNetCore.Mvc.Controller` and `ControllerBase`, both arity zero.
- Added resolved controller ancestry classification and owner-attributed role,
  `HasRoute`, and `ControllerAction` projection.
- Added generation reconciliation for all controller facts.
- Removed the separate request-local ASP.NET ancestry resolver and its query
  projection/filtering authority.
- Prevented written `Controller`/`ControllerBase` recognition from producing
  semantic edges; it remains marker/candidate behavior only.
- Preserved direct `[ApiController]` and `[Controller]` authority behind an
  explicit temporary attribute-only function for the R11 migration.
- Bumped the .NET semantic producer generation because identical source now
  uses a different controller authority proof.

## Regression coverage

Tracked R9 tests cover both MVC base identities, full/global qualification,
ordinary imports, namespace and type aliases, indirect ancestry, partial
composition, local shadowing, competing imports, cross-file reconciliation,
base removal/restoration, and owner deletion.

The partial-symbol regression additionally proves that one composed partial
controller publishes one durable role while route/action payload from another
physical part remains available. The operator reported this regression and the
focused `r9_` gate GREEN.

## Static retirement audit

- `src/dotnet_meta/semantic.rs` is the sole base-derived controller classifier.
- `src/mcp/semantic_compilation/artifact.rs` retires and rebuilds controller
  facts at generation boundaries.
- The former request-local ASP.NET projection module and query overlays were
  removed.
- `src/dotnet_meta/aspnet.rs` may recognize written controller bases only to
  render legacy markers; base-text recognition clears semantic edges.
- `src/dotnet_meta/detect.rs` remains candidate scheduling only, and resolved
  roles bypass detector misses.
- Direct controller attributes remain authority intentionally and explicitly;
  R11 will migrate them through the shared attribute resolver.

No remaining production predicate was found that can publish base-derived
controller identity, route facts, or action facts from unresolved written base
spelling.

## Scope held

- R0-R8 and Issue #148 are unchanged.
- R10 attribute foundation/MSTest and R11 ASP.NET attributes were not started.
- Route and HTTP-verb payload parsing remains behind authorized controller
  identity; attribute identity itself remains the approved R11 boundary.
- No provider, completeness, or public response contract changed.

## Verification status

Agent MaxHeadRoom ran no Cargo command, test, build, Clippy, binary, MCP
runtime, or server. The operator reported the focused R9 and preservation
gates GREEN. Two CBM recovery tests that failed under the complete parallel run
were GREEN when rerun sequentially; they are outside the R9 semantic path.

A reusable production-stdio acceptance harness now lives at
`verification/issue-reconciliation/Invoke-CSharpR9Acceptance.ps1`. It covers
supported base spellings, fail-closed controls, history independence,
cross-file lifecycle, deletion, neutral file truth, the temporary R11
attribute-only boundary, and an optional ready-but-incomplete provider view.
The harness builds nothing and retains evidence under
`target/csharp-r9-live-verification/`.

The operator reported every R9 live scenario PASS against the current binary,
including the registered production boundary, complete positive/negative
matrix, history and warm independence, neutral-file and R11-boundary controls,
ancestry removal/restoration and deletion, and the ready-provider plausible
incomplete snapshot. Evidence is retained at
`target/csharp-r9-live-verification/20261019-174553-b3324525`; the combined
partial-regression/live console record is
`target/r9-partial-role-verification.log`.

Tracked Rust regression results and live MCP acceptance remain distinct
evidence categories. Both are GREEN. Claude field testing remains deferred to
the consolidated campaign field gate.
