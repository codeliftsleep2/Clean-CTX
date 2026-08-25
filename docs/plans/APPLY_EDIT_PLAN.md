# `apply_edit` — Clean-CTX-Native Write Path

**Status:** ✅ Implemented (Phases 1–5 complete)
**Created:** 2026-08-25
**Depends on:** `docs/EDIT_MODE_PLAN.md` (Edit fidelity + `CoreOp::Body`) — complete/landed
**Target Release:** TBD

---

## Executive Summary

Every MCP client host that edits a file still performs a full raw file read *immediately before* the write, even when Clean-CTX already delivered byte-exact content for the exact region being changed via `provide_code_context(fidelity="edit", focusMethods=[...])`. That raw read exists to satisfy the **client host's own** staleness precondition on its write tool — a check Clean-CTX cannot see, populate, or influence — not because Clean-CTX lacks the content.

This plan adds a Clean-CTX-native `apply_edit` MCP tool that performs the write itself, using state Clean-CTX already tracks, so a client host that trusts Clean-CTX's read path never needs the redundant raw read before a write.

**This is additive, not a replacement.** Any client's native edit tool keeps working exactly as before. `apply_edit` is an opt-in fast path for the specific case Clean-CTX already has enough information to handle safely: replacing, inserting, or deleting a single named structural unit (method, field, import) whose exact prior text Clean-CTX has already shown the caller.

---

## Problem Statement

Today's read→edit loop for a Clean-CTX-tracked file:

1. `provide_code_context(file, fidelity="edit", focusMethods=["processOrder"])` — cheap, byte-exact body for `processOrder` only, signature-only for everything else.
2. Client host's write tool requires a fresh raw read of the *entire* file before it will accept a write, to build its own before/after diff and guarantee the edit target still matches. This re-fetches content Clean-CTX already delivered in step 1, in full and uncompressed.
3. Write proceeds.

Step 2's cost scales with file size and is paid on **every edit**, regardless of how small the change is or how recently Clean-CTX confirmed the file's state. On a 500-line service file, that is thousands of raw tokens spent solely to satisfy a precondition Clean-CTX's own `LocalStateCache` already answers for free.

## Goal

Let a Clean-CTX-aware caller skip step 2 by routing the write through Clean-CTX itself, verified against Clean-CTX's own tracked hash/span state instead of a fresh raw read.

---

## Design

### Core idea: unit-granularity optimistic concurrency, not whole-file re-verification

The client host's read→edit convention re-verifies the **entire file's bytes** before every write. `apply_edit` instead re-verifies only the **specific structural unit being changed** (a method, field, or import), at the granularity Clean-CTX already models structure. This is deliberately a narrower guarantee than "the whole file is unchanged" — see [Risk Analysis](#risk-analysis) for why that's the right trade, not a shortcut.

Two existing primitives already provide most of what this needs:

| Primitive | Where it lives today | Role in `apply_edit` |
|---|---|---|
| Whole-file SHA-256 hash registry | `LocalStateCache.registry` (`src/cache.rs`) | Fast path: "has anything in this file changed since we last looked?" |
| Byte-exact method body text | `CoreOp::Body(method_id, verbatim_text)` (Edit fidelity, `src/ir/opcodes.rs`) | The "expected old text" half of an optimistic-concurrency check |

The one thing missing: **`CoreOp::Body` carries text, not position.** To splice a replacement into the file, the write path needs to know exactly which bytes to replace.

### New primitive: span-tracking on `CoreOp::Body`

Extend the opcode from `Body(method_id, verbatim_text)` to `Body(method_id, verbatim_text, start_byte, end_byte)`.

The byte offset is *already computed* during compression — `Capture.start_byte` exists today in `src/compression/capture_pipeline.rs` for capture sorting — it simply isn't threaded through into the IR body opcode yet. The one real gap: `extract_method_body` currently trims a method-level capture down to just its `{ ... }` block and returns text only; it needs to also return *where that trimmed sub-slice starts* relative to the capture's own `start_byte`, so the final absolute offset is `capture.start_byte + body_offset_within_capture`.

This is an additive extension to a subsystem that (per `EDIT_MODE_PLAN.md`) touches exactly four files for any opcode-shape change — `wire.rs`, `binary_wire.rs`, `delta.rs`, `hierarchical.rs` — each of which already has a dedicated "Edit Mode: Verbatim Method Bodies" section to extend.

### The write path

1. **Whole-file fast path.** Compute the current file's SHA-256 and compare against `LocalStateCache.registry` (`update_and_verify`, already implemented). If it matches what was last established via a `provide_code_context`/`apply_edit` call on this file, the byte offsets from that last read are known-good — splice directly.
2. **Unit-level fallback.** If the whole-file hash doesn't match (something else touched the file since Clean-CTX last looked — another tool, a formatter, an out-of-band edit), don't fail immediately. Re-parse the file, re-locate the target unit by name **and** a structural key (see [Risk Analysis](#risk-analysis)), and byte-compare its *current* text against the *expected old text* the caller supplied. If they still match exactly, nothing relevant to this edit changed — proceed and refresh the hash/span cache for the rest of the file as a side effect.
3. **Reject on mismatch.** If the target unit's current text doesn't match what the caller expects, refuse the write and return a structured diff of expected-vs-actual (bounded size) so the caller can decide whether to re-read and retry or abandon the edit — better signal than a generic "text not found" failure.
4. **Syntax gate before commit.** Splice the replacement into an in-memory copy of the file and re-parse it with tree-sitter (already a pipeline dependency). If `tree.root_node().has_error()`, refuse the write and report the parse error location. This catches malformed replacements *before* they hit disk — something a plain text-replace write tool cannot do.
5. **Commit + update state.** Write the file, recompute its hash, update `LocalStateCache`, and (mirroring what `provide_code_context` already does) refresh the stored baseline/IR state for the file so a subsequent `provide_code_context` call in the same session gets a cheap delta instead of a full recompression.
6. **Minimal response.** Return a compact confirmation by default — new file hash, new unit span, byte delta size — not the file or method content the caller just supplied. An opt-in `verify: true` echoes the new verbatim text back for callers that want a receipt.

### Operation shapes

A small, closed set of structural operations — not an arbitrary line-based patch format (that would just reimplement `git apply`, worse, and lose the structural verification that makes this safe):

- `replace_body` — `(unit_qualified_name, expected_old_text, new_text)`. The primary case.
- `insert_after` / `insert_before` — add a new unit adjacent to a named anchor.
- `delete` — remove a named unit.

Each targets a method, field, or import — whatever granularity `CoreOp` already models. Signature changes, renames, and anything that has effects at other call sites are **explicitly out of scope for v1** — those are cross-file concerns better served by the existing read→edit path (or a future dedicated refactor tool) until Clean-CTX's cross-file graph state is part of this verification loop.

### Concurrency

No new locking primitive needed. Per `docs/ARCHITECTURE_OVERVIEW.md` (A-09), `McpState` already serializes writes through an `RwLock` shared by the compression path; `apply_edit` is simply a new code path acquiring that same write lock.

### Module layout

```
src/edit/                      (new)
├── mod.rs
├── ops.rs        # ReplaceBody / InsertAfter / InsertBefore / Delete
├── locate.rs      # re-locate a unit by name + structural key, unit-level re-verification
└── apply.rs        # splice, syntax gate, write, cache/state update

src/ir/opcodes.rs                extend CoreOp::Body to 4-tuple
src/ir/wire.rs                   tuple encode/decode for the extra fields
src/ir/binary_wire.rs            binary encode/decode for the extra fields
src/ir/delta.rs                  delta key functions
src/ir/hierarchical.rs           MethodNode round-trip
src/compression/capture_pipeline.rs   extract_method_body returns (text, body_offset)
src/mcp/tools.rs                 new `apply_edit` tool definition + schema
src/mcp/tool_handlers/edit.rs    (new) handler, dispatched through the existing RwLock write path
src/cache.rs                     no struct changes — LocalStateCache already has what's needed
```

### Wire/compatibility note

Widening `CoreOp::Body` is a breaking change to the IR wire format for anyone holding a serialized `Body` op from before this lands (in-flight session state, persisted SQLite baselines). Since Edit fidelity is itself a very recent addition, the safest path is a version-gated decode: old 3-element `["BODY", id, text]` tuples decode with `start_byte`/`end_byte` as `None` (span-less, `apply_edit`-ineligible until the file is next recompressed), rather than forcing a hard cutover or a persistence migration.

---

## Risk Analysis

**Is unit-level verification actually as safe as the client host's whole-file check?**

It's a narrower guarantee, and that's intentional, not a corner cut:

- The whole-file check verifies bytes the edit doesn't even touch. It's simple, but it means two agents editing *different, unrelated* methods in the same file serialize unnecessarily — the second one's stale-file check fails even though its target text is untouched.
- `apply_edit`'s guarantee — "the bytes I'm about to overwrite are exactly the bytes I last saw" — is the actual invariant that matters for correctness, and it's the same one any optimistic-concurrency database write or three-way merge relies on. It is not weaker where it counts; it's differently scoped, and strictly more permissive where the scoping doesn't matter.

**Where this can actually go wrong:** the unit-relocation step (Design step 2). If a file was restructured (e.g. methods reordered, a wrapping class renamed) between the last known state and this edit, name-only lookup could resolve to the wrong unit that happens to share a name in a different scope (overloads, a same-named method in a different class). Mitigations, both required before this ships:

1. Key relocation on **qualified name + a structural fingerprint** (e.g. parameter types, containing class id) — not bare method name.
2. Treat the [syntax gate](#the-write-path) as a hard, non-bypassable pre-commit check, not a warning — a mis-relocated splice is far more likely to produce a parse error than a valid-but-wrong edit, and the gate exists specifically to catch that class of failure before disk.

A `replace_body` whose `expected_old_text` mismatches *and* fails relocation entirely should fail loudly with both signals surfaced, not silently fall back to "insert somewhere plausible."

---

## Adoption Note (not just an implementation concern)

Shipping `apply_edit` server-side doesn't automatically save anything — an agent has to actually call it instead of falling back to the read→edit convention it already knows. That means:

- Tool-selection guidance (the equivalent of this project's own Clean-CTX usage rules) needs to name `apply_edit` explicitly as the preferred path for single-unit edits once content has been seen via `provide_code_context(fidelity="edit"|"verbatim")` in the same session, the same way those rules currently steer reads toward `provide_code_context` over a raw file read.
- A client host's own write tool will still independently enforce its own raw-read precondition if invoked — `apply_edit` only helps when the agent chooses it *instead of* that tool, not as a wrapper around it.
- The gap this closes is therefore partly a documentation/prompting problem, not purely a server capability gap. Ship the capability and the usage rule together, or the tool exists but nothing routes through it.

---

## Phased Implementation

### Phase 1 — Span tracking
- Extend `CoreOp::Body` to `(method_id, verbatim_text, start_byte, end_byte)`.
- Update `wire.rs`, `binary_wire.rs`, `delta.rs`, `hierarchical.rs` (additive; each already has an Edit-mode section).
- `extract_method_body` returns the body sub-slice's offset within its capture, not just text.
- Version-gate decode for pre-existing 3-element `BODY` tuples (see [Wire/compatibility note](#wirecompatibility-note)).

### Phase 2 — Write path core
- New `src/edit/` module: `ops.rs`, `locate.rs`, `apply.rs`.
- Unit relocation keyed on qualified name + structural fingerprint.
- Splice + in-memory syntax gate (`has_error()`) as a hard pre-commit check.
- Hash/baseline update on successful commit, reusing `LocalStateCache` as-is.

### Phase 3 — MCP surface
- `apply_edit` tool definition: `filePath`, `operations: [ReplaceBody | InsertAfter | InsertBefore | Delete]`, optional `verify: bool`.
- Handler wired through the existing dispatcher/`RwLock` write path (`src/mcp/tool_handlers/edit.rs`).
- Structured mismatch-error payload (expected vs. actual, bounded size) for the reject case.
- Minimal-by-default response shape (hash + span + delta size, no echoed content unless `verify: true`).

### Phase 4 — Tests
- Unit: span relocation on unchanged / reordered / renamed-sibling files; syntax-gate rejection on malformed splices.
- Integration: full `apply_edit` round trip against existing fixture files (the same fixtures `EDIT_MODE_PLAN.md`/`PERFORMANCE.md` already use), followed by a `provide_code_context` call confirming delta transport picks up the change.
- Adversarial: two `apply_edit` calls targeting different units in the same file both succeed without serializing; two calls targeting the *same* unit — the second is rejected with a mismatch error, not silently overwritten.

### Phase 5 — Measurement
- Add a benchmark alongside `examples/fidelity_comparison.rs` that measures the *actual* token delta of `apply_edit` vs. the read→edit convention across the existing 50-edit simulation categories, rather than extrapolating from `docs/PERFORMANCE.md`'s existing read-side numbers. The read-side savings are already measured; the write-side savings this plan targets are not, and should be validated the same rigorous way before claiming a number.

---

## What This Does Not Replace

- Cross-file edits, renames, and signature changes — still go through the client host's native read→edit path, or a future dedicated cross-file refactor tool once Clean-CTX's graph state (blast radius, call sites) is part of the verification loop.
- The client host's own write tool remains fully correct and available for any caller that hasn't adopted `apply_edit` — this is additive, never a deprecation.

---

---

## Implementation Notes (2026-08-25 — as built)

- **Wire compatibility**: `CoreOp::Body` is now `(method_id, verbatim_text, Option<u64> start, Option<u64> end)` with a both-or-neither pairing invariant surfaced via `body_span()`. Span-less bodies serialize as the legacy 3-tuple; spanned bodies as 5-tuples; `tuple_to_op` accepts exactly 3 or 5 — the single compat gate covering named/tagged/positional/string-table decoders. Binary wire is **v0x03** (`[mid_idx, text_idx, has_span_flag, start?, end?]`, offsets as raw varints); 0x01/0x02 streams still decode, producing span-less bodies that are `apply_edit`-ineligible until recompressed.
- **Emission**: `pipeline.rs::locate_method_body()` returns `(text, byte-offset-within-capture)` — C# attribute-strip aware via pointer arithmetic (stripping is prefix-only) — and the Edit-fidelity emission threads the absolute span `capture.start_byte + offset .. capture.end_byte`. The old text-only `extract_method_body` wrapper was removed as dead code.
- **Concurrency deviation from Design §5**: "acquire that same write lock" self-deadlocks today because `compile_file_ir_focused` takes an `ir_context` READ lock internally (`state.file_version()`); a caller cannot hold that lock's WRITE guard across compilation. Commits serialize through a module-local mutex in `tool_handlers/edit.rs` instead — identical no-interleaving guarantee, no new lock-order interactions.
- **Relocation**: every call relocates against a fresh Edit-fidelity compile of the CURRENT bytes, subsuming the whole-file-hash fast path (strictly safer; costs one local parse server-side, zero client tokens). Unit tables materialize in instruction order — a HashMap-iteration nondeterminism (randomized ambiguity-candidate listings) was caught by the adversarial test and fixed. Keys: qualified name + structural fingerprint; bare names only when unambiguous; span-less bodies excluded by construction.
- **Session baseline refresh**: post-commit, the handler recompiles, `load_ir`s the result with the new hash, updates the registry hash, invalidates the source cache entry, and evicts the stale llm-text entry — so the next `provide_code_context` yields a delta (verified e2e).
- **Dead-code note**: the unattached `src/tests/ir/compiler_methods.rs` references the removed wrapper. If that file is ever wired into a module tree, port its assertions to `locate_method_body`.

## Open Questions

1. Does the client host's own permission/confirmation model apply uniformly to an MCP tool call that writes a file, the same way it does for a native write tool? Needs confirming per-host, not assumed.
2. Should `apply_edit` be allowed to create a *new* file (no prior read), or strictly require a prior `provide_code_context` in the same session? Leaning toward the latter for v1 — it keeps the verification story simple (there is always a "last known state" to check against) and pushes new-file creation through the existing, already-correct write tool.
   **✅ Resolved (v1):** prior tracked state IS required; `apply_edit` refuses with a structured `no_tracked_state` error otherwise.
3. Should the persisted SQLite baseline (`docs/ARCHITECTURE_OVERVIEW.md`'s Persistence Layer) be updated synchronously on `apply_edit`, or left to the next `provide_code_context` call the way in-session state is? Persistence writes are already non-fatal/fire-and-forget elsewhere in the codebase; the same pattern likely applies here without a new design.
   **✅ Resolved (v1):** deferred — the SQLite baseline refreshes on the next `provide_code_context` call; `apply_edit` writes nothing to persistence (avoids empty-baseline rows misrepresenting the session).

---

## Known Issues (Post-Ship Verification — 2026-08-25)

### BUG: `replace_body` rejects byte-exact `expectedOldText` on LF-encoded files — tracked span byte length does not match the real file's bytes

**Repro (outcomes repo, `Outcomes.ApiCore.Tests/Populators/UserPopulator.cs`, method `CreateUser`, LF-only line endings — confirmed via `grep -c $'\r'` = 0 on the file):**

1. `provide_code_context(filePath, fidelity="edit", intent="edit")` — renders the method body normally.
2. `apply_edit` with a `replace_body` op targeting `UserPopulator.CreateUser`, `expectedOldText` hand-transcribed from that rendering (a trivial one-line-comment insertion as `newText`).
3. Rejected: `unit changed since last seen (expected 1018 bytes, actual 1048)`.
4. Assumed a transcription error, so re-derived `expectedOldText` **directly from the real file on disk**, independent of any Clean-CTX rendering: `sed -n '16,44p' UserPopulator.cs | wc -c` → **1021 bytes**, byte-for-byte verified (tabs, no trailing newline).
5. Retried `apply_edit` with that byte-exact 1021-byte text. **Still rejected**, against the same `actual 1048`.
6. Forced a fresh compile via `provide_code_context(fidelity="verbatim")` immediately before retrying (rules out a stale "last seen" cache) — no change.

**Evidence 1048 isn't derived from any straightforward slice of the real file:**
- Body only (lines 16–44): 1021 bytes
- Signature + body (lines 15–44): 1167 bytes
- Neither matches 1048.

**Working hypothesis:** the body spans 29 lines → 28 internal newlines. `1021 (true LF byte count) + 28 (one extra byte per newline if internally counted as CRLF) = 1049` — within 1 byte of the reported `actual 1048`. This strongly suggests the tracked span's byte length (wherever `locate_method_body()`'s offset/length or `src/edit/apply.rs`'s expected-text length check is computed) is derived from a CRLF-normalized or otherwise line-ending-transformed representation of the body, while the real file on disk — and everything `provide_code_context` renders back to the caller — is LF-only. This repo's convention (and most non-Windows-authored source) is LF-only, so this would misfire on close to every multi-line `replace_body`/`delete` call against it. The exact +27/+28-byte source is **not yet confirmed** — this is a strong, evidence-backed hypothesis (off by 1 byte from a clean prediction), not a root-caused fix. Recommend live-audited tracing (per this project's usual practice: reproduce with a failing regression test — e.g. an LF-only multi-line fixture round-tripped through `provide_code_context(fidelity="edit")` → `apply_edit` — before patching) to pin the exact line in `pipeline.rs`/`src/edit/` doing the miscount.

**Safety held:** both rejected attempts wrote nothing to disk (`git status`/`git diff` clean after each) — the mismatch gate correctly refused rather than silently overwriting, it's just comparing against a miscounted expectation.

**Impact:** as shipped, `replace_body`/`delete` on a multi-line unit in an LF-encoded file will very likely reject byte-perfect `expectedOldText`, defeating v1's core promise for what is probably the majority of real-world source files.
