# `provide_code_context` Batch Response-Mode Optimization Plan

**Date:** 2026-09-29
**Status:** Phases 0–5 and Checkpoints A–F complete; Phase 6 has not started
**Scope:** Reduce duplicate serialized code in batched `provide_code_context`
responses when the caller knows which MCP visibility channel its host exposes,
without reintroducing silent content loss or changing per-file evaluation.

---

## 1. Problem statement

Batched `provide_code_context` now mirrors every successful exact text block in
both MCP visibility channels:

1. canonical top-level `result.content`; and
2. item-level `result.structuredContent.results[n].content`.

The mirror is required for safe default interoperability. Live testing proved
that at least one structured-output client exposes the structured item while
hiding the separately indexed top-level block. The original indexed-only
contract therefore produced `status="ok"` and truthful fidelity metadata but no
client-visible code.

Mirroring fixes correctness but duplicates serialized code. Current measured
response sizes are:

| Items | Mirrored batch bytes | Equivalent single-response bytes | Batch overhead |
|---:|---:|---:|---:|
| 2 | 2,557 | 1,732 | 825 bytes (47.63%) |
| 4 | 4,865 | 3,174 | 1,691 bytes (53.28%) |
| 8 | 9,326 | 6,348 | 2,978 bytes (46.91%) |

The primary batch economy still exists: one tool invocation, shared setup,
ordered failure isolation, and one caller planning/reasoning step. This plan
adds an opt-in transport projection so informed callers can also avoid the
dual-channel byte mirror.

---

## 2. Goals

1. Keep the safe dual-channel response as the default.
2. Let a caller explicitly select the one channel its host consumes.
3. Preserve exact successful content and per-item fidelity semantics.
4. Preserve ordered failure isolation and all existing state effects.
5. Avoid a second evaluator, compiler path, renderer, or cache.
6. Make the selected response shape machine-readable and schema-declared.
7. Measure serialization separately from tool-call/model-orchestration economy.

## 3. Non-goals

- automatic host capability detection;
- transport compression for stdio JSON-RPC;
- changing singular `filePath` responses;
- changing fidelity, focus, compilation, persistence, or publication behavior;
- aggregate token budgets, truncation, concurrency, retries, or rollback;
- changing the eight-item batch limit;
- making batch reads a cross-file edit transaction.

---

## 4. Why explicit selection is required

MCP initialization does not declare whether a host exposes `content`,
`structuredContent`, or both to its model. The server cannot safely infer the
correct projection from ordinary client capabilities.

JSON Schema and MCP provide no standard sibling-reference mechanism by which an
item can point at a top-level text block and require the host to dereference it.
An integer `content_index` is useful to callers that retain both channels, but
it cannot make hidden content visible.

Therefore:

- automatic projection is unsafe;
- default deduplication would repeat the field failure;
- explicit caller selection is the smallest reliable contract.

---

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

## 8. Phased implementation

### Phase 0 — Contract and baseline

- Record the field-discovered structured-only client behavior.
- Record current mirrored 2/4/8 response sizes.
- Approve names, defaults, invalid combinations, and exact notices.
- Confirm this plan introduces projection policy only.

**Checkpoint A:** explicit approval of Sections 5–7 before production changes.

### Phase 1 — Tracked RED contract

Add regressions under `src/tests/**` for:

- schema enum and batch-only placement;
- omitted mode resolving to `mirrored`;
- explicit mirrored dual-channel byte equality;
- structured item content with compact top-level notice;
- indexed top-level content with item indexes;
- per-mode failure isolation;
- all-failure fallback;
- invalid/unknown/singular mode rejection;
- unchanged single-file response;
- unchanged fidelity/focus metadata and exact bodies.

Run focused RED against the current mirrored-only implementation, stash only
the regression, implement, restore it unchanged, and run identical GREEN.

**Checkpoint B:** RED must fail because mode selection is absent, not because
the test or fixture is invalid.

### Phase 2 — Typed request policy

Introduce a small request-local enum owned by the batch boundary. Parse and
validate `responseMode` before item evaluation. Do not add configuration or
session state.

**Checkpoint C:** parsing/validation is complete and the evaluator remains
unchanged.

### Phase 3 — Response projection

Refactor the existing response assembly around the resolved mode:

- one evaluated `ProvidedContext` per item;
- one mode-specific placement decision;
- no content rewriting;
- no recursive handler calls;
- one final MCP response.

**Checkpoint D:** unchanged Phase 1 regressions are GREEN and every exact block
matches its equivalent singular response.

### Phase 4 — Cache and economics verification

- Include mode in the outer breaker material.
- Verify repeat reuse independently for all three modes.
- Measure 2/4/8 serialized sizes for every mode.
- Report tool-call/model orchestration economy separately from wire bytes.
- Introduce no automatic mode selection based on measurement.

**Checkpoint E:** cache identity and measurements are truthful for each mode.

### Phase 5 — Guidance

Update:

- `AGENTS.md` and the authoritative local engineering projection;
- `docs/agent/tooling.md`;
- `docs/CLAUDE_INTEGRATION_RULES.md`;
- runtime initialization guidance;
- README examples;
- durable architectural invariants.

Guidance must say:

- unknown host: omit mode and receive safe `mirrored`;
- verified structured-only Claude host: request `structured`;
- verified content-channel host: request `indexed`;
- never select a compact mode merely by guessing;
- every item status remains mandatory to inspect.

**Checkpoint F:** prompt tests and documentation terminology are GREEN.

### Phase 6 — Live MCP acceptance

Extend `verification/context-batch/` to verify all three modes through a freshly
built production binary. The structured scenario must model the field client:
discard top-level content and prove code remains in each structured success.
The indexed scenario must discard item content and prove indexed top-level code
remains. Mirrored must prove byte equality across channels.

Generated captures remain operator evidence, never tracked test evidence.

**Checkpoint G:** all live mode projections pass.

### Phase 7 — Final audit and repository gate

Audit the complete lifecycle, remove obsolete unconditional-mirror assumptions,
update the migration record, and run the authoritative final gate from
`docs/agent/verification.md`.

**Checkpoint H:** the optimization is complete only after the architectural
audit, tracked tests, live acceptance, and final gate are GREEN.

---

## 9. Architectural boundaries

```text
tools/call
  -> provide_code_context request-form dispatch
  -> batch structural validation + ResponseMode parsing
  -> shared per-file evaluator (unchanged)
  -> ordered ProvideResult values
  -> mode-specific response projection
  -> mode-scoped outer cache hint
  -> one MCP response
```

Ownership:

| Concern | Owner |
|---|---|
| File validation, fidelity, focus, compilation, persistence, publication | shared single-file evaluator |
| Batch IDs, cap, duplicate canonical files, shared fields | batch coordinator |
| Response-mode parsing and placement | batch response boundary |
| Host-specific mode recommendation | integration guidance, never server inference |

---

## 10. Approval record

Checkpoint A must explicitly approve:

1. field name `responseMode`;
2. values `mirrored`, `structured`, and `indexed`;
3. omitted default `mirrored`;
4. batch-only validity;
5. exact structured-mode top-level notice;
6. required `response_mode` response discriminator;
7. mode-scoped outer cache identity;
8. no automatic host detection or projection fallback.

**Decision (2026-09-29):** Checkpoint A was explicitly approved as written.
Phase 1 uses the contract above as its RED regression authority.

**Phases 1–3 implementation checkpoint (2026-09-29):** The tracked
response-mode contract was observed RED against the mirrored-only production
path, then restored unchanged after implementation and reported GREEN by the
maintainer. The complete `provide_code_context_batch_tests` module was also
reported GREEN. Batch requests now parse a request-local typed mode, reject
unknown values and singular-form mode use before evaluation, default safely to
`mirrored`, declare the resolved `response_mode`, and project exact successful
text as mirrored, structured-only, or indexed-only without changing the shared
per-file evaluator. Phases 1–3 and Checkpoints B–D are complete.

**Phase 4 cache and economics checkpoint (2026-09-29):** A dedicated tracked
test now verifies that all three response modes establish distinct outer cache
identities and that an identical request reuses each mode-specific identity.
The same test reports serialized batch and equivalent repeated-single response
sizes at 2, 4, and 8 items for `mirrored`, `structured`, and `indexed`, without
using those measurements to select a mode automatically. The maintainer
reported the focused all-features test GREEN. Phase 4 and Checkpoint E are
complete.

**Phase 5 guidance checkpoint (2026-09-29):** Portable and local agent policy,
tooling guidance, Claude integration rules, runtime initialization guidance,
README examples, and the durable MCP invariant now distinguish singular and
batch requests and explain the three explicit batch projections. Unknown hosts
omit the mode for safe `mirrored`; verified structured-result consumers request
`structured`; verified top-level content consumers may request `indexed`.
Every path requires per-item status inspection and forbids guessing a compact
mode. The tracked initialization-prompt contract was reported GREEN by the
maintainer, with the runtime instructions still below their size ceiling.
Phase 5 and Checkpoint F are complete.
