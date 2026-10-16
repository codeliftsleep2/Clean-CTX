# Query-history independence and discovery completeness investigation

> **Status: SUPERSEDED — HISTORICAL INVESTIGATION**
>
> This document preserves the pre-approval investigation and alternatives.
> Decisions A-C were later approved and implemented under Issue #142. Current
> authority is `docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md`; acceptance
> evidence is `docs/plans/ISSUE_142_FINAL_CROSS_CUTTING_ACCEPTANCE.md`.

Date: 2026-10-11. Investigator: Agent MaxHeadRoom.
Baseline: `abf8a51543cbbb0eb8a4799ce6a2d6037caf5c70`, branch
`(feat)Semantic_Intelligence_Architecture_Boundaries`.
Scope: the deferred warm/cold hydration portion of Issue #142, per the supplied
investigation handoff. Investigation only; no production changes.

Evidence labels used below: **CODE** = proven control/data flow in current code;
**TEST** = a current tracked test asserts this contract, not a claim it was run
for this investigation; **LIVE** = reproduced this investigation; **INFERENCE**,
**PROPOSAL**, **NOT ESTABLISHED** retain their ordinary distinctions.
There is no new LIVE evidence in this report. No server, Cargo command, or
controlled verification was started. GitHub issue retrieval through `gh issue
view 142` returned HTTP Forbidden; issue scope is taken from the supplied
handoff and the tracked Angular plan's #142 reconciliation, not an invented live
issue snapshot.

## 1. Executive verdict

**MORE EVIDENCE REQUIRED.** Current production code does not establish universal
query-history independence. It contains concrete acquisition/completion gaps,
but current runtime impact and the smallest common correction require targeted
falsification before production implementation or a new shared abstraction.

Three especially important findings:

1. **CODE:** CBM search success with `still_indexing`/`failed` readiness can still
   produce aggregate `discovery_status = completed`. Completion caching correctly
   withholds that project's cache entry; request-time refinement uses the weaker
   aggregate status anyway. This is an internal truthfulness inconsistency,
   not merely a hypothetical performance concern.
2. **CODE:** legacy hydration skips any already-indexed file regardless of its
   semantic fidelity/hash. Only Angular Service edge requests explicitly require
   High coverage. Discovery of a name and adequacy of a semantic projection are
   different requirements and are not uniformly coupled.
3. **CODE:** batches evaluate index-backed answers after all preparation, but C#
   and Angular catalogs are captured during each item's preparation. A shared
   final index guard is not a shared final refinement snapshot.

Historical cold-forward C# and Low→High Angular Service failures have targeted
repairs and tests. Do not reintroduce those fixes as a new platform design.
WorkspaceIndex can remain the sole semantic evidence fabric; there is no evidence
here earning replacement, a second graph, or durable derived resolution.

## 2. Governing invariant and its necessary qualification

The proposed invariant is appropriate for **equivalent authoritative queries**
against a fixed authorized source generation, root set, exclusions, producer
configuration, fidelity requirement and declared scope. Execution cost may vary.
Semantic facts, resolution states, ambiguity and authoritative absence may not.

The unqualified equality of every response is incompatible with intentional
`indexed_evidence_only` and `lower_bound` APIs. `has_cycle` explicitly queries the
currently published graph. Generic exact edge coverage explicitly says
`source_complete: false`. Those contracts permit new valid facts after warming;
they must never imply complete negative answers. Accepting the stronger invariant
for those APIs would change public behavior and requires explicit approval.

Proposed distinction: authoritative answers require history-independent evidence
acquisition; intentional partial answers require stable, truthful limits and no
unjustified uniqueness/absence/completeness. A narrower lower bound can be truthful
while still being inadequate for the desired agent workflow. Do not call those
identical questions by silently normalizing away the difference.

## 3. Current production call paths

All paths start at registered tools dispatch →
`query.rs::handle_workspace_query` → `WorkspaceQueryOperation::prepare`.
`PreparationContext::scope` calls the one `query_scope`/`WorkspaceScope` rule:
primary root + configured additional roots, intersected with `withinPath`.

| Family | Acquisition and evaluation path |
| --- | --- |
| find_entities | `entities::try_prepare_find_entities` → `context.hydrate(...LegacyEdit)` → `hydrate_workspace_index_for` → deferred name lookup in current index |
| forward/reverse | `edges::try_prepare_*` → name hydration → C# target or Angular owner/selector stages → catalog construction → deferred identity lookup, classification, coverage |
| transitive_dependencies | `graph::try_prepare_transitive_dependencies` → starting-name LegacyEdit hydration → scoped BFS; no recursive frontier acquisition |
| has_cycle | `graph::try_prepare_has_cycle` → scoped index witness only, explicit indexed-only coverage |
| entities_in_file | explicit checked path → `read_source`/hash → required semantic fidelity check → candidate compilation → `replace_semantic_projection` → deferred file lookup |
| calls_in_file | `calls::prepare_calls_in_file` → checked current source/canonical compilation → ready unpublished owner/overload answer; no global invocation record |
| batch | `batch::try_execute` → prepare items sequentially using one context → acquire one final index read guard → evaluate each PreparedQuery |

Hydration pipeline: provider/root/name/mode completion lookup → healthy CBM
`ensure_indexed_for` and declaration/inbound path search, or literal filesystem
`scan` → canonical candidate selection → checked path → semantic publication
preflight → `compile_file_ir_focused` → source-hash checked publication ticket →
WorkspaceIndex → conditional completion recording → HydrationReport.

Provider results are file-path hints; language/meta compilation produces facts.
C# classification remains query-time `classification::{forward,reverse}` over
neutral HasBaseType and `csharp_resolution::Catalog`. Angular catalogs consume
HasTemplate, read linked templates, interpret ordered selectors, and resolve
HasSelector candidates. `outcome` and `content` produce the MCP structured/text
surfaces; diagnostics do not independently acquire evidence.

## 4. State that survives requests

| Structure / owner | Lifetime, ingress and removal | Meaning of absence / completeness |
| --- | --- | --- |
| `McpState.workspace_index`, WorkspaceIndex entities/forward/reverse/file maps | Session; compression, hydration, entities publication and relevant lifecycle tools; file/root replacement/removal | Compiled subset, not workspace absence. Per-file semantic coverage exists; no universal complete graph certificate |
| `WorkspaceIndex.semantic_coverage` | Fidelity + source hash for replacement projections; removed with owner | Adequate current projection only if hash and required fidelity match |
| `McpState.hydration_discovery`, HydrationDiscoveryCache | Session; successful provider/root/mode/name/optional fidelity plus generation; edit/refresh/source-staleness invalidation | Completion of one discovery operation, not all producers or dependency closure; stores no candidates or answers |
| `McpState.source_cache` | Session cached Arc source with mtime/size; read, file/root invalidation | Previously read bytes. `cached_source_snapshot` does not stat; catalogs intentionally use snapshot bytes |
| semantic publication clocks/tickets | Per-owner session coordination; preflight/commit and source validation | Publication ordering/current-source protection; not discovery completeness |
| graph bridge / CBM project state | Configured project readiness, searches, dirty/reindex lifecycle | Advisory discovery authority separate from Clean-CTX semantic facts |
| compression cache, dictionary/file identities, persistent artifacts | Source compilation and persistence lifecycle | Artifacts/identity mapping are not evidence of exhaustive current discovery |
| `PreparationContext` scope/hydration map | One single or batch request; dropped on return | Deduplicates name/mode/requirement acquisition within shared request scope |
| C#/Angular catalogs | Captured during item preparation; dropped after evaluation | Derived/refined query-local evidence from the then-indexed subset |
| private invocation evidence | Compilation-local source-id/SHA-bound view; dropped with compile lifecycle | Not in WorkspaceIndex or global discovery completion |

Root refresh retracts indexed owners, invalidates source snapshots and pending
transitions, then invalidates discovery (`hydration/invalidation.rs`). Source
mtime/size change detected during `read_source` invalidates discovery globally.
An unread newly added file is not detected merely by a cached name search;
external edit/new-file generation changes require the supported refresh lifecycle.

## 5. Current-code cases and classification

| Case | Classification / evidence / required observation |
| --- | --- |
| CBM still indexing but successful path search | **CODE completion inconsistency.** `successful_roots` accepts search success independent of readiness; no fallback for that root. With no pending fallback roots, discovery returns completed. Cache insertion excludes non-ready scope, but `HydrationReport::is_complete` only tests aggregate status. Typed uniqueness/absence may be unjustified; reproduce with injected readiness and ambiguous candidates |
| CBM reverse C# missing target declaration | **INFERENCE correctness risk.** Inbound path discovery does not establish declaration discovery. Reverse C# builds its catalog immediately after reverse acquisition; filesystem literal fallback naturally includes target declarations, healthy CBM need not. Compare provider lanes explicitly |
| Legacy file previously published Low | **CODE inadequate requirement boundary; runtime effect family-dependent.** `select_candidates(LegacyEdit)` tests only file_map presence. Low publication can suppress richer compilation even though name discovery succeeds. Angular Service has a local fix; test other producer-specific relations before generalizing |
| A→B→C transitive query | **CODE partial acquisition.** Only the initial name is hydrated, then BFS uses existing adjacency. Warming B can expose C. Generic coverage is a lower bound: this is explicitly incomplete, not proof of false authoritative absence |
| cycle query before/after publication | **CODE intentional indexed view.** `source_complete:false`, indexed-only coverage. Strong equality requires changing the contract, not fixing cache performance |
| reverse Angular Component after unrelated parent hydration | **INFERENCE partial-result variability.** Catalog iterates all indexed Component links; reverse Component intentionally cannot claim source completeness. Parent templates can use a selector without naming target class in source. Compare exact cold/warm observations |
| catalog frozen before later batch preparation | **CODE snapshot split, INFERENCE semantic failure.** Deferred evaluator reads final index but captured catalog/complete flag belongs to earlier preparation. Test late declaration/selector publication and ambiguity |
| name-only identity query with failed discovery | **CODE state conflation risk.** `IdentityRequest::resolve` uses indexed candidates and emits invalid-params no-match/ambiguity; it does not receive completeness. Sparse discovery context may not survive an error response; inspect dedicated failure-path expectations |
| excluded source explicitly published earlier | **INFERENCE scope mismatch.** filesystem honors gitignore/built-in/config exclusions; index scope filters provenance by roots/withinPath. Compare discovery-visible universe against explicitly published excluded owners |
| missing/out-of-scope entities_in_file | **CODE intentional legacy empty shape.** `empty_entities_in_file` collapses these states. Not a proven warm defect, but not a typed absent/unsupported distinction |

No row is labeled LIVE. Static construction of a path is not an executed cold/warm
counterexample. The production inconsistency warrants targeted RED evidence,
not permission to implement an unapproved global completeness policy.

## 6. Historical cases and existing tests

**TEST:** `workspace_query_csharp_cross_file_classification.rs` contains
`cold_reverse_interface_query_classifies_cross_file_implements`,
`cold_reverse_class_query_classifies_cross_file_extends`, and
`cold_forward_query_hydrates_target_before_classifying`. Current forward code
performs staged target hydration. The predecessor report's unrepaired cold
forward scenario is therefore historical for the tested filesystem lane.

**TEST:** `workspace_query_hydration_fidelity.rs` protects Angular Service High
requirements after Low projection. **TEST:** `discovery_cache.rs` separates root,
provider, mode, name and fidelity; publication-retry tests prevent failed selected
publication becoming completion. **TEST:** filesystem discovery tests cover
exclusions and deterministic multi-root scanning. **TEST:** batch preparation
protects a final index snapshot and fresh unpublished calls, but does not by that
fact prove every captured refinement catalog is final-snapshot consistent.

Existing Angular template and invocation live harnesses cover current source,
local linkage and composition. Their successful prior runs do not substitute for
fresh-server query-history permutations or CBM readiness falsification.

## 7. Evidence requirements

| Answer | Evidence needed before authoritative conclusion |
| --- | --- |
| exact declaration identity | Every eligible matching declaration in authorized candidate scope, supported producer and current source generation |
| reverse relation | Target identity plus all relevant asserting owners in scope, adequate producer fidelity, publication success, candidate discovery sufficient for that relation |
| C# typed base | Current neutral source fact, structured namespace/nesting/name/arity, target declarations, candidate completeness and uniqueness; preserve neutral fact on uncertainty |
| Angular selector resolution | Parent owner/HasTemplate, available supported template occurrence, every authorized matching HasSelector candidate, physical occurrence identity, ambiguity and completion |
| transitive closure | Relation-compatible current facts for each visited frontier up to depth, scope applied during traversal, closure acquisition or explicit partial result |
| cycle absence | All eligible dependency producers in declared scope or explicit indexed-only statement; one witnessed cycle is positive evidence, no witness is not complete absence |
| calls_in_file | One current supported file and exact owner/overload selection; cross-file target discovery is unnecessary |

Discovery requirements are not framework interpretations. Domain policy must
choose what constitutes relevant evidence; shared code may acquire/report that
scope without inventing Angular, C#, NgRx or runtime call meaning.

## 8. What completeness currently means

Discovery cache completion = provider/root/name/mode/optional fidelity/generation
operation completed and selected candidates published/current. HydrationReport
completion = aggregate string test. These are not equivalent to complete domain
candidate sets or complete transitive closure. Publication deduplication using
legacy file presence is weaker than required projection coverage.

Exact generic graph coverage = capability established from indexed family or
response evidence, permanently lower-bound/source-incomplete. Angular forward
Component and Template have a narrower source-completeness calculation; reverse
Component does not. Calls declare authoritative current canonical file snapshot.

One global boolean cannot truthfully combine these meanings. Current explicit
states include partial/failed internal discovery, partial/unavailable public
diagnostics, missing identity/capability, unsupported template, missing template,
and selector unique/unresolved/ambiguous/incomplete. C# resolution returns an
Option and preserves neutral facts; it does not expose all uncertainty states.
Zero candidates warrants unresolved only after sufficient candidate acquisition,
not after merely completing a different name search.

## 9. Discovery responsibility

Current architecture is already largely request-owned: handlers derive local
requirements and PreparationContext coordinates acquisition. Producers are
source-local; CBM supplies paths only; WorkspaceIndex stores compiled facts.
This ownership is appropriate. Problems arise where acquisition is less than
what later interpretation assumes, or where completion names the wrong unit.

Do not move candidate interpretation into providers, make producers recursively
hydrate peers, or equate CBM indexing with Clean-CTX semantic publication.

## 10. Models A–F

| Model | Assessment |
| --- | --- |
| A eager workspace truth | Not earned. Expensive startup/multi-root compilation, producer fidelity and refresh still need explicit contracts; conflicts with focused workflow |
| B request-owned discovery | Best preservation fit; already present. Must acquire sufficient requirements and distinguish discovery from projection adequacy |
| C shared planner/acquisition | Some mechanics already earned as PreparationContext/hydration. A new requirement language or generic resolver is not yet earned |
| D producer-owned discovery | Reject as default: peer IO/recursion and cross-domain lifecycle coupling would violate source-local authority |
| E explicit partial truth | Existing and truthful for index-only/lower-bound contracts; cannot justify unique refinement from incomplete candidate scope |
| F hybrid | Best current description: request acquisition for semantic questions, local file authority for calls, explicit partial indexed graph tools. Requires precise contracts, not universal eager work |

## 11. Multi-repository implications

Scope is primary root plus configured additional roots, narrowed by withinPath.
Discovery uses root/provider generations; one repository can be complete while
another remains pending. Current query-result provenance filters prevent an
unauthorized repository's asserting edges extending paths. Model C identities
exclude file; same-name physical occurrences are not automatically distinct
semantic tuples. Do not change identity to solve acquisition.

CBM configured projects are considered by provider discovery; actual compilation
is path-checked. Failed/out-of-bound candidates must not establish completion.
The cache does not encode a whole root-set certificate. Per-root completion can
be reused, but authorization/root-set/exclusion changes and request-local catalogs
must be reevaluated. Runtime config/root removal behavior beyond session reload
is NOT ESTABLISHED. Cross-root truth must be compared with the same declared root
set, not two differently scoped requests.

## 12. Batch-order implications

The final WorkspaceIndex guard prevents incremental reads between evaluations.
It does not defer catalog construction. Early item catalogs can miss evidence
acquired by later items, even while their stored edge read sees that evidence.
Ready file-local queries also capture their answer during preparation; fixed
source control avoids mistaking source mutation for batch-order effects.

A possible correction is acquire requirements before constructing domain
catalogs/evaluating. That is a hypothesis, not automatic authorization for a
planner rewrite. Test [Q,A], [A,Q], Q alone, A→Q, and failed A→Q separately.
For queries intentionally partial, batch's shared final graph can legitimately
expose more evidence than standalone; truthful coverage must remain explicit.

## 13. Frozen invocation boundary interaction

Discovery compiling a newly admitted TS/C# file naturally produces its private
structural evidence for the local consumer. No current catalog path requires
persisting occurrences; catalogs use durable neutral edges and session source
snapshots, Angular templates use their own domain occurrence extraction.
Invocation evidence should remain private and compilation-local. Its validated
59-case boundary does not prove completeness of a workspace candidate set.

## 14. Paused limited resolution extraction

Classification: **DISCOVERY MUST BE SOLVED FIRST.** The supplied predecessor
concepts alone are not a sufficient frozen API specification. C# and Angular
share acquire-before-refine mechanics but retain different domain authority,
ambiguity and source-support rules. A resolver extracted now could formalize
aggregate completion and subset uniqueness as if they were sound guarantees.
Correct the evidence requirement/completion boundary first, then reassess whether
shared resolution states are independently earned. Do not reopen invocation.

## 15. Preservation analysis

Retain WorkspaceIndex, asserting-file ownership, neutral HasBaseType, HasTemplate
and HasSelector producers, source identity/SHA binding, current scope rules,
existing publication tickets and discovery generations. Derived resolution stays
request-local. No second graph, persisted dependency-resolution state or generic
framework branch is justified. Index structure can store acquired facts; the
missing proof is sufficient acquisition, not inability to store them.

## 16. Performance and cost

Literal fallback traverses and reads every supported nonexcluded source for each
uncached name. A target/selector stage costs additional name scans; existing
request/session completion deduplication amortizes repeated requirements.
Compilation cost is candidates × required producer work, not every walked file.
Catalog building reparses cached C# snapshots and interprets known templates;
more indexed unrelated owners can increase refinement cost and candidate sets.

Correct requirement reuse can preserve cold correctness and warm speed. A batch
union may reduce repeated scans but must retain direction, root/provider,
fidelity and generation semantics. Multi-root costs sum over required roots.
Numeric latency/memory estimates are NOT ESTABLISHED; no benchmark was run.
No correctness-through-global-eager-compilation recommendation is made.

## 17. Candidate architecture, not approved implementation

Prefer the existing request boundary with three explicit obligations:
(1) all required provider scopes reached a sufficient readiness/completion state;
(2) every relevant candidate has a current adequate producer projection;
(3) interpretation uses the same post-acquisition evidence generation.

Local handlers keep requirement policy and interpretation. Existing shared
hydration/preparation owns acquisition and completion mechanics. Whether those
obligations require a new type, a local check, or staged evaluators depends on
RED observations. No new global state vocabulary or public coverage schema is
approved by this report.

## 18. Explicit non-goals

No production changes, new MCP tools, invocation persistence, target-call
resolution, WorkspaceIndex replacement, eager whole-workspace indexing,
watchers/TTL/background work, global cap/timeouts, CBM authority merger, session
statistics repair or global error-schema redesign. These remain distinct #142
workstreams unless evidence directly connects them to query truth.

## 19. Adversarial observation / RED matrix

Use existing PowerShell stdio initialization/capture/shutdown patterns. Every
history gets a fresh process over identical fixture bytes/configuration and its
own persistence DB. Reuse a process only within that history. Explicit CBM-off
and controlled CBM-ready/still-indexing/failure lanes are separate experiments.

| ID | Fixed source and Q | Histories / expected falsifier |
| --- | --- | --- |
| H1 | C# Derived→Base/interface typed edges | Q; target entities→Q; unrelated→Q; class/interface collision; neutral facts always retained |
| H2 | Reverse C# base | Separate declaration/referrer files; provider inbound-only vs filesystem; target warm must not manufacture candidate completeness |
| H3 | Semantic producer fidelity | Low publish→Q vs Q; High Q→Low publication→Q; same source/hash; adequate semantics must not disappear behind a completion hit |
| H4 | Angular Template selector | Parent/template/child in separate files; selector candidate absent/unique/ambiguous; Q vs parent/child/unrelated warming |
| H5 | Reverse Component use | Parent never names target class; warm parent → Q; evaluate lower-bound variability separately from false complete claims |
| H6 | Transitive A→B→C | Q(depth 2) vs B→Q; distinguish explicit subset from claimed closure |
| H7 | Cycle A↔B | cold and published subset/full graph; no-witness remains indexed-only unless acquisition contract changes |
| H8 | Batch catalogs | [Q,A], [A,Q], Q alone, A→Q; later identity/selector conflict; include failed sibling and compare catalog-derived facts |
| H9 | Multiple repositories | Same names/kinds/selectors in primary + additional root; warm each root separately; withinPath controls; root-set changes in new session |
| H10 | Lifecycle | edit/delete/add/restore after warm plus explicit refresh; restart with own durable DB; no historical artifact becomes source-current merely by restore |
| H11 | Identity/unsupported/failure | name-only no match with failed discovery; unsupported grammar/template; source read/publication failure; precise outcomes, no false empty success |
| H12 | Current calls control | TS/C# calls_in_file across unrelated warming; authoritative file/owner/arity/order stays equal, no global occurrence expectation |

Compare verbatim: written call/order/arity/spread, relation, entity kind/name,
physical/asserting provenance, selector span/ordinal, resolution/ambiguity states,
completeness/coverage, error classification and candidates. Normalize only known
request IDs, fixture-root spelling across equivalent copies, durations, and
explicit work counters (discovered/compiled/cache statistics). Preserve partial,
failed, unsupported, source_complete, omitted_possible and lower-bound flags.
Sort only semantically unordered sets; never call/selector occurrence order.
Keep exact content and structured captures and independently assess both.

## 20. Migration and lifecycle implications

First isolate REDs against the unfixed implementation. Then choose the smallest
local correction, using the repository RED→stash tests→fix→restore→GREEN rule.
Readiness/completion correction must preserve retries and zero-candidate success
only when sufficient discovery actually completed. Projection adequacy changes
must compose with invalidation and publication clocks. Batch correction must
preserve item failure isolation, ordering, shared scope and hydration dedup.

## 21. Compatibility and generations

A query-only acquisition/refinement repair does not automatically require changing
producer generations or Binary0x04. A changed public completeness/error contract
requires explicit approval. Durable producer changes require their existing
producer-generation discipline; nothing in this investigation authorizes one.
Do not introduce a global query generation without evidence. Restore/current
source handling must retain existing preflight and refresh authority.

## 22. Open questions

Which CBM readiness/search combinations occur in production? Does inbound CBM
include the target declaration by contract? Which non-Service relations have
fidelity-sensitive producers? Can an early batch catalog establish a uniqueness
that a later item invalidates? Does discovered-but-excluded or ignored explicit
publication legitimately participate in candidate completeness? What root/config
changes are supported without restart? Which partial APIs should the owner
upgrade to authoritative scope acquisition? These need focused observations.

## 23. Requirement-by-requirement recommendation check

Properties 1–8: preserve correctness, physical provenance, neutral syntax facts,
domain-owned interpretation, no sanitization/unknown-evidence shortcuts.
Properties 9–12: reuse source-local producers/index, no peer-producer recursion,
no framework branches in shared discovery, no persisted invocation records.
Properties 13–15: test authoritative history/batch independence, retain explicit
partial contracts, never derive authoritative absence from an unacquired subset.
All are design constraints here, not a claim the current system enforces every
one. The qualification in §2 is necessary for existing partial APIs.

## 24. Exact smallest next phase and decision gate

**Phase I: falsification only, no production implementation.** Add registered
tracked tests using existing CBM injection for H1–H3/H8 readiness/projection/
catalog cases, and a small live query-history driver using the existing
`McpSession.ps1` pattern for H1/H4/H8/H12. Save per-history request/response and
semantic comparison evidence under target. Start with readiness aggregate
truthfulness; it has the shortest code-proven mismatch and existing test hooks.
Only after observed RED choose a repair and obtain architectural approval if it
changes externally visible completeness policy.

Existing baseline commands for operator authorization (not run here):

```powershell
cargo test --locked -p clean-ctx --lib --all-features -j 4 workspace_query_csharp_cross_file_classification
cargo test --locked -p clean-ctx --lib --all-features -j 4 workspace_query_hydration_fidelity
cargo test --locked -p clean-ctx --lib --all-features -j 4 workspace_query_batch_preparation
cargo test --locked -p clean-ctx --lib --all-features -j 4 discovery_cache
pwsh -NoProfile -File ./verification/workspace-query/scripts/Verify-BatchQueriesLive.ps1
```

These establish existing focused baselines, not the whole new history matrix.
The new driver/regression commands must be concrete before execution authorization.
No build is assumed necessary while using the current unchanged binary; no full
repository suite may run locally. Broader implementation remains paused.

Primary verdict: **MORE EVIDENCE REQUIRED**.
Paused limited-resolution work: **DISCOVERY MUST BE SOLVED FIRST**.
