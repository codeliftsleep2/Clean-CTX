---
document_type: decision
status: accepted
decision_id: SDI-001
---

# Semantic Authority Decision

## Authority Boundary

Deterministic Markdown structure and explicitly approved repository metadata
are authoritative. Embeddings, classifier output, inferred roles, and semantic
drift scores are advisory evidence only.

## Rationale

Statistical similarity can improve retrieval but cannot establish that prose
is current, binding, or correct. Repository authority must come from explicit
and reviewable mechanisms.

## Rejected Alternative

The project rejects treating a high-confidence `architectural_invariant`
classification as sufficient authority to block or approve an edit.

## Status

This decision is accepted for the benchmark fixture. It does not establish a
production metadata syntax for Clean-CTX.
