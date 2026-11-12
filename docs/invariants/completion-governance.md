<!-- pcg:family-standing:begin -->
# Invariant family: completion-governance

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:ARCH-004:begin -->
### ARCH-004 Production Integration Is the Completion Boundary

| Property | Value |
|----------|-------|
| **Intent** | Prevent implemented but unreachable features from being reported as complete. |
| **Invariant** | A component or semantic family is not complete until its real default production lifecycle is traced and evidenced: producer, production pipeline, result boundary, persistent owner, workspace/session lifecycle, actual consumer, MCP/API exposure, and applicable live reachability. Code existence, populated test-only state, custom-pipeline tests, and isolated unit tests are implementation evidence only. |
| **Enforcement** | Phase 9 records concrete entry points, owners, lifecycle behavior, consumers, external exposure, tracked production-path coverage, and removal of obsolete bypasses. The completed `0.8.0` production certification is the reference application of this invariant; future work must satisfy the same boundary independently. |
| **Authority** | `AGENTS.md` §11, `docs/architecture/IR_ARCHITECTURE_CERTIFICATION.md`, and the Phase 9 audit records |
| **Type** | GOVERNANCE and INTEGRATION |
| **Gate** | Production lifecycle trace, applicable tracked integration tests, and applicable live end-to-end verification |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 129-141, original section `ARCH-004`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `191aebaa0301b5253eafae229af494b43585ced58b4f88c3994b06770c463f16`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: ARCH-004
kind: INVARIANT
status: IMPLEMENTED
title: ARCH-004 Production Integration Is the Completion Boundary
track: FOUNDATION
artifact: docs/invariants/completion-governance.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#ARCH-004
  - qualification self#family-standing
```

<!-- pcg:ARCH-004:end -->
<!-- pcg:INV.MECHANISM_REUSE_001:begin -->
### MECHANISM-REUSE-001 Equivalent Mechanics Are Shared; Semantics Keep Their Owners

**Status:** DOCUMENTED (architectural and certification review; not a new executable gate)

Cross-cutting semantic mechanics SHOULD be shared across language and meta-layers
whenever their authority, lifecycle and correctness contracts are equivalent.
Specialized layers own semantic interpretation and policy; they SHOULD NOT
independently recreate equivalent lower-layer authority or shared infrastructure.
A parallel implementation of equivalent machinery requires an explicit architectural
reason. Equivalence of contract, rather than similarity of code, governs reuse.

When a lower layer already establishes a fact authoritatively, a higher layer
MUST NOT manufacture competing authority for that fact merely for convenience.
Prove at the lowest authoritative owner, retain and transport the evidence,
reuse it, then specialize its meaning above that boundary. This complements
AUTH-001, ARCH-002/003/004 and the question-driven semantic envelope; it does not
replace any authority, identity, completeness or production-integration obligation.

For each material addition, first distinguish mechanics from semantic interpretation.
For mechanics, inspect existing acquisition/snapshots, project context, identity
transport, provenance/witnesses, dependencies/currentness, generations/compatibility,
persistence/restoration, publication, relation deduplication/selection/replacement/
retirement, completeness and query transport. Reuse or extend an existing mechanism
when those contracts match. When none matches, choose the smallest correctly owned
mechanism justified by demonstrated consumers. Interpret specialized meaning at its
authoritative layer: language binding in the language authority, Angular roles in
Angular, and DI/NgRx/forms/routes or other specialized relationships in their owners.

Different evidence, authority, lifecycle dependencies, identity forms, completeness,
publication, invalidation or query contracts can justify specialized implementations.
Record the actual difference; a shared consumer does not by itself require a shared
producer. Repeated code is a review signal, not automatic proof of a defect. Compare
concrete requirements before extracting common mechanics; do not build a universal
framework around hypothetical future layers or remove specialized machinery before
equivalence has been demonstrated.

Material consumer reviews MUST identify identity/authority origins; acquisition and
retained provenance; currentness and persistence/restoration; publication, duplicate
prevention and retirement; completeness representation; shared infrastructure and
specialized interpretation. Explain equivalent parallel paths and planned migration
or their explicit architectural justification. Correct point answers alone do not
prove correct ownership. Review applies to every layer/meta-layer touched, while
unrelated historical debt remains separately recorded rather than triggering an
unbounded repository-wide refactor.

**Enforcement:** existing architecture audit and Language/Meta-Layer Certification
Standard §9 review. This convention does not claim automatic enforcement, certify
current implementations, authorize a redesign or change the existing runtime gate.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 715-763, original section `MECHANISM-REUSE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `28c629c4a4e0819dc25af3b6a33635348ad6b8e7054c319375dc853805ab3748`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: INV.MECHANISM_REUSE_001
kind: INVARIANT
status: INVESTIGATING
title: MECHANISM-REUSE-001 Equivalent Mechanics Are Shared; Semantics Keep Their Owners
track: FOUNDATION
artifact: docs/invariants/completion-governance.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.MECHANISM_REUSE_001
  - qualification self#family-standing
```

<!-- pcg:INV.MECHANISM_REUSE_001:end -->
<!-- pcg:INV.META_LAYER_COMPLETENESS_001:begin -->
### META-LAYER-COMPLETENESS-001 Supported Framework Semantics Are Complete

| Property | Value |
|----------|-------|
| **Invariant** | When Clean-CTX claims semantic support for a framework or language construct, materially used semantic forms within that supported capability must be implemented completely enough to preserve the framework relationships Clean-CTX claims to understand. A known capability gap is not resolved merely by documenting, labeling, degrading, or explaining away missing semantics when sufficient authoritative evidence can be modeled. |
| **Safety boundary** | Unsupported/degradation metadata is a truthful safety boundary, not a substitute for required semantic capability. Truthful degradation is a safety mechanism, not a completion criterion. |
| **Engineering rule** | Do not explain away an implementable semantic gap. Either implement the required structured authority or explicitly narrow the meta-layer's supported envelope. A materially used idiom inside the intended envelope must not remain a permanent omission merely because support requires additional identity, reconciliation, or lifecycle machinery. |
| **Constraint** | Completeness never authorizes guessing: framework roles require structured identity and framework-specific authority; lookalike syntax fails closed. Meta-layers must provide no fabricated semantics, no silent gaps, and no knowingly half-implemented contract inside their declared envelope. |
| **NgRx application** | Object-member actions require exact Store dispatch authority. Shorthand/reference-valued members preserve qualified member identity and carry a separate exact declaration reference. Bare switch cases join through that declaration identity, never spelling or literal equality, and ambiguity fails closed. Direct reducers and reducer-map members require exact NgRx registration authority; map shape alone grants none. Aliases normalize to declaration identity; handlers publish only after Action, Reducer, and case identity reconcile. Supported registration queries acquire their bounded dependency closure before claiming exact completeness. Derived roles and handlers retract with their last supporting proof. |
| **Type** | STRUCTURAL and ENFORCED |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 1057-1069, original section `META-LAYER-COMPLETENESS-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `b20ac0b26f1c59f1277dd9cda45f24fb87144693595ee3fc772fbc4372c5ef62`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: INV.META_LAYER_COMPLETENESS_001
kind: INVARIANT
status: IMPLEMENTED
title: META-LAYER-COMPLETENESS-001 Supported Framework Semantics Are Complete
track: FOUNDATION
artifact: docs/invariants/completion-governance.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.META_LAYER_COMPLETENESS_001
  - qualification self#family-standing
```
<!-- pcg:INV.META_LAYER_COMPLETENESS_001:end -->
