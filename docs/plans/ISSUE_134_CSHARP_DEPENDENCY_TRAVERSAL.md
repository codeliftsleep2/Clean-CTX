# Issue #134  Exact C# Dependency Traversal and Constructor Cycles

**Agent:** Agent MaxHeadRoom  
**Status:** Implemented; focused local and native acceptance green  
**Date:** 2026-10-15

## Outcome

`workspace_query` now supplies a request-local, declaration-proven C# dependency
adjacency to `transitive_dependencies` and `has_cycle`. The durable workspace
index and its global relation allowlists are unchanged.

The refinement accepts only exact C# class relationships supported by the
scoped source catalog:

- already-resolved `Extends` relationships;
- written `HasBaseType` references resolving to one class declaration; and
- written `HasConstructorParameterType` references resolving to one class
  declaration.

Unresolved, ambiguous, interface, and unrelated structural endpoints do not
enter the exact adjacency. Traversal continues to use the shared BFS depth
contract, while cycle witnesses retain ordered edge relation and asserting-file
provenance.

## Production integration

The integration path is:

1. structured C# declarations and written type facts are published normally;
2. the scoped C# catalog reads session-owned source snapshots;
3. `DependencyGraph` joins each admitted fact to an exact declaration;
4. `transitive_dependencies` uses the request-local adjacency for an exact
   `builtin/Class` request and otherwise preserves the generic index path;
5. `has_cycle` preserves the existing generic witness first, then uses an exact
   C# witness when the generic relation policy has none; and
6. the existing response envelopes expose depth, coverage, cycle ordering, and
   provenance unchanged.

No resolved dependency edge is persisted, so no semantic generation bump or
cross-file invalidation policy was introduced.

## Tracked regression evidence

The registered test module
`src/tests/mcp/workspace_query_csharp_dependencies.rs` covers:

- the `A -> B -> C -> D` depth matrix for depths 1, 2, 3, and 0;
- exclusion of unresolved and ambiguous targets; and
- a deterministic closed `P -> Q -> P` constructor-cycle witness.

User-run results reported green for:

- `workspace_query_csharp_dependencies`;
- `workspace_query_cycle_witness`;
- `workspace_query_scope_traversal`;
- `workspace_query_within_path_traversal`;
- the adjacent `workspace_query_csharp_` regression batch; and
- `cargo clippy --all-targets --all-features -- -D warnings`.

These are tracked Rust-test and lint results, distinct from the live evidence
below.

## Native acceptance evidence

The provider-disabled debug-binary case
`134 / exact-csharp-dependency-depths-and-constructor-cycle` passed in:

`target/issue-reconciliation/20261015-105307-a7ec2d71/`

`134-csharp-dependencies.json` records:

- depth 1: `B`;
- depth 2: `B, C`;
- depths 3 and 0: `B, C, D`; and
- a two-edge closed `P/Q` witness whose relations are both
  `HasConstructorParameterType` and whose asserting file is
  `DependencyGraph.cs`.

The run also retained the previously known, unrelated #139 failure. The initial
artifact contains two identical PASS verdicts for this #134 case because the
harness block had been duplicated; the reusable harness was deduplicated after
the run and its PowerShell syntax was revalidated.

## Remaining verification

This slice is not a claim that all of Issue #134 or the repository-wide Final
Verification Gate is complete. Finalization still follows
`docs/agent/verification.md`.
