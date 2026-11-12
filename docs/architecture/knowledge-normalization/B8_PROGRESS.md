# B8 implementation and architectural investigation

Agent: **Agent MaxHeadRoom**, 2026-11-12.

## Authorization and delivery baseline

Owner consolidated B8 authorization: remove architectural document token-budget
enforcement, preserve unrelated contracts, investigate representation and retrieval
read-only, and deliver a correctly stacked PR. The later owner clarification makes
independently queryable proposition content a primary next-mission concern; it
does not authorize implementing the proposed representation change in B8.

Branch: `feat/pcg-b8-document-budget`, based on B7
`dcb60495b80335c33c8d27607ff3ba1886968a9b`. Hosted PR #186 was verified OPEN with
that exact head and base `docs/pcg-b6-history`; no upstream branch was changed.
Preserved baseline: 218 nodes, 221 typed relationships, generation
`8b91c92e82c9edf31fb4c111e5ba8b7c38ee02ca5942d3f69910f8d1d0582529`.

## Implementation

- Removed request/profile token enforcement and shared document response overflow
  refusal; no replacement token ceiling, automatic transport change or result pruning.
- Legacy request/profile budget hints are optional, accepted and explicitly retired.
  Exact existing token accounting remains descriptive; response limit is null and
  enforcement is false. The actual installed profile remains unchanged.
- Updated the controlling source `docs/architecture/PCG_DOCUMENT_AUTHORING.md`.
  Historical B2–B7 reports and the frozen B7 evaluation remain unchanged.
- Added tracked regressions for large discovery and complete contexts, retired hints,
  omitted configuration, format/reconstruction correctness and retained finite limits.
  Source/currentness, atomic reconciliation, schema, FTS, traversal, exact pooling,
  enrolled-evidence, code-context, workspace and proxy implementations are preserved.

## Verified scoped result

29 tracked Rust tests passed. Scoped all-feature Clippy passed with zero warnings;
the all-feature production build passed. All 33 actual stdio requests are ready and
current, including all ten exact B7 searches and all six mandatory complete contexts.
Four context pairs reconstruct exactly with the independent existing consumer.
No existing structural limit prevented the required retrievals.

Final graph: 218 nodes / 221 typed relationships, unchanged topology and declarations.
216 current revisions are exact; only AUTH.PCG_DOCUMENTS and AUTH.PCG_GLS advance
through the authorized normative source edit. Every original revision remains stored.
Final generation: `abb04e766d381da87b1dcd7ec18c7b22d6c98949276d83d523f209f0f6c5e0da`.
Production executable SHA-256: `d6af36341dc15b96548b9c3e404e96f7af215dea082490650bc792cafb4f2dcc`.
Actual unchanged profile SHA-256: `0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.

Changed Rust formatting passes. Full repository rustfmt reports pre-existing
differences in untouched `src/pcg/mod.rs`, `src/pcg/security.rs`, and
`src/tests/pcg/persistence.rs`. The full repository gate is UNRUN; it is not GREEN.
Actual commands, scope and remaining obligations are in `B8_VERIFICATION.csv` and
`B8_VERIFICATION.md`. Delivery remains a stacked draft PR pending repository gates.

## Architectural result and proposal

The active interface is a document graph with structured metadata/relationships
and complete exact source history, not independently bounded proposition/rationale/
obligation storage. SQLite stores the actual words as whole-source BLOBs; it does
not store just external pointers. Rust cannot select an exact proposition because
no admitted source-owned selector exists. Whole-source FTS can select a node because
another proposition in its shared source contains the query term.

`B8_REPRESENTATION_AND_RETRIEVAL_PROPOSAL.md` answers the owner's seven questions,
assesses all eight PassPipeline results and recommends explicit source-owned content
boundaries and deterministic Rust/SQLite selection. GLS shares the pointer/state
representation limitation; its implementation does not demonstrate this SQLite
FTS/materialization mechanism. No corrective representation change is implemented.

## Evidence index

- Implementation: `B8_PREFLIGHT.csv`, `B8_BUDGET_ENFORCEMENT_INVENTORY.csv`,
  `B8_POLICY_DISPOSITION.md`, `B8_UNRELATED_OPERATION_ISOLATION.csv`.
- Production acceptance: `B8_DISCOVERY_VERIFICATION.csv`, `B8_CONTEXT_VERIFICATION.csv`,
  `B8_TRANSPORT_VERIFICATION.csv`, `B8_GRAPH_PRESERVATION.csv`, `B8_VERIFICATION.csv`.
- Investigation: `B8_KNOWLEDGE_ARCHITECTURE_INVESTIGATION.md`,
  `B8_SQLITE_STORAGE_INVENTORY.csv`, `B8_KNOWLEDGE_UNIT_ASSESSMENT.csv`,
  `B8_QUERY_EXECUTION_TRACES.md`, `B8_OPERATION_CONTRACTS.csv`,
  `B8_FTS_SEMANTICS_ASSESSMENT.md`, `B8_EXECUTION_RESPONSIBILITY_MATRIX.csv`,
  `B8_DECISION_REPRESENTATION_ASSESSMENT.csv`,
  `B8_DETERMINISTIC_QUERY_CAPABILITIES.csv`, `B8_PAYLOAD_COMPOSITION.csv`,
  `B8_ARCHITECTURAL_FINDINGS.md`, and the next-mission proposal.
- Raw runtime captures and database snapshots remain under `target/pcg-b8/**`;
  reusable request sets, production driver and capture analysis are tracked under
  `verification/pcg-175/`. They are operator verification, not Rust regression tests.

## Outstanding integration and certification

All 16 independent A/B evaluations remain UNRUN due to repository-specific memory
contamination. Frozen scenarios and answer keys are unchanged. Host source-reference
support, uncontaminated evaluation infrastructure, upstream integration and any
representation/schema/API implementation are separate obligations. No independent
certification, global closure, reclosure or banking is claimed.
