# ANG-R2 Phase A compatibility and governance proposal

Agent: **Agent MaxHeadRoom**. Proposed owner disposition; no grant submitted.

## Compatibility disposition

| Surface | Proposed disposition |
| --- | --- |
| Existing ResolvedBy membership and direction | Preserve one exact route/target support and all old default responses/counts/ordering. |
| Existing EntityRef equality, CanonicalEndpoint and EdgeKey | Preserve. No registration key, mapping span or new target identity injected into them. |
| Existing forward_edges/reverse_edges/name validation | Preserve, including old rejection of empty name. The new operation uses its own route_path field, which can be empty. |
| Public API | Add explicit workspace_query type resolver_registrations and only its direction/selector fields/result. Old operations remain unchanged. No default opt-in or generic canonical selector. |
| Public text versus structured output | New operation renders the same registration semantics/records in both; old content envelope remains version 1. |
| Semantic producer | Recommend **Angular semantic generation 78 -> 79** after approval/implementation because the persisted produced semantic result gains registration facts. Phase A remains 78. |
| Angular markers / canonical pipeline / CoreOps / Binary 0x04 | Preserve. No marker/canonical/wire meaning change is demonstrated. |
| Durable physical SQLite schema | Existing JSON semantic payload can carry an optional versioned typed collection; no SQL migration or global durable generation bump is currently demonstrated necessary. Verify, do not assume merely because serde works. |
| Non-Angular/feature-restricted projections | Preserve existing absence and behavior. No registration capability invented from None. |
| Old Angular snapshots | Existing producer-generation validation must refuse them as current generation-79 semantic production and reacquire normally. Preserve historical bytes/receipt status. |
| Generation-79 payload missing required applicable collection | Structural non-success, not successful empty registrations. Missing is different from a produced current empty list. |
| Unsupported collection format or forged/stale proof | Refuse before semantic admission/publication using existing compatibility/currentness categories as applicable; no reinterpretation from names or old edges. |

The generation change is proposed, not made. If concrete codec/API inspection
reveals a genuinely necessary broader change, stop for that exact decision.
Existing binary/canonical histories remain histories; no silent recertification.

## Lifecycle outcomes required in Phase B

| Transition | Required result |
| --- | --- |
| Initial/repeated acquisition | Current exact records plus unchanged membership; no duplicate records. |
| Warm replay/restart | Exact typed collection survives normal artifact/session/SQLite/replay with current source/witness/producer proof. |
| Key-only edit | Replace old collection even if rendered ResolvedBy is unchanged. New snapshot hash coordinate is expected; no cross-edit stable route identity. |
| Target edit, unchanged owner | Old dependency witness invalidates before consumption; exact new target coordinate appears only after normal reacquisition. |
| Mapping/field/Object/route removal | Retire only owned registration facts; retain independently valid siblings and other relation families. |
| Owner relocation | Old source-owned entry retires; newly authorized physical owner and source coordinates are acquired. Do not invent stable IDs across moves. |
| Root refresh/delete/reset | Registration data follows the same SemanticCoverage/snapshot owner retirement. |
| Failed acquisition/publication | No mixed-generation collection or stale success. Preserve existing ticket/failure rules; no new global transaction/race guarantee. |
| Stale/old payload | Normal compatibility/currentness refusal and reacquisition; no public registration reconstructed from an unqualified edge alone. |

Registration admission must reject extra, missing, crossed, borrowed or altered
records against the expected owner projection, not merely validate target existence.
The supplied immutable source versus on-disk dependency distinction remains intact.
Typed witnesses retain their current observation/validation semantics; the filesystem
does not become an atomic snapshot through this proposal.

## Active governed sources and protected components

The active registry is `docs/architecture/knowledge-governance/corpus.csv`, protected
selectors are in `protected.csv`; the existing actual profile scans `docs`.
The proposal is ordinary evidence under `verification`, not a new admitted source.

The accepted predecessor knowledge generation is
`4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3`.
Exact relevant revisions:

| Authority | Accepted revision | Relevant protected selectors |
| --- | --- | --- |
| AUTH.ANGULAR_RESOLVER | `6c796231ed5b92ed63a6bb4f8ed9f2bb85b2cfb03335d08b7507aaa958af8f1e` | RESOLVER-questions; B11-resolver-policy; standing/current-source/routing qualifications |
| AUTH.ANGULAR_CANONICAL_RELATIONS | `57d4e32a32f56b9e9fb182afe0821172c5cfc0b498d62d9044636fd4c8689adb` | B11-boundary-1 contract; B11-boundary-2 accepted scope; B11-boundary-3 standing |
| AUTH.ANGULAR_FOUNDATION | `e39c065d0de8897b36be94fbbbffa5fdb55d749bfb9d0649d2034cf571ee90a6` | Existing native identity/lifecycle/bounded-work/public-scope limits remain in force |
| AUTH.TS_RETAINED_EVIDENCE | `f50b18412798d5ed658a5826c70882d6d16dfaa1a5b9de531b50089d9c45d64c` | Existing transport/currentness and qualifications remain unchanged |
| AUTH.TOOL_WORKFLOW | `9cd976e168473b01c1fe256cf3fb4f86abe475a45889db6e964e8a20b12d8765` | TOOLING-selection tools statement; current-verification and antipatterns remain preserved |

Question-matrix/verification/performance authority is acquired, not weakened.
Document the new query in the protected TOOLING-selection tools statement without
rewriting unrelated historical checkpoints or claiming a new verified result.
Shared artifact hash change alone does not decide protection. No existing selector
or corpus owner is removed.

## Proposed controlling amendment content, for review

No text below is currently controlling and no protected source is edited yet.
After contract acceptance, prepare exact source-owned changes within existing
registered authorities, retaining historical generation-78 clauses and qualifications.
Recommended normative addition to the resolver's protected question/policy scope:

> The separately authorized written resolver registration projection retains each
> independently admitted (RouteOccurrence, decoded mapping key, exact resolver
> declaration) fact and its source-qualified field/Object/pair/key/value evidence.
> It supplements the existing unqualified ResolvedBy membership without changing
> that relationship's identity or deduplication. One shared Angular evaluation
> produces both; the source-owned semantic result, compatibility, session/durable
> snapshot, publication and index retirement carry them together. The explicit
> resolver_registrations query exposes bounded exact records in occurrence buckets;
> it establishes no callable eligibility, execution, navigation or effective DI.

The complete public contract in ANG_R2_PUBLIC_CONTRACT.md, including validation,
refusal, source nomination, missing-proof distinction and snapshot identity, is
the proposal to be normalized into the authorized controlling source; the short
paragraph above is not a license to omit those requirements.

Recommended bounded exception in canonical-relations contract/accepted scope:

> A separately approved resolver registration collection may accompany existing
> semantic edges in their existing source-owned lifecycle. Its explicit new query
> may expose the approved route/declaration snapshot coordinates and mapping
> evidence. Existing EntityRef equality, endpoint/edge identity, public responses
> and name-based query selectors retain their established meanings. This is not
> a general public canonical identity or generic evidence extension.

Add a source-owned typed dependency from the resolver authority to the canonical
relationship contract if the admitted new clause consumes that contract. Preserve
all existing dependency, guard preservation, historical evidence and qualification
associations. No broad foundation/TS/performance rewrite is proposed. Source grammar
and the Rust-computed descriptor, not this draft's assertion, determine the exact
candidate membership/association/protected-content delta.

The tooling addition must describe the new explicit operation, fixed direction/
selector fields, required root, asserting-source scope, lower-bound coverage,
source-coordinate meaning and lack of runtime semantics. Reference the accepted
resolver contract rather than copying a second normative definition. Existing
operations, batch semantics and old execution evidence remain unchanged.

## Required B11 sequence

1. Owner reviews and accepts or revises the exact Phase A contract.
2. Prepare the accepted controlling source changes; retain historical scopes and
   include all mandatory qualifications. Do not assert implementation or PASS.
3. Obtain the exact Rust-computed candidate descriptor using existing
   `pcg documents describe --profile <absolute existing profile>` workflow.
4. Present complete descriptor/predecessors/deltas to the owner. **Only the owner**
   submits the explicit exact-digest authorization. Contract acceptance is not a
   substitute for that procedural grant, and an agent never invokes authorize.
5. Normal knowledge reconciliation must admit the exact authorized candidate;
   verify current generation, relevant revisions/typed chains and retained history.
6. Begin production implementation only under that accepted controlling contract.

This is the existing mission §11/§12 and B11 workflow, not another approval system.
Phase A does not prepare a fictitious digest for future bytes, alter the corpus/
protected set, modify SQLite directly or claim governance admission for this proposal.

## Final disposition after owner disposition 01

This section supersedes the draft decision wording above. Architecture is accepted
in principle; the following exact candidate is prepared, not admitted or implemented.

| Compatibility boundary | Final bounded disposition |
| --- | --- |
| Semantic compatibility | Advance Angular semantic generation **78 -> 79** with implementation; current source remains 78. Existing generation checks invalidate old producer semantics, including otherwise current source bytes. |
| Durable format | Add optional versioned registration collection to existing semantic snapshot JSON and its retained/projection transport. Keep the existing physical SQLite schema, durable global generation, binary 0x04 and canonical/marker generations. No separate table/store. Version is **1**. |
| Old generation-78 payload with no collection | May remain historical/readable as old representation; cannot become current generation-79 Angular semantics. Normal owner/dependency reacquisition is required before registration success. No edge/name reconstruction. |
| Applicable generation-79 payload with no collection | Invalid/incomplete current production. Reacquire; if required proof/acquisition fails, non-success. A deserialization default of None is not semantic acceptance. |
| Current produced empty collection | Valid only after complete expected-owner projection validation. May contain proved omissions; zero records is not source completeness. |
| Unknown version, malformed or forged/crossed/extra/missing records | Existing compatibility/admission refusal, then normal reacquisition if applicable; never silently drop bad records and report empty success. |
| Non-Angular/non-applicable feature mode | Preserve prior absence and behavior; no invented collection or capability. A missing applicable producer cannot serve registration success. |
| Restart and warm reuse | Same source/hash, generation, dependency/package/project witnesses and record proof must survive durable restore. No new freshness shortcut. |
| Public compatibility | Add resolver_registrations only, operation format_version 1, unchanged global envelope/old edge schemas, selectors, identity/dedup/order. Exact optional attribution fields and both omission arrays are in the new output schema only. |

No SQL migration is selected: the existing durable JSON snapshot is the extension
point, and producer compatibility handles the changed semantic meaning. This is a
bounded architectural disposition, not a demonstrated codec round-trip. Phase B
must verify the exact old/missing/empty/unknown-version matrix through actual SQLite,
not just serde. Any genuinely necessary wider format/API change returns to the owner.

### Exact controlling amendments prepared

| Owner / existing protected selection | Exact candidate effect |
| --- | --- |
| AUTH.ANGULAR_RESOLVER: statement and verification_obligation:public_keys_and_current_producer, both RESOLVER-questions | Append the complete prospective registration contract inside the existing selected boundary: identity, records, omission attribution, cold nomination, API, compatibility, shared lifecycle, refusal, performance and required verification. Explicitly qualify earlier OPEN wording as the preserved generation-78 checkpoint. |
| AUTH.ANGULAR_CANONICAL_RELATIONS: statement:contract B11-boundary-1 and qualification:accepted_scope B11-boundary-2 | Add the additive collection/query snapshot-coordinate exception, preserving all old equality/edge/public meanings and other limits. Exact resolver contract remains source-owned by resolver authority. |
| AUTH.TOOL_WORKFLOW: statement:tools TOOLING-selection | Add the prospective explicit operation and its selector/scope/coverage limits, referring to resolver authority. Do not recertify historical results. |
| Resolver governing relationships | Add DEPENDS_ON AUTH.ANGULAR_CANONICAL_RELATIONS. Preserve existing TS dependency, guard preservation and DOC-228 evidence association. |
| Tooling governing relationships | Add DEPENDS_ON AUTH.ANGULAR_RESOLVER. Preserve every existing invariant preservation relationship. |
| Corpus / protected membership | **Unchanged**, including selector names and all required qualifications. No new source owner, identity, protection removal or registry change. Existing selected boundaries protect the appended operative clauses. |

Foundation, neutral TS, shared query scope, verification and performance authorities
are unchanged. Original historical wording, receipts and qualification text remain.
Protection compares exact selected bytes and associations, not a semantic-equivalence
assertion. The candidate is in the three actual registered source files, not merely
a disconnected patch. No candidate is called governing until normal admission.

### Descriptor boundary

Accepted predecessor generation and five authority revisions are pinned above.
ANG_R2_CANDIDATE_PINS.json additionally binds exact HEAD predecessor/candidate
source hashes, unchanged corpus/protection files, executable and actual-root profile.
These are static byte pins, **not the Rust governance descriptor or authorization**.

The complete Rust descriptor and its exact digest are **NOT YET CAPTURED**: owner
disposition 01 grants no runtime invocation permission. Run the bounded operator
capture in ANG_R2_GOVERNANCE_HANDOFF.md; return descriptor.json before authorizing.
Do not substitute a hash of the patch, manifest or this document. Describe must bind
the actual accepted predecessor, membership/protection/state and candidate view.
No owner grant is generated or submitted by Agent MaxHeadRoom. Phase B stays held.

## Current implementation status after governed admission

The source constant is now Angular semantic generation 79. Complete index/session/
durable admission is wired to the version-1 collection; no physical SQL schema,
marker/canonical generation, old edge schema or global query envelope change was
made. Current applicable missing support and unknown/forged collections refuse
through existing admission/compatibility categories. The generation-78 simulation
and missing-generation-79 collection use separate actual-SQLite tests, not merely
serde round-trips. All current-source compatibility execution is still pending.

Producer scope is runtime metadata from the actual configured evaluation, skipped
by serde and rebound from the existing source applicability predicate after decode.
Unknown scope cannot grant registration eligibility. A conservative producer hint
does not force unrelated decorated TS into this producer's collection requirement.
This adds no serialized scope flag or new authority boundary. Registered public
schemas expose only the approved new operation's exact records and diagnostics.
