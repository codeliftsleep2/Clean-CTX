# Clean-CTX Language and Meta-Layer Certification Standard

**Status:** NORMATIVE  
**Applies to:** New language layers, framework/meta-layers, and material expansions of an existing supported semantic envelope

## 1. Purpose

Clean-CTX does not consider a language or framework supported merely because its syntax can be parsed, its files can be indexed, or a semantic producer exists.

A language or meta-layer is certified only when Clean-CTX can answer its declared architectural questions through the production interface using authoritative, current, queryable semantic knowledge with an explicit completeness contract.

The purpose of this standard is to make that requirement repeatable for every ecosystem Clean-CTX adds.

```text
SUPPORTED ECOSYSTEM
        ↓
CANONICAL QUESTIONS
        ↓
REQUIRED SEMANTIC FACTS
        ↓
REQUIRED AUTHORITY
        ↓
IDENTITY MODEL
        ↓
ACQUISITION / DEPENDENCY CLOSURE
        ↓
SEMANTIC IMPLEMENTATION
        ↓
PROVENANCE / CURRENTNESS
        ↓
PUBLIC QUERYABILITY
        ↓
COMPLETENESS CONTRACT
        ↓
ADVERSARIAL VERIFICATION
        ↓
EXTERNAL-ORACLE CONFORMANCE
        ↓
LIVE LLM VERIFICATION
        ↓
DUAL SIGN-OFF
        ↓
CERTIFIED
```

## 2. Governing Principles

The following rules apply throughout certification:

- Correctness over compression.
- Resolve by identity, render by position.
- Share mechanics, specialize semantics.
- Apply [MECHANISM-REUSE-001](ARCHITECTURAL_INVARIANTS.md#mechanism-reuse-001-equivalent-mechanics-are-shared-semantics-keep-their-owners): reuse equivalent authority/lifecycle/correctness mechanics and justify genuine specialization; correct answers alone do not prove correct ownership.
- Names are not identities.
- Proximity is not ownership.
- Framework syntax is not framework authority.
- A refinement layer may join authorities; it must not manufacture them.
- Store what source proves; synthesize what the system implies.
- Failure to understand evidence is not authority to discard it.
- Failure to sanitize is not authority to expose it.
- Source-cache validity, canonical-IR validity, and derived-semantic validity are distinct.
- Semantic dependencies govern derived validity.
- Correctness defines required work; performance removes repetition.
- Result filtering is not an acquisition budget.
- Warm state may improve performance. It must never improve or degrade truth.
- Equivalent authoritative evidence under equivalent semantic requirements must converge to equivalent truth regardless of cold/warm state, query order, batching, root order, cache population, or optional acquisition accelerators.

## 3. Certification Definition

```text
PARSER SUPPORT ≠ CERTIFIED LANGUAGE SUPPORT

SEMANTIC PRODUCER ≠ CERTIFIED META-LAYER

CERTIFIED =
    DECLARED ENVELOPE
  + CANONICAL QUESTIONS
  + AUTHORITATIVE IDENTITY
  + REQUIRED ACQUISITION
  + SEMANTIC CORRECTNESS
  + PROVENANCE
  + CURRENTNESS
  + PUBLIC QUERYABILITY
  + COMPLETENESS
  + ADVERSARIAL PROOF
  + CONFORMANCE PROOF WHERE FEASIBLE
  + LIVE CONSUMER PROOF
  + AGENT APPROVAL
  + CLEAN-CTX OWNER APPROVAL
```

A capability that is implemented but cannot answer the canonical question that justified it is incomplete.

A capability that can answer a question but cannot prove why the answer is authoritative, current, and complete to the claimed degree is incorrect.

Neither agent approval nor owner approval is sufficient alone. Certification requires both.

## 4. Required Certification Gates

### Gate 1 — Supported Envelope Declaration

Before certification, define the semantic envelope explicitly.

The declaration MUST identify supported language/framework versions or version families, project/configuration forms, syntax and semantic constructs relevant to Clean-CTX claims, package/module/composition mechanisms, known exclusions, intentionally unsupported constructs, and degradation behavior outside the supported envelope.

Broad labels such as `TypeScript support`, `Angular support`, or `EF support` are insufficient. The supported envelope is the boundary of the claim.

### Gate 2 — Canonical Question Matrix

Every material semantic capability MUST map to one or more real architectural questions that a user or LLM can ask.

Every in-envelope canonical question MUST identify:

```text
CANONICAL QUESTION
        ↓
SEMANTIC FACTS REQUIRED
        ↓
AUTHORITY / IDENTITY REQUIRED
        ↓
ACQUISITION / DEPENDENCY CLOSURE
        ↓
PRODUCTION QUERY / MCP PATH
        ↓
EXPECTED ANSWER SHAPE
        ↓
COMPLETENESS CONTRACT
        ↓
ADVERSARIAL / LIFECYCLE PROOF
```

Required question classes should be considered where applicable: Identity, Forward, Reverse, Why/Provenance, Scope/Visibility, Composition, Impact/Change, Completeness, Lifecycle, and Cross-Layer.

No semantic capability is complete merely because its internal data structure exists.

### Gate 3 — Authority Definition

For every fact produced by the layer, define what evidence is permitted to establish that fact.

The implementation MUST distinguish authoritative evidence from heuristics, hints, acquisition accelerators, and presentation conveniences.

Names, filenames, path proximity, decorator/annotation spelling, package-name resemblance, framework-looking syntax, cached conclusions, optional provider output, and model inference MUST NOT silently become authority unless the supported ecosystem itself defines them as authoritative for the claimed fact.

### Gate 4 — Canonical Identity Model

Define how relevant entities obtain durable semantic identity.

The model MUST address all identity-changing or identity-obscuring mechanisms inside the supported envelope, including where applicable files and physical/logical source identity, declarations, aliases, imports/exports, re-exports/barrels, packages/package instances, projects, namespaces/modules, declaration merging, generated/synthesized framework constructs, framework roles, and composition boundaries.

A new language or meta-layer MUST reuse existing lower-layer canonical identity where that authority already exists. It MUST NOT create a competing identity system for the same semantic entity.

### Gate 5 — Acquisition and Dependency Closure

Prove that Clean-CTX acquires all evidence required to answer each claimed canonical question.

Certification MUST establish required source, configuration and manifest/package acquisition; dependency traversal and transitive closure; framework composition requirements; and evidence dependencies for derived facts.

Result limits, display filters, or query scope MUST NOT prevent acquisition of evidence required for semantic correctness.

### Gate 6 — Semantic Implementation

Implement the semantics necessary to answer the declared canonical questions inside the supported envelope.

Clean-CTX does **not** need to reimplement an entire compiler, framework, runtime, IDE, or language service.

```text
CANONICAL QUESTION
       ↓
What facts are required?
       ↓
What semantics establish those facts?
       ↓
What authority is required?
       ↓
IMPLEMENT THAT
       ↓
VERIFY IT
       ↓
STOP
```

Semantics outside the declared Clean-CTX question envelope are not required merely because an external compiler or framework implements them.

Conversely, a known semantic requirement inside the declared envelope MUST NOT be omitted and explained away as unsupported after certification is claimed. The envelope must either be implemented or explicitly narrowed.

### Gate 7 — Provenance and Evidence Witnesses

Every authoritative derived fact MUST be traceable to the authoritative evidence that established it to the degree required by its claim.

Where applicable, retained proof should identify source snapshot/exact bytes/hash, declaration/source span, project/configuration context, package/module context, authority or semantic-contract version, dependency/composition witnesses, and generation/publication identity.

The system must be able to explain **why** a fact is believed, not merely return the fact.

### Gate 8 — Lifecycle and Currentness

Certification MUST prove that semantic truth changes when authoritative evidence changes.

Tests MUST include applicable mutations such as edit, rename, move, delete, dependency/configuration/package/version/composition changes, authority loss, and ambiguity introduction/removal.

Affected semantic facts MUST update or retract correctly.

Equivalent truth MUST NOT depend on warm versus cold state, previous query order, batching, cache population, root order, partial prior acquisition, or optional provider state.

### Gate 9 — Persistence and Publication

If semantic knowledge survives beyond immediate computation, prove that retained/public state corresponds to the authoritative generation from which it was derived.

Certification MUST address persistence format, semantic dependency retention, invalidation, publication ordering, stale-fact retraction, generation compatibility, and reconstruction after restart where applicable.

Intermediate correctness does not certify final published state.

### Gate 10 — Production Query Reachability

Every in-envelope canonical question MUST have a production-reachable path from authoritative evidence to the public Clean-CTX interface.

```text
authoritative source
    ↓
acquisition / dependency closure
    ↓
canonical identity
    ↓
semantic producer
    ↓
retained fact + proof
    ↓
publication / persistence
    ↓
query evaluator
    ↓
MCP / public tool response
    ↓
canonical question answer
```

An internal fact that cannot be queried through the production path is not a completed feature.

### Gate 11 — Completeness Contract

Every public answer MUST classify its completeness honestly.

The layer MUST distinguish, where applicable: exact/complete, complete within a declared bound, incomplete/partial, ambiguous, unsupported, authority unavailable, stale/not-current, and indeterminate.

`These are the results found` MUST NOT be represented as `these are all results` without proof of the required closure.

### Gate 12 — Adversarial Regression Suite

The certification suite MUST deliberately attempt to falsify the implementation.

Applicable cases include identity collisions, same-name unrelated entities, aliases, re-exports, cycles, shadowing, declaration merging, ambiguous resolution, type/value or equivalent semantic spaces, malformed/partial source, unsupported versions, missing dependencies, partial acquisition, configuration conflicts, mutation/deletion, cold/warm transitions, cache invalidation, query/batch ordering, persistence/restart, and framework-specific edge cases.

Every discovered semantic failure MUST become a regression when it represents an in-envelope correctness requirement.

### Gate 13 — Official-Implementation Differential Conformance

When a supported semantic claim overlaps behavior established by an authoritative external implementation, Clean-CTX SHOULD maintain development-time differential conformance evidence against that implementation where technically feasible.

Examples include official language compilers, framework compilers, analyzers, or runtime metadata systems.

The external implementation is a **development-time oracle**, not a production dependency.

```text
                     SAME FIXTURE
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
      Official Implementation    Clean-CTX
              │                     │
              └──────────┬──────────┘
                         ▼
                      COMPARE
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
            MATCH                MISMATCH
              │                     │
          evidence            investigation
                                  ↓
                            regression/repair
```

A mismatch MUST be investigated rather than automatically classified as a Clean-CTX defect. The investigation must determine whether the fixture is inside the supported envelope, both systems consumed equivalent evidence, both answer the same semantic question, external behavior is version-specific, or Clean-CTX is incorrect.

#### ORACLE-CONFORMANCE-001

> When a supported semantic claim overlaps behavior established by an authoritative external implementation, Clean-CTX SHOULD maintain development-time differential conformance evidence against that implementation where technically feasible. External implementations may falsify or corroborate Clean-CTX during development but MUST NOT become required production authorities or runtime dependencies.

#### Conformance Scope Rule

> Conformance scope follows Clean-CTX's declared semantic envelope. Clean-CTX does not implement semantics merely because an external compiler or framework implements them.

```text
PRODUCTION
Repository → Clean-CTX native authority → semantic graph → queries → LLM

DEVELOPMENT
Fixture → Official implementation ─┐
                                   ├→ differential comparison
Fixture → Clean-CTX ───────────────┘
```

### Gate 14 — Performance Characterization

Performance work follows semantic correctness.

Certification SHOULD characterize cold/warm acquisition cost, semantic derivation cost, persistence/publication cost, query and batch-query cost, repeated work, memory impact, and invalidation/recomputation behavior.

Optimizations MUST remove redundant work without weakening required acquisition, authority, provenance, currentness, or completeness.

### Gate 15 — Live LLM / Consumer Verification

Where the layer exists to support model reasoning, certification MUST include representative live consumer verification.

The verification should force the consumer to answer canonical architectural questions using the production representation/query path and establish that required knowledge is available; identity survives presentation; ambiguity is preserved; completeness metadata is usable; provenance supports reasoning where required; and the consumer can reach the intended conclusion without reconstructing missing semantics from raw source.

Presence of semantic data is not sufficient if the intended consumer cannot correctly use it.

### Gate 16 — Certification Artifact and Dual Sign-Off

Certification MUST produce a durable evidence artifact containing at least:

- layer name and version;
- certification date;
- supported envelope;
- known exclusions;
- Canonical Question Matrix reference;
- authority model;
- identity model;
- acquisition/dependency requirements;
- production query paths;
- completeness classifications;
- lifecycle/currentness evidence;
- adversarial suite evidence;
- external oracle(s) and oracle version(s), where used;
- differential-conformance results;
- performance characterization;
- live consumer verification;
- unresolved limitations;
- proposed certification verdict;
- agent attestation;
- Clean-CTX owner approval.

Certification is not complete until the evidence artifact supports the claimed verdict **and both required sign-offs approve the same certification scope**.

Neither the implementation/investigation agent nor the Clean-CTX owner may unilaterally certify a layer.

## 5. Language-Layer Certification

A language layer establishes language-level authority consumed by higher layers.

Depending on the language and supported envelope, certification may require authority for syntax/declarations, lexical scope, symbol/declaration identity, imports/exports/modules, package/project resolution, aliases/re-exports, type/value or equivalent semantic spaces, inheritance/implementation relationships, references, dependency relationships, declaration merging/augmentation, and configuration-driven semantics.

A language layer MUST expose authoritative identity and relationships in a form reusable by dependent meta-layers. Higher layers MUST NOT independently rediscover language identity when certified language authority already exists.

## 6. Meta-Layer Certification

A meta-layer begins above certified lower-layer authority.

```text
CERTIFIED LANGUAGE AUTHORITY
            ↓
FRAMEWORK API AUTHORITY
            ↓
FRAMEWORK IDENTITY / ROLES
            ↓
COMPOSITION / VISIBILITY
            ↓
SPECIALIZED SEMANTICS
            ↓
FRAMEWORK CANONICAL QUESTIONS
```

A meta-layer MUST declare its authority dependencies explicitly.

```text
NgRx Meta-Layer
    requires → Angular Composition Authority
    requires → Angular API Authority
    requires → TypeScript Identity Authority
```

A meta-layer cannot claim stronger certification than the lower authority on which its claimed semantics depend.

If a required lower-layer capability is uncertified, unsupported, stale, ambiguous, or incomplete, the dependent meta-layer MUST preserve that limitation rather than manufacture certainty.

## 7. Semantic Certification DAG

Clean-CTX SHOULD treat certified semantic layers as a dependency graph rather than isolated feature modules.

Each certification artifact should identify authorities it consumes, facts it produces, downstream layers that may consume those facts, invalidation dependencies, and certification prerequisites.

```text
Language Identity Authority
          ↓
Framework API Authority
          ↓
Framework Composition Authority
          ↓
Specialized Meta-Layer Authority
          ↓
Canonical Questions
```

## 8. External Dependency Boundary

Clean-CTX's production architecture SHOULD remain independent of heavyweight ecosystem compilers/framework runtimes unless a separate architecture decision explicitly changes that rule.

> Implement only the semantics Clean-CTX needs to prove its declared architectural claims, and use official implementations during development to verify that those semantics match where the claims overlap.

Adding support for a new ecosystem MUST NOT automatically imply shipping or requiring that ecosystem's compiler, framework compiler, runtime, language server, package manager, or development environment.

## 9. Required Certification Checklist

A reviewer MUST be able to mark every applicable item before certification is granted.

### Scope
- [ ] Supported semantic envelope is explicitly documented.
- [ ] Supported versions/version families are documented.
- [ ] Known exclusions and degradation behavior are documented.
- [ ] No broad support claim exceeds the declared envelope.

### Canonical Questions
- [ ] Material capabilities map to canonical questions.
- [ ] Required question classes have been considered.
- [ ] Every in-envelope question maps to required semantic facts.
- [ ] Every in-envelope question has a production query path.

### Authority and Identity
- [ ] Authoritative evidence is explicitly defined.
- [ ] Heuristics/hints are distinguished from authority.
- [ ] Canonical identity is defined for relevant entities.
- [ ] Aliases/merges/re-exports/composition forms are handled where applicable.
- [ ] Existing lower-layer identity is reused rather than duplicated.

### Acquisition
- [ ] Required source/configuration/package evidence is acquired.
- [ ] Dependency closure is proven for claimed questions.
- [ ] Result limits cannot truncate required authority acquisition.

### Mechanism Reuse and Specialized Semantics
- [ ] Each material consumer traces identity/authority origins, acquisition, provenance, currentness, persistence/restoration, publication, deduplication, selection/replacement/retirement and completeness to their actual owners.
- [ ] Available lower-layer authority is retained, transported and consumed rather than independently reconstructed through competing paths.
- [ ] Equivalent cross-cutting contracts reuse shared machinery; parallel implementations have an explicit architectural reason or are recorded as unresolved architectural defects, not silently certified by passing point tests.
- [ ] Specialized producers identify the evidence, authority, lifecycle, identity, completeness or query contract that requires their difference; source similarity or a common consumer alone is not grounds for consolidation.
- [ ] Proposed sharing follows demonstrated requirements and preserves behavioral/integration obligations; no hypothetical universal framework or unexplained independent lifecycle was introduced.

### Semantics
- [ ] Required in-envelope semantics are implemented.
- [ ] Known in-envelope gaps are resolved or the envelope is explicitly narrowed.
- [ ] No unnecessary attempt has been made to reimplement semantics unrelated to Clean-CTX claims.

### Provenance and Lifecycle
- [ ] Derived facts retain sufficient provenance/witnesses.
- [ ] Mutation updates affected facts.
- [ ] Deletion retracts affected facts.
- [ ] Authority loss retracts or degrades affected claims.
- [ ] Cold/warm state does not change truth.
- [ ] Query/batch/root order does not change truth.

### Persistence and Publication
- [ ] Persisted/public facts correspond to authoritative generations.
- [ ] Semantic dependencies participate in invalidation.
- [ ] Restart/reload behavior is verified where applicable.
- [ ] Final published state, not merely intermediate state, is certified.

### Completeness
- [ ] Public answers classify completeness correctly.
- [ ] Partial results cannot masquerade as complete results.
- [ ] Ambiguity is preserved.
- [ ] Unsupported/out-of-envelope evidence is represented honestly.

### Adversarial Verification
- [ ] Identity collisions are tested.
- [ ] Alias/re-export/merge/shadowing cases are tested where applicable.
- [ ] Cycles and ambiguity are tested where applicable.
- [ ] Missing/partial evidence is tested.
- [ ] Mutation/deletion/configuration changes are tested.
- [ ] Cache/order/batching/lifecycle behavior is tested.
- [ ] Discovered in-envelope failures have regression coverage.

### Differential Conformance
- [ ] Applicable official implementation/oracle has been identified.
- [ ] Differential conformance is implemented where technically feasible.
- [ ] Oracle version and fixture environment are recorded.
- [ ] Mismatches are investigated rather than blindly attributed.
- [ ] Oracle tooling is development-only unless separately authorized architecturally.
- [ ] Conformance scope does not expand the Clean-CTX semantic envelope unnecessarily.

### Consumer Verification
- [ ] Canonical questions are answered through the production interface.
- [ ] Live consumer/LLM verification exists where applicable.
- [ ] The consumer can use identity, ambiguity, provenance, and completeness correctly.

### Sign-Off
- [ ] Certification artifact exists.
- [ ] All required evidence is referenced.
- [ ] Known limitations are explicit.
- [ ] Lower-layer certification prerequisites are satisfied.
- [ ] Agent attestation is complete.
- [ ] Clean-CTX owner approval is complete.
- [ ] Both approvals cover the same semantic envelope and verdict.
- [ ] Final verdict is recorded as CERTIFIED, BOUNDED CERTIFIED, or NOT CERTIFIED.

## 10. Certification Verdicts

### CERTIFIED

All required gates are satisfied for the declared envelope and no known limitation weakens the claimed completeness or authority contract. Both required sign-offs approve this scope.

### BOUNDED CERTIFIED

All required gates are satisfied for an explicitly constrained envelope. The boundary is public, testable, preserved in query/completeness behavior, and approved by both required signatories.

This is a valid production state. It MUST NOT be marketed or represented as broader support.

### NOT CERTIFIED

One or more required gates remain unsatisfied, required authority is unavailable, production queryability is missing, lifecycle correctness is unproven, completeness claims exceed evidence, or both required approvals have not been obtained.

A parser, prototype, experimental semantic producer, or partially migrated meta-layer may be useful while remaining NOT CERTIFIED.

## 11. Dual Certification Sign-Off

Certification requires independent approval from both the implementation/investigation agent and the Clean-CTX owner.

The two signatories serve different purposes. The agent attests that the technical investigation and evidence support the claimed verdict. The owner explicitly accepts that proven scope into Clean-CTX's production semantic contract.

Neither approval is sufficient alone.

### Agent Attestation

```text
Agent: __________________________
Role: ___________________________
Date: ___________________________
Evidence revision/commit: _______

I attest that:

- I evaluated every applicable certification gate.
- I disclosed all known failures, ambiguities, unsupported cases,
  contradictory evidence, and unresolved semantic risks discovered
  during certification.
- The cited evidence supports the declared certification verdict.
- I did not treat passing tests as proof beyond the scope they exercise.
- I did not knowingly omit evidence that weakens or contradicts the claim.
- External-oracle agreement, where used, was treated as conformance
  evidence rather than production authority.
- The supported envelope does not exceed what the evidence proves.

Agent verdict:

[ ] CERTIFIED
[ ] BOUNDED CERTIFIED
[ ] NOT CERTIFIED

Agent signature/identifier: __________________
```

### Clean-CTX Owner Approval

```text
Owner: __________________________
Date: ___________________________
Reviewed revision/commit: _______

I have reviewed:

- the declared supported envelope;
- Canonical Question Matrix coverage;
- authority and identity model;
- mechanism reuse and justified semantic specialization, including explained parallel authority/lifecycle paths;
- acquisition/dependency closure;
- lifecycle/currentness evidence;
- completeness contract;
- adversarial findings;
- differential-conformance evidence where applicable;
- production query reachability;
- live consumer verification where applicable;
- known limitations and exclusions;
- final certification verdict.

Owner decision:

[ ] APPROVED
[ ] APPROVED AS BOUNDED CERTIFIED
[ ] REJECTED / ADDITIONAL EVIDENCE REQUIRED

Owner signature: __________________
```

### Final Certification State

```text
AGENT APPROVAL + OWNER APPROVAL = CERTIFICATION

Either approval missing = NOT YET CERTIFIED

Scope mismatch between approvals = NOT YET CERTIFIED
```

A layer MUST NOT enter `CERTIFIED` or `BOUNDED CERTIFIED` state unless both attestations approve the same semantic envelope and compatible verdict.

### RECERTIFICATION-001

> Any material change to a certified layer's semantic envelope, authority model, canonical identity model, completeness contract, authority dependencies, or production truth path requires recertification of the affected scope.

A change that cannot affect the certified semantic contract does not automatically require full recertification, but the certification artifact MUST identify the basis for treating the existing certification as still applicable.

## 12. Definition of Done

A new language or meta-layer is done only when:

```text
SEMANTIC IMPLEMENTATION
    +
AUTHORITATIVE IDENTITY
    +
REQUIRED DEPENDENCY CLOSURE
    +
CURRENTNESS / PUBLICATION PROOF
    +
CANONICAL QUESTION COVERAGE
    +
PUBLIC QUERY REACHABILITY
    +
CORRECT COMPLETENESS SEMANTICS
    +
ADVERSARIAL / LIFECYCLE VERIFICATION
    +
DIFFERENTIAL CONFORMANCE WHERE FEASIBLE
    +
LIVE CONSUMER VERIFICATION WHERE APPLICABLE
    +
AGENT ATTESTATION
    +
CLEAN-CTX OWNER APPROVAL
    =
CERTIFIED LANGUAGE / META-LAYER
```

The governing question is not:

> Did we implement support for this language or framework?

It is:

> Can Clean-CTX prove the architectural facts it claims to know about this ecosystem, keep those facts current, expose them through the production interface with an honest completeness contract, demonstrate that the intended consumer can rely on them, and obtain independent agent and owner approval of that exact claim?

Only then is the layer complete.
