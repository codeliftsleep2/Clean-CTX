---
document_type: architecture
status: accepted
protected_sections:
  - Authority Model
---

# Document Intelligence Architecture

## Current Capability

Clean-CTX can parse Markdown structure, retrieve task-relevant sections, and
resolve explicit cross-document links. Cross-document ranking remains
experimental.

## Authority Model

Semantic classification establishes authoritative repository facts and may
override deterministic document structure when classifier confidence is high.

## Planned Capability

The next experiment will compare lexical retrieval with local embeddings.
