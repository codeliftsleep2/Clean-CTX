# Coverage Semantics / Epistemic State Investigation

**Date:** 2026-10-18  
**Investigator:** Agent MaxHeadRoom  
**Scope:** Static repository investigation only. No Cargo command, build, test,
Clippy, binary, MCP runtime session, or live verification was run.

## 1. Executive verdict

**Verdict: CURRENT MODEL CONFLATES KNOWN INCOMPLETENESS WITH UNPROVEN
EXHAUSTIVENESS.**

**Should original Issue #144 remain closed? YES.**

**Should a new coverage-semantics issue be opened? YES.**

The existing fields are truthful under their documented strong contract:

```text
source_complete = false
result_semantics = lower_bound
omitted_possible = true
```

correctly mean that Clean-CTX lacks authority to claim that the returned set is
exhaustive. The mixed C#/TypeScript query must retain those values. Nothing in
this investigation justifies granting a completeness certificate, removing the
TypeScript unit, or inferring exactness from `207 discovered == 207 compiled`.

The model is nevertheless materially under-expressive for an LLM consumer. It
projects the same public state for:

1. operationally incomplete acquisition or publication, where known work
   remains and an identical retry may help; and
2. terminal processing of all selected evidence with a structural proof
   limitation, where no known candidate work remains and an identical retry
   cannot certify exhaustiveness.

Clean briefly possesses enough typed information to distinguish these cases.
It then discards that information while constructing provisional acquisition.
The first loss is not in `exact_identity_coverage`; it occurs when typed
`CleanProofRejection` and `ProofRejection` values are converted to absence by
`filter_map(... .ok()?)`.

**EARLIEST EPISTEMIC INFORMATION LOSS:**
`prepare_clean_type_ref_acquisition` discards each coverage unit's typed proof
failure reason and retains only successfully validated proofs.

The smallest justified improvement is additive: retain a bounded per-unit
proof disposition internally, derive an aggregate acquisition/proof/retryability
detail, and expose it alongside—not instead of—the existing strong fields.

## 2. Existing public coverage contract

For exact edge queries, `exact_identity_coverage` currently exposes:

- `status`;
- `identity_indexed`;
- `capability_established`;
- `source_complete`;
- `result_semantics`;
- `omitted_possible`;
- occasionally `discovered_not_compiled_this_cycle`; and
- occasionally an `alternative_query`.

An earned `CompletenessCertificate` changes the central tuple to:

```json
{
  "status": "authoritative_complete",
  "source_complete": true,
  "result_semantics": "exact",
  "omitted_possible": false
}
```

Otherwise the tuple is:

```json
{
  "source_complete": false,
  "result_semantics": "lower_bound",
  "omitted_possible": true
}
```

Documentation establishes the intended meaning as epistemic rather than a
claim that returned facts are invalid. `SEMANTIC_AUTHORITY_ARCHITECTURE.md`
states that failure to earn a certificate means Clean cannot claim facts are
exhaustive; it does not make them wrong. `WSC-014` similarly says absence of a
certificate means "not authorized to claim complete."

The public `discovery` diagnostic carries provider, partial/unavailable status,
fallback reason, counts, and exceptional project states. It does not describe
coverage-unit proof dispositions. A completed discovery diagnostic therefore
cannot explain why source completeness was not earned.

## 3. Internal acquisition/proof state model

The pipeline represents the requested dimensions unevenly.

### 3.1 Acquisition/process completion

`DiscoveryOutcome` knows whether discovery was attempted and completed.
`HydrationReport` retains `discovery_status`. Candidate compilation returns one
of:

```text
Published
PublishedCSharp
Current
Failed
```

`hydrate_workspace_index_for_target` reduces candidate results to:

- a count of newly compiled publications; and
- one `publication_failed` boolean.

It therefore retains coarse terminal success/failure but not a per-candidate
failure inventory.

### 3.2 Evidence evaluated

Filesystem inventory records every successfully read supported owner with its
source kind, source hash, and source snapshot. The candidate predicate selects
owners containing the written target. Compilation/publication then evaluates
selected owners or accepts current adequate projections.

The public response exposes discovered and compiled counts but not the set of
evaluated/current owners, nor a terminal "no known processing work remains"
claim.

### 3.3 Required and proven coverage

`AcquisitionRequirement` owns the complete required `CoverageUnit` set.
`AcquisitionSatisfaction` owns that set plus validated proofs. It can compute
`residual_units()` and `is_source_complete()`.

It does not retain why a residual unit lacks proof.

### 3.4 Positive facts

Returned edges come from Clean-published semantic projections and retain their
relationship, endpoint identities, layer, and asserting-file provenance. A
missing completeness certificate does not revoke this positive evidence.

### 3.5 Exhaustive-negative authority

Only `CompletenessCertificate::incoming_relationships` authorizes absence of
additional facts. It requires `AcquisitionSatisfaction::completeness_generation`
and therefore proof of every required unit after successful publication.

This positive-versus-negative authority distinction exists architecturally but
is not explicitly named in the lower-bound public payload.

## 4. Earliest epistemic information loss

The trace is:

```text
candidate discovery
  -> InventoryManifest(attempted, complete, entries, digest)
  -> AcquisitionRequirement(required CoverageUnits)
  -> clean_type_ref_proof_claim
       Result<Claim, CleanProofRejection>
  -> validate_proof_claim
       Result<ValidatedProof, ProofRejection>
  -> filter_map(... .ok()?)
       rejection reason discarded
  -> ProvisionalAcquisition(requirement, successful proofs only)
  -> finish(publication status)
  -> AcquisitionSatisfaction(required, proven)
  -> HydrationReport(coarse discovery/publication + satisfaction)
  -> exact_identity_coverage
  -> exact certificate or undifferentiated lower_bound
```

`CleanProofRejection` already distinguishes:

- `IncompleteInventory`;
- `UncertifiedPredicate`;
- `SourceKindMismatch`; and
- `OwnerOutsideRoot`.

`ProofRejection` further distinguishes unit, target, scope, inventory,
admission, predicate-version, contract-version, exhaustiveness, truncation, and
limitation failures.

At `prepare_clean_type_ref_acquisition`, both result types are converted into
`None`. From that point onward a TypeScript `UncertifiedPredicate` and an
incomplete inventory are identical residual units.

A second loss occurs after candidate processing: all compilation, source-race,
admission, and publication-ticket failures collapse into
`publication_failed: bool`. This is sufficient to withhold completeness but
insufficient to explain whether retry is useful.

The final projection is therefore not where the distinctions originate, only
where their prior loss becomes visible.

## 5. TypeScript UncertifiedPredicate case

For the mixed workspace, `manifests_by_source_kind` constructs a RootC ×
TypeScript unit for `BuiltinTypeRefIncoming` and both neutral relationship
families. Assume the filesystem traversal/read inventory is complete.

The exact sequence is:

```text
complete TypeScript inventory
  -> clean_type_ref_proof_claim
  -> has_certified_type_ref_name_bearing_predicate(TypeScript) == false
  -> Err(UncertifiedPredicate)
  -> rejection discarded
  -> unit remains required but unproven
  -> completeness_generation() == None
```

That sequence does not establish any known omitted TypeScript candidate. It
establishes that Clean cannot prove its literal selection predicate captured
every TypeScript owner capable of emitting a matching TypeRef edge.

Current internal types can distinguish the requested cases at the instant they
occur:

| Case | Available internal signal | Survives into satisfaction/public coverage? |
|---|---|---|
| Inventory incomplete | `InventoryManifest.complete=false` → `IncompleteInventory` | No reason; only residual unit/coarse discovery may survive |
| Complete inventory, uncertified predicate | `UncertifiedPredicate` | No |
| Predicate valid, candidate processing failed | `CandidatePublication::Failed` | Only aggregate publication failure/partial discovery |
| Publication failed | `CandidatePublication::Failed`/ticket result | Only aggregate publication failure |
| Processing terminal, proof structurally unavailable | `UncertifiedPredicate` plus no publication failure | Not retained as a combined state |

Thus the architecture can observe the distinguishing facts, but the durable
request-local result model does not preserve enough of them to derive a public
retryability classification later.

## 6. Positive versus exhaustive-negative authority

The two returned `HasConstructorParameterType` edges are positive semantic
facts authored by current Clean semantic publication. Their authority does not
depend on proving that no third edge exists.

There are two independent propositions:

```text
Positive authority:
  ConsumerOne and ConsumerTwo declare the reported relationship.

Exhaustive-negative authority:
  No other admitted owner declares that relationship.
```

Current edge payloads and provenance support the first. Only a completeness
certificate supports the second.

The lower-bound tuple formally preserves that distinction: it returns the
facts while denying exhaustiveness. It does not, however, explicitly tell an
LLM that the returned facts remain authoritative positive evidence. A consumer
may incorrectly treat every edge as provisional rather than treating only the
closed-world conclusion as unavailable.

A richer contract should make the distinction explicit without changing edge
authority or exactness. Conceptually it needs two axes:

- observed semantic evidence is current/authoritative; and
- exhaustive coverage is proven/unproven.

## 7. lower_bound semantic audit

The proven intended meaning is:

> Returned facts are valid indexed evidence, but Clean cannot prove that
> additional matching facts do not exist in the required source universe.

That is closest to option C from the investigation prompt: inability to prove
exhaustiveness regardless of whether operational acquisition completed.

It is not limited to knowingly incomplete acquisition. Tests require
`lower_bound` for an unproved language unit even when the observed C# answer is
complete, and architecture documentation describes it as the absence of
exhaustiveness authority.

Therefore `result_semantics="lower_bound"` is not semantically wrong in the
TypeScript case. Changing it would risk weakening an established closed-world
contract. The defect is that it is the only public semantic category beneath
`exact`, not that this category is false.

## 8. omitted_possible semantic audit

Current code sets `omitted_possible=true` whenever an exact certificate is not
issued. It does not require evidence of a known omitted file or result.

Its actual meaning is:

> Clean lacks authority to prove that nothing was omitted.

It does not mean:

> Clean has identified a candidate or edge that it omitted.

The name is defensible as modal language—omission is possible—but existing
tooling text often explains lower bounds through files that "may omit
uncompiled source files." That explanation fits known acquisition gaps better
than an uncertified predicate with zero known pending candidates.

The field is truthful but ambiguous to an LLM because it does not distinguish
"possible because known work failed" from "possible because the selection
proof is unavailable." Additive detail should clarify the basis; the boolean
should remain unchanged for compatibility.

## 9. Retryability analysis

Retryability is not currently a first-class contract.

| Condition | Identifiable at occurrence? | Identical retry potentially useful? |
|---|---:|---:|
| Incomplete filesystem inventory/transient read | Yes | Yes |
| Candidate compile/source race failure | Yes | Yes |
| Publication failure/supersession | Yes | Yes |
| Provider unavailable and Clean fallback incomplete | Yes | Yes |
| Budget/cancellation interruption | Not represented end-to-end in current request control | Policy/new request may help |
| `UncertifiedPredicate` | Yes | No |
| Proof version/compatibility rejection | Yes | Usually no without state/version change |
| Provider unavailable but Clean proves all units | Yes | No retry needed; exact |

The decisive limitation is retention. Because proof rejection reasons are
discarded and publication failures are aggregated, `exact_identity_coverage`
cannot authoritatively classify a residual unit as structurally blocked versus
operationally retryable.

Repeated unchanged queries cannot convert `UncertifiedPredicate` into a proof.
Publicly exposing a generic lower bound without this fact can induce pointless
retries, filesystem searches, or provider calls.

Before publishing retryability, Clean must retain typed dispositions through
the request. It should not infer retryability from `discovery_status` alone.

## 10. LLM-consumer consequences

A rational LLM reading only:

```json
{
  "source_complete": false,
  "result_semantics": "lower_bound",
  "omitted_possible": true
}
```

cannot tell whether:

- known candidates failed and a retry may improve the result;
- discovery did not finish;
- all selected work completed but one source-language predicate lacks a proof;
- the result is index-only and no acquisition was attempted; or
- a broader policy/scope is required.

Material consequences include:

- unnecessary fallback grep/search after terminal observed processing;
- repeated identical queries that cannot improve proof state;
- unnecessary provider status/index queries;
- refusal to rely on positively authoritative returned edges;
- conflation of "cannot prove no third edge" with "the two edges may be
  wrong"; and
- inability to decide whether narrowing scope, changing policy, retrying, or
  accepting open-world uncertainty is the appropriate next action.

These consequences are particularly important for an LLM-facing intelligence
layer because the coverage object is the consumer's only structured epistemic
guide. The current contract protects correctness but does not sufficiently
guide follow-up behavior.

## 11. Scenario matrix

| Scenario | Acquisition terminal? | Known missing work? | Exhaustiveness proven? | Same retry useful? | Current public state |
|---|---:|---:|---:|---:|---|
| Complete C# proof | Yes | No | Yes | No | `authoritative_complete`, exact, no omission possible |
| Mixed C#/TS; TS `UncertifiedPredicate` | Yes for selected work | No known candidate work | No | No | lower-bound, `source_complete=false`, `omitted_possible=true` |
| Incomplete inventory | No/unknown | Yes | No | Yes | same lower-bound tuple, usually partial discovery |
| Candidate compilation failure | No | Yes | No | Yes | same lower-bound tuple plus partial discovery |
| Publication failure | No | Yes | No | Yes | same lower-bound tuple plus partial discovery |
| Budget exhaustion | No | Yes | No | Not identical; changed policy may help | same lower-bound family if represented |
| Provider unavailable, Clean proves all units | Yes | No | Yes | No | authoritative complete; discovery may disclose fallback |
| Complete authoritative zero result | Yes | No | Yes | No | authoritative complete with `count=0` |
| Interrupted/cancelled request | No terminal response contract today | Yes | No | New request may help | no dedicated coverage state |
| Index-only unsupported query family | No authoritative acquisition | Unknown | No | Query-family change may help | lower-bound/indexed capability state |

Multiple materially different rows collapse into the same exact/lower-bound
tuple. Discovery diagnostics sometimes distinguish operational failure, but
they do not represent proof limitation or retryability and are absent/default-
elided in steady states.

## 12. Backward compatibility

Existing fields should retain their strong meanings:

- `source_complete=true` only with an authoritative certificate;
- `result_semantics=exact` only when absence of more facts is authoritative;
- `lower_bound` whenever exhaustiveness is not proven; and
- `omitted_possible=true` whenever Clean cannot prove otherwise.

Changing `result_semantics` would force existing consumers to understand a new
closed-world category and risks accidental treatment of an uncertified result
as exact. Additive evolution is safer.

A compatible shape could conceptually be:

```json
{
  "source_complete": false,
  "result_semantics": "lower_bound",
  "omitted_possible": true,
  "coverage_detail": {
    "processing_state": "terminal",
    "proof_state": "required_units_unproven",
    "positive_evidence": "authoritative_observed_facts",
    "absence_authority": "not_established",
    "retryability": "structural_change_required",
    "residual_unit_count": 1,
    "residual_reasons": ["uncertified_candidate_predicate"]
  }
}
```

These labels are illustrative, not approved API names. Public detail should be
bounded and should avoid exposing filesystem roots, provider internals, or an
unbounded unit list. Counts and stable reason categories are sufficient for an
LLM to select the next action.

## 13. Relationship to Issue #144

Issue #144 repaired acquisition correctness, completeness authority,
performance, and startup/provider lifecycle behavior. This investigation does
not falsify those repairs.

The broad mixed-language query correctly refuses a certificate. Its two C#
edges may be observationally complete, but the TypeScript source universe lacks
an exhaustive candidate-predicate proof. Original Issue #144 should remain
closed.

The new concern is the expressiveness of public epistemic metadata after
correct certificate refusal. It affects consumer behavior across query families
and failure modes, so it should be tracked as a separate coverage-semantics
issue rather than appended as an acquisition regression.

## 14. Smallest justified contract improvement

The smallest coherent internal change is to preserve a bounded disposition for
every required coverage unit through provisional acquisition and satisfaction.
At minimum, a disposition must distinguish:

```text
Proven
OperationallyIncomplete(reason category)
ProofUnavailable(reason category)
```

The exact enum design requires approval, but it should be typed rather than
reconstructed from strings. `UncertifiedPredicate` belongs to
`ProofUnavailable`; incomplete inventory and publication failure belong to
operational incompleteness. Proof compatibility failures may need a separate
non-retryable/stale category rather than being guessed into either group.

Candidate processing should similarly retain whether all selected owners are
terminal-current/published or whether a known owner failed. Once those facts
survive into `HydrationReport`, response projection can derive a bounded
additive detail:

- processing terminal versus known incomplete;
- proof complete versus residual;
- positive observed facts remain authoritative;
- exhaustive-negative authority absent/present; and
- identical retry useful, not useful, or indeterminate.

This does not require changing `CompletenessCertificate`. Exactness remains a
separate terminal condition over all proven units.

Because this changes the public tool/API contract and shared coverage behavior,
it requires explicit architectural approval before implementation.

## 15. Required regressions if approved

Tracked regressions under `src/tests/**` should first establish RED behavior for
the additive detail without changing existing exactness assertions.

Required cases:

1. complete C# proof reports terminal processing, proof complete, no retry;
2. mixed C#/TypeScript `UncertifiedPredicate` remains lower-bound while
   reporting terminal observed processing, structural proof limitation, and no
   benefit from an identical retry;
3. incomplete inventory remains lower-bound and reports known incomplete,
   retryable acquisition;
4. candidate compilation failure remains lower-bound and reports known failed
   processing;
5. publication failure remains lower-bound and reports retryable publication
   failure;
6. provider unavailable plus successful complete Clean fallback remains exact
   and does not recommend retry;
7. exact authoritative zero retains the same complete state as a positive
   exact result;
8. cold/warm equivalent requests expose the same epistemic classification;
9. singular/batch forms project identical per-item detail; and
10. schemas and model-facing content carry the bounded additive object without
    removing or weakening legacy fields.

RED/GREEN procedure must follow repository policy. Agent MaxHeadRoom must not
run the Cargo commands; the exact focused commands must be handed to the user.

## 16. Final recommendation

Keep Issue #144 closed and open a new architecture issue for additive coverage
epistemics.

The new issue should authorize design—not immediate implementation—of:

1. retained typed per-unit proof dispositions;
2. retained terminal/failed candidate-publication state;
3. an aggregate retryability classification derived only from those facts; and
4. bounded additive public `coverage_detail` metadata.

Do not change the completeness certificate, remove residual units, reinterpret
207/207 as proof, or weaken `source_complete`, `exact`, `lower_bound`, or
`omitted_possible`.

The target distinction is:

```text
known acquisition/publication work remains
```

versus:

```text
observed processing is terminal, positive facts are authoritative,
but exhaustive-negative proof is structurally unavailable
```

Clean-CTX already observes the facts needed to begin making that distinction.
It must preserve them through the acquisition result model before exposing a
new public claim.

