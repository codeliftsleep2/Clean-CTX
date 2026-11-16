# ANG-R2 Phase A representation disposition

Agent: **Agent MaxHeadRoom**. Architectural direction accepted by owner disposition 01; not implemented.
The alternative assessment below is retained Phase A reasoning, not reopened work.

## Actual loss map

| Boundary / source | What survives now | Actual loss / required change |
| --- | --- | --- |
| declaration_facets/resolve_mappings.rs::Demand::resolve_object | Neutral owner/field/Object/pair/key/value joins and exact selected references | No demonstrated missing carrier for the established forms. Reuse unchanged. |
| TypeScriptAuthority::resolve_mapping_at | Exact key decoding, source/hash/containment, demanded/runtime reference and current witness proof | Proof accessor returns the mapping and declaration; it does not erase the key. |
| route_resolvers::project | Native ownership, approved refusal and one edge per object/target | **Earliest semantic loss:** targets BTreeSet suppresses a second key to one target; SemanticEdge has no registration field. Add paired registration output before membership-only dedup. |
| route_resolvers::owned_objects | Object spans for selective weak-branch suppression | Preserve unchanged meaning even on canonical refusal. |
| route_resolvers::admits_projected / relation_evidence.rs | Exact membership replay and operation-local projection reuse | Existing contract validates unqualified edges. Extend the production projection validation to check registration records using that same resolver result once. |
| MetaLayerEvaluation -> LayerRegistry -> MetaLayerPass -> PassContext/IRCompiler | Markers, semantic edge Vec and dependencies | These carriers currently have no registration collection; one richer internal result must transport it without a second evaluation. |
| SemanticCompilationArtifact / into_parts / published_semantics | Edges and retained TS proof through separately taken authority | Add support to semantic decomposition and every publication caller; canonical-only discard remains explicitly classified. |
| CompatibleSemanticProjection / SemanticSnapshot / DurableSemanticSnapshot | Same owner/hash/edges/dependencies and authority | Add optional typed support in the same lifecycle/payload; do not claim old snapshots already contain it. |
| DurableSemanticEdge / index EdgeKey | Existing endpoint identity, Calls-only evidence, whole edge storage | They are not the first loss. Recommended design leaves their identity/codec untouched. |
| index/coverage.rs SemanticCoverage / source replacement | Source-owned current retained authority | Retain the registration collection in that same owner entry, replacing/removing it with the projection. No new reverse index or coordinator. |
| query/edges.rs / SemanticEdge Serialize / query/content.rs | Existing edge fields; canonical endpoint is skipped publicly | Existing responses intentionally expose no mapping support. Add a separate explicit operation; preserve old responses. |

## Alternatives assessed

| Alternative | Identity, multiplicity and traversal | Compatibility, lifecycle and cost | Disposition |
| --- | --- | --- | --- |
| Put keys into Resolver/Route names or layer strings | Turns display/provenance into authority; cannot reliably represent same-name declarations or route occurrences | Changes name buckets and consumers; loses exact proof | REJECT. |
| Reuse CallEvidence | Wrong meaning; fields encode arity/spread, not mappings | Durable validation explicitly rejects call evidence on non-Calls; violates other consumers | REJECT. |
| Change ResolvedBy key to include mapping key | Makes two keys into two existing membership edges | Changes approved dedup/count/meaning and default forward/reverse answers | REJECT. |
| New key-qualified relation per registration | Can retain exact support with a new typed relationship-evidence carrier | Requires new relation/edge identity rules, persistence and default-query filtering; dynamic relation strings are unbounded vocabulary | Viable with a larger explicit contract; not preferred. |
| New registration occurrence node plus route/target edges | Correctly models ternary ownership when every node/edge has exact proof | Requires a new canonical endpoint, vocabulary, entity visibility and forward/reverse joins; existing public hidden identity still needs scoped exposure | Viable but more graph surface than the demonstrated questions require. |
| Typed registration support on SemanticEdge | Endpoints and old membership key can remain unchanged if support is separately compared | New field touches shared public struct construction; bounded rg census found 245 SemanticEdge literals in 83 files, not a complete impact analysis. Every producer/DTO/comparator needs review; active-file rules still apply | Semantically viable; disproportionate shared carrier surface for this bounded field. Not rejected on correctness or file-size convenience. |
| Derive registrations only during the new query from retained TS mappings | Existing neutral payload already persists the required source joins | If it invokes an independent evaluator it creates competing semantics; even a shared projector adds query-time reconstruction and lacks a retained produced registration result | Not preferred. No second interpreter or per-row replay. |
| Typed collection beside existing semantic edges in the same source-owned projection | Exact (route, key, target) records; old edge dedup unchanged; F/R filter one owned collection | Requires transport/persistence/admission integration, but no global identity, new graph vocabulary, edge fields, store or lock | **RECOMMEND**, subject to complete lifecycle and cost verification. |

This choice does not avoid lifecycle work. It uses the existing persistent owner
rather than leaving the only registration result in a request-local view. It is
a specific typed result for one demonstrated capability, not an extensible registry,
generic evidence bag or new framework.

## One production evaluator, two projections

Refactor the existing resolver routine internally to produce memberships and
registration records from one native route/mapping proof traversal. For every
admitted mapping, record its registration before deduplicating membership by
target. Keep the original order, exact membership endpoints and broad written-use
policy. Retain known source-proven omission reasons in that same result, not by
reparsing refused forms later. Do not conflate stale/forged payload with an ordinary
unsupported mapping. `project` can remain a membership-only compatibility wrapper around the
shared routine for old callers; the default production Angular evaluation consumes
the richer result **once**, not that wrapper followed by another registration pass.

The canonical carrier belongs in the existing layers/meta semantic boundary using
the existing neutral CanonicalDeclaration/RouteOccurrence shapes. Do not import
IR-owned TypeScript types into semantic.rs or store Angular-produced records inside
the neutral TypeScriptAuthority capture. Precise private Rust struct/module names
are routine only after this ownership contract is accepted.

Production publication/restore validation shares one operation-local resolver
projection between membership checks and registration checks. The registration
collection must match the complete expected admitted registration multiset for
that owner, and its canonical memberships must agree. Checking only each supplied
record would let removal/empty payload masquerade as successful production.
Existing direct membership-admission APIs that intentionally validate a subset
retain their old meaning; do not globally turn unrelated relation validation into
a complete-set contract.

## Proposed lifecycle integration map

1. Native capture unchanged; shared route/mapping proof supplies both results.
2. Angular combined evaluation emits the optional typed collection with its edges.
3. Existing registry/default MetaLayerPass/PassContext/compiler result transport
   carries the collection into SemanticCompilationArtifact. No standalone detector.
4. Artifact decomposition, provide, compress, hydration, explicit-file entities,
   edit and delta transfer the same collection with authority and dependencies.
5. Compatibility and publication validate source/producer/witness identity and
   matching record proof before committing the normal source owner.
6. Session snapshot, pending transitions, SQLite JSON snapshot and restoration carry
   the optional collection. No separate table/database or persistence transaction.
7. WorkspaceIndex SemanticCoverage retains it alongside that owner's authority.
   Existing remove_file already removes the entry; root reset/replacement follows
   the same owner. Validation precedes replacement; failed publication must not
   expose mixed old/new edges and registrations.
8. The new query prepares current candidate owners, reads their admitted collections
   once, filters records by route-path or resolver-name bucket, applies asserting
   scope, validates the consumed owner footprints at the existing final boundary,
   sorts and returns exact records. It does not reinterpret names into proof.

Required source regions: `layers/meta/{context,angular_semantic,mod}.rs`,
`layers/registry.rs`, `ir/{pipeline,compiler}.rs`, `pipeline/meta_layer.rs`,
`angular_meta/route_resolvers.rs`, `typescript_resolution/relation_evidence.rs`,
`mcp/semantic_compilation/{artifact,retained}.rs`, compatibility validator,
durable semantics/SQLite semantic state, index coverage, actual core/provide/
compress/delta/edit/hydration/entities publication callers, and workspace query
request/schema/content/batch integration. Existing paths are confirmed by source;
any new helper path is a proposed addition, not a discovered implementation.
Both `mcp/tool_schemas.rs` input variants and `mcp/tools.rs` registered output schema
are affected; permissive serialization alone is not a schema contract. The new
coverage status/fact meaning uses existing vocabulary and its bounded reason uses
the existing object form; existing global string enums are not widened.

Public Rust evaluation interfaces need preservation review. Prefer an internal
richer result/compatible wrapper over breaking unrelated external callers of the
existing registry tuple or public struct construction. Do not silently broaden
the approved public Rust compatibility surface; return a concrete unavoidable
break to the owner before implementing it.

`angular_semantic.rs` is 601 lines and `pipeline.rs` 603 at this baseline;
`durable_semantics.rs` 575. Any active expansion must stay within 615 via cohesive
decomposition if necessary. File-size cost does not weaken identity/lifecycle.

## Accepted query-specific acquisition direction

Cold reverse registration discovery cannot rely only on the resolver's display
name appearing in every asserting source: aliases/barrels/namespaces can hide it.
Likewise path strings are not native route authority. For this **new operation
only**, nominate TypeScript owners through the existing configured-root
`source_inventory`, `select_candidates` and `compile_candidates_for` mechanics,
using High semantic acquisition/currentness. The existing bootstrap nomination
helper demonstrates these mechanics but does not itself authorize resolver use.

This is conservative candidate acquisition, not a new scanner, global discovery
default, provider fallback or completeness certificate. Reuse current compatible
owners through existing selection; do not unconditionally recompile the workspace.
`withinPath` filters answers only. Preserve existing inventory/acquisition failure
and retry behavior; no caps, budgets, timeouts, lease or cache is introduced.
Measure enclosing cold/warm nomination cost, not only the new Vec filter.

The record query may scan prepared current per-source coverage collections once;
there is no new maintained inverse index. If measured cost makes that insufficient,
return the measured tradeoff rather than adding one under this approval.

WSC-014 also requires naming detected unsupported semantic forms. The proposed
collection therefore carries bounded source-qualified omission records alongside
positive registrations. They are not extra relationship facts or a new global
diagnostic framework. Preserve existing acquisition/currentness failure categories
and the old public operation diagnostics.

## Final nomination and omission disposition

The exact final rules are in the public contract's owner-disposition-01 sections.
`select_candidates` deduplicates and checks eligibility; the current semantic
source/hash/fidelity shortcut is in `publication.rs::prepare_candidate`, not in
selection itself. The shared compiler batch and publication owner are reused.
`bootstrap.rs::hydrate_bootstrap_owners` is inspected evidence for the mechanics,
not a resolver implementation or a complete required-failure adapter.

Selected-bucket omissions require actual route or resolver association proof.
Unassociated inspected-source omissions have their own response array; general
discovery limits remain coverage. No unsupported syntax invents target identity.
No further general representation investigation is requested or performed.
