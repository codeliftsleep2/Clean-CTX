# Issue #134 LGR-001 Declaration-Proven .NET Test Subjects

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Verdict:** #134 LGR-001 IMPLEMENTATION COMPLETE — VERIFICATION REQUIRED

## 1. Root cause

The .NET testing producer stripped `Tests` from a test-class name and immediately
published `TestClass --Tests--> builtin/Class`. WorkspaceIndex correctly registered
that relationship endpoint as an occurrence, but no authoritative Class declaration
was required. Candidate evidence therefore became durable semantic truth.

## 2. Files changed

- `src/dotnet_meta/testing.rs`: separates TestClass declaration publication from
  candidate collection; structured persistent construction outranks convention;
  multiple structured candidates fail closed.
- `src/dotnet_meta/mod.rs`: publishes TestClass self-`Defines`, removes durable
  candidate `Tests`, and increments .NET semantic generation 1 -> 2.
- `src/mcp/tool_handlers/query/{dotnet_testing,edges}.rs` and `query.rs`: staged
  target hydration and ephemeral declaration-proven forward/reverse projection.
- `src/mcp/tool_handlers/query/csharp_resolution.rs`: exposes a conservative unique
  Class-declaration provenance lookup while preserving existing resolution behavior.
- `src/mcp/compatibility/mod.rs` and
  `src/tests/mcp/dotnet_test_subject_compatibility.rs`: generation compatibility.
- `src/tests/mcp/workspace_query_dotnet_test_subjects.rs` and registration:
  production/public-path regression matrix.
- `src/tests/dotnet_meta/testing.rs`: producer, precedence, opt-out, and durable-
  declaration guards updated to the new authority boundary.

## 3. Repaired authority chain

```text
structured TestClass declaration
  -> durable dotnet/TestClass self-Defines declaration only
  -> collect persistent constructed-SUT or suffix candidate
  -> structured candidate wins; multiple structured candidates fail closed
  -> staged scoped hydration of the candidate name
  -> exact C# Class declaration catalog + WorkspaceIndex declaration authority
  -> require one admitted declaration and preserve its file provenance
  -> request-local TestClass --Tests--> builtin/Class
```

Relationship endpoints never participate in the declaration check. CBM/provider
semantics are not consumed. The same ephemeral catalog serves forward TestClass and
reverse Class queries.

## 4. Durable versus ephemeral decision

Candidate `Tests` edges are no longer durable. Durable .NET output contains only the
source-proven TestClass declaration carrier. The cross-file relationship is
ephemeral because its authority spans the test source, target declaration, active
scope, and current hydration result. This reuses file-owned declaration lifecycle
and avoids inventing cross-file invalidation ownership.

The edge is produced only from current cached sources with exact declaration
authority. Reverse filesystem fallback may return declaration-proven edges while
retaining lower-bound coverage when inbound discovery cannot establish
completeness; it never promotes an occurrence or claims complete results. Target
removal or replacement recomputes from current declarations and cannot leave a
stored stale edge.

## 5. Generation decision

`dotnet_meta::SEMANTIC_PRODUCER_GENERATION` advances from 1 to 2. Identical source
now emits a TestClass self-`Defines` declaration instead of an unverified durable
`Tests` edge, so durable semantic compatibility must change. Marker generation
remains 1; canonical producer/configuration identities are unchanged.

## 6. Tracked regressions

The new public-path suite covers:

- GhostTests without a declaration;
- a Class relationship endpoint without a declaration;
- conventional exact target and exact target provenance;
- differently named persistent construction;
- structured construction conflicting with the suffix;
- ambiguous same-terminal declarations;
- multiple persistent constructed candidates;
- unsupported transient/local construction;
- repeated query deduplication;
- target replacement/retraction;
- reverse-query parity.

Compatibility coverage proves generation-1 .NET semantic snapshots differ from the
new producer while canonical and configuration identities remain unchanged.

## 7. Preservation guards

- Test markers and Moq/setup/verification rendering remain in the .NET producer.
- Testing configuration opt-out suppresses both markers and TestClass declarations.
- Conventional relationships still resolve when exactly one Class declaration
  exists.
- Differently named persistent constructed subjects resolve and outrank convention.
- Non-test .NET semantic extraction is unchanged.
- Forward and reverse public queries use one declaration-proven catalog.
- Repeated publication/query returns one current relationship.

## 8. Negative guards

- no declaration: no edge;
- endpoint occurrence only: no edge;
- ambiguous terminal identity: no edge;
- multiple persistent constructed candidates: no edge, even if a suffix exists;
- transient/local construction: not structured candidate authority;
- incomplete inbound discovery: only currently cached declaration-proven edges,
  with lower-bound coverage and possible omissions reported;
- removed/replaced target: no stale edge;
- convention alone never bypasses the exact declaration requirement.

## 9. Commands run

Only bounded read-only inspection, patching, and static diff inspection were used:

```text
Get-Content <targeted policy/source/test files and ranges>
rg -n <targeted symbols> <scoped paths>
git status --short
git diff --check
git diff / git diff --stat
```

The patch utility was invoked through its executable in the approved shell because
the Windows restricted-token sandbox could not create workspace files.

## 10. Commands intentionally not run

No Cargo test/check/build/Clippy/fmt command, test binary, MCP server, live script,
benchmark, CBM/provider call, or CI-equivalent gate was run. No RED/GREEN runtime
claim is made.

## 11. Exact bounded verification recommended

Run the focused tracked coverage first:

```powershell
cargo test --all-features dotnet_test_subject -- --nocapture
cargo test --all-features dotnet_meta::testing::tests -- --nocapture
```

Then rerun the focused operator gate (separate from Rust test evidence):

```powershell
pwsh -NoProfile -File .\verification\issue-reconciliation\Invoke-NativeCausalControls.ps1 `
  -BinaryPath .\target\debug\clean-ctx.exe
```

Inspect case `134 / test-target-must-exist`. It must prove both no Ghost target and
the differently named positive control. After focused GREEN, run the applicable
user-owned final verification commands from `docs/agent/verification.md`.

## 12. Completion pending verification

Implementation and tracked coverage are complete for LGR-001 only. Static diff
inspection is complete. Runtime correctness, zero-warning compilation, the focused
live assertion, and the final gate remain explicitly unverified and user-owned.

LGR-004, EF configuration semantics, C# dependency traversal/cycles, #135, and #142
were not implemented.
