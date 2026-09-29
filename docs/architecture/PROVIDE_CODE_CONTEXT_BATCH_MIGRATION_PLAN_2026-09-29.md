# `provide_code_context` Batch Migration Plan

**Date:** 2026-09-29  
**Status:** Complete — Phases 0–8, Checkpoints A–H, tracked regressions, live
acceptance, architectural audit, and final repository gate are GREEN
**Scope:** Add an ordered, failure-isolated multi-file request form to the
existing `provide_code_context` MCP tool while preserving the complete
single-file contract, byte-exact Edit/Verbatim authority, session lifecycle,
and token-accounting behavior.

---

## 1. Executive summary

`provide_code_context` currently accepts one source file per MCP call. A caller
that needs a service, its model, and a related repository must issue several
tool calls even when every file belongs to the same authorized workspace.
Those calls repeat tool invocation, caller reasoning/orchestration, workspace
validation, and tokenizer selection.

This plan proposes a second, mutually exclusive request form:

```json
{
  "workspaceRoot": "C:/work/repo",
  "tokenizer": "claude",
  "files": [
    {
      "id": "service",
      "filePath": "src/orders/order-service.ts",
      "intent": "debug"
    },
    {
      "id": "model",
      "filePath": "src/orders/order.ts",
      "fidelity": "edit",
      "focusMethods": ["Order.validate"]
    }
  ]
}
```

The existing single-file form remains supported without response-shape or
behavior changes. A structurally valid batch is best-effort and
non-transactional: each file produces an ordered `ok` or `error` outcome, and
one file failure does not suppress successful siblings.

The implementation must reuse the existing single-file evaluator. It must not
create a second compilation, heuristic, rendering, persistence, or baseline
publication path.

Successful code text has a stricter boundary than a workspace-query result:
Edit and Verbatim content may be byte-exact edit authority. The batch envelope
therefore must not prepend an item label to a successful content block. Ordered
structured outcomes correlate each item to its unchanged content block through
`content_index`.

This is an additive public-contract migration. It does not authorize changes
to fidelity selection, heuristic classification, focus selector resolution,
token economics, source/cache ownership, persistence policy, or edit authority.

---

## 2. Motivation

### 2.1 Current caller cost

Multi-file investigation is a normal code-understanding workflow:

- inspect a service and its interface;
- inspect a caller and callee together;
- retrieve several focused methods before a coordinated edit;
- compare a framework component with its configuration and model;
- read files discovered by one graph or workspace query.

Today every file requires a separate `tools/call`. Even when source and IR
caches avoid recompilation, the caller still pays repeated protocol envelopes,
tool selection, scope parsing, response parsing, and round-trip latency.

### 2.2 Desired outcome

One request can retrieve several independent code contexts while retaining the
correctness properties of separate calls:

- each item has an explicit caller-owned ID;
- input and output order are stable;
- each item independently selects intent, fidelity, and focus methods;
- one invalid or unsupported file does not erase sibling successes;
- every successful item is semantically equivalent to the same legacy call;
- byte-exact content remains byte-exact;
- cache and session side effects match ordered legacy execution;
- no content is silently omitted, merged, or truncated.

### 2.3 Why this is not a generic batching framework

`workspace_query` batching provides a useful contract pattern, but code-context
retrieval has different authority and payload characteristics. A generic batch
framework would hide rather than simplify these differences:

- code text can be byte-exact edit authority;
- individual items can publish session baselines and semantic projections;
- fidelity and heuristic selection are file-specific;
- token counts and raw-passthrough decisions are file-specific;
- aggregate response size is materially larger than graph data;
- source compilation and persistence have lifecycle effects.

The batch coordinator should therefore be purpose-built and reuse only small,
already-valid conventions such as unique IDs and ordered item outcomes.

---

## 3. Current production lifecycle to preserve

Before implementation, Phase 0 must trace and record the exact live path from
the MCP entry point through response delivery:

```text
tools/call: provide_code_context
  -> registered dispatcher
  -> handle_provide_code_context
  -> argument/focus/fidelity validation
  -> trusted path resolution and source read
  -> heuristic or explicit fidelity selection
  -> focused IR compilation / source-cache reuse
  -> semantic publication and session state
  -> persistence/baseline lifecycle where applicable
  -> rendering and raw-passthrough economics
  -> token counting/cache hints
  -> MCP content + _meta delivery
```

The implementation must identify precisely which steps can fail after state is
published. A batch does not introduce rollback semantics; its item must behave
exactly like the corresponding legacy call at every failure point.

### 3.1 Existing public arguments

The single form currently owns:

- `filePath`;
- optional `workspaceRoot`;
- optional `intent`;
- optional `fidelity`;
- optional `focusMethods`;
- optional `tokenizer`.

The current rules remain authoritative, including:

- non-empty focus implies Edit only when intent and fidelity are both omitted;
- explicit non-Edit modes conflict with focus;
- an empty focus list is meaningful only under the existing Edit rule;
- ambiguous focus selectors fail rather than guessing;
- same-owner overload selection keeps its existing behavior;
- unsupported extensions and unsafe paths retain their existing errors;
- raw passthrough remains an economics decision, not a batch policy.

### 3.2 State and ownership concerns

The existing handler is not a pure reader in the narrow in-memory sense. It may
populate caches, publish semantic/session context, establish an edit baseline,
and update usage statistics. The migration must preserve those owners and may
not replace them with batch-owned shadow state.

Sequential execution is the v1 recommendation. Parallel file evaluation is
not approved because it could change publication order, cache races, baseline
ownership, persistence ordering, response determinism, and test isolation.

---

## 4. Architectural invariants

The following are non-negotiable unless separately approved:

1. **Single-call compatibility:** the legacy request and response remain
   byte-for-byte/field-for-field compatible for equivalent state.
2. **One evaluator:** singular and batch paths call the same internal
   single-file evaluation function.
3. **Content completeness:** no successful item is silently truncated or
   replaced by a summary.
4. **Byte authority:** Edit/Verbatim output is not prefixed, suffixed, escaped,
   or otherwise changed by the batch envelope.
5. **Ordered equivalence:** executing a batch is behaviorally equivalent to
   executing its valid items as legacy calls in input order within one session.
6. **Failure isolation:** after structural acceptance, an item-local failure
   does not suppress unrelated siblings.
7. **No rollback claim:** successful cache/session/persistence work remains
   valid when another item fails.
8. **Trusted paths:** the existing path resolver remains the only filesystem
   authorization boundary.
9. **Per-file semantics:** intent, fidelity, focus resolution, raw-passthrough,
   content kind, and token accounting remain item-local.
10. **Shared tokenizer truth:** a top-level tokenizer, when present, is applied
    consistently to every item and reported per item using existing metadata.
11. **No hidden concurrency:** v1 evaluation is deterministic and sequential.
12. **No generic batch framework:** extraction stops at the narrow evaluator
    and response boundaries needed by this tool.

---

## 5. Proposed public contract

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

### 5.3 Duplicate paths

Phase 0 found that existing same-file Edit reuse is keyed by fidelity/source
but not focus coverage. V1 therefore recommends one canonical file occurrence
per batch. A later duplicate becomes an item-local error directing callers to
combine selectors in one `focusMethods` array. Section 14.4 records the evidence
and the deferred condition for supporting duplicates safely.

### 5.4 Structural versus item-local errors

Reject the whole request before file work when:

- `files` is missing, not an array, or empty;
- the batch exceeds the approved item limit;
- an item is not an object;
- an item lacks a unique non-empty string `id`;
- an item contains shared-only fields;
- both request forms are present;
- shared scope/tokenizer input is structurally invalid.

After structural acceptance, isolate these failures to the item:

- missing or invalid `filePath`;
- path authorization failure;
- unsupported language/extension;
- invalid fidelity or intent;
- focus/fidelity conflict or unresolved selector;
- source read failure;
- compilation/projection failure;
- persistence or lifecycle failure already classified as item-local by the
  legacy evaluator.

Phase 0 must confirm the last category. The batch layer may not reclassify an
existing server-fatal invariant failure merely to claim isolation.

### 5.5 Proposed structured result

The accepted batch returns one ordered outcome per item:

```json
{
  "batch": true,
  "results": [
    {
      "id": "service",
      "status": "ok",
      "content_index": 0,
      "meta": {
        "content_kind": "skeleton_with_verbatim_bodies"
      }
    },
    {
      "id": "missing",
      "status": "error",
      "error": {
        "code": -32602,
        "message": "..."
      }
    }
  ]
}
```

Rules:

- `results` order exactly matches `files` order;
- every accepted item produces exactly one result;
- `content_index` is present only for success;
- `content_index` points into the outer MCP `content` array;
- only successful items create content blocks;
- when every item fails, one non-authoritative fallback text block keeps the
  MCP success envelope valid and directs the caller to structured item errors;
- every successful block contains the exact text the legacy call would return;
- errors retain the legacy code/message/data shape;
- overall `isError` remains false for an accepted batch with item failures;
- a structurally rejected batch returns the normal top-level JSON-RPC error.

The exact `meta` projection is a Phase 0/2 decision. It should reuse the
single-item metadata type rather than copy fields by hand. Full code text must
not also be duplicated inside `structuredContent`.

### 5.6 Model-readable presentation

The `content` array is the model-readable authority for successful code. It
must not contain batch headers inside successful source text.

If correlation cannot be represented safely without altering code blocks, the
batch may add one small manifest text block before the success blocks and make
all `content_index` values account for it. That option requires explicit tests
proving Edit/Verbatim block content itself remains exact. The recommended first
choice is structured correlation without a manifest block.

---

## 6. Capacity and response-size decision

Code-context payloads are substantially larger than workspace-query payloads.
This plan does not approve silent aggregate truncation, partial code blocks, or
an implicit token budget.

### 6.1 Options

**Option A — item cap only (recommended for v1)**

- enforce a small fixed item count;
- retain complete normal output for every successful item;
- rely on callers to choose appropriate fidelity/focus;
- observe real payloads before designing aggregate budgeting.

Tradeoff: several large raw-passthrough items can still create a large response.

**Option B — caller-declared aggregate token budget**

- add a shared `maxOutputTokens`;
- define deterministic admission/order behavior;
- return explicit budget errors for items that do not fit.

Tradeoff: this creates new partial-completeness and token-estimation semantics
and can perform stateful work for content that is later withheld.

**Option C — server-global aggregate limit**

- reject or stop when a configured global threshold is reached.

Tradeoff: externally observable global policy, configuration complexity, and
potentially surprising behavior. Not recommended.

### 6.2 Recommended v1 limit

Recommend **8 items maximum**, sequential execution, and no aggregate
truncation. Eight covers common caller/callee/service/model investigations
without importing the workspace-query limit of 32 into a much heavier tool.

The number is an externally observable limit. **Checkpoint A must explicitly
approve or replace it before schema or production implementation.**

---

## 7. Intended internal architecture

### 7.1 Extract a typed single-file evaluator

The existing handler currently combines evaluation and response transmission.
Introduce a narrow internal boundary conceptually equivalent to:

```rust
fn evaluate_provide_code_context(
    args: &Value,
    state: &McpState,
) -> Result<ProvidedContext, ProvideFailure>
```

`ProvidedContext` must own the exact content text plus the existing metadata
needed to construct the legacy response. `ProvideFailure` must preserve the
existing error code, message, and optional structured data.

The legacy handler becomes thin orchestration:

```text
evaluate one item -> serialize the unchanged legacy response -> send once
```

No behavior change is allowed during this extraction.

### 7.2 Batch coordinator

The coordinator owns only:

- structural batch validation;
- shared argument projection;
- deterministic sequential item iteration;
- ordered outcome collection;
- success-block indexing;
- one final MCP response.

It does not own compilation, path authorization, heuristics, rendering,
publication, persistence, focus resolution, or token counting.

### 7.3 Suggested module boundaries

The current `core/provide.rs` should not become a monolith. If materially
editing it approaches the active-file target, decompose along real ownership:

```text
core/provide.rs            legacy/batch entry-point orchestration
core/provide/evaluate.rs   single-file evaluator and typed outcome
core/provide/batch.rs      batch validation and ordered coordination
core/provide/response.rs   legacy and batch response projection
```

The exact split follows existing code discovered in Phase 0. File-size pressure
is not a reason to weaken the architecture; decomposition is required when the
boundary warrants it.

### 7.4 Cache behavior

V1 should first rely on existing caches:

- source cache;
- compiled IR/session state;
- raw token-count cache;
- persistence baseline reuse;
- semantic publication reuse.

Do not add a batch cache. Do not memoize complete MCP responses. Do not dedupe
items solely by path because fidelity, intent, focus, tokenizer, source
freshness, and session state all affect behavior.

---

## 8. Phased implementation plan

### Phase 0 — Production-path audit and decision closure

**Goal:** establish the exact current behavior and approve the v1 contract.

Work:

1. Trace creation, compilation, rendering, publication, persistence, cache,
   and response paths for successful and failed legacy calls.
2. Inventory every early return and classify it as item-local or server-fatal.
3. Capture representative single-call responses for Low/Medium/High/Edit,
   focused Edit, Verbatim, raw passthrough, unsupported file, and invalid path.
4. Confirm whether `_meta` can be represented by one reusable internal type.
5. Confirm whether duplicate same-file calls are order-sensitive.
6. Measure representative aggregate sizes for 2, 4, and 8 items.
7. Resolve Checkpoint A decisions below.

Deliverable:

- a short audit section appended to this document;
- approved contract decisions;
- no production behavior change.

**Checkpoint A — explicit approval required**

- public field name: `files`;
- maximum item count: recommended 8;
- shared fields: `workspaceRoot` and `tokenizer` only;
- later duplicate canonical paths rejected item-locally;
- sequential best-effort execution;
- `content_index` correlation design;
- no aggregate truncation/budget in v1.

### Phase 1 — RED contract regressions

**Goal:** prove the legacy implementation lacks the approved behavior.

Add tracked tests under `src/tests/mcp/**` for:

- tools/list exposes mutually exclusive single and batch forms;
- ordered heterogeneous fidelity/intent items;
- one failed item between two successes;
- exact parity with equivalent legacy calls;
- byte-exact Edit and Verbatim content blocks;
- stable `content_index` correlation when an earlier item fails;
- duplicate IDs, empty files, oversized batch, and item-level shared fields;
- focus/fidelity conflicts remain item-local;
- duplicate path behavior;
- source/token cache reuse without semantic drift;
- no response-content duplication in `structuredContent`.

Follow the repository RED/GREEN procedure exactly: run focused RED, stash the
test and registration only, implement, restore the unchanged regression, and
run the identical command GREEN.

**Checkpoint B:** RED must fail behaviorally because batching is absent, not
because the test or schema is malformed.

### Phase 2 — Extract the single-file evaluation boundary

**Goal:** separate evaluation from JSON-RPC transmission without changing the
legacy contract.

Work:

1. Introduce typed success and failure outcomes.
2. Move existing handler logic mechanically behind the evaluator.
3. Project the legacy response from that outcome.
4. Preserve logging, cache hints, metadata, and error classification.
5. Decompose `provide.rs` along semantic boundaries if activated size requires
   it.

Verification:

- all existing provide-context tests remain unchanged and green;
- captured single responses before/after are equivalent;
- no batch schema or production entry point exists yet.

**Checkpoint C:** legacy behavior equivalence must be established before batch
orchestration begins.

### Phase 3 — Batch schema and structural validation

**Goal:** expose the approved request union without executing items yet.

Work:

1. Extract reusable schema fragments for one file item.
2. Add the mutually exclusive `filePath`/`files` forms.
3. Declare ordered result, error, metadata, and `content_index` output shapes.
4. Implement request-level structural validation.
5. Keep item semantic validation delegated to the evaluator.

**Checkpoint D:** tools/list must describe the full input/output contract and
reject ambiguous scope ownership.

### Phase 4 — Sequential batch orchestration

**Goal:** evaluate accepted items through the single authoritative evaluator.

Work:

1. Project shared workspace/tokenizer fields into each item.
2. Iterate in input order.
3. Preserve each evaluator success or failure unchanged.
4. Append exact success text blocks and record their indexes.
5. Return one canonical MCP result envelope.
6. Retain successful state changes when siblings fail.

No concurrency, cancellation policy, retry policy, or new cache is introduced.

**Checkpoint E:** the unchanged Phase 1 regression must be GREEN, including
legacy parity and byte-exact content.

### Phase 5 — Economics and cache verification

**Goal:** confirm batching reduces call overhead without corrupting accounting.

Work:

1. Compare repeated calls with equivalent batches for protocol-envelope size.
2. Verify raw and compressed token counts remain per-file truthful.
3. Verify tokenizer selection is shared and consistently reported.
4. Confirm existing source/IR/token caches are reused where expected.
5. Record aggregate response sizes for representative 2/4/8-item scenarios.
6. Do not introduce a new limit or truncation behavior based on measurements
   without a new architectural approval.

**Checkpoint F:** measurements must show no semantic or accounting drift. A
performance gain is desirable but does not override correctness.

### Phase 6 — Model guidance and documentation

**Goal:** make the capability discoverable and hard to misuse.

Update:

- `docs/agent/tooling.md`;
- `docs/CLAUDE_INTEGRATION_RULES.md`;
- runtime initialization guidance in `src/mcp/prompts.rs`;
- tool schema descriptions;
- architecture overview/invariants only where a durable fact is established;
- discovery registry if implementation or live testing reveals a field issue.

Guidance must explain:

- use singular form for one file;
- use `files` for two or more independently useful contexts;
- use per-item fidelity so only edit targets request exact bodies;
- inspect every item status;
- do not treat sibling success as proof a failed file was read;
- batching does not create cross-file transactional edit authority.

### Phase 7 — Live MCP acceptance

**Goal:** verify the freshly built production binary through stdio.

Add a tracked operator harness under:

```text
verification/context-batch/
  README.md
  fixtures/
  scripts/Verify-ProvideContextBatchLive.ps1
```

The live harness must verify:

- tools/list publishes the approved union;
- mixed overview and focused Edit items succeed;
- one missing/invalid file is isolated;
- content indexes correlate correctly across failures;
- successful Edit text equals the legacy single response exactly;
- a repeated run retains semantics and demonstrates expected cache reuse;
- generated captures are written beneath `target/` and clearly labeled as
  operator evidence, not tests.

**Checkpoint G:** live acceptance is required because this changes an
externally visible MCP contract.

### Phase 8 — Final architectural audit and verification

**Goal:** prove the migration is complete and leave one production path.

Audit:

- singular and batch paths share one evaluator;
- no duplicated compiler/rendering/persistence logic remains;
- no hidden content mutation exists in the batch envelope;
- no obsolete handler path remains;
- all active files remain within the repository ceiling;
- docs and runtime prompts describe actual behavior;
- the production lifecycle is traced end to end.

Then run the single authoritative final verification gate from
`docs/agent/verification.md`. The agent hands long-running commands to the
operator unless explicitly authorized for that run.

**Checkpoint H:** the plan may be marked Complete only after the architectural
audit, tracked gate, and live acceptance are green.

---

## 9. Test matrix

| Boundary | Required coverage |
|---|---|
| Schema | Single/batch exclusivity, shared fields, unique IDs, item cap |
| Legacy parity | Every supported fidelity/intent path matches a single call |
| Focus | Bare/qualified selectors, overload family, ambiguity, conflict |
| Content | Exact text, raw passthrough, content kind, byte authority |
| Ordering | Input order, success-block indexes, failures between successes |
| Isolation | Missing file, unsupported extension, invalid path, compile failure |
| State | Session publication, baseline ownership, persistence, repeated calls |
| Cache | Source, IR, raw-token count, source freshness |
| Tokenizer | Shared override and per-item truthful counts |
| Features | Default and all-feature compilation with zero warnings |
| Live | tools/list plus real stdio execution against a fresh binary |

---

## 10. Explicit non-goals

This migration does not include:

- cross-file `apply_edit`;
- transactional rollback across files;
- parallel compilation;
- generic MCP batching for unrelated tools;
- a batch response cache;
- automatic graph discovery of which files to include;
- aggregate summarization or truncation;
- a global token budget or response-size configuration;
- changing heuristics, fidelity defaults, or focus semantics;
- merging multiple files into one synthetic context document;
- changing persistence, semantic identity, or edit-authority boundaries.

---

## 11. Risks and mitigations

### Oversized responses

Mitigation: small approved item cap, per-item fidelity guidance, no silent
truncation, live size measurement, and deferred budget policy.

### Byte-exact content corruption

Mitigation: never inject labels into success blocks; correlate through
structured `content_index`; regression-test exact equality.

### Duplicate production logic

Mitigation: extract one typed evaluator before adding orchestration; prohibit
calling the response-sending handler recursively.

### Partial side effects after failure

Mitigation: document best-effort ordered semantics and preserve legacy
per-item lifecycle. Do not claim transactionality.

### Cache/state order drift

Mitigation: sequential execution and parity tests against the same ordered
legacy calls.

### File-size pressure

Mitigation: decompose the activated handler along evaluation, batch, and
response boundaries. Do not route around the repository ceiling.

### Client confusion between `focusMethods` and `files`

Mitigation: schema and Claude guidance must state that `files` batches source
documents, while `focusMethods` selects method bodies within one item.

---

## 12. Rollback strategy

Each phase should remain independently reversible:

1. evaluator extraction can remain as internal cleanup if batch work stops;
2. schema must not be published before an executable path exists in the same
   completed phase;
3. batch orchestration can be removed without changing the evaluator or legacy
   path;
4. no persistence migration or durable-format change is required;
5. no stored batch artifact becomes a new source of truth.

Rollback must never delete valid contexts, baselines, or persistence state
created by successful legacy-equivalent item evaluation.

---

## 13. Definition of done

The migration is complete only when:

- Checkpoint A decisions are explicitly approved;
- the unchanged tracked RED regression is GREEN;
- singular responses remain compatible;
- batch results are ordered and failure-isolated;
- Edit/Verbatim success blocks remain exact;
- one evaluator owns both request forms;
- state, cache, persistence, and token behavior match ordered legacy calls;
- Claude/runtime/schema guidance is synchronized;
- live MCP acceptance passes on a freshly built binary;
- the final architectural audit finds no duplicate or obsolete path;
- the authoritative repository verification gate is green;
- this document is updated from Proposed to Complete with evidence.

---

## 14. Phase 0 production-path audit (2026-09-29)

### 14.1 Production integration trace

The registered path is live and direct:

```text
tools/call
  -> registry entry for provide_code_context
  -> core::handle_provide_code_context
  -> focus/mode validation
  -> trusted path resolution + exclusion/resource checks
  -> pending-edit recovery preflight
  -> source cache read + session alias allocation
  -> Angular-template specialization OR normal source path
  -> heuristic decision + tokenizer/raw-token cache
  -> Verbatim short circuit OR cached/new IR compilation
  -> checked hierarchy + focus resolution
  -> presentation/economics selection
  -> optional atomic durable baseline persistence
  -> session IR/fidelity/semantic/index/text publication
  -> compression statistics + response metadata/cache hint
  -> send_response
```

There is no intermediate typed outcome. The main handler, Angular template
specialization, and focus/projection helpers can each transmit a response.
Phase 2 must convert all three sending boundaries before batch orchestration;
calling the current handler repeatedly would emit multiple JSON-RPC responses.

### 14.2 Observable success families

| Family | Content/state behavior |
|---|---|
| Verbatim source | Raw source; records compression statistics; no IR compile/publication; non-template path remembers alias-to-path ownership |
| Angular template | Separate template compressor/economics path; no canonical IR or durable row; records statistics; may raw-pass through |
| Structural source | Cached or newly compiled IR; hierarchy/focus projection; economics selection; optional durable checkpoint; then session/index/text publication |
| Raw passthrough | Selected after structural compilation when raw is not more expensive; still persists and publishes canonical IR/semantics while returning exact source |

Every success currently returns one text content block and result-level `_meta`.
Normal metadata includes version, strategy, fidelity, Angular classification,
decision summary, content kind, byte-exact regions, degradation, and an optional
baseline cache hint. Angular templates have a related but distinct metadata
shape; Verbatim omits version and several structural fields.

### 14.3 Failure and mutation ordering

| Boundary | Code | State may already have changed? |
|---|---:|---|
| Focus/mode request conflict | `-32602` | No |
| Missing file path / path authorization | `-32602` | No |
| Excluded, oversized, unreadable file | `-32603` | Pending-edit recovery may already have restored durable state for the resolved path |
| Heuristic/fidelity decision | `-32602` | Source cache, alias, and persisted-path map may be populated |
| Projection/focus resolution | `-32602` or mapped `-32603` | Source/raw-token caches and alias/path ownership may be populated; no new canonical baseline published |
| IR compilation unavailable | `-32603` with `ir_unavailable` data | Same preparatory caches/ownership; no new baseline publication |
| Durable baseline persistence | `-32603` | Compilation and token caches exist; new session IR/index publication has not occurred |

After durable persistence succeeds, remaining publication operations are
in-memory and currently non-fallible. Therefore accepted batch execution can
isolate all existing request/file failures, but it must explicitly remain
best-effort: even an error item may have performed recovery or preparatory cache
work. Rollback would contradict existing lifecycle behavior.

### 14.4 Cache and duplicate-file finding

Existing reuse keys prove these safe facts:

- source cache validates modification time and size;
- raw token counts key by source hash plus tokenizer;
- canonical IR reuse requires matching fidelity and source hash;
- semantic edges must exist for an IR cache hit;
- compilation version derives from current session file version.

Focus coverage is not part of the canonical IR cache key. A focused Edit read
publishes the body-filtered IR; a later same-file Edit request with another
focus set can reuse that fidelity/source snapshot. Separate legacy calls are
therefore order-sensitive for this duplicate-path/different-focus case.

Phase 0 recommends that v1 reject every later occurrence of the same canonical
file as an **item-local** `-32602` error, directing the caller to combine method
selectors in one item. This prevents the new API from advertising unsafe
same-file heterogeneity without changing legacy behavior. Supporting duplicate
paths should be deferred until focus coverage becomes an explicit cache
dimension or the evaluator can safely recompile for broader coverage.

### 14.5 Cache-hint finding

The single response owns one result-level baseline cache hint derived from its
visible text. A batch cannot preserve several independently functional
result-level hints by nesting the old `_meta` objects in structured outcomes;
clients consume hints at the outer result boundary.

Recommendation: successful items retain their semantic metadata under their
ordered result entries, excluding nested cache hints. The outer batch response
receives one baseline hint derived from an unambiguous length-prefixed sequence
of successful content blocks in response order. This represents the actual
cacheable batch response and remains stable for identical ordered outcomes.
The requested tokenizer continues to govern per-item economics; baseline-hint
token estimation preserves the existing config-tokenizer behavior.

### 14.6 Payload evidence

No new server run was required. Existing tracked verification captures provide
48 paired tokenizer measurements:

- largest representative compressed item: 1,995 tokens (focused C# Edit);
- representative Angular focused Edit: 1,472–1,536 tokens;
- representative C# High: 1,728–1,820 tokens;
- representative Angular Low: 882–925 tokens;
- raw counterparts in the largest fixtures: approximately 3,590–4,091 tokens.

At the observed maxima, eight compressed items are approximately 16,000 tokens
and eight raw items approximately 33,000 tokens before batch-envelope overhead.
The configured per-file safety limit is 10 MiB, so an item cap cannot guarantee
a small aggregate response. Nevertheless, silent truncation or post-publication
budget rejection would create worse correctness semantics.

Recommendation remains eight items, sequential, complete, and without an
aggregate budget in v1. Guidance must favor mixed low/medium context plus
focused Edit only for actual edit targets. Phase 5 must measure live batch
payloads and may only propose a later budget through a new approval decision.

### 14.7 Metadata and correlation decision

The proposed `content_index` design is viable:

- successful content blocks remain unchanged;
- failed items create no content block;
- ordered structured outcomes correlate IDs to success blocks;
- per-item metadata is projected from the same typed result used by the legacy
  response;
- full code text is not duplicated in `structuredContent`.

An extra manifest content block is not recommended. It would consume model
tokens and complicate exact-content indexing without adding machine authority.

### 14.8 Revised Checkpoint A recommendation

Approval is requested for this complete v1 policy:

1. Add mutually exclusive top-level `filePath` and `files` forms.
2. Limit a batch to eight items; reject larger requests structurally.
3. Keep `workspaceRoot` and `tokenizer` shared at the top level.
4. Allow per-item `intent`, `fidelity`, and `focusMethods`.
5. Execute sequentially in input order with best-effort item isolation.
6. Reject later duplicate canonical files item-locally in v1.
7. Return unchanged success text blocks plus ordered `content_index` outcomes.
8. Store per-item semantic metadata without nested cache hints.
9. Emit one outer batch baseline cache hint over the ordered successful blocks.
10. Introduce no aggregate truncation, token budget, concurrency, retry, or
    rollback policy in v1.

Phase 1 must not start until these externally observable decisions are
explicitly approved.

**Decision (2026-09-29):** Checkpoint A was explicitly approved as written.
Phase 1 uses this policy as its regression authority.

**Phase 3 response decision (2026-09-29):** An accepted batch whose items all
fail returns one text block stating `No context items succeeded; inspect
structuredContent.results for item errors.` No item points to that block with a
`content_index`; ordered structured errors remain authoritative. This is a
response-contract rule, not part of the Phase 2 evaluator extraction.

**Phases 3–4 implementation checkpoint (2026-09-29):** The public schema now
declares the mutually exclusive single/batch request forms and ordered batch
outcomes. The production coordinator evaluates at most eight items
sequentially, isolates item failures, rejects later canonical-file duplicates,
correlates exact successful content blocks by `content_index`, and emits one
outer cache hint. The complete tracked `provide_code_context_batch_tests`
regression target was reported GREEN by the maintainer. Legacy single-call
equivalence was subsequently reported GREEN; later verification phases remain.

**Phase 5 verification checkpoint (2026-09-29):** The tracked
`batch_preserves_token_accounting_cache_reuse_and_records_envelope_sizes`
regression was reported GREEN by the maintainer. It verified truthful per-file
raw/compressed token accounting with the shared `o200k` tokenizer, stable
semantics on repeated execution, and reuse of the outer batch cache identity.
The original pre-mirror response-envelope measurements were superseded by the
post-field-correction measurements below. Exact content is now deliberately
present in both MCP visibility channels, so serialized byte size increases:

| Items | Batch bytes | Equivalent single-response bytes | Batch overhead |
|---:|---:|---:|---:|
| 2 | 2,557 | 1,732 | 825 bytes (47.63%) |
| 4 | 4,865 | 3,174 | 1,691 bytes (53.28%) |
| 8 | 9,326 | 6,348 | 2,978 bytes (46.91%) |

The batch economy is therefore not raw response serialization. It is one model
tool call, one planning/reasoning step, shared setup, ordered failure isolation,
and no repeated caller round trip per file. Correct dual-channel content
reachability takes priority over byte deduplication. No new budget, truncation
rule, or performance-dependent behavior was introduced. Phase 5 and Checkpoint
F remain complete with the corrected interpretation.

**Later response-projection note (2026-09-29):** These figures and the
dual-channel statement describe the backward-compatible default `mirrored`
projection. The subsequent response-mode optimization added explicit
`structured` and `indexed` batch projections for verified hosts while retaining
`mirrored` when `responseMode` is omitted. See
`PROVIDE_CODE_CONTEXT_BATCH_RESPONSE_MODE_OPTIMIZATION_PLAN_2026-09-29.md`.

**Phase 6 guidance checkpoint (2026-09-29):** The authoritative tooling guide,
Claude integration rules, and runtime initialization instructions now explain
when to use singular versus batched `provide_code_context`, the eight-item cap,
shared versus per-item fields, item-level exact `content`, ordered
`status`/`content_index` correlation, failure isolation, duplicate
canonical-file handling, selective exact fidelity, retained successful side
effects, and the absence of cross-file transactional edit authority. Runtime
guidance remains inside its enforced
2,000-byte compactness budget at 1,978 bytes. The tracked
`mcp::prompts::tests` target was reported GREEN by the maintainer. Phase 6 is
complete; live MCP acceptance has not started.

**Phase 7 live-acceptance checkpoint (2026-09-29):** The tracked operator
package under `verification/context-batch/` drove a freshly built production
binary over MCP stdio and was reported fully passing by the maintainer. It
confirmed the published schema, ordered mixed-mode outcomes, item-local failure
isolation, exact `content_index` correlation, byte-identical focused Edit
parity with the legacy single form, and stable repeated semantics with reuse of
the batch cache identity. Generated captures are stored beneath
`target/provide-context-batch-verification/captures/`. Those captures are
operator evidence, not tests; the authoritative regression contract remains
the GREEN tracked tests under `src/tests/**`. Phase 7 and Checkpoint G are
complete. Phase 8 has not started.

**Phase 8 architectural-audit checkpoint (2026-09-29):** The production
lifecycle was traced from registered `tools/call` dispatch through the thin
single/batch entry point, structural batch validation, shared typed per-file
evaluation, persistence/publication/statistics, exact content projection, and
the one outer response/cache envelope. `provide::evaluate::evaluate` is the
sole owner of source reading, heuristics, Angular specialization, compilation,
focus resolution, persistence-before-publication, workspace/session
publication, rendering, and compression statistics. `provide::batch` owns only
batch validation, shared-field projection, canonical duplicate prevention,
ordered coordination, and response correlation. No nested response sender,
duplicated evaluator, obsolete execution path, hidden successful-content
mutation, rollback claim, or cross-file edit authority remains.

The audit tightened the declared output schema so successful and failed item
shapes are mutually exclusive and required, and updated the durable invariant
catalog to name the extracted evaluator authority and the batch contract.
Fast guards reported valid UTF-8/no BOM, clean diff hygiene, and all active
files below the 615-line ceiling. No critical or high-severity architectural
issue remains. At audit time, Checkpoint H remained pending the complete
user-run repository verification gate recorded below.

**Final verification checkpoint (2026-09-29):** After restoring the established
`workspaceRoot` schema guidance exposed by the first full-gate run, the focused
`schema_guidance_uses_current_model_workflow_terms` regression was reported
GREEN. The maintainer then reran the complete authoritative gate from
`docs/agent/verification.md` against the corrected final state and reported all
seven commands GREEN: formatting, all-target/all-feature Clippy with warnings
denied, the complete workspace/all-target/all-feature test suite, file-size
validator tests, active-file validation, UTF-8 validation, and the Rust encoding
test. Phase 8 and Checkpoint H are complete; this migration is complete.

**Post-completion field correction (2026-09-29):** Live use through a
structured-only client exposed that successful code text existed in top-level
MCP `content` but not in each `structuredContent.results` item. Fidelity and
rendering were correct, but the client-visible projection showed plausible
`status="ok"` metadata without code. Successful items now mirror their exact
content block under item-level `content` while retaining `content_index` and
the byte-identical canonical top-level block. This is intentional dual-channel
reachability, not repeated evaluation. The primary batch economy remains one
tool invocation, shared setup, and one caller reasoning step rather than
separate calls per file.
