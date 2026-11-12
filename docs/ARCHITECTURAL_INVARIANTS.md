# Clean-CTX Architectural Invariants

**Purpose:** Make Clean-CTX's important architectural decisions visible, identify how they are currently enforced (type system, compiler, tests, or convention), and establish a pattern for future architectural governance.

**Audience:** Developers contributing to the Clean-CTX codebase.

**Current role:** Navigation and invariant-governance procedure. Individual normative sections now live under `docs/invariants/`; stable legacy anchors below route to their sole current owners. This index does not duplicate the relocated normative propositions. Source history and enforcement attributes are retained there.

---

## Status Classifications

| Status | Meaning |
|--------|---------|
| **ENFORCED** | Actively enforced by tests or tooling. Failure blocks CI. |
| **STRUCTURAL** | Enforced by Rust's type system or compiler. Violation requires changing type signatures. |
| **DOCUMENTED** | Architectural convention currently not machine-enforced. Violation is possible but should trigger a design discussion. |
| **DEFERRED** | Important architectural decision intentionally postponed. |
| **PROPOSED** | Under consideration but not yet accepted. |
| **RESOLVED** | Previously documented architectural debt that has been completed. |

## Architectural Gate

The architectural gate is the existing CI pipeline:

```
cargo test
+
cargo clippy --all-targets -- -D warnings
```

No separate executable, trait, registry, or framework is used. Each invariant below identifies which part of the gate enforces it.

---

## Invariant Catalog

### WIRE-001 Canonical IR Serialization Stability

Current normative owner: [WIRE-001](invariants/canonical-ir.md#wire-001-canonical-ir-serialization-stability). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### VALID-001 IR Structural Validity

Current normative owner: [VALID-001](invariants/canonical-ir.md#valid-001-ir-structural-validity). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### DELTA-001 Delta Correctness

Current normative owner: [DELTA-001](invariants/delta-lifecycle.md#delta-001-delta-correctness). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ARCH-001 Inference State Is Ephemeral

Current normative owner: [ARCH-001](invariants/canonical-ir.md#arch-001-inference-state-is-ephemeral). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ARCH-002 Language-Agnostic Canonical IR Boundary

Current normative owner: [ARCH-002](invariants/canonical-ir.md#arch-002-language-agnostic-canonical-ir-boundary). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ARCH-003 Canonical IR and LLM Projection Are Separate Contracts

Current normative owner: [ARCH-003](invariants/projection-fidelity.md#arch-003-canonical-ir-and-llm-projection-are-separate-contracts). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CONTENT-001 Visible Content Metadata Describes Only the Visible Text

Current normative owner: [CONTENT-001](invariants/projection-fidelity.md#content-001-visible-content-metadata-describes-only-the-visible-text). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ARCH-004 Production Integration Is the Completion Boundary

Current normative owner: [ARCH-004](invariants/completion-governance.md#arch-004-production-integration-is-the-completion-boundary). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### PIPELINE-001 Compilation Pipeline Ordering

Current normative owner: [PIPELINE-001](invariants/canonical-ir.md#pipeline-001-compilation-pipeline-ordering). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### C-22 — Meta-Layer Source Context from Canonical Capture Identity

Current normative owner: [C-22](invariants/meta-evidence.md#c-22--meta-layer-source-context-from-canonical-capture-identity). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### META-001 Framework Annotations Retain Declaration-Local Ownership and Intelligible Values

Current normative owner: [META-001](invariants/meta-evidence.md#meta-001-framework-annotations-retain-declaration-local-ownership-and-intelligible-values). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### META-002 Compilation-Scoped Single Evaluation

Current normative owner: [META-002](invariants/meta-evidence.md#meta-002-compilation-scoped-single-evaluation). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-ID-001 Canonical CBM Project Identity & Multi-Root Lifecycle

Current normative owner: [CBM-ID-001](invariants/cbm.md#cbm-id-001-canonical-cbm-project-identity--multi-root-lifecycle). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-LIFE-001 Optional Provider Never Gates Core MCP Readiness

Current normative owner: [CBM-LIFE-001](invariants/cbm.md#cbm-life-001-optional-provider-never-gates-core-mcp-readiness). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-CACHE-001 Canonical Project-Owned Graph Cache

Current normative owner: [CBM-CACHE-001](invariants/cbm.md#cbm-cache-001-canonical-project-owned-graph-cache). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-E-001 Explicit CBM Error Propagation

Current normative owner: [CBM-E-001](invariants/cbm.md#cbm-e-001-explicit-cbm-error-propagation). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-IDENTITY-001 Trace Source Resolution Never Guesses

Current normative owner: [CBM-IDENTITY-001](invariants/cbm.md#cbm-identity-001-trace-source-resolution-never-guesses). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-WIRE-001 Verified CBM `trace_path` Wire Contract

Current normative owner: [CBM-WIRE-001](invariants/cbm.md#cbm-wire-001-verified-cbm-trace_path-wire-contract). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-WIRE-002 Verified CBM `query_graph` Wire Contract & Column-Shape-Driven Edge Extraction

Current normative owner: [CBM-WIRE-002](invariants/cbm.md#cbm-wire-002-verified-cbm-query_graph-wire-contract--column-shape-driven-edge-extraction). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-VERIFY-001 Per-Result Caller Verification for `search_graph`

Current normative owner: [CBM-VERIFY-001](invariants/cbm.md#cbm-verify-001-per-result-caller-verification-for-search_graph). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-VERIFY-002 Verified Caller Identity Is Retained and Candidate Discovery Reuses the Project-Scoped Graph Cache

Current normative owner: [CBM-VERIFY-002](invariants/cbm.md#cbm-verify-002-verified-caller-identity-is-retained-and-candidate-discovery-reuses-the-project-scoped-graph-cache). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IDENT-001 One Physical File, One Stable Alias

Current normative owner: [IDENT-001](invariants/identity-occurrences.md#ident-001-one-physical-file-one-stable-alias). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### EDIT-001 apply_edit Unit Verification & EOL Preservation

Current normative owner: [EDIT-001](invariants/framework-and-tooling.md#edit-001-apply_edit-unit-verification--eol-preservation). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### MCP-001 MCP Result Envelope Conformance

Current normative owner: [MCP-001](invariants/framework-and-tooling.md#mcp-001-mcp-result-envelope-conformance). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### MCP-002 Focused Context Requests Have One Explicit Fidelity Meaning

Current normative owner: [MCP-002](invariants/framework-and-tooling.md#mcp-002-focused-context-requests-have-one-explicit-fidelity-meaning). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### MCP-003 Batched Context Reads Reuse One Authoritative File Evaluator

Current normative owner: [MCP-003](invariants/framework-and-tooling.md#mcp-003-batched-context-reads-reuse-one-authoritative-file-evaluator). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRWIRE-001 Semantic Binary Round Trip

Current normative owner: [IRWIRE-001](invariants/identity-occurrences.md#irwire-001-semantic-binary-round-trip). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRDELTA-001 Exact Occurrence-Preserving Delta Replay

Current normative owner: [IRDELTA-001](invariants/delta-lifecycle.md#irdelta-001-exact-occurrence-preserving-delta-replay). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRDELTA-002 Explicit Code-Side Delta Acknowledgement

Current normative owner: [IRDELTA-002](invariants/delta-lifecycle.md#irdelta-002-explicit-code-side-delta-acknowledgement). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRDELTA-003 Delta Statistics Mirror the Successful Response Lifecycle

Current normative owner: [IRDELTA-003](invariants/delta-lifecycle.md#irdelta-003-delta-statistics-mirror-the-successful-response-lifecycle). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRPAT-001 IR Identity Preservation During Consumptive Pattern Transformations

Current normative owner: [IRPAT-001](invariants/identity-occurrences.md#irpat-001-ir-identity-preservation-during-consumptive-pattern-transformations). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### PATID-001 Method Identity Survives Pattern Recognition (F2)

Current normative owner: [PATID-001](invariants/identity-occurrences.md#patid-001-method-identity-survives-pattern-recognition-f2). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRFACT-001 Derived Facts Use Declaration-Local Evidence

Current normative owner: [IRFACT-001](invariants/source-authority.md#irfact-001-derived-facts-use-declaration-local-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IRFACT-002 Observable and Promise Classifications Are Semantically Disjoint

Current normative owner: [IRFACT-002](invariants/source-authority.md#irfact-002-observable-and-promise-classifications-are-semantically-disjoint). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### SEL-001 Selector-Value Invariant

Current normative owner: [SEL-001](invariants/framework-and-tooling.md#sel-001-selector-value-invariant). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IDX-001 Index-Encoding Invariant

Current normative owner: [IDX-001](invariants/identity-occurrences.md#idx-001-index-encoding-invariant). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### IDX-002 Edge Occurrence Identity Is Source-Occurrence Aware

Current normative owner: [IDX-002](invariants/identity-occurrences.md#idx-002-edge-occurrence-identity-is-source-occurrence-aware). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-001 Authoritative Facts Do Not Imply Authoritative Coverage

Current normative owner: [WSC-001](invariants/source-authority.md#wsc-001-authoritative-facts-do-not-imply-authoritative-coverage). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-002 CBM Discovers Candidates; Clean-CTX Alone Determines WorkspaceIndex Semantics

Current normative owner: [WSC-002](invariants/workspace-query.md#wsc-002-cbm-discovers-candidates-clean-ctx-alone-determines-workspaceindex-semantics). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-003 Hydration Discovery Is Session-State Work

Current normative owner: [WSC-003](invariants/workspace-query.md#wsc-003-hydration-discovery-is-session-state-work). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-019 Provider Health and Provider Authority Are Independent

Current normative owner: [WSC-019](invariants/source-authority.md#wsc-019-provider-health-and-provider-authority-are-independent). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-004 Workspace Queries Respect Declared Workspace Scope

Current normative owner: [WSC-004](invariants/workspace-query.md#wsc-004-workspace-queries-respect-declared-workspace-scope). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-005 File-Local Call Inspection Retains Canonical Owner and Occurrence Evidence

Current normative owner: [WSC-005](invariants/workspace-query.md#wsc-005-file-local-call-inspection-retains-canonical-owner-and-occurrence-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-006 File-Local Semantic Coverage Is Fidelity- and Source-Aware

Current normative owner: [WSC-006](invariants/projection-fidelity.md#wsc-006-file-local-semantic-coverage-is-fidelity--and-source-aware). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-007 Dependency-Cycle Results Are Witnessed, Scoped Index Evidence

Current normative owner: [WSC-007](invariants/workspace-query.md#wsc-007-dependency-cycle-results-are-witnessed-scoped-index-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-008 Incomplete Query Identity Resolves Once Without Guessing

Current normative owner: [WSC-008](invariants/workspace-query.md#wsc-008-incomplete-query-identity-resolves-once-without-guessing). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-009 Workspace Query Batches Isolate Outcomes and Share Final Evidence

Current normative owner: [WSC-009](invariants/workspace-query.md#wsc-009-workspace-query-batches-isolate-outcomes-and-share-final-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ANG-DI-001 Angular Constructor Injection Is Modifier- and Formatting-Independent

Current normative owner: [ANG-DI-001](invariants/framework-and-tooling.md#ang-di-001-angular-constructor-injection-is-modifier--and-formatting-independent). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CTX-001 Canonical Semantics and Representation Authorities Stay Distinct

Current normative owner: [CTX-001](invariants/projection-fidelity.md#ctx-001-canonical-semantics-and-representation-authorities-stay-distinct). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### PERSIST-001 Automatic Checkpoints Are Policy; Accepted Mutations Are Durable Authority

Current normative owner: [PERSIST-001](invariants/persistence-publication.md#persist-001-automatic-checkpoints-are-policy-accepted-mutations-are-durable-authority). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### PERSIST-002 Durable Authority Requires Scoped Compatibility Evidence

Current normative owner: [PERSIST-002](invariants/persistence-publication.md#persist-002-durable-authority-requires-scoped-compatibility-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### AUTH-001 Refinement Joins Authority; It Never Manufactures It

Current normative owner: [AUTH-001](invariants/source-authority.md#auth-001-refinement-joins-authority-it-never-manufactures-it). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### MECHANISM-REUSE-001 Equivalent Mechanics Are Shared; Semantics Keep Their Owners

Current normative owner: [MECHANISM-REUSE-001](invariants/completion-governance.md#mechanism-reuse-001-equivalent-mechanics-are-shared-semantics-keep-their-owners). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-012 Workspace Tuple Identity Is Not Exact Declaration Identity

Current normative owner: [WSC-012](invariants/identity-occurrences.md#wsc-012-workspace-tuple-identity-is-not-exact-declaration-identity). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-013 Authoritative Query Truth Is History-Independent

Current normative owner: [WSC-013](invariants/workspace-query.md#wsc-013-authoritative-query-truth-is-history-independent). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-014 Completeness Is Earned from Evidence, Not Population

Current normative owner: [WSC-014](invariants/source-authority.md#wsc-014-completeness-is-earned-from-evidence-not-population). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-018 Coverage Detail Cannot Strengthen Completeness

Current normative owner: [WSC-018](invariants/workspace-query.md#wsc-018-coverage-detail-cannot-strengthen-completeness). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### SEMDEP-001 Semantic Validity Follows Dependencies

Current normative owner: [SEMDEP-001](invariants/source-authority.md#semdep-001-semantic-validity-follows-dependencies). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### PROJ-001 Acquisition Strength Does Not Upgrade Publication Fidelity

Current normative owner: [PROJ-001](invariants/projection-fidelity.md#proj-001-acquisition-strength-does-not-upgrade-publication-fidelity). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CBM-OPTIONAL-001 Optional Providers Change Cost, Not Core Truth

Current normative owner: [CBM-OPTIONAL-001](invariants/cbm.md#cbm-optional-001-optional-providers-change-cost-not-core-truth). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### REFINE-001 Reconciliation Ownership Is Domain-Local

Current normative owner: [REFINE-001](invariants/source-authority.md#refine-001-reconciliation-ownership-is-domain-local). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

## Concurrent Authority

### PUB-001 Same-Owner Semantic Publication Is Source-Ordered

Current normative owner: [PUB-001](invariants/persistence-publication.md#pub-001-same-owner-semantic-publication-is-source-ordered). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-010 Authoritative Refresh Retracts Absent Semantic Owners

Current normative owner: [WSC-010](invariants/workspace-query.md#wsc-010-authoritative-refresh-retracts-absent-semantic-owners). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ANG-TPL-001 Angular Template Ownership and Identity

Current normative owner: [ANG-TPL-001](invariants/framework-and-tooling.md#ang-tpl-001-angular-template-ownership-and-identity). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### ANG-TPL-002 Angular Selector Refinement Is Occurrence-Preserving and Ephemeral

Current normative owner: [ANG-TPL-002](invariants/framework-and-tooling.md#ang-tpl-002-angular-selector-refinement-is-occurrence-preserving-and-ephemeral). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### INVOCATION-001 Written Invocation Evidence Is Source-Bound and Ephemeral

Current normative owner: [INVOCATION-001](invariants/invocation.md#invocation-001-written-invocation-evidence-is-source-bound-and-ephemeral). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### INVOCATION-002 RxJS Pipe Structure Uses the Shared Occurrence Authority

Current normative owner: [INVOCATION-002](invariants/invocation.md#invocation-002-rxjs-pipe-structure-uses-the-shared-occurrence-authority). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### INVOCATION-003 Exact C# Calls Use Capability-Gated Artifact Authority

Current normative owner: [INVOCATION-003](invariants/invocation.md#invocation-003-exact-c-calls-use-capability-gated-artifact-authority). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### WSC-011 Direct Declaration Authority Is Independent of Derived Relationships

Current normative owner: [WSC-011](invariants/workspace-query.md#wsc-011-direct-declaration-authority-is-independent-of-derived-relationships). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### CSHARP-RESOLUTION-001 Structured C# Resolution Owns Framework Authority

Current normative owner: [CSHARP-RESOLUTION-001](invariants/framework-and-tooling.md#csharp-resolution-001-structured-c-resolution-owns-framework-authority). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### DOTNET-001 Framework Role Identity Is Independent of Relationship Evidence

Current normative owner: [DOTNET-001](invariants/framework-and-tooling.md#dotnet-001-framework-role-identity-is-independent-of-relationship-evidence). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

### META-LAYER-COMPLETENESS-001 Supported Framework Semantics Are Complete

Current normative owner: [META-LAYER-COMPLETENESS-001](invariants/completion-governance.md#meta-layer-completeness-001-supported-framework-semantics-are-complete). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

## Architectural Debt

### ARCH-DEBT-001 PassPipeline Migration (RESOLVED)

Current normative owner: [ARCH-DEBT-001](invariants/historical-debt.md#arch-debt-001-passpipeline-migration-resolved). Enforcement attributes, witnesses and original source provenance moved with the proposition. See [migration progress](architecture/knowledge-normalization/B0_B1_PROGRESS.md).

## How to Add a New Invariant

When a new architectural invariant is needed:

1. **First:** Can the Rust type system or compiler enforce it? If yes, do that (classify as STRUCTURAL).
2. **Second:** Does an existing test already cover it? If yes, document it here (classify as ENFORCED).
3. **Third:** Can Clippy or `cargo check` enforce it? If yes, add the appropriate lint (classify as STRUCTURAL).
4. **Only if none of the above suffice:** Add a new `#[test]` function (classify as ENFORCED).

Do not create a fitness-function framework, trait, registry, or gate abstraction. The architectural gate is `cargo test` + `cargo clippy --all-targets -- -D warnings`.

---

## Invariants That Are NOT Documented Here

The following are important architectural properties but are **not** formalized as architectural invariants:

- **Module dependency direction:** Currently enforced by Rust's module and visibility system within a single crate. The existing dependency patterns (MCP → IR, IR → compression, no reverse dependencies) are healthy but not independently tested. If a dependency becomes important enough to require hard enforcement, the appropriate mechanism is splitting into separate crates.
- **Meta-layer additivity:** Meta-layers currently append to compressed output rather than modifying it. However, the `MetaLayer::enrich()` trait signature permits modification, and "additivity" has not been established as a formal architectural contract. This is a candidate for future formalization if the contract is explicitly defined.
- **Meta-layer per-class source isolation:** Previously an uncovered concern — each meta-layer could accidentally inspect neighboring type declarations or whole-file text when trying to extract framework annotations. This is now **formally covered by C-22** (see above). The canonical capture path (`PassContext.captures` → `class_source_from_capture()` → `MetaLayer::enrich(class_captures)`) ensures that a meta-layer receives only the exact source span belonging to the type it is enriching. Multi-class cross-contamination tests (9 tests across Angular/Spring/.NET at all three fidelity levels) enforce this structurally: a class's `@Component` / `@RestController` / `[ApiController]` marker never leaks to sibling classes.
