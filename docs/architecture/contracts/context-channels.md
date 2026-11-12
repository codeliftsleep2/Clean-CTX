# Explicit context response channels

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

## 5. Proposed public contract

### 5.1 Request field

Batch requests gain one optional top-level field:

```json
"responseMode": "mirrored" | "structured" | "indexed"
```

Rules:

- the field is valid only with top-level `files`;
- omission means `mirrored`;
- it is shared batch policy and may not appear inside an item;
- unknown values return top-level `-32602` before item evaluation;
- singular `filePath` requests reject `responseMode` rather than silently
  ignoring it.

### 5.2 `mirrored` mode — safe default

Every success contains exact text in both channels:

```text
result.content[content_index]
result.structuredContent.results[n].content[0]
```

The blocks are byte-identical. Successful items carry `content_index`,
item-level `content`, and `meta`. This is the compatibility-first default and
the only mode callers should use when host visibility is unknown.

### 5.3 `structured` mode — structured-only hosts

Every success carries exact item-level `content` and `meta`.

Top-level `result.content` contains one compact text notice:

```text
Batch context is available in structuredContent.results; inspect every item status and content.
```

Successful items omit `content_index` because no top-level code block exists to
index. Failures carry only their item-local error.

This is the recommended mode for Claude integrations whose live projection has
been verified to expose structured results and hide separate indexed blocks.

### 5.4 `indexed` mode — content-channel hosts

Every successful exact text block appears once in top-level `result.content`.
Successful structured items carry `content_index` and `meta` but omit
item-level `content`.

This is valid only when the caller knows its host preserves the canonical MCP
content channel. It reproduces the original compact response shape deliberately,
not as the default.

### 5.5 Structured response discriminator

Every accepted batch includes the resolved mode:

```json
{
  "batch": true,
  "response_mode": "mirrored",
  "results": []
}
```

The discriminator lets clients validate the selected projection and prevents
absence of item content from being mistaken for accidental data loss.

### 5.6 All-failure behavior

All modes retain one non-authoritative top-level fallback block:

```text
No context items succeeded; inspect structuredContent.results for item errors.
```

No failed item has `content` or `content_index`. Ordered structured errors remain
authoritative.

---

## 6. Cache identity

Per-file source/IR/token caches are unchanged because response mode is a
projection concern after evaluation.

The outer baseline cache breaker must include:

1. resolved response mode; and
2. the existing length-delimited ordered successful texts.

This prevents two different public response projections from sharing one outer
cache identity while keeping repeated calls in the same mode reusable.

---

## 7. Output-schema strategy

The declared batch output schema must expose:

- `batch: true`;
- required `response_mode` enum;
- ordered results;
- error items with neither content representation;
- mirrored success: `content` + `content_index` + `meta`;
- structured success: `content` + `meta`, no `content_index`;
- indexed success: `content_index` + `meta`, no item `content`.

Because output shape depends on request policy, the schema documents all three
valid success shapes. Runtime tests pin the relationship between
`response_mode` and the permitted item shape.

---


Source: [docs/architecture/PROVIDE_CODE_CONTEXT_BATCH_RESPONSE_MODE_OPTIMIZATION_PLAN_2026-09-29.md](../PROVIDE_CODE_CONTEXT_BATCH_RESPONSE_MODE_OPTIMIZATION_PLAN_2026-09-29.md); original section SHA-256 `d125b2ab7bca27cb60a7ea1422fcd310d404eddeb4330b0e56d3afd10af10fec`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.CONTEXT_CHANNELS
kind: AUTHORITY
status: IMPLEMENTED
title: Explicit context response channels
track: REPOSITORY
artifact: docs/architecture/contracts/context-channels.md
depends_on:
  - AUTH.CONTEXT_BATCH
preserves:
  - MCP-001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
