# Invariant family: source-authority

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### IRFACT-001 Derived Facts Use Declaration-Local Evidence

| Property | Value |
|----------|-------|
| **Intent** | Compiler-derived facts are presented as verified conclusions, so their evidence must belong to the declaration receiving the fact. |
| **Invariant** | An additive fact keyed to method `M` may use only operations whose canonical owner is `M` and may not scan across the next declaration boundary for supporting evidence. Every evidence component required by the fact's established contract must be present; independent partial signals are not interchangeable. Downstream hierarchy and presentation layers project the canonical fact and must not infer, relocate, or repair its ownership. This rule applies to every derived fact family, not only `PatternFact::Observable`. |
| **Enforcement** | `CodePatternRecognizer::try_observable_pattern` stops at the next `DefMethod` and accepts a qualifying Observable return only when its owner is the triggering method ID. Unit regressions reject neighboring-method evidence and non-Observable evidence; registered MCP regressions cover both `provide_code_context` and the initial-full `delta_code_context` path. |
| **Authority** | `src/ir/layers/patterns.rs`, `src/tests/ir/layers/patterns.rs`, `src/tests/mcp/pattern_fact_ownership.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |
| **Relationship to IRPAT-001 and PATID-001** | IRPAT-001 and PATID-001 preserve declaration identity during consumptive transformations. IRFACT-001 governs the separate additive-producer boundary: a surviving identity must not receive a fact derived from another declaration. |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 443-456, original section `IRFACT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `c1df87b199c63578e8ed0a2d0e79915f0fe59aed4729fbe85f0c27b659ee0c94`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IRFACT-001
kind: INVARIANT
status: IMPLEMENTED
title: IRFACT-001 Derived Facts Use Declaration-Local Evidence
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### IRFACT-002 Observable and Promise Classifications Are Semantically Disjoint

| Property | Value |
|----------|-------|
| **Intent** | A typed pattern name is a verified semantic conclusion. Observable and Promise are distinct contracts and must never be inferred from one another or from an unrelated execution modifier. |
| **Invariant** | `PatternFact::Observable` and `PatternOp::Observable` require an owner-matched declared `Observable` or `IObservable` return type. `PatternOp::Promise` requires a declared `Promise` return or its canonical `$P` opcode. `Async` is independent method metadata: it neither establishes Observable identity nor prevents Promise identity. Body-level `df:reads:observable` remains independent because consuming or transforming an Observable does not imply returning one. Canonical classifications remain typed and available code-side; SCHEMA-vNext suppresses `pf:OBSERVABLE` and `P OBSERVABLE` only when the rendered `→ Observable<...>`/`IObservable<...>` return already states the same fact. This suppression is presentation-only and does not alter canonical IR or wire compatibility. |
| **Enforcement** | Shared identifier-aware return predicates in `src/ir/patterns.rs` are the single semantic authority used by both additive and consumptive recognizers. The focused `observable_semantics` regressions cover genuine Observable returns, async Promises, declaration ownership, and SCHEMA-vNext redundancy collapse; production-shape identity and two-pass regressions retain the declarations and modifiers around both classifications. |
| **Authority** | `src/ir/patterns.rs`, `src/ir/patterns/recognize.rs`, `src/ir/layers/patterns.rs`, `src/ir/render_llm.rs`; `src/tests/ir/observable_pattern_semantics.rs`, `src/tests/ir/layers/patterns.rs`, `src/tests/ir/render_llm.rs`, `src/tests/ir/pattern_identity.rs`, `src/tests/ir/patterns_pipeline.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |
| **Relationship to IRFACT-001** | IRFACT-001 governs which declaration may supply evidence. IRFACT-002 governs what that owner-matched evidence is allowed to mean. |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 457-470, original section `IRFACT-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `1fd6e65d8472ddea9ab59f5455ffeffa848f684120882939703b395a0e0f282d`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IRFACT-002
kind: INVARIANT
status: IMPLEMENTED
title: IRFACT-002 Observable and Promise Classifications Are Semantically Disjoint
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### WSC-001 Authoritative Facts Do Not Imply Authoritative Coverage

| Property | Value |
|----------|-------|
| **Intent** | WorkspaceIndex is demand-populated and session-scoped. Its facts are authoritative for the semantic evidence compiled into the index, but the index must never be presented as a complete picture of the workspace. |
| **Invariant** | Absence of an entity or relationship from a partially populated `WorkspaceIndex` must never be represented as confirmed absence from the workspace. `workspace_query` results are authoritative for what Clean-CTX currently knows — never evidence of completeness beyond the configured discovery coverage. Result cardinality (including `0`) must never drive hydration decisions or completeness claims: `initial count == 0` is not "needs hydration" and `initial count > 0` is not "sufficiently covered". Discovery-provider absence or failure must never be represented as a successful zero-candidate workspace search. Hydration must process every discovered valid, previously-unindexed candidate needed for the query; a resource guard may not silently truncate semantic coverage. Any future guard that prevents exhaustive processing must explicitly report partial/truncated coverage. |
| **Enforcement** | `src/mcp/tool_handlers/query/prepare.rs` owns request-local hydration execution and final-view deferral; the eligible family preparers in `query/{entities,edges,graph}.rs` perform registered hydration independently of initial result cardinality. `src/tests/mcp/workspace_query_2.rs::red9_partial_nonzero_hydration` proves a non-zero initial result remains hydration-eligible; `red10_fresh_index_hydration` covers a fresh empty index. Batch preparation and final-snapshot behavior are covered by `src/tests/mcp/workspace_query_batch_preparation.rs`. |
| **Authority** | `src/mcp/tool_handlers/query/prepare.rs`, `src/mcp/tool_handlers/query/{entities,edges,graph}.rs`, `src/mcp/tool_handlers/hydration.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 510-522, original section `WSC-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `cd2ef064cba110884a6858aad1c3339bd17a14d036293492af5e40097594fcc3`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: WSC-001
kind: INVARIANT
status: IMPLEMENTED
title: WSC-001 Authoritative Facts Do Not Imply Authoritative Coverage
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### WSC-019 Provider Health and Provider Authority Are Independent

| Property | Value |
|----------|-------|
| **Intent** | Sparse discovery diagnostics must distinguish provider failure from a healthy provider response that cannot authorize the request's required proof scope. |
| **Invariant** | Provider health/readiness and provider authority are independent. `cbm_scope_unavailable` means CBM did not provide authoritative coverage for the request's required scope or proof contract. CBM may still have responded successfully and contributed advisory candidates; this reason does not by itself indicate a CBM failure. `cbm_unavailable`, `cbm_discovery_failed`, and `cbm_partial_failure` retain their existing provider-state meanings. Optional provider output never becomes implicit authority for Clean-CTX semantic truth or completeness. |
| **Enforcement** | The `workspace_query` output schema documents the public interpretation. `src/tests/mcp/workspace_query_diagnostics.rs::red_diag3_fallback_reason_surfaced` pins the compatible combined-provider response after a ready successful CBM search; `src/tests/mcp/workspace_query_3.rs::workspace_query_schema_declares_sparse_discovery_diagnostics` pins the schema wording. |
| **Authority** | `src/mcp/tools.rs`, `src/mcp/tool_handlers/hydration.rs`, `src/tests/mcp/workspace_query_diagnostics.rs`, `src/tests/mcp/workspace_query_3.rs` |
| **Type** | DOCUMENTED and ENFORCED (test) |
| **Gate** | Focused workspace-query diagnostic and schema tests; complete repository gate in CI |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 558-570, original section `WSC-019`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `b41b1600cc442358339133329ba3e59c194edbd1143927e5362a9ce99a776292`. Original text remains in Git history; this file is the sole current normative owner of this section.

### AUTH-001 Refinement Joins Authority; It Never Manufactures It

**Status:** ENFORCED

Framework/domain classification, exact declaration binding, and source
occurrence evidence retain their own producers. A refinement stage may publish
a joined fact only when every required authority is current and compatible.
Name, spelling, proximity, or shared storage never supplies a missing authority.

**Enforcement:** producer-specific extraction plus exact Angular/C# refinement.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 704-714, original section `AUTH-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `8ed61eb177c11d38ea059065c3fb74556778d64aa68ccf6eeb8452a8f2d97ab2`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: AUTH-001
kind: INVARIANT
status: IMPLEMENTED
title: AUTH-001 Refinement Joins Authority; It Never Manufactures It
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### WSC-014 Completeness Is Earned from Evidence, Not Population

**Status:** ENFORCED

Completeness requires declared scope, complete proof coverage of every required
provider-neutral coverage unit, satisfied evidence requirements, a compatible
source generation, producer support, and valid dependencies.
File-scoped exactness is therefore scoped to supported producer patterns. A
detected unsupported framework form must be named in `semantic_omissions`; an
exact result inside that explicit scope cannot be interpreted as absence of the
omitted semantic class.
For NgRx object actions, the omission is removed per fact only when current
workspace reconciliation proves the qualified action member through exact
Store dispatch authority and proves the switch handler through exact reducer
registration plus case identity. Shorthand and reference-valued members retain
their qualified member identity while referencing an exact value declaration.
A bare switch case joins only through that declaration identity; equal text or
equal string values never join facts, and multiple authorized members backed by
one declaration remain ambiguous. Reducer-map members gain reducer authority
only through an exact NgRx registration consumer. Syntax recognition alone
never removes an omission.
Populated caches/indexes, provider success, or stable empty results never grant
exhaustiveness. Lack of a certificate means “not authorized to claim complete,”
not “the returned facts are false.”

**Enforcement:** `AcquisitionRequirement`, `CoverageUnit`, validated acquisition
proofs, `AcquisitionSatisfaction`, the proof-backed completion cache,
`CompletenessCertificate`, and producer-owned exact/lower-bound projection in
the workspace-query coverage path. Acquisition/cache/hydration tests and
`workspace_query_reverse_edge_completeness.rs` are the test authorities.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 791-821, original section `WSC-014`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `bc17720b212dde6ee412991d0404e7d22fb6c9f67b906c2173ddbbcef26a735e`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: WSC-014
kind: INVARIANT
status: IMPLEMENTED
title: WSC-014 Completeness Is Earned from Evidence, Not Population
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### SEMDEP-001 Semantic Validity Follows Dependencies

**Status:** ENFORCED

Source-cache, canonical-IR, and derived-semantic validity are separate. The file
owning an assertion need not be the full set of files proving it. Refresh,
restore, and publication validate/retract semantic dependencies rather than
assuming storage ownership proves current meaning.

**Enforcement:** WorkspaceIndex dependency projection, compatibility validation,
durable semantics, hydration invalidation, and refresh.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 851-862, original section `SEMDEP-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `7da11b157a7696d8ef40a522a47d991715169bb570d2253ebebb9ff9e0d33ea9`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: SEMDEP-001
kind: INVARIANT
status: IMPLEMENTED
title: SEMDEP-001 Semantic Validity Follows Dependencies
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### REFINE-001 Reconciliation Ownership Is Domain-Local

**Status:** ENFORCED

A reconciler may replace/refine only facts in the semantic domain it owns.
Sharing WorkspaceIndex, caches, or durable storage does not grant authority over
unrelated producer-owned facts.

**Enforcement:** domain-specific Angular template, NgRx, C#, and ASP.NET
reconciliation paths plus occurrence-preserving publication.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 908-918, original section `REFINE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `3b37c6e302437332270c4df2ba34577b1e175c637c2ea2a57f5435abd72c20c2`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: REFINE-001
kind: INVARIANT
status: IMPLEMENTED
title: REFINE-001 Reconciliation Ownership Is Domain-Local
track: FOUNDATION
artifact: docs/invariants/source-authority.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

```architecture-graph
node: WSC-019
kind: INVARIANT
status: IMPLEMENTED
title: WSC-019 Provider Health and Provider Authority Are Independent
track: REPOSITORY
artifact: docs/invariants/source-authority.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
