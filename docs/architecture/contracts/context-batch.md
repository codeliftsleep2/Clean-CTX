# Ordered authoritative context batches

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

### 5.1 Mutually exclusive forms

`provide_code_context.inputSchema` will accept exactly one of:

1. the existing single form containing `filePath`; or
2. the batch form containing `files`.

Supplying both `filePath` and `files` is invalid. A batch request contains:

- optional shared `workspaceRoot`;
- optional shared `tokenizer`;
- required `files` array.

Each item contains:

- required unique non-empty string `id`;
- required non-empty string `filePath`;
- optional `intent`;
- optional `fidelity`;
- optional `focusMethods`.

Items may not override `workspaceRoot` or `tokenizer`. This keeps authorization
and accounting policy explicit at one level and avoids two competing scopes in
one request.

### 5.2 Why items are heterogeneous

Heterogeneity here means per-item retrieval mode, not different tools. One item
may request an overview while another requests an Edit-focused body. This is
essential for the main workflow: inexpensive surrounding context plus exact
authority only where an edit is planned.


Source: [docs/architecture/PROVIDE_CODE_CONTEXT_BATCH_MIGRATION_PLAN_2026-09-29.md](../PROVIDE_CODE_CONTEXT_BATCH_MIGRATION_PLAN_2026-09-29.md); original section SHA-256 `80976deae849bbd462181c3d6a6361a7117a7ed31cef5fbfcf504dfef8a338e1`. Original wording and tense are retained; admission is limited to the implemented scope described here.


The shipped batch is one to eight items. The original plan remains a historical evaluator/migration record; its initial output envelope is the historical mirrored baseline. The current channel choices are owned by [context channels](context-channels.md). Duplicate-path and failure isolation details remain governed by MCP-003, whose original witness references are retained.

```architecture-graph
node: AUTH.CONTEXT_BATCH
kind: AUTHORITY
status: IMPLEMENTED
title: Ordered authoritative context batches
track: REPOSITORY
artifact: docs/architecture/contracts/context-batch.md
preserves:
  - MCP-003
  - MCP-002
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
