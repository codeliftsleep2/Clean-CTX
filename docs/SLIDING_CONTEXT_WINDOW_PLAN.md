# Sliding Context Window — Concept & Design

**Status:** 📋 proposed — not yet in ROADMAP.md
**Proposed ID:** R-41 (Tier 1), R-42 (Tier 2)
**Audience:** Architects / Contributors
**Depends on:** Existing proxy (`docs/PROXY.md`), A-04 (observability for R-41), A-07 (proptest for R-42)

---

## 1. Problem this solves

Clean-CTX currently optimizes token cost at two points:

| Stage | Mechanism | What it compresses |
|---|---|---|
| Ingestion (MCP tools) | AST/IR compression, 3 fidelities | Source code, as it enters context |
| Ingestion (proxy) | 26 TOML tool-output filters | Build/test/lint/git output, as it enters context |

Neither stage touches content **after** it has already entered the conversation history. In a short session this doesn't matter. In a long-running agent session (hours, dozens of tool calls), it does:

- Every prior turn — including already-compressed source and already-filtered tool output — gets re-transmitted in full on every subsequent API call, because LLM APIs are stateless and the full message array is resent each time.
- A file read and compressed at turn 5 is still being billed at its turn-5 size at turn 80, even if nothing about it is still relevant to the current step.
- Tool-output filtering reduces the *size* of each entry once; it does not reduce the *number* of stale entries accumulating over a long session.

This is a distinct cost category from anything currently in the roadmap — R-29 (Intelligence Layer) ranks and packs what to *include* going forward; nothing currently *removes or shrinks what's already included*.

## 2. Where this fits architecturally

This is **not** an MCP-tool-layer feature. MCP tools only see what they're explicitly invoked with — they have no visibility into the full conversation array, so they structurally cannot prune history.

It **is** a proxy-layer feature, and the proxy already does the right kind of work today: it sits between client and provider, sees the full outbound request body (including full message history) on every call, and already has a transform-pipeline pattern in place (see `proxy/src/pipeline.rs`). The existing pipeline order is:

```
Client → drop_tools → strip_ansi → trim_bash_git → model_override → scrub_secrets → tool_filters → [sliding_window] → cache_control injection → Provider
```

The sliding-window transform slots in **after** tool filtering (so it ages already-compact entries, not raw ones) and **before** cache_control injection (so cache breakpoints are placed against the post-aging message array, not invalidated by it).

This maps directly to `proxy/src/server.rs:337` where `inject_breakpoints` is called after `pipeline.run()`.

## 3. Two-tier design (final, after review)

### Tier 1 (MVP) — Rule-based age truncation

Deterministic, no LLM calls, consistent with the project's "fully deterministic, rule-based" design philosophy used everywhere else in the codebase.

- Each tool-result block in the message array gets an implicit "age" = distance from the end of the array (most recent message = position 0).
- Past a configurable age threshold (`sliding_window.max_age_turns`), tool-result content is truncated to a fixed-format stub: `[aged: cargo build output, 1,847 tokens, turn 12]`.
- **Force-preserve rules** (never aged, regardless of turn count):
  1. The system prompt — always preserved
  2. The last N turns floor (configurable `force_preserve_floor`, default 15) — never aged
  3. **Path cross-reference check:** if a file path string appearing in an aged candidate also appears anywhere later in the message array (even past the floor), skip aging it. This is a **stateless textual substring match** across the message array the proxy already holds — no MCP-layer coupling, no semantic inference. This specifically handles the "circling back" pattern (file touched at turn 5, revisited at turn 40).
- **Assistant responses are NEVER aged** — tool output is mechanically reproducible (agent can re-run the command); assistant responses contain reasoning, decisions, and tradeoffs that the model relies on for consistency. Stubbing them risks the model contradicting itself, which is a worse failure mode than "model re-reads a file it already read."
- This is the safe, ship-first version. Worst case on a wrong threshold: the model loses access to something it needed and asks for it again (a visible, recoverable failure) — not a silent wrong answer.

### Tier 2 (stretch) — Score-based pruning

- Replace fixed age-threshold with a relevance score combining recency, reference count (was this file/output referenced again in later turns), and path overlap with current active work.
- Higher precision, higher risk — a low-scoring-but-actually-relevant item could be pruned with no visible failure signature, just a wrong or incomplete answer downstream. This tier should not ship without:
  - Logging of every pruning decision (what got aged, score, why) surfaced via the existing audit/observability path (A-04)
  - A `--dry-run` mode that reports what *would* be pruned without acting
  - Regression tests asserting force-preserve invariants hold under adversarial inputs (ties into A-07 proptest work)

### Explicitly out of scope (for now)

- **LLM-based summarization** (asking an auxiliary model to summarize aged-out turns). This reintroduces a network dependency clean-ctx has deliberately avoided everywhere else. If ever pursued, it should be opt-in and clearly labeled as breaking the zero-network-footprint guarantee for that specific code path.
- **Assistant message aging** (see §3 above — different risk profile requires a different mechanism, more like a running decision-log/checkpoint than blind truncation).

## 4. Failure mode comparison (why this is riskier than existing features)

This is worth stating explicitly because it changes the bar for shipping:

- **AST/IR compression failure mode:** loud. A malformed parse, an unsupported syntax construct — these throw, get caught, and degrade to an error message or raw passthrough. The user sees something is wrong immediately.
- **Tool-output filter failure mode:** loud-ish. A filter that doesn't match falls through to unfiltered output — verbose, but not wrong.
- **History pruning failure mode:** silent. If something relevant gets aged out, the model doesn't error — it just doesn't know something it needed, and may produce a plausible-looking but incorrect response. There's no exception to catch.

This asymmetry is the main argument for shipping Tier 1 only, conservatively configured (long age threshold, generous force-preserve floor) before any Tier 2 work, and for treating this as a feature requiring more scrutiny in review than typical roadmap items of similar size.

## 5. Dashboard panel

Existing per-domain savings (IR compression, tool filters) are all the same metric shape: one-shot, measured before/after, "raw → compressed, X% saved" against an actual prior state. Sliding-window savings are structurally different — they include a **counterfactual** component ("this content would have been resent N more times at full size").

**Decision:** separate panel, with exact measurements separated from estimates:

```
── Sliding Window (Session Compaction) ──
  Items aged this request: 3
  Bytes removed this request: 14,280
  Cumulative bytes removed (session): 142,800
  Cumulative resend-tokens avoided (estimated): ~356,000
```

The first three rows are **exact** measurements (the proxy measures bytes before/after on each request). The last row is the **counterfactual estimate** (bytes_removed × average expected resends), clearly labeled as such.

## 6. Suggested roadmap placement

Proposed as a **Next (v0.3.0)** item, gated behind:
- The proxy's existing transform-pipeline pattern (already shipped — no new dependency)
- A-04 (Observability) — pruning decisions should be traceable before this ships, given the silent-failure risk above. Recommend sequencing this *after* A-04 lands, not in parallel.

| ID | Title | Description | Effort | Priority | Depends on |
|----|-------|-------------|-------:|---------:|---|
| **R-41** | Sliding Context Window (Tier 1) | Age-based tool-result truncation in the proxy pipeline, with force-preserve rules (floor + path cross-reference). Opt-in via `SLIDING_WINDOW=1`. | 3-5 days | 🟡 Medium | A-04 (observability, for pruning audit trail) |
| **R-42** | Sliding Context Window (Tier 2 — scored pruning) | Relevance-scored pruning beyond simple age. Dry-run mode required before default-on. | 4-6 days | 🟢 Low | R-41, A-07 (proptest coverage for force-preserve invariants) |

## 7. Configuration

Global default with optional per-platform override, following the existing A-15 precedence chain (tool arg > env var > config file > default):

```jsonc
{
  "sliding_window": {
    "enabled": false,
    "max_age_turns": 20,
    "force_preserve_floor": 15,
    "platform_overrides": {
      "anthropic": { "max_age_turns": 30 },
      "openai": { "max_age_turns": 15 }
    }
  }
}
```

Env var gate: `SLIDING_WINDOW=1` enables it (default: disabled).

## 8. Key invariants (for testing)

1. System prompt is never aged, regardless of turn count or configuration.
2. Messages within `force_preserve_floor` turns of the end are never aged.
3. If a path string from a candidate aged item appears anywhere later in the array, the item is preserved.
4. Assistant messages (`role: "assistant"`) are never aged, even if they contain tool-like content.
5. Aged tool results are replaced with a stub, not deleted entirely (the message structure is preserved).
6. The stub always contains the original tool name, original token count, and approximate position.
7. Bytes-removed is an exact measurement (before/after diff of the serialized content array).

## 9. Resolved decisions

### 9.1 Config scope: global, with optional per-platform override — not per-combo

"Per-combo" config doesn't map onto anything that exists in clean-ctx today. What the proxy *does* legitimately vary by is **platform** — Anthropic, OpenAI, and Generic targets have different context-window sizes and pricing curves, so a different default `max_age_turns` per platform is justified.

**Decision:** global default in `.clean-ctx.json`, with optional per-platform override, following the existing precedence chain documented in A-15.

### 9.2 Reversibility: not needed — the proxy is a stateless relay, not the source of truth

The proxy never becomes the authoritative copy of the conversation: on every call, the *client* sends the full message array, the proxy thins it before forwarding to the provider for that one call, and the client's own session memory still holds the full, unaged content afterward — nothing was deleted at the source, only thinned on the wire for that request. If the model later needs the detail back, the agent loop driving the session still has it locally and either re-includes it or re-runs the tool call. The proxy doesn't need a "give it back" path because it never owned the only copy.

**Decision:** no read-path into `source_cache`/`llm_text_cache` is needed for Tier 1.

### 9.3 Corollary: Tier 1 needs no persistent state in the proxy process

Because each call already carries the full message array, "age" can be computed by counting position from the end of *that array*, fresh, on every call. No turn-counter or session state needs to persist in the proxy between calls for Tier 1. This keeps the feature consistent with the rest of the project's stateless, local-first design posture — it also means it scales cleanly across multiple proxy instances with zero shared state.

### 9.4 Force-preserve floor over active-file inference

A flat floor (last 15 turns) is a cleaner invariant than trying to infer "active files" from message text — easy to state, easy to test, no coupling to the MCP tool-call layer the proxy can't see. The path cross-reference check (§3, rule 3) patches the one gap a pure floor misses (file touched at turn 5, revisited at turn 40).

### 9.5 Dashboard: exact metrics separated from estimates

Bytes-removed-this-request and cumulative-bytes-removed are **exact** measurements (the proxy measures them directly). Only the resend-multiplier projection is counterfactual. Separating them in the same panel is the more honest design.

---

## 10. Implementation Plan

### Phase 1: Config + Pipeline hookup
1. Add `sliding_window` config fields to `proxy/src/config.rs` (`ProxyConfig`)
2. Add `sliding_window` to `PipelineConfig` in `proxy/src/pipeline.rs`
3. Add `SlidingWindowStats` to `TransformStats` in `proxy/src/transform.rs`

### Phase 2: Core transform logic
4. Implement `age_tool_results()` in `proxy/src/transform.rs`:
   - Parse the `messages[]` array
   - Identify tool result blocks (via platform adapter)
   - Apply force-preserve rules (system prompt, last N floor, path cross-reference, assistant messages)
   - Replace aged blocks with stubs
   - Track bytes-removed metrics

### Phase 3: Pipeline integration
5. Add sliding_window transform to `Pipeline::build()` in `pipeline.rs`
6. Wire into `handle_messages_request` in `server.rs`

### Phase 4: Dashboard integration
7. Add sliding window stats to proxy stats endpoint (`GET /stats`)
8. Add proxy-side capture in `src/mcp/proxy_stats.rs`
9. Add dashboard panel to `session_stats.rs` renderer