# File/workspace presentation boundary — re-partition plan

**Status:** In execution (2026-09-24). §1-§3 done (invariants reconciled, reasoning suite re-partitioned, DeepSeek backend added); §6 records the content-boundary decision; the content fix (Option A) is in progress.

**Scope:** Determine what a normal `provide_code_context` / `compress_code_context`
call must put in front of the LLM (versus what belongs in `workspace_query` or in
the code-side reversible contract), re-partition the reasoning evaluation into a
file-local lane and a `workspace_query` lane, add a DeepSeek-v4 Pro model backend,
and correct the architectural assumptions that caused the file context to over-ship.

---

## 1. Findings (hard evidence)

### 1.1 Two repository invariants contradict each other

- **ARCH-003** (`docs/ARCHITECTURAL_INVARIANTS.md:101-110`) — "Canonical IR and
  LLM Projection Are Separate Contracts":
  > "LLM text is a separate compact projection that may abbreviate presentation
  > only while preserving complete meaning; it is not the canonical storage or
  > validation model."
  Enforcement is pinned to `render_hierarchical_for_llm`, which is no longer the
  production content path.

- **CTX-001** (`docs/ARCHITECTURAL_INVARIANTS.md:425-433`) — "Model-Visible Context
  Is a Correctness-Complete Projection":
  > "MCP content is a versioned CONTROL-FULL projection of checked hierarchical IR
  > plus the complete semantic-edge snapshot. It preserves explicit canonical IDs,
  > typed ownership, occurrence order/grouping/duplicates, unresolved written call
  > names and written-arity/spread evidence, injection facts, edge relation/layer/file
  > provenance, and exact bodies/spans when compiled."

These cannot both hold for the same `content`. The current production content follows
CTX-001 (`src/mcp/tool_handlers/core/content.rs:17-30` -> `compact_a::render_file_context`),
which is the source of the bloat.

### 1.2 The A3 grammar already declares the correct boundary

- `A3_GRAMMAR.md:20-21`:
  > "Workspace graph edges, file-local calls, and core injection occurrences are
  > outside this grammar; they are served on demand by workspace_query."
- `A3_GRAMMAR.md:170` (fidelity matrix): "Workspace edges, calls, injections |
  query | query | query | query".

### 1.3 The current content is a half-measure hybrid

`src/ir/compact_a.rs:246-262` (`render_file_context`) removes `E` (semantic edges)
from `g` but keeps `K` (calls), `n.D` (injections), `n.V` (behavior navigation),
the 13-column method rows with occurrence groups, canonical IDs, and the full legend.

### 1.4 Calls and DI are already duplicated in `workspace_query`

`A3_BOUNDARY_MAP.md` documents the double-write directly:

- Lines 34-38:
  > "The same semantic facts ship twice. The file wire carries calls (`K`), DI (`D`),
  > and derived/inferred facts (`cs`/`fc`/`fd`/`se`/`ec`) even though `workspace_query`
  > already serves calls and DI from the workspace index."
- Lines 156-159:
  > "`workspace_query` (on demand): calls, DI/framework relations, cross-file edges,
  > graph traversal."
- Lines 169-171:
  > "Calls exist twice: `compact_a.rs:138-152` serializes `K` rows into the file
  > payload, while `semantic_projection.rs:99-117` serializes the identical facts
  > (`caller`, `callee`, `explicit_arg_count`, `has_spread`) as `Calls` edges into
  > [the workspace index]."

`workspace_query` already serves those edges via `forward_edges` / `reverse_edges`
(`domain="builtin"`, `entity_type="Method"` for calls; the framework layer for
`Injects`) plus `entities_in_file` (`src/mcp/tool_handlers/query.rs:82-88`). No new
operation is required for the normal "who calls X / what injects Y" case.

### 1.5 The reasoning harness feeds the oracle as the payload

- `Prepare-ReasoningWorksheet.ps1:28`: `control_full_capture = "...\control-full.txt"`
  (the CONTROL-FULL oracle).
- `Run-CodexReasoning.ps1:118-133`: the whole file is the "MODEL-VISIBLE PAYLOAD".
- `oracles.json` scores the LLM on workspace-served facts from that file payload:
  `provenance` (#10, cross-file asserting-file), `framework-edge-provenance` (#17,
  framework Injects edge), `call-order` (#3) / `dense-graph` (#4, file-local calls),
  and the "Injects edges" half of `dependency-injection` (#7).
- `Verify-Captures.ps1:179-185` requires cross-file `semantic_edges` in the single-file
  context and flags their absence as "unsupported", not "correctly deferred".

---

## 2. The plan

### 2.1 Re-partition the evaluation

**(a) Single-file correctness lane.** Only file-local, reasoning-necessary oracles,
fed the file-context payload (not `control-full.txt`). Keep: `ownership`, `overloads`,
`inheritance`, `focused-edit`, `source-escalation`, `implement-source-escalation`,
`body-byte-fidelity`, `focused-overload-family`; `behavior-facts` re-scoped to collapsed
flags only (drop the "nested occurrence groups and order" requirement).

**(b) `workspace_query` correctness lane.** Only genuine **cross-file** facts, driven
through real `workspace_query` calls: `provenance` -> `find_entities` (asserting-file
provenance); `framework-edge-provenance` -> `forward_edges` (framework layer).

File-local calls (`call-order`, `dense-graph`, `unresolved-honesty`) stay in the **file**
lane: `workspace_query` keys on `(domain, entity_type, name)` — **name only, no owner** —
so `forward_edges` on `"run"` cannot isolate `Alpha.run` from `Beta.run`. See 2.3.

**(c) DeepSeek-v4 Pro model backend.** Abstract `Run-CodexReasoning.ps1`'s hard-coded
`codex exec` into `-ModelRunner codex|deepseek`. The DeepSeek lane routes through
**Cline's OpenAI-compatible API** (`https://api.cline.bot/api/v1/chat/completions`, key
`$env:CLINE02_LOCAL_ENV_API_KEY`) so it bills the Cline account (ClinePass); the model id
must carry the `cline-pass/` prefix (e.g. `cline-pass/deepseek-v4-pro`). No CLI install.

### 2.2 SHOULD / SHOULDN'T in the normal-call presentation

**SHOULD be in `content` (file-local, reasoning-necessary):** file identity
(source path, version); class/interface identity (name + kind); `extends`/`implements`;
fields (name + type + owner); method identity (name, params, return); overload
disambiguation (arity); collapsed modifiers/flags (`async`, `static`, `export`);
imports + type aliases; exact bodies (Edit + focused only).

**SHOULDN'T be in `content`:** semantic edges (framework/cross-file) -> `workspace_query`;
canonical IDs (`C1/M1/F1/P1`) -> regenerate positionally (code-side); occurrence groups +
duplicates -> code-side (round-trip); navigation index (`n.D`/`n.V`) -> code-side (decoder
index); grammar legend -> code-side (decoder contract); body byte-length framing ->
code-side.

**Stays until the codec changes:** file-local calls (`K`) and injection occurrences
(`n.D`) — see 2.3: `workspace_query` cannot serve them by owner, so removing them from the
file wire would lose owner-qualified facts.

This is structurally main's SCHEMA v2 (names-only) plus explicit ownership clarity.

### 2.3 File-local calls: corrected finding

**Finding (measured).** `workspace_query`'s `forward_edges` / `reverse_edges` select by
`(domain, entity_type, name)` — **name only, no owner** (`src/mcp/tool_handlers/query/edges.rs`).
A call's caller identity is `(builtin, Method, "run")`, not `(..., "Alpha.run")`, so
`forward_edges` on `"run"` returns the calls of **every** method named `run` mixed
together, with no owner attribution and no recoverable per-owner order.

**Consequence.** The earlier "workspace_query already serves calls" conclusion was only
half right — it serves calls *coarsely by name*, but **cannot** serve the owner-qualified
order/duplicates/spread that `call-order` / `dense-graph` / `unresolved-honesty` require.
Those facts therefore remain **file-local**: they stay in the **file** lane and test the
file context (which still carries `K`). Removing `K` from the file wire is **not** safe
while this gap exists.

**Decision.** Do **not** extend `workspace_query` for file-local calls now (scope creep —
it is cross-file by design). Either keep file-local calls in the file context, or, if a
future decision wants them out, add a first-class owner-aware file-local surface — not a
widened cross-file query.

### 2.4 Baked-in wrong assumptions to correct

1. **CTX-001 itself** declares `content` must be "correctness-complete CONTROL-FULL +
   complete semantic-edge snapshot", conflating the reversible oracle with the
   presentation, in direct contradiction to ARCH-003.
2. "Occurrence order/grouping/duplicates are LLM-relevant" — they are a code-side
   round-trip property; the LLM reasons over the semantic set.
3. "Canonical IDs are LLM-relevant" — the LLM reasons over names (main's SCHEMA v2
   shipped names-only and worked); IDs are for `apply_edit`/delta.
4. "File-local calls + injections belong in the file context" — `A3_GRAMMAR.md:20` says
   they are outside the grammar, and they are already in `WorkspaceIndex`.
5. "The correctness oracle is the payload" — `Prepare-ReasoningWorksheet.ps1` ->
   `control-full.txt` -> "MODEL-VISIBLE PAYLOAD" makes the evaluator unable to detect
   over-provisioning.
6. "A single file payload should answer cross-file questions" — the `provenance` /
   `framework-edge-provenance` oracles + `Verify-Captures.ps1:179-185` contradict the
   file/workspace authority split.
7. "Structured modes never fall back to a cheaper payload" — CTX-001's clause conflicts
   with the live economics gate (`content.rs:45-51`) that already returns raw when the
   candidate is not cheaper.

---

## 3. Proposed invariant reconciliation (exact wording)

**CTX-001 — reframe as the code-side reversible contract (not the presentation).**
Retitle "Model-Visible Context Is a Correctness-Complete Projection" to
"Reversible Codec Is Correctness-Complete", and change the invariant body to state that
`CONTROL-FULL` (the reversible oracle) preserves canonical IDs, ownership, occurrence
order/grouping/duplicates, call facts, injection facts, edge provenance, and exact
bodies/spans as the round-trip/persistence authority — not as the model-visible
presentation.

**ARCH-003 — restore as the presentation contract.** State that the model-visible
`content` is a compact projection over the checked IR that preserves complete meaning
(names, typed ownership, signatures, collapsed modifiers, extends/implements, imports,
aliases, exact bodies when requested) while omitting code-side machinery (canonical IDs,
occurrence groups, navigation, legend, body framing) and workspace-served facts (edges,
calls, injections), which remain available via the reversible codec and `workspace_query`
respectively.

---

## 4. Execution order

1. **Reconcile the invariants** (CTX-001 -> reversible contract; ARCH-003 -> presentation)
   — define the target before touching code.
2. **Re-partition** `oracles.json` + `Verify-Captures.ps1` + `Run-CodexReasoning.ps1`
   into the two lanes (2.1a/2.1b) and add the DeepSeek runner (2.1c). Test/harness edits.
3. **Measure** the bulk-pattern frequency to decide whether a whole-file facts operation
   is ever warranted (2.3).
4. **Cut the presentation payload** to the 2.2 SHOULD set, regenerate IDs positionally,
   and re-run the re-partitioned gate.

---

## 5. Open decisions

- ~~DeepSeek exact API model id + where the `DEEPSEEK_API_KEY` is provided.~~
  Resolved: Cline's OpenAI-compatible API (`api.cline.bot`, `CLINE02_LOCAL_ENV_API_KEY`,
  model `cline-pass/deepseek-v4-pro`).

---

## 6. Decision — model-visible content is the presentation, not the codec

**Decided 2026-09-24. Approved: implement Option A now, then Option C.**

**Decisive evidence (not inference):** the codec's decode side has no production
caller. `compact_a3::decode_cold` / `decode_declarations` / `decode_facts` are
referenced only by one another (`document.rs:270-271`); no MCP handler decodes a
caller-supplied content document — `delta`, `apply_edit`, `restore_context`,
`replay_history` operate from the server's persisted canonical IR and exchange
`from_version`/`to_version`, never documents. The only caller-text parse in
`src/mcp` is `buffered_store.rs:164`, unrelated. The codec is therefore
encoder-only in production: its preamble, grammar legend, envelope schema id and
`§BODIES` framing are paid in every prompt while required by nothing in the
protocol.

**Decision.** `content` is the model-facing presentation
(`render_hierarchical_for_llm`, SCHEMA v5) on every path; the reversible codec
stays code-side (`result.ir` + persistence). Option A is the minimal correct
enforcement of ARCH-003 vs CTX-001; Option C (a purpose-built presentation
authored against §2.2's SHOULD/SHOULDN'T table) follows.

**Rejected — Option B** (keep the dense codec as content, drop the decoder
contract): a positional grammar without its interpretive key is a codec the
reader cannot decode, not a presentation. It optimizes tokens by removing the
information the reader needs — the exact failure mode this plan corrects — and
the reasoning runs already showed the dense form struggling.

**Consequence owned.** The A2/A3 token-savings gate (42–43%) stops describing
the model-visible wire; production savings become the presentation's density
(v5's historical 56–65% on large files). A3's correct home is the code-side
reversible wire, whose decoder has never been load-bearing in production.

**Enforcement.** `src/tests/mcp/presentation_boundary.rs` — five RED tests
pinning that content is not the codec document and does not carry its decoder
legend, envelope schema id, or body framing, plus one guard that the typed
owner and method identity survive. Written RED, frozen (stash), then the code
is fixed and the tests restored byte-identically.

- Whether the "whole-file local facts" `workspace_query` operation is worth building
  (deferred behind the step-3 measurement).
