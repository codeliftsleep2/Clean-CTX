# Canonical IR authority and contract composition

**Standing:** Current production architecture. Agent: Agent MaxHeadRoom.

Canonical source structure belongs to `CompiledIR` and the typed `CoreOp`
instruction stream. The [CoreOp contract](CORE_OP_CONTRACT_MATRIX.md) owns
operation identity, ownership, cardinality, ordering, validation and transformation
requirements. This document owns their composition as the canonical IR boundary;
it does not create a second operation matrix or replace language/domain authority.

The existing production pipeline lowers supported source languages to that
common instruction stream. [ARCH-002](../invariants/canonical-ir.md#arch-002-language-agnostic-canonical-ir-boundary)
governs the common boundary; [PIPELINE-001](../invariants/canonical-ir.md#pipeline-001-compilation-pipeline-ordering)
retains the production pass order and its tracked witness.

[ARCH-001](../invariants/canonical-ir.md#arch-001-inference-state-is-ephemeral)
keeps inference state outside canonical serialized IR. Neutral written references
and exact resolved declarations remain different evidence. Domain refinement may
join proven authorities but must not manufacture them; that rule remains owned by
[AUTH-001](../invariants/source-authority.md#auth-001-refinement-joins-authority-it-never-manufactures-it).

The [Binary 0x04 contract](BINARY_V04_CONTRACT.md) owns exact physical
serialization. Canonical equality includes ordered repeated occurrences and
complete operands. Model-facing projection is a distinct contract under
[ARCH-003](../invariants/projection-fidelity.md#arch-003-canonical-ir-and-llm-projection-are-separate-contracts);
compact display or fidelity cannot weaken canonical authority.

The [compiler guide](../COMPILER_IR.md) explains the existing pipeline, formats,
explicit deltas and replay. Its walkthrough is subordinate to the current owners
above. No runtime type, pipeline, wire format or semantic interpretation changes
in this documentation normalization.

## Source provenance and disposition

This composition is normalized from `docs/COMPILER_IR.md` §§1–3 and the
CoreOp contract at commit `655f0a75`; exact original source hashes remain in
`docs/architecture/knowledge-normalization/inventory.csv`. Invariant sections
retain their own original section/source hashes and current owner mapping in
`B0_B1_PROVENANCE.csv`. Those historical sources explain the derivation; they are
not attached as broad graph evidence payloads.

GLS IMPLEMENTED records the source-described current production boundary, not
new execution, certification, authenticated approval, reclosure or banking.

```architecture-graph
node: AUTH.CANONICAL_IR
kind: AUTHORITY
status: IMPLEMENTED
title: Canonical IR authority and operation contract composition
track: FOUNDATION
artifact: docs/architecture/CANONICAL_IR_AUTHORITY.md
commit: 655f0a75
depends_on:
  - AUTH.CORE_OPS
preserves:
  - ARCH-001
  - ARCH-002
  - PIPELINE-001
  - AUTH-001
  - ARCH-003
```
