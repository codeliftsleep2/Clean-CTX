# Coverage Epistemic Detail Architecture Design 1

**Date:** 2026-10-18  
**Architect:** Agent MaxHeadRoom  
**Repository:** `codeliftsleep2/Clean-CTX`  
**Authoritative branch:** `(fix)Performance-Regression-Issues`  
**Phase:** Architecture design only; no production implementation authorized.

## 1. Executive decision

**Final decision: DESIGN C — SEPARATE REQUEST-LOCAL ACQUISITION OUTCOME.**

**Legacy completeness semantics changed: NO.**

**Production implementation authorized: NO.**

Keep `AcquisitionSatisfaction` as the narrow mathematical authority for the
question "is every required unit proven?" Compose it with separately retained
proof and processing dispositions in one request-local `AcquisitionOutcome`.
Project a small additive `coverage_detail` object from that outcome.

This ownership preserves three independent truths:

```text
returned positive facts are authoritative observations
exhaustive-negative authority is earned only by CompletenessCertificate
operational/proof limitations explain why exhaustive authority is absent
```

The first tracked RED boundary is the existing mixed-language regression
`src/tests/mcp/workspace_query_reverse_edge_completeness.rs::
issue_144_unproved_language_unit_remains_lower_bound`. Extend it, before any
production change, to require unchanged legacy lower-bound fields plus bounded
detail identifying terminal processing, an uncertified predicate, authoritative
positive facts, and `change_required` retryability.

## 2. Problem statement

The legacy coverage tuple correctly refuses exactness whenever any required
coverage unit lacks proof:

```text
source_complete=false
result_semantics=lower_bound
omitted_possible=true
```

It does not tell an LLM why. Known failed work, incomplete inventory, an
unsupported proof predicate, and an index-only query can all appear as a lower
bound. Those cases demand different next actions.

The design must explain epistemic state without changing it. In particular:

- `207 discovered == 207 compiled` is never completeness proof;
- terminal selected processing is not exhaustive source proof;
- TypeScript remains a required unit for the cross-language TypeRef capability;
- provider state remains discovery information, not semantic authority; and
- no new metadata may manufacture absence authority.

## 3. Authority boundaries

Four authorities remain separate.

### 3.1 Process authority

Inventory traversal and candidate processing can establish whether the work
Clean selected reached terminal success, encountered known failures, or is not
represented precisely enough to classify.

### 3.2 Positive semantic authority

Published `SemanticEdge` values and their provenance establish that the
returned relationships exist. Lower-bound coverage does not revoke this
authority.

### 3.3 Coverage-proof authority

Validated per-unit acquisition proofs establish that the candidate-selection
and source universe are exhaustive for the requested capability and generation.

### 3.4 Exhaustive-negative authority

Only `CompletenessCertificate` establishes that no additional matching facts
exist in the declared scope. It alone authorizes `source_complete=true` and
`result_semantics=exact`.

Process completion and semantic interpretation remain independent. The new
model describes their conjunction without collapsing either into the other.

## 4. Current information-loss boundaries

The first loss occurs in `prepare_clean_type_ref_acquisition`:

```text
clean_type_ref_proof_claim -> Result<Claim, CleanProofRejection>
validate_proof_claim      -> Result<ValidatedProof, ProofRejection>
filter_map(... .ok()?)    -> successful proofs only
```

Typed distinctions such as `IncompleteInventory` and
`UncertifiedPredicate` disappear.

The second loss occurs in candidate execution:

```text
CandidatePublication::{Published, PublishedCSharp, Current, Failed}
  -> compiled count + publication_failed bool
```

Known candidate failure, source race, admission failure, and publication
supersession are no longer distinguishable. The response can safely withhold a
certificate but cannot explain retryability.

## 5. Architecture options

| Criterion | Design A: hydration aggregate | Design B: satisfaction ledger | Design C: separate outcome |
|---|---|---|---|
| Semantic cohesion | Convenient projection point, but mixes discovery, execution, and proof | Strong unit accounting, but pollutes mathematical proof authority with diagnostics | Keeps proof authority narrow and composes process/proof facts explicitly |
| Authority ownership | `HydrationReport` becomes too authoritative | `AcquisitionSatisfaction` would mean both proof and failure history | `AcquisitionOutcome` owns request epistemics; satisfaction still owns completeness |
| Diagnostic contamination | High | High | Low |
| Cache implications | Tempts caching transient failures with hydration | Tempts caching negative dispositions with proof satisfaction | Defaults entire diagnostic outcome to request-local; cache stores only complete satisfaction |
| Hot-path cost | Low, but coarse | O(units) | O(units + fixed reason vocabulary) |
| Batch composition | Aggregate can be accidentally shared too broadly | Ledger keys help, but satisfaction reuse may contaminate items | Per-hydration-key outcome naturally remains item-specific |
| Future capabilities | Risks generic hydration framework | Risks generic proof/failure framework | Capability-specific outcome can grow only when earned |
| Testability | Easy projection, weak causality | Strong proof tests, awkward processing tests | Direct unit and end-to-end tests for each axis |
| Framework branching risk | High in `HydrationReport` | Medium in satisfaction | Low if first implementation stays TypeRef-specific internally |

Design C is the smallest model that preserves ownership correctly.

## 6. Chosen internal state model

Introduce a request-local result beside, not inside, mathematical satisfaction:

```rust
struct AcquisitionOutcome {
    satisfaction: AcquisitionSatisfaction,
    proof_dispositions: BTreeMap<CoverageUnit, CoverageProofDisposition>,
    processing: ProcessingSummary,
}
```

Every required unit must have exactly one `CoverageProofDisposition` when an
`AcquisitionOutcome` is produced. This is structurally enforced by constructing
the map from `AcquisitionRequirement::units`, initially as unassessed, then
replacing each value during proof construction. Before return, unassessed is
converted to an internal indeterminate disposition; it must never be silently
omitted.

Recommended internal types:

```rust
enum CoverageProofDisposition {
    Proven,
    Unproven(ProofBarrier),
    Indeterminate,
}

enum ProofBarrier {
    InventoryIncomplete,
    CandidatePredicateUncertified,
    ClaimIncompatible(ProofCompatibilityBarrier),
}

enum ProofCompatibilityBarrier {
    Unit,
    Target,
    Scope,
    InventoryGeneration,
    AdmissionPolicy,
    PredicateVersion,
    ContractVersion,
    Exhaustiveness,
    Truncation,
    Limitation,
}
```

Raw internal rejection enums are mapped to this stable internal model; they are
not serialized directly.

Processing remains orthogonal:

```rust
struct ProcessingSummary {
    inventory: InventoryProcessing,
    candidates: CandidateProcessing,
    publication: PublicationProcessing,
}

enum InventoryProcessing { Complete, Incomplete, Indeterminate }
enum CandidateProcessing { Terminal, Failed, NotAttempted, Indeterminate }
enum PublicationProcessing { Terminal, Failed, NotRequired, Indeterminate }
```

Counts may remain in `HydrationReport`; owner lists do not enter this summary.

## 7. Rejected internal models

### 7.1 One combined state enum

A single enum such as `ObservedCompleteUncertified` cannot faithfully represent
mixed states—for example, one structural TypeScript barrier plus one failed C#
publication. Proof and processing are orthogonal.

### 7.2 Expanding AcquisitionSatisfaction

`AcquisitionSatisfaction` answers a precise set relation: required minus proven.
Adding transient execution failures would make cached mathematical satisfaction
depend on request history and weaken its authority.

### 7.3 HydrationReport as the source of truth

Hydration owns provider/discovery orchestration. Reconstructing proof barriers
there from status strings or counts repeats the current information loss.

### 7.4 Public raw rejection enums

Internal variants may evolve with implementation. Public API requires stable,
bounded authority categories, not Rust debugging detail.

## 8. Required type changes

No existing public or certificate type is removed.

Minimal proposed changes:

1. `ProvisionalAcquisition` gains one disposition per required unit instead of
   successful proofs alone.
2. Proof-building helpers return or preserve typed rejection values.
3. Candidate execution returns a bounded `ProcessingSummary` alongside existing
   publication outcomes/counts.
4. `ProvisionalAcquisition::finish` returns `AcquisitionOutcome`.
5. `HydrationReport` carries `Option<AcquisitionOutcome>` rather than only
   `Option<AcquisitionSatisfaction>`, with an accessor for satisfaction.
6. `exact_identity_coverage` uses satisfaction exactly as today for certificate
   construction and uses the outcome only for additive detail.

The completion cache continues to store only source-complete
`AcquisitionSatisfaction`, never transient `AcquisitionOutcome`.

## 9. Data-flow changes

| Boundary | Current information | Current loss | Proposed handoff | Owner/lifetime | Authority/cache/public status |
|---|---|---|---|---|---|
| `InventoryManifest` → requirement | Entries, digest, attempted, complete | None yet | Unchanged manifest; initialize unit ledger | Acquisition; request-local | Inventory authority; not public |
| Requirement → claim | Exact units and target | None | Unchanged | Acquisition; request-local | Proof obligation |
| `clean_type_ref_proof_claim` | Typed `CleanProofRejection` | Rejection discarded | Map rejection into unit disposition | Acquisition outcome; request-local | Explains proof absence; no negative authority |
| `validate_proof_claim` | Typed `ProofRejection` | Rejection discarded | Map into compatibility barrier | Acquisition outcome; request-local | Compatibility fact; not cached |
| Provisional acquisition | Requirement + proofs | No complete ledger | Requirement + proof ledger + validated proofs | Acquisition; request-local | Internal only |
| Candidate processing | Per-candidate outcomes | Collapsed to count/bool | Aggregate bounded processing summary while iterating | Execution; request-local | Operational fact; not cached |
| Publication → satisfaction | One global complete/failed status | Mixed causes collapse | Record proven units only if publication terminal; preserve processing separately | Outcome | Satisfaction retains strong meaning |
| Outcome → HydrationReport | Satisfaction only | No reason/retryability | Carry request-local outcome | Query preparation | Internal; clone cost O(units) |
| Coverage projection | Certificate or lower bound | All lower bounds look alike | Add bounded `coverage_detail` | Query coverage | Public additive API |

## 10. Proof-disposition semantics

`Proven` means a validated proof exists for the exact required unit and the
required publication condition succeeded. It may contribute to
`completeness_generation()`.

`Unproven(InventoryIncomplete)` means Clean knows the inventory could not
establish a complete source universe. It is operationally incomplete and an
identical retry may help.

`Unproven(CandidatePredicateUncertified)` means the inventory may be complete,
but Clean lacks a certified narrowing predicate for that producer. Repeating
unchanged source/scope/code cannot create proof.

`Unproven(ClaimIncompatible(...))` means a claim exists but does not match the
current requirement/context. Retryability depends on the category:

- inventory/admission generation races may be retryable;
- predicate/contract capability mismatch requires a structural/version change;
- non-exhaustive/truncated/limited claims require a changed acquisition method
  or policy;
- unknown combinations remain indeterminate.

`Indeterminate` is required so future or currently unmodeled failures do not get
misclassified as structural or retryable. It never contributes proof.

## 11. Processing-disposition semantics

Inventory processing is `Complete` only when traversal was attempted and every
supported enumerated owner was successfully classified/read. It is
`Incomplete` when current code proves traversal/read incompleteness.

Candidate processing is `Terminal` only when every selected candidate ended as
`Current`, `Published`, or `PublishedCSharp`. Any `Failed` yields `Failed`.
Empty selection after a complete certified predicate is terminal; empty
selection after an uncertified predicate says nothing about exhaustiveness.

Publication processing is `Terminal` when every required selected candidate is
current or committed successfully. A source race, compilation failure,
publication-ticket supersession, or commit failure is `Failed`. Where current
code cannot distinguish preparation failure from publication failure, use
`Indeterminate` until the execution result type is refined; do not fabricate
precision.

Cancellation, deadline, and budget statuses are not added in this slice. The
repository lacks end-to-end execution-control authority. Future variants are
reserved conceptually, not serialized now.

## 12. Retryability derivation

Public retryability is derived only from retained typed state:

```text
not_needed
same_request_may_help
change_required
indeterminate
```

Precedence is conservative:

1. authoritative complete → `not_needed`;
2. any known operational failure/incomplete inventory →
   `same_request_may_help` unless mixed with an unresolved indeterminate state;
3. only structural proof barriers with terminal processing → `change_required`;
4. mixed operational + structural barriers → `same_request_may_help` because a
   retry can improve part of the state, though not earn completeness;
5. unsupported/index-only families or unmodeled causes → `indeterminate`.

`same_request_may_help` does not predict success. It means repetition against
the current contract can repair known transient work. `change_required` means
unchanged repetition cannot remove every current barrier; source scope,
capability implementation, proof contract, or policy must change.

## 13. Positive vs exhaustive-negative authority

Existing edge provenance is the underlying authority for positive facts, but
it is not sufficiently obvious to an LLM when paired with `lower_bound`.
Therefore the additive detail should include one aggregate field:

```text
fact_authority = authoritative_observed
```

This means only that serialized semantic facts were produced by Clean's
authoritative current evaluation path. It does not certify completeness, target
resolution beyond the edge's own contract, or the truth of absent facts.

No separate `absence_authority` field is needed. It would duplicate
`source_complete`/`result_semantics`. Exactness remains the sole public absence
authority signal.

For query families whose positive facts are merely indexed witnesses, the same
value is valid: the observed fact is authoritative as an indexed fact, while
the set remains open-world. If any future surface returns speculative facts, it
must use a different authority contract rather than reusing this value.

## 14. Public coverage_detail contract

Add `coverage_detail` inside existing `coverage` only where an exact-identity
coverage object is already emitted:

```json
{
  "coverage": {
    "source_complete": false,
    "result_semantics": "lower_bound",
    "omitted_possible": true,
    "coverage_detail": {
      "processing_state": "terminal",
      "proof_state": "structurally_unproven",
      "fact_authority": "authoritative_observed",
      "retryability": "change_required",
      "residual_unit_count": 1,
      "reasons": ["candidate_predicate_uncertified"]
    }
  }
}
```

Stable enums:

```text
processing_state:
  terminal | incomplete | not_attempted | indeterminate

proof_state:
  complete | structurally_unproven | operationally_unproven |
  mixed_unproven | unsupported | indeterminate

fact_authority:
  authoritative_observed

retryability:
  not_needed | same_request_may_help | change_required | indeterminate

reasons (fixed vocabulary, deterministic order):
  inventory_incomplete
  candidate_predicate_uncertified
  proof_incompatible
  candidate_processing_failed
  publication_failed
  capability_not_supported
  processing_not_attempted
  indeterminate
```

`reasons` is a set of categories, never one entry per unit/file. Its maximum
length is the vocabulary size. `residual_unit_count` counts required units not
proven; it exposes no root, language, owner, or provider. For unsupported
query families without an acquisition requirement, omit the count rather than
inventing zero.

`proof_state=complete` is emitted only with a certificate. Detail never
strengthens the legacy tuple.

## 15. Legacy-field compatibility matrix

| Condition | `source_complete` | `result_semantics` | `omitted_possible` | Additive explanation |
|---|---:|---|---:|---|
| Authoritative complete | true | exact | false | terminal, proof complete, retry not needed |
| Structural proof limitation | false | lower_bound | true | terminal, structurally unproven, change required |
| Known operational incompleteness | false | lower_bound | true | incomplete, operationally unproven, retry may help |
| Mixed barriers | false | lower_bound | true | incomplete/mixed, retry may help |
| Unsupported/index-only | false | lower_bound | true | unsupported or indeterminate |

No legacy field is renamed, removed, or reinterpreted.

## 16. Discovery/coverage ownership boundary

`discovery` answers how paths were sought:

- provider(s) used;
- fallback reason;
- partial/unavailable discovery;
- discovered/compiled counts; and
- exceptional provider-project states.

`coverage_detail` answers why semantic exhaustiveness was or was not earned:

- whether selected processing terminated;
- whether required units were proven;
- whether the barrier is structural, operational, mixed, or unknown; and
- whether repeating the same request can improve the state.

Provider readiness remains only in discovery. `coverage_detail.reasons` uses
provider-neutral authority facts. The two objects may coexist but cannot derive
claims from one another.

## 17. Cache and generation semantics

`AcquisitionOutcome`, dispositions, and processing summaries are request-local.
They are not persisted or inserted into `AcquisitionCompletionCache`.

The existing cache continues to accept only source-complete
`AcquisitionSatisfaction`. Its key remains target, roots, admission policy,
producer generations, predicate version, proof contract, and evidence
requirement.

On a valid complete cache hit, construct a request-local outcome with proof
complete and processing terminal-for-reused-authority; do not replay historical
work diagnostics.

Negative dispositions are never cached:

- `UncertifiedPredicate` must be reevaluated after producer/predicate-version
  changes;
- publication failure must not poison a later request; and
- inventory incompleteness must be allowed to recover.

If negative caching is proposed later, it requires a separate architectural
decision and a compatibility key including every causal dependency.

## 18. Provider neutrality

Equivalent Clean-authoritative evidence yields identical detail whether CBM is
ready, unavailable, initializing, or disabled.

CBM can affect discovery cost and candidate hints. It cannot change:

- required units;
- Clean proof validation;
- fact authority;
- certificate eligibility; or
- proof-state categories.

No public reason contains `cbm`, provider project identifiers, readiness, or
transport status. Those remain in `discovery`.

## 19. Batch semantics

`PreparationContext` already caches hydration by a key containing direction,
name, fidelity requirement, and optional acquisition target. A batch item may
reuse an equivalent request-local `AcquisitionOutcome`; non-equivalent items
must receive separate outcomes.

Each prepared item captures the outcome associated with its own acquisition
requirement and projects its own `coverage_detail`. Shared filesystem traversal
or compilation may improve economics, but proof dispositions are evaluated
against each item's target, unit set, scope, and requirement fingerprint.

One item's structural barrier cannot contaminate a sibling with a different
target/requirement. Batch output preserves ordered item-local detail under each
`result`. A failed item never supplies detail for another.

## 20. Warm-state equivalence

Equivalent cold and warm requests over compatible source, scope, producer
generations, predicate version, and requirement must produce equivalent:

- legacy completeness tuple;
- proof state;
- fact authority; and
- retryability.

Processing economics may differ. To avoid truth varying with work reuse,
`processing_state=terminal` describes whether adequate selected evidence is
terminal for this request, not whether files were newly compiled. `Current` and
successful publication are semantically equivalent terminal outcomes.

Warm state cannot turn an uncertified predicate into proof. A recovered
transient failure must not remain sticky because negative outcomes are not
cached.

## 21. Performance/boundedness analysis

The design consumes facts already produced during inventory, proof validation,
candidate execution, and publication.

Complexity:

```text
internal aggregation: O(required coverage units + selected candidate outcomes)
public projection:    O(required coverage units + fixed reason vocabulary)
response size:        O(fixed vocabulary)
```

The unit ledger stores compact enums and existing `CoverageUnit` keys. It does
not clone source text or owner lists beyond existing proof structures. Public
projection aggregates categories and a count; it never serializes the ledger.

The design adds no traversal, compilation, source read, reconciliation,
provider wait, global invalidation, or per-edge work.

## 22. Normative scenario table

| Condition | Processing | Proof | Positive facts | Absence authority | Retryability | Legacy tuple |
|---|---|---|---|---|---|---|
| Complete C# proof | terminal | complete | authoritative observed | established | not needed | exact / complete / no omission |
| Mixed C#/TS `UncertifiedPredicate` | terminal | structurally unproven | authoritative observed | not established | change required | lower-bound / incomplete / omission possible |
| Incomplete inventory | incomplete | operationally unproven | authoritative observed facts only | not established | same request may help | lower-bound tuple |
| Candidate compilation failure | incomplete | operationally unproven | successfully published facts authoritative | not established | same request may help | lower-bound tuple |
| Publication failure | incomplete | operationally unproven | retained prior/current facts authoritative; failed candidate contributes none | not established | same request may help | lower-bound tuple |
| Proof compatibility/version failure | terminal if processing terminal; otherwise indeterminate | structurally unproven or indeterminate by barrier | authoritative observed | not established | change required or indeterminate | lower-bound tuple |
| Provider unavailable + complete Clean fallback | terminal | complete | authoritative observed | established | not needed | exact tuple; discovery reports fallback |
| Exact authoritative zero | terminal | complete | authoritative observed empty set | established | not needed | exact tuple with count zero |
| Unsupported/index-only family | not attempted or indeterminate | unsupported | authoritative indexed observations | not established | indeterminate | lower-bound tuple |
| Current cancellation/budget state | indeterminate | indeterminate | any returned facts retain their own authority | not established | indeterminate | no fabricated specialized state |

## 23. Proposed architectural invariants

Add after implementation and verification, not during design:

### WSC-015 Positive and Exhaustive Authority Are Independent

Returned current semantic facts retain their positive authority when exhaustive
coverage is unproven. Only an earned completeness certificate authorizes the
absence of additional facts.

### WSC-016 Epistemic Cause Is Preserved When Known

When Clean possesses typed evidence distinguishing operational incompleteness
from structural proof unavailability, that distinction survives request-local
acquisition through public bounded coverage detail. Unknown causes remain
indeterminate rather than guessed.

### WSC-017 Retryability Is Derived

Retryability is derived only from retained typed processing and proof state. It
is never inferred from lower-bound status, candidate counts, provider readiness,
or discovery completion alone.

### WSC-018 Detail Cannot Strengthen Completeness

Additive epistemic detail may explain the legacy tuple but cannot authorize
`source_complete=true`, `exact`, or `omitted_possible=false`.

## 24. Required RED regressions

All tests are tracked under `src/tests/**` and follow the repository's operator-
run RED/GREEN procedure.

| Regression | Existing/new location | RED assertion before implementation |
|---|---|---|
| Complete C# proof detail | `workspace_query_reverse_edge_completeness.rs` | exact tuple unchanged; terminal/complete/not-needed detail present |
| Mixed TypeScript structural barrier | same file; extend `issue_144_unproved_language_unit_remains_lower_bound` | lower-bound unchanged; structural reason and change-required retry |
| Incomplete inventory | `acquisition_hydration.rs` plus response case in reverse completeness file | operationally unproven and retryable |
| Candidate compile/source-race failure | `workspace_query_hydration_publication_race.rs` or existing publication-race authority file | processing failure is not labeled structural |
| Publication failure | extend existing `issue_144_publication_failure_remains_lower_bound` | publication reason and retryable state |
| Provider unavailable + Clean proof | `workspace_query_provider_readiness.rs` or reverse completeness file | exact detail independent of provider diagnostic |
| Authoritative zero | extend `issue_144_complete_zero_result_is_exact` | same complete detail as positive exact result |
| Cold/warm equivalence | extend existing cold/warm Issue #144 regression | complete detail equality |
| Singular/batch equivalence | `workspace_query_batch.rs` | per-item detail equals singular result |
| Deterministic bounded aggregation | new focused tests in `acquisition_proof.rs` or `acquisition_requirement.rs` | fixed order, unique reasons, bounded vocabulary |
| Schema serialization | `workspace_query_exact_identity_coverage.rs` | schema declares exact enums and optional count |
| Model-facing envelope | `workspace_query_content.rs` | structured detail survives unchanged in content result |
| Legacy compatibility | reverse completeness and content files | all old fields byte/value compatible when detail ignored |

Do not create a live harness as primary regression authority.

## 25. Migration/schema impact

The MCP output schema gains optional `coverage.coverage_detail` with closed
properties and enums. Existing consumers that ignore unknown additive fields
retain behavior.

Model-facing content already mirrors structured coverage. The new object must
survive that path without a second renderer-specific interpretation.

Documentation updates after implementation:

- `docs/agent/tooling.md`;
- `docs/CLAUDE_INTEGRATION_RULES.md`;
- `docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md`;
- `docs/ARCHITECTURAL_INVARIANTS.md`; and
- the `workspace_query` MCP schema descriptions.

No input schema, query semantics, completeness certificate, or provider
contract changes.

## 26. Smallest implementation slices

1. Add the first tracked RED contract regression for mixed C#/TypeScript detail;
   user runs the focused command and reports intended RED.
2. Introduce typed proof dispositions and preserve all claim/validation reasons
   in `ProvisionalAcquisition`.
3. Introduce typed processing summary while retaining existing counts and
   failure behavior.
4. Add request-local `AcquisitionOutcome`; keep completion cache restricted to
   complete `AcquisitionSatisfaction`.
5. Derive bounded aggregate `CoverageDetail` and retryability from the outcome.
6. Project the additive object from `exact_identity_coverage` without changing
   certificate or legacy fields.
7. Update output schema and model-facing contract tests.
8. Add operational failure, authoritative-zero, provider-independence,
   cold/warm, batch, boundedness, and compatibility regressions.
9. Update architectural/tooling documentation and invariants.
10. Hand off the authoritative final verification gate; do not claim it passed
    until the user reports results.

Each slice preserves existing semantic tests before the next begins. The exact
RED test is stashed/restored around production implementation per repository
policy.

## 27. Explicit non-goals

- Changing `lower_bound` to `observed_complete`.
- Granting exactness after terminal processing.
- Using discovered/compiled equality as proof.
- Treating an uncertified predicate as evidence of absence.
- Removing TypeScript units based on observed C# results.
- Changing or bypassing `CompletenessCertificate`.
- Exposing raw rejection enums, roots, files, source, or provider internals.
- Predicting retry success.
- Inventing cancellation, deadline, or budget facts.
- Negative-caching transient or structural dispositions.
- Recomputing acquisition for diagnostics.
- Adding a generic multi-capability framework before another capability earns
  it.
- Global cache invalidation, workspace recompilation, or provider blocking.

## 28. Final recommendation

Approve **Design C — Separate Request-Local Acquisition Outcome** for a future
implementation phase.

Keep mathematical satisfaction and the completeness certificate unchanged.
Retain typed proof barriers and processing outcomes in one request-local
composition, then emit a bounded additive `coverage_detail` that tells an LLM:

- returned facts are authoritative observations;
- exhaustive authority is established or absent;
- known work is terminal, incomplete, unsupported, or indeterminate; and
- repeating the same request is unnecessary, may help, requires a structural
  change, or cannot yet be classified.

The first RED boundary is the existing mixed-language TypeRef completeness
test. The smallest ordered sequence is: preserve proof reason → preserve
processing state → compose request-local outcome → derive bounded detail →
project schema/content → complete equivalence and failure regressions.

Legacy completeness semantics remain unchanged, and no production
implementation is authorized by this design.

