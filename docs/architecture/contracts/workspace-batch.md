# Workspace query batch scope and identity

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

### 5.1 Two mutually exclusive request forms

`workspace_query.inputSchema` will accept exactly one of:

1. the existing single-query form, unchanged; or
2. the new batch form with shared scope plus `queries`.

The request must not contain both top-level `type` and `queries`. The batch
form is:

```json
{
  "workspaceRoot": "C:/repo",
  "withinPath": "src/orders",
  "queries": [
    { "id": "q1", "type": "find_entities", "name": "OrderService" },
    { "id": "q2", "type": "reverse_edges", "name": "OrderService" }
  ]
}
```

Rules:

- `queries` must be a non-empty array;
- every item must be an object;
- every item must carry a non-empty string `id`;
- IDs must be unique within the request;
- every item carries one existing query specification;
- items may select different query types within the same batch;
- item fields use the same validation and meaning as the corresponding legacy
  query;
- `workspaceRoot` and `withinPath` exist only at the batch top level;
- item-level scope fields are rejected rather than ignored;
- input order is semantically significant and preserved in output order.


Source: [docs/architecture/WORKSPACE_QUERY_BATCH_MIGRATION_PLAN_2026-09-28.md](../WORKSPACE_QUERY_BATCH_MIGRATION_PLAN_2026-09-28.md); original section SHA-256 `85ee6024fff7c2d31e41168300c0b6b468165390e96ec5cfc15cb53b9b4839b1`. Original wording and tense are retained; admission is limited to the implemented scope described here.


The original batch-size discussion remains a historical approval record; this extraction does not establish a new limit. Final acquired evidence and isolated outcomes are governed by WSC-009 and WSC-013.

```architecture-graph
node: AUTH.WORKSPACE_BATCH
kind: AUTHORITY
status: IMPLEMENTED
title: Workspace query batch scope and identity
track: REPOSITORY
artifact: docs/architecture/contracts/workspace-batch.md
preserves:
  - WSC-009
  - WSC-013
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
