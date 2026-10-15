# Issue #134 LGR-004 Exact ASP.NET Ancestry

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Verdict:** #134 LGR-004 CHECKPOINT VERIFIED

## Scope

This slice implements LGR-004 only. It does not implement generic C# dependency
traversal/cycles, EF configuration registration, #133, #135, or #142.

## Root cause

ASP.NET classification was source-local. A class was admitted only when its own
capture contained an ASP.NET attribute or a direct textual `ControllerBase` /
`Controller` base. Project-owned base chains therefore lost framework meaning.

## Repaired authority chain

```text
candidate Class declaration
  -> structured first base reference
  -> staged exact-name hydration
  -> namespace/nesting/import-resolved project Class declaration
  -> repeat with visited exact declaration identities
  -> qualified or import-resolved Microsoft.AspNetCore.Mvc endpoint
  -> reuse the original class's ASP.NET route/action extraction
  -> request-local Controller occurrence and route/action edges
```

An unqualified framework terminal is admitted through the MVC import only after
exact project resolution finds no shadowing declaration. Missing or ambiguous
declarations, interfaces, cycles, incomplete hydration, and unrelated endpoints
fail closed.

## Production changes

- `src/dotnet_meta/aspnet.rs` exposes the existing route/action analysis behind an
  internal authority-proven entry point; direct-controller behavior is unchanged.
- `src/mcp/tool_handlers/query/csharp_resolution.rs` owns exact ancestry walking,
  import provenance, project-shadowing precedence, ambiguity, and cycle handling.
- `src/mcp/tool_handlers/query/entities.rs` stages base discovery and projects the
  indirect Controller occurrence for `find_entities`.
- `src/mcp/tool_handlers/query/edges.rs` stages the same ancestry and projects the
  original endpoint's `HasRoute` and `ControllerAction` edges.
- No durable semantic output changes, so the .NET semantic producer generation
  remains unchanged.

## Regression coverage

`src/tests/mcp/workspace_query_dotnet_aspnet_ancestry.rs` covers:

- two project bases below the framework base;
- Controller, route, and complete action projection;
- project-owned `ControllerBase` shadowing an MVC import;
- missing and ambiguous hops;
- ancestry cycles;
- namespace/import selection amid terminal-name collisions;
- target replacement and projection retraction.

Existing direct ASP.NET tests remain the preservation authority for source-local
controllers.

## Focused live gate

The existing `134 / controller-two-project-bases-below-framework` case now asserts
the derived Controller plus the original endpoint's route and action in one native
session.

After rebuilding the debug binary, run:

```powershell
pwsh -NoProfile -File .\verification\issue-reconciliation\Invoke-NativeIssueReconciliation.ps1 `
  -BinaryPath .\target\debug\clean-ctx.exe
```

## Verification results

The maintainer reported GREEN for:

- `cargo test --all-features workspace_query_dotnet_aspnet_ancestry -- --nocapture`;
- direct ASP.NET preservation tests;
- .NET semantic preservation tests;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- the rebuilt debug-binary live case
  `134 / controller-two-project-bases-below-framework`.

Live evidence is under
`target/issue-reconciliation/20261015-095730-ea6aecbf`. It records binary SHA-256
`465F0E81E32F8FB3B5DF9E6FFD3F35FEDA525CE3B004BE238E9096287E9A3E34` and proves
one exact `dotnet/Controller` occurrence, one `HasRoute -> orders`, and one
`ControllerAction -> GetOne`, all with `OrdersEndpoint.cs` provenance.

The same multi-issue harness reports an unrelated #139 failure for duplicate
Service declaration selection. That failure is not #134/LGR-004 evidence and is
not included in this slice.

The agent itself ran only bounded source inspection, patching, UTF-8/file-size
checks, static Git diff inspection, and read-only evidence inspection. User-run
tracked tests, Clippy, build, and live evidence remain reported separately.
