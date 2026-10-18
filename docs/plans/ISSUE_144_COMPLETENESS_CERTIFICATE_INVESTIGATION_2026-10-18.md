# Issue #144 Exact Reverse-Lookup Completeness Certificate Investigation

**Date:** 2026-10-18  
**Investigator:** Agent MaxHeadRoom  
**Scope:** Static repository investigation only. No Cargo command, build, test,
Clippy, binary, MCP runtime session, or live verification was run.

## 1. Executive verdict

**Primary verdict: LOWER_BOUND IS CORRECT — REQUIRED COVERAGE REMAINS
UNPROVEN.**

**Observed live result: OBSERVATIONALLY EXACT BUT NOT INTERNALLY PROVEN.**

**Final recommendation: CLOSE #144 — CURRENT LOWER_BOUND IS CORRECT.**

The live request already enters the only implemented exact incoming-relationship
certificate family:

```json
{
  "type": "reverse_edges",
  "name": "ServiceX",
  "domain": "builtin",
  "entity_type": "TypeRef",
  "workspaceRoot": "<RootA>"
}
```

It has an explicit workspace scope, so `exact_incoming_type_ref` is true. The
certificate is not blocked by the syntactic gate in `coverage.rs`, and no
certificate is constructed and later lost. It is blocked earlier because the
effective workspace contains TypeScript source and current Clean-owned proof
construction certifies the TypeRef name-bearing candidate predicate only for
C#.

The current public query is not C#-specific. `builtin/TypeRef` and its incoming
`HasConstructorParameterType`/`HasBaseType` relationships are produced through
the language-agnostic canonical IR projection. A TypeScript owner in scope can
therefore contribute a matching edge. The native check cited in the live report
searched `.cs` files only; it strongly validates the two returned C# edges but
does not prove that no in-scope TypeScript owner can contribute another
`builtin/TypeRef/ServiceX` edge.

The exact first point of ineligibility is:

```text
RootC / TypeScript inventory
  -> required CoverageUnit(BuiltinTypeRefIncoming, TypeScript,
                           HasConstructorParameterType + HasBaseType)
  -> clean_type_ref_proof_claim
  -> UncertifiedPredicate
  -> no ValidatedAcquisitionProof for that unit
  -> AcquisitionSatisfaction::residual_units is non-empty
  -> completeness_generation() returns None
  -> CompletenessCertificate::incoming_relationships() returns None
  -> truthful lower_bound projection
```

An existing tracked regression deliberately pins this behavior:
`issue_144_unproved_language_unit_remains_lower_bound` in
`src/tests/mcp/workspace_query_reverse_edge_completeness.rs`.

There is one adjacent, statically proven limitation: `withinPath` narrows final
query occurrences but does not narrow the acquisition roots, acquisition cache
key, inventory manifests, or proof scope. Therefore the proposed
`withinPath=RootB` experiment is not expected to remove RootC's TypeScript
obligation. This is conservative and does not overclaim completeness, but it
prevents a narrower query from earning a certificate using a narrower source
universe. Changing that behavior is a separate externally observable
certificate-design decision, not evidence that this broad-scope response is
wrong.

## 2. Exact live-query certification path

The production path is:

```text
workspace_query dispatch
  -> query::edges::prepare_reverse_edges
  -> IdentityRequest::exact(builtin, TypeRef, ServiceX)
  -> PreparationContext::scope
  -> WorkspaceScope(workspaceRoot + all configured additional_roots)
  -> PreparationContext::hydrate_incoming_type_ref
  -> hydrate_workspace_index_for_acquisition
  -> discover_candidate_paths(InboundReference)
  -> filesystem inventory for configured roots
  -> prepare_clean_type_ref_acquisition
  -> AcquisitionRequirement::incoming_type_ref
  -> per-root/per-source-kind CoverageUnit construction
  -> provisional validated proofs for certified units
  -> compile/current publication processing
  -> ProvisionalAcquisition::finish
  -> AcquisitionSatisfaction
  -> reverse-edge evaluation
  -> exact_identity_coverage
  -> CompletenessCertificate::incoming_relationships
  -> exact or lower-bound coverage projection
```

For the live request, identity resolution is exact rather than inferred. The
handler recognizes exact `builtin/TypeRef` before hydration and passes
`AcquisitionTarget::exact_type_ref("ServiceX")`. This is the production hook
that asks hydration to construct proof-backed satisfaction.

The final evaluator calls `exact_identity_coverage` with reverse direction, the
same effective `WorkspaceScope`, and the resulting `HydrationReport`. Nothing
after this call overwrites the returned `coverage` object.

## 3. AcquisitionSatisfaction authority model

`AcquisitionSatisfaction` contains:

- the exact set of required `CoverageUnit` values;
- validated proofs keyed by the exact unit they prove;
- the complete acquisition-requirement fingerprint; and
- the scope-policy fingerprint.

It does not infer authority from candidate counts, provider readiness,
successful discovery, an indexed identity, or observed edges.

A proof enters `proven` only when:

1. its claim validated against the exact requirement and `ProofContext`;
2. its unit belongs to the requirement; and
3. semantic publication completed without a candidate publication failure.

`is_source_complete()` requires a non-empty requirement and no residual unit.
`completeness_generation()` returns `Some(hash)` only in that state. Its hash
binds the requirement fingerprint and every proven inventory generation.

Accordingly, all discovered candidates may compile while final coverage remains
lower-bound. Compilation counts describe work performed; satisfaction describes
whether the complete capable source universe was proven.

## 4. Required coverage-unit derivation

`prepare_clean_type_ref_acquisition` receives every root inventory returned by
filesystem discovery. `manifests_by_source_kind` groups each root's inventory
entries by `SourceKind`. Every non-empty `(canonical root, source kind)` group
becomes a required unit containing:

```text
canonical root
source kind
BuiltinTypeRefIncoming producer capability
source-kind-specific producer-generation fingerprint
{ HasConstructorParameterType, HasBaseType }
```

For the reported workspace the conceptual requirement is at least:

```text
RootA × CSharp
RootB × CSharp
RootC × TypeScript
RootD × CSharp
```

Only source kinds actually present in each inventory produce units. Additional
roots participate independently because `configured_roots` always combines the
requested root with configured additional roots.

The requirement is relationship-producer relevant rather than tied to where a
declaration named `ServiceX` lives. Incoming lookup asks which in-scope owners
can emit the two neutral TypeRef relationship families. Each capable language
producer must be covered.

## 5. Why completeness_generation() is None

`clean_type_ref_proof_claim` rejects a unit for four reasons:

- incomplete inventory;
- uncertified name-bearing predicate;
- an owner outside the unit root; or
- a source-kind mismatch.

In current code,
`has_certified_type_ref_name_bearing_predicate(SourceKind)` returns true only
for `SourceKind::CSharp`. It returns false for TypeScript, Rust, and Java.

Therefore a complete RootC TypeScript manifest still receives
`CleanProofRejection::UncertifiedPredicate`. The unit remains required but has
no proof. `AcquisitionSatisfaction::residual_units()` remains non-empty and
`completeness_generation()` returns `None`.

This reconciles the apparent contradiction:

```text
207 discovered == 207 compiled
```

does not mean:

```text
every required coverage unit is proven
```

The TypeScript files can be compiled as candidates or otherwise processed
without establishing the exhaustive negative authority needed to prove that
the literal narrowing found every possible TypeRef-producing owner.

## 6. exact_incoming_type_ref rationale

The `builtin/TypeRef` restriction is both an intentional semantic boundary and
the first implemented exact-completeness slice.

It is semantic because the implemented acquisition target and proof contract
are specifically `AcquisitionTarget::exact_type_ref`,
`ProducerCapability::BuiltinTypeRefIncoming`, and the two TypeRef relationship
families. Other identities do not automatically share that source-universe or
candidate-predicate proof.

It is also an implementation boundary because no generalized acquisition
requirement exists for other identity/relationship families. Removing the gate
would permit a TypeRef proof to certify unrelated reverse relations and would
overclaim.

The live query does resolve to `builtin/TypeRef`, so this boundary is not the
cause of its lower-bound result. It successfully reaches certificate
construction; construction returns `None` because satisfaction is incomplete.

## 7. Mixed-root and mixed-language analysis

The implemented architecture follows query-relevant coverage, but
"query-relevant" is defined by the actual public semantic capability—not by the
language of the two observed results.

This is neither a naive workspace-global model nor a C#-only model:

- unrelated source kinds with no participating unit would not be required;
- every admitted source kind that can feed the shared
  `BuiltinTypeRefIncoming` projection is required;
- the current proof implementation has certified only C# narrowing; and
- an unproved participating TypeScript unit conservatively prevents exactness.

The shared `ir::semantic_projection` projects constructor `CoreOp::Param` values
to `HasConstructorParameterType` and `CoreOp::BaseTypeRef` values to
`HasBaseType` without a C# discriminator. The architectural invariant
`ARCH-002` likewise defines canonical IR as language-agnostic across TypeScript,
C#, Rust, and Java.

Consequently an exact C#-only answer could be certified in a mixed workspace
only if the request contract or scope excludes other capable producers, or
those producers also supply valid coverage proofs. The present request does
neither.

## 8. Scope gate and withinPath

`scope.is_some()` means the caller supplied an explicit `workspaceRoot`; it does
not mean acquisition is complete. Requested root plus additional roots forms
one `WorkspaceScope`. An optional `withinPath` narrows occurrence provenance
inside that authorized set.

Root-less queries receive `None` and cannot claim workspace completeness. This
is an authority-bearing defensive gate: the server cannot certify an unnamed
global/session source universe.

However, query scope and acquisition scope are not fully aligned for
`withinPath`:

- `WorkspaceScope::narrowed` applies `withinPath` to returned occurrences;
- `hydrate_workspace_index_for_target` builds its cache key from
  `configured_roots(state, workspace_root)`;
- discovery and filesystem inventories use those same configured roots; and
- neither path receives `withinPath`.

Thus `withinPath=RootB` still acquires and requires RootA/RootB/RootC/RootD
units, even though final edges are filtered to RootB. Broader completeness
implies subset completeness, so this is safe. It is nevertheless an incomplete
narrow-scope certificate capability and means the proposed experiment cannot
isolate RootB under current code.

## 9. Interpretation of 207 / 207

The counters establish:

- 207 candidate paths survived discovery/deduplication for this cycle; and
- 207 selected candidates were newly published successfully enough to count as
  compiled.

They do not independently establish:

- that every capable source owner was discovered exhaustively;
- that literal name matching is a sound narrowing predicate for every language;
- that every required producer executed with compatible generation;
- that every relevant dependency is current;
- that every coverage unit received a validated proof; or
- that publication completed for the complete required universe.

The inventory manifest and validated proof—not equality of counts—own those
claims. Budget exhaustion would revoke completeness; no numeric budget or
filter can redefine the source universe.

## 10. Certificate construction and projection

`CompletenessCertificate::incoming_relationships` has one fallible step:

```rust
source_generation: satisfaction.completeness_generation()?
```

If satisfaction is absent or incomplete, `exact_identity_coverage` falls
through to lower-bound metadata. If construction succeeds, `project()` emits:

```text
status = authoritative_complete
source_complete = true
result_semantics = exact
omitted_possible = false
```

It then adds `identity_indexed` and `capability_established`. The resulting
object is installed directly into the response. `discovery_field` is separate
and does not overwrite it. Batch wrapping preserves each item's structured
result. No downstream certificate-loss path was found.

## 11. Static scenario matrix

| Scenario | Current expected certificate | Authority |
|---|---|---|
| One C# root; complete inventory, proof, and publication | `authoritative_complete` | Covered by `issue_144_proof_backed_reverse_type_ref_earns_exact_coverage` |
| Two C# roots; every unit proven | `authoritative_complete` | Conjunctive per-root units; no contrary branch |
| C# plus TypeScript; C# units proven, TypeScript unit unproved | `lower_bound` | Covered by `issue_144_unproved_language_unit_remains_lower_bound` |
| Additional C# root with one unit unproven | `lower_bound` | `residual_units` remains non-empty |
| Interrupted/incomplete inventory | `lower_bound` | `IncompleteInventory` prevents proof |
| Candidate publication failure | `lower_bound` | Covered by `issue_144_publication_failure_remains_lower_bound` |
| Budget exhaustion/truncation | `lower_bound` | Exhaustive proof cannot validate |
| Provider unavailable, Clean proves all units | `authoritative_complete` | Provider-independent Clean proof contract |
| Warm equivalent request with compatible cached proof | Same as cold | Covered by `issue_144_exact_coverage_is_stable_across_cold_and_warm_requests` |
| Complete authoritative zero result | `authoritative_complete` | Covered by `issue_144_complete_zero_result_is_exact` |
| Root-less session query | `lower_bound` | Covered by `issue_144_unscoped_session_view_cannot_claim_workspace_completeness` |
| `withinPath=RootB` while another configured root has unproved TypeScript | `lower_bound` today | Acquisition ignores narrowing and retains broad units |

## 12. Existing tests and uncovered cases

Existing tracked authority includes:

- provider success cannot omit Clean-discovered consumers;
- ready empty provider results have no negative authority;
- unproved provider subsets cannot become warm completion authority;
- proof-backed TypeRef reverse lookup earns exact coverage;
- cold/warm coverage equivalence;
- exact authoritative empty results;
- mixed C#/TypeScript lower-bound behavior;
- publication-failure lower-bound behavior; and
- unscoped lower-bound behavior.

The principal uncovered case is not the live broad-scope response; it is the
interaction between `withinPath` and acquisition proof scope. A future design
may choose to make a narrowed subtree the proof universe, with a cache key and
scope fingerprint that bind that exact narrowing. That requires an explicit
contract because it changes when public results become exact and changes
acquisition/caching behavior.

A second possible future slice is to certify name-bearing predicates for
TypeScript, Rust, or Java. Such certification requires producer-specific tests
that prove every emitted TypeRef target is present in the inventoried source
under the exact matching semantics. Simply changing the boolean guard would be
unsound.

## 13. Relationship to the completed #144 repair

The completed repair correctly separated:

- candidate hints from coverage proof;
- provider readiness from completeness;
- candidate compilation from terminal satisfaction; and
- indexed capability from authoritative source completeness.

The live result demonstrates those guards working: it returns useful indexed
facts quickly while refusing an unsupported negative claim over a mixed source
universe. This is not a regression to the old incomplete-acquisition behavior.

The closure summary's "mixed-scope lower-bound behavior" matches both current
production code and the tracked regression. The later comment supplies a real
example of that designed boundary; it does not falsify it.

## 14. Smallest proven boundary and regression disposition

No production defect is proven for the reported broad-scope request.

The certificate stops at the correct boundary:

```text
clean_type_ref_proof_claim(TypeScript unit)
  -> UncertifiedPredicate
```

No RED regression is warranted for changing the broad request to exact. The
existing regression asserts the opposite behavior and is architecturally
consistent.

If maintainers explicitly approve narrowed proof scopes, the narrowest future
RED regression should construct:

```text
workspaceRoot = RootA
additional roots = RootB + RootC(TypeScript)
withinPath = RootB
exact reverse builtin/TypeRef query
```

and require the acquisition requirement, proof scope, final edge scope, and
cache identity all to bind the same canonical RootB narrowing. The production
change must not merely hide residual units at response projection.

If maintainers instead approve TypeScript predicate certification, the RED
regression must first prove a TypeScript constructor/base reference can produce
the relevant neutral edge, and then prove complete inventory plus publication
earns the TypeScript unit without missing formatted, qualified, generic, or
aliased written types supported by the producer.

## 15. Files and symbols inspected

- `src/mcp/tool_handlers/query/edges.rs` — exact reverse identity, acquisition
  target, evaluation, and coverage call.
- `src/mcp/tool_handlers/query/coverage.rs` — TypeRef/scope gate,
  `CompletenessCertificate`, and exact/lower-bound projection.
- `src/mcp/tool_handlers/query/prepare.rs` — request-local hydration key and
  incoming-TypeRef dispatch.
- `src/mcp/tool_handlers/query.rs` — effective `WorkspaceScope` construction.
- `src/workspace/scope.rs` — authorized roots and `withinPath` occurrence
  narrowing.
- `src/mcp/tool_handlers/hydration/execution.rs` — acquisition cache,
  discovery, compilation/publication, satisfaction, and report construction.
- `src/mcp/tool_handlers/hydration/acquisition.rs` — requirement/unit derivation
  and proof finalization.
- `src/mcp/tool_handlers/hydration/filesystem.rs` — configured roots,
  inventory construction, source-kind grouping input, and literal candidates.
- `src/mcp/acquisition/requirement.rs` — targets, coverage units, relationship
  families, evidence, and producer-generation fingerprints.
- `src/mcp/acquisition/clean.rs` — certified C# predicate and rejection rules.
- `src/mcp/acquisition/proof.rs` — claim validation authority.
- `src/mcp/acquisition/satisfaction.rs` — residual units and completeness
  generation.
- `src/ir/semantic_projection.rs` — language-agnostic TypeRef relationship
  projection.
- `src/tests/mcp/workspace_query_reverse_edge_completeness.rs` — exact,
  mixed-language, failure, warm-state, and unscoped regressions.
- `src/tests/mcp/acquisition_clean.rs`, `acquisition_requirement.rs`,
  `acquisition_proof.rs`, and `acquisition_cache.rs` — unit proof authority.
- `docs/ARCHITECTURAL_INVARIANTS.md` — `ARCH-002` and `WSC-014`.
- Issue #144's 2026-10-18 closure and follow-up live comment.

## 16. Recommended next phase

**CLOSE #144 — CURRENT LOWER_BOUND IS CORRECT.**

Do not change the TypeRef gate, infer completeness from 207/207, or exclude
TypeScript merely because the observed edges are C#. The current query contract
is cross-language and the reported native validation did not establish absence
of TypeScript contributors.

If narrower certificate behavior is desired, open a separate design issue for
one of two explicit changes:

1. bind acquisition requirements and proof/cache scope to authorized
   `withinPath` narrowing; or
2. certify additional source-language name-bearing predicates with tracked
   producer-completeness regressions.

Either option changes externally visible exactness and shared acquisition/cache
behavior, so it requires architectural approval before implementation.

