# Semantic Intelligence Architecture Hardening Investigation

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-10  
**Status:** Investigation complete; implementation not authorized  
**Baseline:** Angular template/component-usage vertical GREEN through Phase 6

## Executive verdict

**PROCEED WITH LIMITED EXTRACTION ONLY.**

Clean-CTX has two proven query-time refinement implementations—C# base-type classification and Angular selector/component occurrence resolution—that repeat important mechanics: acquire scoped evidence, preserve a neutral fact, require complete discovery before unique typing, retain ambiguity, compose coverage, and return an ephemeral result. That is enough evidence to extract a small request-scoped resolution protocol.

It is not enough evidence to introduce a general semantic-synthesis layer, policy registry, persisted enriched graph, generic occurrence model, or Task Context Compiler. The materially different third case, NgRx/RxJS-to-service and HTTP/backend flow, exposes producer and identity deficiencies before it exposes a missing generic reasoner. Its current facts are largely durable typed triples produced by domain-local textual extraction; they lack the occurrence, declaration-resolution, protocol, and completeness evidence required for safe cross-domain composition.

The smallest preservation-first next step is:

1. Keep WorkspaceIndex as the durable, framework-agnostic evidence fabric.
2. Keep structured language and domain producers authoritative for meaning.
3. Keep occurrence models producer-specific until a second equivalent consumer exists.
4. Extract only a private request-local evidence-requirement, resolution-state, and provenance-result contract from the C# and Angular implementations.
5. Validate that contract against one narrow NgRx resolution before considering cross-domain synthesis.
6. Leave cross-file and cross-domain conclusions ephemeral unless dependency ownership and invalidation are proven.

This is hardening, not a new semantic subsystem.

## Evidence labels

- **PROVEN BY CODE** — present in production code.
- **PROVEN BY TEST** — protected under src/tests.
- **PROVEN BY LIVE EVIDENCE** — observed through a built MCP process or issue report.
- **INFERRED** — conclusion drawn from proven facts.
- **PROPOSED** — future architecture.
- **UNRESOLVED** — insufficient evidence.

## Current architecture map

~~~text
source
  +-> captures / canonical IR ------------------+
  +-> MetaLayerPass -> registered domain layers |
                                                v
                                         SemanticEdge[]
                                                |
                                                v
                       WorkspaceIndex durable evidence fabric
                       identity + provenance + file ownership
                                                |
                           workspace_query preparation
                           scope + hydration + coverage
                              /                 \
                             v                   v
                  C# request catalog    Angular request catalog
                  Extends/Implements    UsesSelector/UsesComponent
                              \                 /
                               +-------+--------+
                                       v
                              MCP structured answer
~~~

**PROVEN BY CODE:** IRCompiler drains semantic edges from PassContext; LayerRegistry invokes applicable producers; WorkspaceIndex records file-owned occurrences; query/edges.rs performs hydration and invokes C# and Angular request-local catalogs.

### What does not flow downward

~~~text
query-time classification/synthesis
              X must not rewrite
parser captures -> canonical IR -> durable producer facts

rendered skeleton text
              X must not become
semantic extraction authority

CBM discovery candidates
              X must not become
Clean-CTX semantic proof
~~~

## Repository census

| Responsibility | Current authority | Finding |
|---|---|---|
| Canonical structure | src/queries.rs, src/ir/**, src/ir/semantic_projection.rs | **PROVEN BY CODE:** declarations and written calls become generic facts. Calls retain written argument count/spread but remain unresolved. |
| Framework production | MetaLayer, MetaLayerPass, LayerRegistry | **PROVEN BY CODE:** registered layers return presentation and semantic edges. This is already the durable-domain extensibility seam. |
| Semantic model | src/layers/meta/semantic.rs | **PROVEN BY CODE:** identity is domain/type/name; file is provenance. SemanticEdge supports typed CallEvidence, not arbitrary evidence. |
| Durable fabric | src/workspace/index.rs and index/** | **PROVEN BY CODE:** forward/reverse lookup, scoped occurrences, source-owned replacement/removal, fidelity, and source hash exist. |
| Scope | src/workspace/scope.rs | **PROVEN BY CODE:** primary/additional roots and withinPath constrain evidence. |
| Hydration | src/mcp/tool_handlers/hydration.rs and hydration/** | **PROVEN BY CODE:** discovery mode, fidelity requirements, publication, reports, and completion cache exist. |
| Query orchestration | query/prepare.rs, edges.rs, coverage.rs | **PROVEN BY CODE:** request preparation deduplicates hydration and evaluates a final index view; domain branches remain explicit. |
| C# refinement | query/csharp_resolution.rs, classification.rs | **PROVEN BY CODE/TEST:** namespace/nesting/name/arity resolution classifies neutral HasBaseType only with complete discovery. |
| Angular refinement | template_occurrences.rs, query/angular_templates.rs | **PROVEN BY CODE/TEST:** spans/control ancestry survive; neutral and typed occurrence results are ephemeral and additive. |
| NgRx/RxJS | angular_meta/ngrx/**, rx/**, angular_semantic.rs | **PROVEN BY CODE:** import-gated shapes use substantial text scanning and project directly to typed durable edges; no request-local identity resolution exists. |
| .NET/Spring | dotnet_meta/semantic.rs, spring_meta/semantic.rs | **PROVEN BY CODE:** endpoint, route, DI, entity, and mapping facts exist independently; no authorized frontend/backend bridge exists. |
| Compatibility | mcp/compatibility/** | **PROVEN BY CODE/TEST:** durable producer generations are isolated; ephemeral Angular refinement has no generation. |
| Consumers | workspace_query, provide_code_context, CBM tools | **PROVEN BY CODE:** no production Task Context Compiler exists. |

## Existing assets to preserve

1. Model C semantic identity: file provenance is separate from domain/type/name identity.
2. Source-owned edge occurrences: EdgeKey includes asserting file and typed call evidence.
3. Atomic projection replacement with source hash and fidelity.
4. Scoped hydration and final-view batch evaluation.
5. Producer-scoped compatibility generations.
6. Neutral-before-typed behavior in C# and Angular.
7. Angular’s producer-owned occurrence authority.
8. Generic written-call evidence without false target resolution.
9. Existing domain producers and their specialized authority.
10. CBM as advisory discovery, never semantic proof.

## Evidence authority taxonomy

The proposed levels are useful as a diagnostic taxonomy, not a stored type hierarchy.

| Level | Example | Minimum authority | Current status |
|---|---|---|---|
| 0 observation | tag/call/string/span exists | parser/source | Strong in Angular occurrences and generic calls; uneven elsewhere. |
| 1 structural | caller contains written call | language grammar/canonical projection | Available through CoreOp::Call and CallEvidence. |
| 2 contextual | reference uniquely resolves to declaration | language-aware policy plus complete scoped candidates | Implemented locally for C# bases. |
| 3 domain | Angular Component or Spring endpoint | domain producer/interpreter | Durable producers exist; extraction quality varies. |
| 4 composite | selector occurrence uniquely uses Component | domain policy + occurrence + complete discovery | Implemented ephemerally for Angular. |
| 5 cross-domain | frontend HTTP reaches backend endpoint | explicit protocol bridge | Missing. |
| 6 causal/system | click can eventually write entity | authorized causal hop composition | Not justified. |

No lower level automatically implies a higher one.

## Capability evidence matrix

| Capability | Structural evidence | Domain evidence | Missing evidence | Minimum authority | Current owner | Candidate future owner |
|---|---|---|---|---|---|---|
| C# base | structured base node | class/interface declarations | complete candidate universe | C# namespace/name/arity policy | C# query catalog | Domain policy using shared result mechanics |
| Angular child use | element span/control ancestry | HasTemplate, HasSelector | complete candidates | Angular selector policy | Angular query catalog | Remain Angular-local |
| Generic call | caller, written callee, argc/spread | none | receiver/type/declaration binding | language resolver | builtin projection | future contextual resolver |
| NgRx dispatch | call-like syntax | import gate and shape | occurrence, import/symbol identity, owner, completeness | NgRx policy | NgRx extractor | first validation case |
| NgRx service call | nested call text | effect shape | resolved service method and occurrence | NgRx + TS resolution | NgRx extractor | NgRx validation case |
| Frontend HTTP | calls/literals may exist | incomplete | method, URL structure, client identity, occurrence | HTTP producer | missing | future producer |
| Backend endpoint | attributes/annotations/methods | route facts | normalized protocol identity | .NET/Spring HTTP policy | domain producer | producer plus bridge adapter |
| Persistence write | calls/entities | partial mappings | proven operation occurrence | persistence policy | partial | future producer |

## Proof-case comparison

### C# base-type refinement

~~~text
durable Class --HasBaseType--> TypeRef/written
 + request-local structured C# declarations
 + namespace/nesting/name/generic arity
 + complete hydration
 -> ephemeral Extends or Implements
~~~

**PROVEN BY CODE:** csharp_resolution::Catalog builds structured declarations and references from current source; classification preserves neutral facts on failure.

**PROVEN BY TEST:** workspace_query_csharp_cross_file_classification.rs covers cold hydration, collisions, refresh, struct/record exclusion, qualified names, generic arity, and mismatch.

### Angular selector/component refinement

~~~text
durable Component --HasTemplate--> Template
 + current ordered Template source
 + SelectorOccurrence(span, control ancestry)
 + durable Component --HasSelector--> selector
 + complete discovery
 -> ephemeral UsesSelector + optional UsesComponent
~~~

**PROVEN BY CODE:** template_occurrences::extract produces source-ordered occurrences. angular_templates::Catalog always emits UsesSelector and adds UsesComponent only for one candidate under complete discovery, carrying identical evidence.

**PROVEN BY TEST:** workspace_query_angular_selector_occurrences.rs covers repeated uses, mixed ancestry, unresolved/ambiguous/unique states, refresh, selector changes, and no producer bump.

### NgRx/RxJS toward HTTP/backend

Current evidence ends early:

~~~text
Component --Dispatches--> Action
Effect --HandlesAction--> Action
Effect --CallsService--> ServiceMethod/string
Effect --ProducesAction--> Action

Spring/.NET endpoint facts exist separately.
~~~

**PROVEN BY CODE:** NgRxShape::to_ngrx_semantic_edges emits typed durable edges directly. extract_ngrx_shape_with_regions is import-gated, but its extractors use source scans and lexical exclusions. ServiceMethod is name-like, not a proven declaration occurrence. No production bridge proves a service call reaches an HTTP request or endpoint.

**INFERRED:** Traversing these triples as a causal chain would overstate identity and behavior. The missing work is stronger producer evidence, not a generic walker.

| Concern | C# | Angular | NgRx/cross-domain | Share now? |
|---|---|---|---|---|
| scoped acquisition | yes | yes | needed | use existing hydration |
| complete discovery before typing | yes | yes | needed | yes, narrow contract |
| neutral preservation | yes | yes | inconsistent | yes, invariant/result |
| ambiguity state | neutral fallback | explicit status/candidates | needed | yes |
| occurrence model | not exposed | template-specific | missing | no |
| matching policy | C# | Angular | protocol/domain-specific | no |
| path causality | none | none | missing | defer |

## Duplication and gaps

Worth sharing:

- declare scoped discovery requirements;
- acquire and merge hydration reports;
- use the final scoped index view;
- represent unique, unresolved, ambiguous, incomplete, failed, unsupported;
- preserve neutral evidence;
- establish exact capability without overstating completeness;
- attach provenance to ephemeral results.

Must remain local:

- C# namespace/nesting/arity matching;
- Angular selector and control ancestry semantics;
- NgRx ofType/reducer/effect/RxJS meaning;
- HTTP route normalization;
- persistence semantics.

Local producer gaps include intertwined NgRx observation/interpretation, declaration-looking identities used for some references, and relation-specific edge evidence that cannot represent all occurrences. Missing capabilities include resolved TS receiver/call targets, occurrence-bearing NgRx facts, frontend HTTP requests, normalized endpoint identities, and persistence operations.

Missing cross-domain protocols include HTTP, messaging, RPC/schema, persistence, and UI event identities. A shared core must never derive them from strings.

## Recommended limited extraction

Conceptual private contract:

~~~text
ResolutionRequirement
  discovery requests: mode, lookup name, semantic fidelity
  authorized WorkspaceScope
  required completion

ResolutionState<T>
  Unique(T)
  Unresolved
  Ambiguous(Vec<T>)
  Incomplete
  Failed(diagnostic)
  Unsupported(diagnostic)

DerivedEvidence
  domain policy identifier
  supporting durable identities
  optional producer-owned occurrence payload
  completion facts used
~~~

These names are proposals, not approved APIs.

Responsibilities:

- let a domain refiner declare evidence needs;
- execute them through existing hydration;
- supply the final scoped index view;
- standardize resolution states and completeness input;
- compose an ephemeral result with provenance;
- preserve the neutral input.

Non-responsibilities:

- no matching rules or framework branches;
- no policy/capability registry yet;
- no persisted derived facts;
- no generic occurrence schema;
- no path or causal inference;
- no replacement of MetaLayer, LayerRegistry, WorkspaceIndex, or compatibility;
- no automatic hydration inferred from relation names.

First extraction target: C# and Angular query orchestration, protected by exact response regressions.

First validation target: NgRx action reference to one authoritative action declaration, but only after structured occurrence-bearing neutral evidence exists. Keep the typed result ephemeral.

## Strongest falsification case

The case against a new architecture is strong:

- only two mature refiners exist;
- their occurrence needs differ;
- PreparationContext, hydration, WorkspaceScope, coverage, and WorkspaceIndex already provide most mechanics;
- a new layer could only rename query orchestration;
- cross-domain failure currently reflects missing evidence, not missing traversal;
- a registry is speculative until a third consumer fits;
- persistence creates unowned multi-file dependencies;
- path traversal could fabricate causality.

Therefore revisit broader synthesis only when:

1. a third production domain uses the same requirement/result contract;
2. one explicit cross-domain protocol bridge is proven end-to-end;
3. two consumers need the same hop/completeness semantics.

## Occurrence, identity, durability, and compatibility

SemanticEdge is sufficient for file-owned entity-level facts and typed CallEvidence. It is not sufficient for every occurrence-sensitive future fact, but this does not justify generic JSON evidence.

- Keep SelectorOccurrence private and ephemeral.
- Keep call evidence typed.
- New producers first own typed occurrence structures locally.
- Byte spans are within-snapshot evidence, never durable identity across edits.
- Retain domain/type/name identity only where name is authoritative.
- Keep file as physical provenance.
- Use neutral reference types instead of fake declarations.
- Future protocol identities must be explicit; basename, final segment, and string equality are insufficient.
- Persist only single-owner source-proven facts.
- Keep multi-owner refinement, ambiguity, bridges, and paths ephemeral.
- Durable producer changes bump only their producer generation.
- The limited private contract introduces no producer key or generation.

## Causality, completeness, scope, and performance

Current dependency traversal and CBM call reachability do not prove causality. A future path hop must declare whether it is containment, resolution, invocation, event, protocol, persistence, or render/use. A path is causal only if every policy authorizes causal composition in that direction.

A returned path proves that path, not uniqueness or exhaustive discovery. Per-hop status must identify ambiguity, incomplete discovery, unsupported boundaries, and termination.

Every lookup must use the existing WorkspaceScope. Evidence outside primary/additional roots and withinPath is ineligible. External artifacts require explicit admission.

Performance should reuse request-level hydration deduplication and final-view catalogs. Cache completion separately from answers; do not cache derived conclusions without dependency invalidation. Future traversal budgets require separate approval.

## Options

- **A, domain-local only:** safest immediately, but repeats proven orchestration for a third refiner.
- **B, shared mechanics plus domain policies:** accept only the narrow requirement/result/provenance subset.
- **C, central reasoner:** reject; it would hardcode domains or infer authority.
- **D, persisted mega-graph:** reject; ownership, invalidation, compatibility, and scale are unproven.
- **E, durable evidence plus ephemeral synthesis:** directionally right; implement only the limited contract now.

## Future onboarding stress test

**Django/FastAPI/SQLAlchemy:** requires Python structure, domain producers, local occurrences, and explicit HTTP/persistence policies. It must not modify Angular, C#, or NgRx. Status: **POSSIBLE AFTER SPECIFIC PRODUCER WORK**.

**Spring + Kafka:** Spring facts exist; Kafka needs producer/consumer occurrences plus topic/schema identity. Status: **POSSIBLE AFTER SPECIFIC PRODUCER WORK AND LATER SYNTHESIS WORK**.

**OpenAPI/Kubernetes/Terraform:** require artifact parsers, identity, scope, lifecycle, compatibility, and explicit bridges. Status: **LONG-TERM / RESEARCH**.

## Issue #133–#142 mapping

The public issue bodies were not retrievable from this environment. This table uses finding families recorded in current plans, invariants, tests, and the supplied brief; it does not invent exact issue wording.

| Issue | Known family | Bucket | Timing | Architectural meaning |
|---|---|---|---|---|
| #133 | C# cross-file discovery/hydration | CROSS-FILE / DISCOVERY | existing fixes before hardening | first proven refiner |
| #134 | C# neutral base identity | IDENTITY / STRUCTURAL | existing fixes | neutral-before-typed |
| #135 | class/interface classification | RESOLUTION / COMPLETENESS | existing fixes | unique/ambiguous/incomplete |
| #136 | structured headers and coverage | LOCAL PRODUCER / REPORTING | existing fixes | structured authority; generic layer cannot fix parser defects |
| #137 | qualified/generic resolution and refresh | IDENTITY / LIFECYCLE | existing fixes | domain keys plus current lifecycle |
| #138 | Angular DI/providers/signals/member APIs | MISSING PRODUCER | after hardening, local | mostly producer work |
| #139 | NgModule/routing/TS identity/constraints | MIXED LOCAL/IDENTITY/STRUCTURAL | split by finding | must not be bundled |
| #140 | NgRx action/effect/selector identity/linkage | PRODUCER/OCCURRENCE/RESOLUTION | first validation candidate | strongest third proof case |
| #141 | Angular templates/linkage | completed vertical | completed | second proven refiner |
| #142 | hydration/coverage/refresh/index/platform | TEMPLATE parts resolved; broader PLATFORM | separate | reuse mechanics, do not import wholesale |

Exact issue-title reconciliation is a follow-up when those bodies are available.

## What becomes possible

| Capability | Classification |
|---|---|
| consistent C#/Angular refinement states | POSSIBLE WITH CURRENT EVIDENCE |
| NgRx action-reference resolution | POSSIBLE AFTER SPECIFIC PRODUCER WORK |
| template-to-component slice | POSSIBLE WITH CURRENT EVIDENCE |
| Angular event -> NgRx action -> effect | POSSIBLE AFTER SPECIFIC PRODUCER WORK |
| effect -> service -> HTTP request | POSSIBLE AFTER SPECIFIC PRODUCER WORK |
| frontend HTTP -> backend endpoint | POSSIBLE AFTER PRODUCER AND SYNTHESIS WORK |
| cross-language blast radius | POSSIBLE AFTER SYNTHESIS WORK |
| targeted tests and semantic regression slices | POSSIBLE AFTER SYNTHESIS WORK |
| architecture invariant checks | POSSIBLE AFTER SYNTHESIS WORK for modeled relations |
| dead-path detection | POSSIBLE AFTER COMPLETE PRODUCER COVERAGE |
| security/taint analysis | LONG-TERM / RESEARCH |
| Task Context Compiler | LONG-TERM / RESEARCH |

## Migration plan

1. Freeze exact C# and Angular preservation regressions.
2. Define a private resolution-state contract.
3. Define private request-local evidence requirements using existing hydration.
4. Migrate C# orchestration only; keep C# policy local.
5. Migrate Angular orchestration only; preserve occurrence JSON and diagnostics.
6. Harden NgRx structured neutral occurrence evidence.
7. Validate one NgRx neutral-to-typed resolution.
8. Separately investigate one HTTP protocol bridge.
9. Defer paths, causality, task projection, registries, caches, and additional domains behind new approval.

No dual authority may survive migration. Remove an old local orchestration path only after equivalence is proven.

## Risks and rejected alternatives

Risks: domain branches leaking into shared code, changed public diagnostics, hydration mistaken for proof, occurrence flattening, stale caches, cross-root leaks, and a “shared” module that merely renames edges.rs. Mitigate with one-domain-at-a-time migration, opaque producer evidence, no derived cache, existing WorkspaceScope, and a stop condition if NgRx cannot fit unchanged.

Rejected: universal meta-layer, generic durable JSON evidence, global registry now, name-based bridges, durable cross-file conclusions without owners, graph reachability as causality, early Task Context Compiler, and canonical-IR redesign for framework conclusions.

## Open decisions

### Blocking before implementation

1. Exact private Rust API shape.
2. Whether failed acquisition remains QueryFailure or a ResolutionState without changing public behavior.
3. Internal mapping of C# neutral fallback to explicit states.
4. Exact preservation regression set.

### Non-blocking

- internal module naming;
- lazy versus eager internal provenance rendering;
- future capability descriptors after NgRx validation.

### Deferred

- generic occurrence identity;
- protocol schemas;
- path semantics and budgets;
- derived caching/persistence;
- Task Context Compiler;
- global coverage redesign;
- broader #142 platform work.

## Acceptance criteria

1. C# and Angular public outputs remain compatible.
2. Neutral evidence survives unique refinement.
3. Ambiguous/incomplete/failed/unsupported never become typed.
4. Angular occurrence evidence remains required.
5. Existing scope and final index view govern every lookup.
6. No new producer key, generation, durable edge, cache, registry, or public relation.
7. Domain policies remain local.
8. NgRx can later consume the contract without C#/Angular edits.
9. If NgRx requires domain branches in shared code, abandon the extraction.

## Recommended near-term architecture

~~~text
Source + structured language authorities
                  |
      domain and builtin evidence producers
                  |
                  v
WorkspaceIndex durable source-owned evidence
                  |
 existing scope + hydration + final query view
                  |
                  v
LIMITED shared request mechanics
requirements | resolution state | provenance
NO domain meaning | NO persistence | NO paths
          /          |          \
      C# policy  Angular policy  future NgRx policy
          \          |          /
                  v
ephemeral typed answer + retained neutral evidence
                  |
                  v
                 MCP
~~~

Synthesized meaning never flows back into parser, canonical, or durable producer authority.

## Final decision

Proceed with a limited private extraction of request-scoped evidence requirements, resolution outcomes, and provenance/completeness composition. Do not build shared path synthesis, a semantic policy registry, a persisted mega-graph, generic occurrence storage, protocol bridges, or a Task Context Compiler in the same change.

The first extraction target is duplicated C#/Angular query orchestration. The first falsifying migration target is one narrowly defined NgRx neutral-to-typed resolution after NgRx obtains structured occurrence and identity evidence.

> **Mission:** Establish the smallest preservation-first resolution boundary that reuses scope, hydration, completeness, and provenance mechanics while leaving every semantic conclusion with the language or domain authority that can actually prove it.
