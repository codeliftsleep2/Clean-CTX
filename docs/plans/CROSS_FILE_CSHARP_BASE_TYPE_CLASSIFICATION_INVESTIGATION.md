# Cross-File C# Base-Type Classification Investigation

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-08  
**Status:** Implemented; focused regression suite GREEN on 2026-10-09; final verification pending

## Question

How should Clean-CTX classify a neutral cross-file C# class base-list fact as Extends or Implements without creating stale edges, order-dependent false claims, or incorrect Struct/Record behavior?

## Finding

The reported gap is real. v0.9.1 correctly preserves an ambiguous first C# class base-list entry as Class --HasBaseType--> TypeRef, but the workspace query layer never refines that neutral fact using declarations already present in the WorkspaceIndex.

The report is slightly broader than the implementation gap:

- C# interface inheritance is already emitted directly as InterfaceExtends and projects to typed Extends edges, including cross-file symbolic targets.
- Every class base-list entry after the first is already emitted as Implements.
- The unresolved case is the first entry of a C# class base list, because C# syntax alone does not say whether it is a base class or the first implemented interface.

## Verified production behavior

### Same-file refinement is canonical and intentionally local

src/ir/compiler_methods.rs builds separate class and interface name maps from one file's complete canonical instruction stream. A BaseTypeRef becomes Extends only when that same file contains a unique matching class and no matching interface; it becomes Implements only for the inverse case. Ambiguous and absent names remain BaseTypeRef.

docs/architecture/CORE_OP_CONTRACT_MATRIX.md states that cross-file canonical refinement is undefined.

### v0.9.1 preserves the unresolved source fact

src/ir/semantic_projection.rs projects every surviving Class-owned BaseTypeRef to HasBaseType with a TypeRef target. That is the authoritative v0.9.1 behavior and must remain additive.

### The index cannot safely persist a derived cross-file edge today

WorkspaceIndex owns every stored edge occurrence by one asserting file. replace_semantic_projection removes and replaces only that file's facts; remove_file performs exact file-local cleanup.

A typed edge derived from both Child.cs and Target.cs has no correct single-file owner under this model:

- owned by Child.cs, it becomes stale when Target.cs changes kind or disappears;
- owned by Target.cs, it becomes stale when Child.cs changes or disappears;
- duplicated under both, removal can leave half of the derived authority behind.

No dependency table records which target declaration justified refinement, and no lifecycle hook recomputes dependent child files. Persisting the typed edge would therefore require a new cross-file derivation ownership and invalidation architecture.

### Reverse hydration already discovers both sides

Filesystem hydration scans supported source files and retains candidates whose text contains the queried name. A reverse query for IFoo therefore discovers both IFoo.cs and FooCross.cs in the normal case. After hydration, the index contains:

- the Interface/IFoo declaration occurrence;
- FooCross --HasBaseType--> TypeRef/IFoo.

The query-time evaluator receives the hydrated WorkspaceIndex, so it has enough current indexed evidence to derive the typed reverse answer without persisting it.

### Forward hydration does not guarantee the target

A forward query for FooCross discovers source files containing FooCross. It does not necessarily compile IFoo.cs or BaseX.cs. The target may already be indexed, but that depends on earlier publication or queries.

Consequently, query-time classification can be complete for the ordinary reverse query while a cold forward query may truthfully retain only HasBaseType. Eliminating this asymmetry requires a second hydration stage or a broader discovery contract.

### Existing coverage metadata is intentionally incomplete

Every exact edge answer reports source_complete false and lower-bound semantics. Capability evidence is family-level, not exact-identity-level. An unrelated typed edge can establish a family while the requested cross-file relationship remains unclassified.

The Interface-only alternative query is also incomplete: Class reverse queries need the same TypeRef/HasBaseType fallback guidance.

## Interaction with Struct and Record support

The new Struct/Record projection also emits HasBaseType to TypeRef. A generic query-time classifier that consumes every HasBaseType edge would incorrectly reinterpret those facts.

Any class-focused refinement must require subject.entity_type == Class.

Do not infer typed Record relationships from the current neutral facts. Record identity does not retain whether the source declaration was record class or record struct, and the neutral edges do not retain base-list ordinal. That is insufficient evidence for correct classification.

Struct syntax permits only implemented interfaces, so a future Struct-specific Implements refinement may be possible, but it is a separate explicitly scoped behavior change and is not required to close the v0.9.1 Class gap.

## Smallest correct implementation boundary

Add one read-time C# class-base refinement helper at the workspace-query boundary. It should never mutate WorkspaceIndex.

For a neutral edge whose subject is builtin/Class and object is builtin/TypeRef:

1. Look for admitted builtin/Class and builtin/Interface declaration occurrences with the written target name in the effective WorkspaceScope.
2. If Class exists and Interface does not, derive Extends to builtin/Class.
3. If Interface exists and Class does not, derive Implements to builtin/Interface.
4. If both or neither exist, derive nothing and retain the neutral HasBaseType edge.
5. Preserve the neutral fact in all cases.
6. Preserve the neutral edge's asserting-file provenance on the derived occurrence.

The same helper should be used by forward and reverse edge evaluation so classification rules cannot diverge. Its output is ephemeral response evidence, never a stored edge.

## Options and tradeoffs

### Option A — Read-time refinement with existing hydration semantics

Reverse queries derive typed results after name-based hydration. Forward queries use the same resolver but classify only when the target declaration is already indexed; otherwise they return the neutral edge with existing lower-bound coverage.

Benefits:

- no stale durable edge;
- no cross-file dependency tracker;
- exact removal and replacement semantics remain unchanged;
- ambiguity remains neutral;
- the v0.9.1 HasBaseType fact remains queryable;
- one classifier serves both directions;
- no semantic producer generation change because durable producer output is unchanged.

Costs:

- a cold forward query may remain neutral until the target has been indexed;
- forward completeness can improve after another query, consistent with the existing lower-bound contract but observable;
- documentation and coverage must state this explicitly.

This is truthful but rejected as the final design because forward classification would depend on prior session history.

### Option B — Read-time refinement plus staged forward hydration (recommended)

After hydrating the child, inspect its neutral TypeRef targets, hydrate declaration candidates for each written type, then evaluate the shared classifier.

Benefits:

- forward and reverse typed answers become substantially more symmetric from a cold index;
- still avoids persistent derived edges.

Costs:

- an exact C# Class forward query may perform one additional target-name discovery/compilation cycle;
- hydration diagnostics must aggregate the child and target stages rather than hide partial target discovery;
- the behavior changes query cost for this exact query family and therefore needs explicit architectural approval.

### Option C — Persist typed derived edges with dependency tracking

Introduce explicit derivation ownership spanning child and target files, plus recomputation on publication, deletion, kind change, restore, and root refresh.

Benefits:

- symmetric ordinary adjacency queries;
- derived facts can participate in every graph consumer.

Costs:

- new cross-file dependency and invalidation architecture;
- persistence and compatibility changes;
- substantially larger lifecycle and concurrency surface;
- disproportionate to the requested query capability.

Do not choose this without a broader approved redesign.

### Option D — Persist the edge under the child file only

Rejected. A target deletion or Class/Interface kind change cannot retract or recompute the child's stored edge under current file-local replacement semantics.

### Option E — Treat every cross-file target as Interface or Class by naming convention

Rejected. Prefix conventions such as I are not semantic authority and would fabricate facts.

## Regression coverage

Tracked production-path coverage was added before implementation and proven RED for:

1. reverse Interface/IFoo derives FooCross --Implements--> IFoo when both files were published separately;
2. reverse Class/BaseX derives DerivedX --Extends--> BaseX;
3. the original HasBaseType/TypeRef edge remains queryable after classification;
4. same-name Class and Interface declarations keep the edge neutral;
5. an absent target keeps the edge neutral;
6. changing the published target from Interface to Class changes the next derived answer without a stale stored edge;
7. removing the target through the supported refresh/removal lifecycle removes the typed answer;
8. forward Class/child classification is pinned to the approved hydration policy;
9. Struct and Record HasBaseType edges are not classified by the Class resolver;
10. Class reverse coverage exposes the same HasBaseType alternative query as Interface;
11. existing same-file Extends/Implements behavior remains unchanged.

## Revised production design

Further tracing shows staged forward hydration fits the existing preparation boundary without a general redesign:

1. The exact builtin/Class forward query performs its existing child-name hydration.
2. Preparation reads the now-indexed Class child's outgoing HasBaseType edge and obtains its single ambiguous first written base name.
3. Preparation performs target-name hydration using the same inbound-reference discovery mode as a reverse query.
4. Final evaluation invokes one shared ephemeral classifier for both forward and reverse results.

One C# class declaration contributes at most one ambiguous BaseTypeRef, but Model C may merge same-named Class occurrences from multiple files under one exact identity. The forward path therefore performs one hydration per distinct neutral target name, deduplicated before hydration. It does not hydrate the known implemented-interface entries.

Classification must be gated on completed discovery for every relevant hydration stage. If discovery is partial or failed, the response retains only HasBaseType; it must not claim uniqueness from incomplete evidence.

The staged reports must be merged for coverage and diagnostics. A successful child stage cannot conceal a partial target stage.

Within the current identity model, all cases means all exact name identities in the authorized workspace scope. Qualified-name and using-directive resolution cannot be made complete without changing the repository's name-only semantic identity model; those references remain neutral rather than guessed.

Supported lifecycle events remain authoritative. apply_edit publication and explicit repository refresh/reindex update or retract indexed declarations. Unobserved external edits remain outside Clean-CTX's documented freshness contract.

## Compatibility impact

The implemented Option B changes workspace_query response behavior but does not change canonical instructions, stored semantic edges, producer output, or snapshot compatibility. CSharpSemanticProjection therefore remains at generation 2 after the Struct/Record change.

If implementation instead persists new edges or changes semantic publication, the producer and restore compatibility decision must be revisited.

## Implemented architectural decision

The approved implementation adopts Option B:

- exact builtin/Class forward queries hydrate the child and then each distinct neutral target name;
- exact builtin/Class or builtin/Interface reverse queries classify after their existing inbound hydration;
- typed edges are derived only after completed relevant discovery proves exactly one target kind;
- incomplete discovery, an absent target, or Class/Interface ambiguity retains only HasBaseType;
- derived typed edges remain ephemeral and the authoritative neutral fact is preserved;
- Struct and Record HasBaseType facts are not consumed by the Class classifier;
- the existing CSharpSemanticProjection producer remains generation 2 because no durable output changed.

The focused tracked regression suite `workspace_query_csharp_cross_file_classification` was reported GREEN on 2026-10-09. The repository-wide final verification gate remains pending.
