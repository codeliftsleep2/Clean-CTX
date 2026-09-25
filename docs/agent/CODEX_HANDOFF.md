# Clean-CTX hand-off: SCHEMA-v5 presentation boundary + annotation-redundancy A/B

You are resuming work on Clean-CTX (a token-waste reducer / context compiler for
LLM tooling). This note corrects an earlier architectural mistake and brings you
current.

## The architectural fact (do not re-litigate — ARCH-003)

Clean-CTX has two **separate** contracts:

1. **Canonical IR / codec** — `CoreOp[]`, CONTROL-FULL, COMPACT-A2/A3. This is
   the **reversible wire**: its only job is lossless round-trip (encode IR ↔
   decode IR) for deltas, persistence, and replay. It is a *reversibility*
   mechanism, **not a compressor**.
2. **LLM projection / presentation** — SCHEMA-v5, produced by
   `render_hierarchical_for_llm`. This is the **model-visible `content`**; its
   job is maximum information per token for the model.

**Invariant:** `content` is always the SCHEMA-v5 presentation. The codec stays
code-side (`result.ir` + persistence). Workspace facts (semantic
edges/calls/injections) are served separately via `workspace_query`. This is
mechanically enforced (ARCH-003) and tested.

## Why the earlier "compact-full duplication on the LLM side" was 100% wrong

The branch wired the CONTROL-FULL codec as the model-visible `content` across
~17 handler sites. That decision was incorrect for three measured reasons:

1. **The codec expands — it does not compress.** On the large fixtures, A2/A3
   emit *more* tokens than raw source (preamble, grammar legend, envelope schema
   id, canonical IDs, occurrence groups, `§BODIES` framing). SCHEMA-v5 is the
   only thing that actually compresses: **56–76% vs raw**.
2. **It shipped machinery nothing consumes.** The codec's decode side
   (`decode_cold`/`decode_declarations`/`decode_facts`) has **no production
   caller** — so the legend/schema id forced into every prompt was required by
   *nothing* in the protocol. The model paid for a reversibility legend it
   doesn't read and a decode path that doesn't use it.
3. **It conflated two contracts and dropped the real one.** Using the codec as
   `content` simultaneously (a) violated the ARCH-003 boundary and (b) left
   `render_hierarchical_for_llm` (the actual presentation renderer) with **zero**
   production callers.

Net effect: the model received a *larger, lower-signal* document, while the
genuinely compressed presentation was never emitted.

## What has been fixed (committed baseline: `5021889`, `a2899cd`)

- `content` is the SCHEMA-v5 presentation on every path; the codec is code-side
  only.
- Delta presentation is the minimal summary
  `Δ delta for … (v{from} → v{to}): +N ~N -N ops`, with the op list in
  `result.delta` — never a full presentation or a `FILE-CONTEXT-DELTA v1`
  envelope.
- `content_kind` is a typed `ContentKind` enum (was stringly-typed).
- Prompts/vocabulary teach SCHEMA-v5, not the retired COMPACT-A codec.
- Harness split: `verification/context-compression/codec/` (reversibility) vs
  `schema-v5/` (presentation), plus measurement and a task-based edit eval
  (**3/3 green**, deterministic grading + `apply_edit` round-trip).

## Current work: annotation-redundancy collapse (A/B)

**Finding:** the `async` fact was reported up to three times in the High
presentation — `mod:ASYNC` + `se:async` + `ec:async` — densest in C# (async
methods + interface mirror). This is presentation-only; the codec still stores
all three facts.

**Implemented in `src/ir/render_llm.rs`:**
- **Collapse #1** — drop `se:async`/`ec:async` when `mod:ASYNC` is present.
- **Collapse #2** — drop `ec:async` when `se:async` is present (covers the
  `IOrderService` interface mirror, which has no `mod:ASYNC`).

Tests updated in `src/tests/ir/render_llm_edit.rs` and
`src/tests/mcp/tool_helpers.rs`.

**Result (measured, o200k High):**

| Fixture | before | after | Δ |
|---|---:|---:|---:|
| typescript | 65.5% | 67.1% | +1.6pp |
| angular | 60.4% | 61.8% | +1.3pp |
| csharp | 41.3% | 43.9% | +2.6pp |

Low/Medium are unchanged (`se:`/`ec:` are High-only). C# gains the most, as
expected (densest async population + interface mirror). The remaining C# gap is
the verbose `CancellationToken`/`Task<ActionResult<T>>` signatures, not
annotation redundancy.

## What to do next

1. Commit the annotation-redundancy collapse (source + tests + doc updates).
2. If pursuing further C# density, the next lever is collapsing the verbose
   async signature boilerplate in the presentation (e.g. eliding repeated
   `CancellationToken cancellationToken = default`), which is a separate,
   presentation-only A/B with its own density + reasoning gate.

## Non-negotiable constraints

- Strict UTF-8, no BOM — see `.clinerules/encoding.md`; run
  `scripts/check-utf8.ps1` after any text edit.
- Zero warnings: `cargo clippy --all-targets --all-features -- -D warnings`.
- Files ≤ 615 lines; tests live in `src/tests/**` via `#[path]`, never inline.
- Do **not** start long-running processes yourself — hand off `cargo`/binary
  commands and report results accurately.
