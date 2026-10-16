# Residual Non-EF C# Scan Measurement — 2026-10-16

**Investigator:** Agent MaxHeadRoom  
**Baseline:** authoritative release gate GREEN  
**Status:** implementation, adversarial regression, affected verification, and authoritative release gate GREEN  
**Recommendation:** **NO FURTHER DEPENDENCY-GRAPH SCAN OPTIMIZATION JUSTIFIED**

## Inventory

| Consumer | Collection and predicates | Frequency | Existing reference index |
| --- | --- | --- | --- |
| `recognizes_edge` | All references; exact file, owner spelling, written type | Per unresolved neutral edge | Exact mechanical fit for file/owner prefilter |
| `resolve_edge` | All references; exact file, owner spelling, written type, then semantic resolution | Per neutral edge | Exact mechanical fit |
| `written_references_to` | Resolves every reference; retains matching kind and target name | Per reverse classification | Not compatible; target is known only after resolution |
| `base_reference` | All references; file, owner, namespace, nesting | Per inheritance step | Partial fit; namespace/nesting remain semantic |
| `resolve_declaration` | Declarations for each candidate path and arity; rejects ambiguity | Per reference resolution | A separate path/arity prefilter could only narrow |
| `resolve` | Declarations for each candidate path/arity; preserves kind/name ambiguity | Per classified reference | Reference index cannot remove declaration work |
| `unique_named_class` | Class declarations with exact name; requires one | Per ASP.NET refinement | Not applicable |
| `unique_class_declaration_file` | Class declarations with terminal written name; requires one | Per .NET testing candidate | Not applicable |
| `DependencyGraph::build` reference lookup | Per edge, all references for exact file/owner/written target | Per admitted dependency edge | Exact mechanical fit |
| `DependencyGraph::build` target lookup | Linear admitted-class position search after resolution | Per resolved dependency edge | Build-local position map is mechanically possible |

## Dependency Graph Production Path

```text
workspace_query transitive_dependencies / has_cycle
 -> graph preparation
 -> request-local C# Catalog
 -> DependencyGraph::build
 -> scoped WorkspaceIndex edges for each class occurrence
 -> matching dependency edge
 -> global reference lookup
 -> ambiguity-preserving declaration resolution
 -> linear admitted-class target-position lookup
 -> ordered/deduplicated adjacency
```

## Test-Only Counters

- `dependency_graph_edges_considered`
- `dependency_graph_reference_comparisons`
- `csharp_declaration_resolution_comparisons`
- `dependency_graph_target_position_comparisons`

The counters are thread-local, test-only, and observational. They do not
participate in authority, candidate selection, resolution, or results.

## Scaled Characterization

One published C# file contains an unambiguous inheritance chain. The small
fixture has four declarations and three dependency edges. The large fixture
has sixteen declarations and fifteen edges. File count, scope, ambiguity, and
query family remain fixed.

The production `transitive_dependencies` request starts from the final class.
Controls require every preceding identity exactly once. The focused user-run
characterization passed and reported:

| Work | 4 declarations / 3 edges | 16 declarations / 15 edges | Growth |
| --- | ---: | ---: | ---: |
| Catalog builds / parses | 1 / 1 | 1 / 1 | Constant |
| Candidate-index keys / references | 3 / 3 | 15 / 15 | 5x |
| Maximum candidate bucket | 1 | 1 | Constant |
| Admitted dependency edges | 3 | 15 | 5x |
| Global reference comparisons | 6 | 120 | 20x |
| Declaration-resolution comparisons | 12 | 240 | 20x |
| Target-position comparisons | 6 | 128 | 21.3x |

All three residual scan stages amplify much faster than the distinct edge set.
The existing candidate index is already constructed once and proves a stronger
mechanical opportunity for the first stage: every measured file/owner bucket
contains exactly one reference, yet the graph ignores those buckets and scans
the global reference vector for every edge.

## Index Applicability

The existing `(file, owner spelling)` reference-candidate index may safely
prefilter `recognizes_edge`, `resolve_edge`, `base_reference`, and the graph
reference lookup because those consumers already apply those exact predicates.
It must not establish ownership or identity, resolve namespaces/nesting,
collapse duplicates, change order, or replace ambiguity checks.

`written_references_to` is not compatible: resolving all references is how it
discovers which written spellings reach the requested identity.

No declaration index is implemented. Potential mechanical predicates are
exact path plus arity for resolution, kind plus exact name for unique-name
helpers, and occurrence identity to vector position inside one graph build.
Each could only narrow candidates; all current ambiguity logic must remain.

## Preliminary Classification

| Candidate | Classification | Reason |
| --- | --- | --- |
| Graph reference lookup | **PROCEED NEXT** | 6→120 comparisons for 3→15 edges despite existing singleton ordered buckets |
| Graph declaration resolution | **DEFER AFTER FIRST TARGET** | 12→240 is material, but declaration identity/ambiguity risk is higher |
| Graph target-position lookup | **DEFER AFTER FIRST TARGET** | 6→128 is material and mechanical, but do not bundle optimizations |
| `recognizes_edge` / `resolve_edge` | **MEASURE MORE** | Exact fit but current frequency unmeasured |
| `written_references_to` | **REQUIRED LINEAR WORK** provisionally | Every reference is a semantic input; repetition unproven |
| `base_reference` | **MEASURE MORE** | Partial fit; inheritance frequency unmeasured |
| Unique-name helpers | **DEFER** | Correct ambiguity scans; material cost unproven |

## Decision

**PROCEED NEXT — dependency-graph reference lookup only.**

The smallest safe phase replaces only this predicate-equivalent global scan:

```text
catalog.references.iter().find(
  file == declaration.file
  && owner == declaration.name
  && written == edge.object.name
)
```

with iteration over the existing ordered `(file, owner spelling)` candidate
bucket followed by the unchanged written-type predicate. The bucket is
mechanical candidate narrowing only. It must not establish ownership, target
identity, declaration identity, namespace/nesting resolution, or ambiguity.

Expected structural result for this fixture:

```text
reference comparisons: 6 / 120 -> 3 / 15
declaration comparisons: remain 12 / 240
target-position comparisons: remain 6 / 128
edges considered: remain 3 / 15
semantic results, order, and provenance: unchanged
```

Do not combine declaration prefiltering or target-position mapping with this
phase. Their independent measurements remain as evidence for later selection
after the first migration is verified.

The selected production migration is now implemented: `DependencyGraph::build`
uses the existing ordered file/owner candidate bucket and retains the exact
written-type predicate. Declaration resolution and target-position lookup are
unchanged. No new index, authority, cache, generation, or lifecycle exists.

The post-migration focused run produced the exact expected work counts:

| Work | Small after | Large after | Prior small / large |
| --- | ---: | ---: | ---: |
| Reference comparisons | 3 | 15 | 6 / 120 |
| Declaration-resolution comparisons | 12 | 240 | 12 / 240 |
| Target-position comparisons | 6 | 128 | 6 / 128 |
| Edges considered | 3 | 15 | 3 / 15 |
| Candidate-index builds | 1 | 1 | 1 / 1 |
| Maximum candidate bucket | 1 | 1 | 1 / 1 |

This proves the optimization removed only the redundant global reference scan.
Required downstream semantic work was not accidentally optimized away.

## Verification Evidence

```text
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::residual_csharp_dependency_graph_scans_are_structurally_characterized" -- --exact --nocapture
```

The user ran the unchanged focused command GREEN after migration and supplied
the post-migration counts above. The affected C# dependency suite was also
reported GREEN. This reference-bucket phase is complete.

The remaining `6/128` target-position comparisons have now been addressed as a
separate phase. `DependencyGraph::build` creates one ephemeral
`HashMap<*const Declaration, usize>` from the already-ordered admitted
declaration vector. `resolve_declaration` returns a reference into that same
immutable catalog declaration vector, so address equality maps only that exact
existing occurrence to its graph position. It neither hashes copied source text
nor defines or infers semantic identity. The expected structural result is zero
linear position comparisons, `4/16` map entries, and `3/15` exact lookups. The
map dies before its borrowed catalog and never dereferences its raw pointer keys.

Declaration resolution remains unchanged at `12/240` expected comparisons.
No declaration index or declaration-authority change is included.

The user reported both the focused structural regression and affected C#
dependency suite GREEN. The target-position migration therefore preserves
dependency results, traversal depth, cycle behavior, unresolved-target
exclusion, ambiguity refusal, ordering, and provenance.

Verified post-migration structure:

```text
reference comparisons:             3 / 15
linear target-position comparisons: 0 / 0
target-position map entries:        4 / 16
exact target-position lookups:       3 / 15
declaration-resolution comparisons: 12 / 240 (unchanged)
edges considered:                    3 / 15
```

This phase is complete. The remaining declaration-resolution amplification
must receive a separate design and authorization because any candidate
prefilter must preserve path/arity matching and duplicate-declaration
ambiguity exactly; the position map provides no authority for that decision.

The declaration phase is now implemented separately. The request-local C#
catalog owns a mechanical ordered index from exact full declaration path and
arity to every matching declaration position. Index construction iterates the
already sorted and admitted declarations once and appends every position; it
does not deduplicate buckets or select a preferred file.

`resolve_declaration` was relocated intact to the declaration-candidate module
and retains candidate-path generation, ordering, namespace/nesting expansion,
import fallback, arity matching, first-candidate-path precedence, and the
second-match ambiguity refusal. An empty bucket continues to the next existing
candidate path. Shared callers benefit only because they already call this
resolver; no caller was independently redesigned.

Expected scaled result:

```text
declaration comparisons: 12 / 240 -> 3 / 15
declaration index builds: 1 / 1
indexed declaration occurrences: 4 / 16
maximum bucket size in scaled fixture: 1 / 1
reference comparisons: remain 3 / 15
target-position map entries/lookups: remain 4/3 and 16/15
semantic results, ambiguity, ordering, provenance: unchanged
```

The adversarial regression covers one unique declaration, identical path/arity
occurrences in multiple files remaining ambiguous, equal paths with different
arity remaining separate, empty early paths falling through to an imported
path, and namespace/nesting-sensitive resolution. The index is owned and
discarded with its request-local catalog and introduces no separate validity or
authority lifecycle.

The focused post-declaration user-run characterization is GREEN and reports:

| Work | Small | Large | Interpretation |
| --- | ---: | ---: | --- |
| Dependency edges | 3 | 15 | Distinct required inputs |
| Reference comparisons | 3 | 15 | One ordered-bucket inspection per edge |
| Declaration comparisons | 3 | 15 | One exact path/arity candidate per edge |
| Linear target-position comparisons | 0 | 0 | Removed |
| Exact target-position lookups | 3 | 15 | One per resolved edge |
| Reference-index builds | 1 | 1 | Once per catalog |
| Declaration-index builds | 1 | 1 | Once per catalog |
| Indexed declaration occurrences | 4 | 16 | One per admitted declaration |
| Maximum declaration bucket | 1 | 1 | Fixture remains unambiguous |

The original large-fixture decomposition was `120 / 240 / 128` for reference,
declaration, and target-position comparisons. It is now `15 / 15 / 0`, plus
15 exact target-position lookups. Work scales with distinct edges and admitted
declarations rather than repeatedly scanning global vectors.

No further dependency-graph scan optimization is justified by this fixture.
Any future performance work must return to the remaining independently listed
consumers and earn a new target through representative measurement; this graph
result does not authorize migrating `recognizes_edge`, `resolve_edge`,
`base_reference`, unique-name helpers, or `written_references_to`.

## Final Verification

The user reported all gates GREEN, including:

- the exact path/arity declaration-candidate adversarial regression;
- the scaled three-stage dependency-graph work-count regression;
- the affected C# dependency suite;
- the authoritative repository release gate.

The verified implementation preserves unique resolution, duplicate ambiguity,
generic arity separation, candidate-path fallthrough, imports,
namespace/nesting reasoning, multiple-file occurrence preservation, dependency
traversal, cycle behavior, ordering, and provenance. The performance campaign
entered and exits this phase with an authoritative GREEN release baseline.
