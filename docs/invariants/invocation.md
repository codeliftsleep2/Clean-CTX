<!-- pcg:family-standing:begin -->
# Invariant family: invocation

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:INV.INVOCATION_001:begin -->
### INVOCATION-001 Written Invocation Evidence Is Source-Bound and Ephemeral

| Property | Value |
|----------|-------|
| **Invariant** | TypeScript and C# invocation occurrences retain written UTF-8 byte spans, callee structure, ordered arguments and qualifiers, type arguments, immediate invocation parents, and supported declaration contexts. They contain no resolved target or framework meaning. Source slices require a view bound to the producing source identity and SHA-256. Edits cannot reuse the old set as current evidence. |
| **Boundary** | Evidence is compilation-local, cleared before compilation and published only after success. It remains outside CompiledIR, canonical formats, semantic edges, WorkspaceIndex, producer generations, and persistence. CoreOp::Call and CallEvidence remain their existing compact canonical authorities. Unsupported targets retain occurrences without invented terminals. |
| **Enforcement** | Private structural types and validated slicing; same-parse capture assembly; cross-language consumer parity, identity/hash mismatch, edit/empty replacement, and populated-failure regressions. |
| **Authority** | `src/ir/invocation_evidence.rs`; `src/ir/invocation_evidence/collect.rs`; `src/tests/ir/invocation_generic_consumer.rs`; `src/tests/ir/invocation_evidence_lifecycle.rs` |
| **Type** | STRUCTURAL + ENFORCED |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 973-982, original section `INVOCATION-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `3c7116fc92556fa7ba3766d3df7578e7c09d46579e7e0e6e858ad0785a4efbf7`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.INVOCATION_001:end -->
<!-- pcg:INV.INVOCATION_002:begin -->
### INVOCATION-002 RxJS Pipe Structure Uses the Shared Occurrence Authority

| Property | Value |
|----------|-------|
| **Invariant** | Production RxJS pipe discovery and top-level operator argument boundaries consume the compiler's borrowed validated occurrence view, including authoritative empty evidence. They do not reparse or reconstruct pipe bodies with substring scans or comma splitting. The standalone source API performs one TypeScript capture walk through the same collector because it has no upstream parse. |
| **Domain boundary** | RxJS retains operator interpretation, textual owner policy, minimum-operator filtering, shape and rendering. Combinator scanning remains outside the migrated slice. Unavailable structural evidence does not fall back to textual pipe discovery. |
| **Enforcement** | Production and standalone parse-count assertions, spaced/commented syntax regression, frozen shape/operator assertions, lexical-decoy and ownership preservation suites. |
| **Authority** | `src/ir/pipeline/meta_layer.rs`; `src/layers/meta/context.rs`; `src/angular_meta/rx/pipes.rs`; `src/tests/angular_meta/rx_invocation_evidence.rs` |
| **Type** | ENFORCED |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 983-992, original section `INVOCATION-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `bbe6ed63ad11e6ecc36d9cbe2e2427b16b26559032a04f6502943f622d9c6eb9`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.INVOCATION_002:end -->
<!-- pcg:INV.INVOCATION_003:begin -->
### INVOCATION-003 Exact C# Calls Use Capability-Gated Artifact Authority

| Property | Value |
|----------|-------|
| **Invariant** | A durable C# `Calls` edge to a `csharp/Method` identity is published only when current structured receiver evidence and the complete admitted C# declaration authority select one owner/signature-bearing target. Missing, incompatible, or ambiguous targets publish no exact edge. Generic written-call evidence remains unchanged. |
| **Lifecycle** | One parse produces a source-bound semantic artifact containing compiled IR, local declarations, invocation evidence, non-call semantic facts, source identity, fidelity, and producer generations. Publication retains the artifact and dirties exact-call authority. Reverse TypeRef and unrelated consumers never prepare Calls authority. The first exact C# Calls consumer validates or compiles only missing/stale artifacts, derives one deterministic capability fingerprint from the complete admitted manifest, projects from retained artifacts without rereading or reparsing them, and advances exact-call coverage only after a complete generation succeeds. Additions, removals, renames, signature changes, ambiguity, scope policy, and producer changes can therefore establish or revoke exact facts independent of file order or warm history. |
| **Concurrency and commit** | Compilation uses the session-owned bounded Rayon executor and returns provisional immutable artifacts in canonical input order. Workers do not mutate shared authority. The coordinator rejects stale source/generation results and publishes projections plus exact-call coverage under one index write boundary; a failed or superseded preparation remains dirty and cannot be read as current. Concurrent consumers serialize/coalesce at the lifecycle boundary. |
| **Fidelity preservation** | Exact-call projection may strengthen exact-call evidence, but it preserves each artifact publication's semantic fidelity. Preparing Calls authority never upgrades a Low or Medium projection to High. A later explicit higher-fidelity request remains the only authority for that upgrade. Dirty Calls authority does not make independently current TypeRef evidence inadequate. |
| **Boundary** | Retained invocation evidence remains source-bound under INVOCATION-001 and is owned only by the in-session artifact store; artifacts are not durable persistence. Exact-call currentness is capability-scoped through `CSharpAuthorityFingerprint`, not modeled as per-edge all-authority-file dependencies. Provider availability and query target identity do not participate in semantic truth or the fingerprint. |
| **Enforcement** | `SemanticCompilationArtifact` is the compiler return boundary. `CSharpCallAuthorityState` owns artifacts and dirty/current generation state; query preparation is the sole exact-call capability gate; `WorkspaceIndex` separately owns capability coverage. Structural work counters pin K-only compilation, zero eager TypeRef projection, artifact-backed projection, one manifest per generation, and O(1) warm currentness. Lifecycle, artifact-reconciliation, fingerprint, ambiguity, deletion/restore, and fidelity regressions enforce fail-closed behavior. |
| **Compatibility** | Exact C# call output is owned by `CSharpSemanticProjection` generation 5. |
| **Authority** | `src/ir/csharp_exact_calls.rs`; `src/mcp/semantic_compilation/`; `src/mcp/csharp_calls/`; `src/workspace/index/capability_coverage.rs`; `src/tests/mcp/issue_144_performance_work_counts.rs`; `src/tests/mcp/csharp_call_authority_lifecycle.rs`; `src/tests/mcp/csharp_call_artifact_reconciliation.rs`; `src/tests/mcp/csharp_call_authority_fingerprint.rs` |
| **Type** | ENFORCED |
| **Gate** | Focused Issue #144 artifact, parallelism, lifecycle, reconciliation, fingerprint, and work-count regressions; then the authoritative Final Verification Gate. |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 993-1009, original section `INVOCATION-003`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `9e00ada0bb79964e4b4cb9ef953176ea3334f2d9872e1befb362e7ec72a24d31`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: INV.INVOCATION_001
kind: INVARIANT
status: IMPLEMENTED
title: INVOCATION-001 Written Invocation Evidence Is Source-Bound and Ephemeral
track: REPOSITORY
artifact: docs/invariants/invocation.md
preserves:
  - CTX-001
note: Current source-described contract; no witness execution or independent certification is inferred.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.INVOCATION_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.INVOCATION_003
kind: INVARIANT
status: IMPLEMENTED
title: INVOCATION-003 Exact C# Calls Use Capability-Gated Artifact Authority
track: REPOSITORY
artifact: docs/invariants/invocation.md
depends_on:
  - INV.INVOCATION_001
preserves:
  - WSC-012
  - PROJ-001
note: Current source-described contract; no witness execution or independent certification is inferred.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.INVOCATION_003
  - qualification self#family-standing
```

```architecture-graph
node: INV.INVOCATION_002
kind: INVARIANT
status: IMPLEMENTED
title: INVOCATION-002 RxJS Pipe Structure Uses the Shared Occurrence Authority
track: REPOSITORY
artifact: docs/invariants/invocation.md
depends_on:
  - INV.INVOCATION_001
note: Operative bounded source contract; historical approval and execution remain attributed, not language certification or runtime permission.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.INVOCATION_002
  - qualification self#family-standing
```
<!-- pcg:INV.INVOCATION_003:end -->
