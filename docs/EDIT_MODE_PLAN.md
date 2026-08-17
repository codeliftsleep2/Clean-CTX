# Edit Mode & Gap Closure Plan

## Overview

Six gaps were identified from real-world usage of Clean-CTX with Claude. This document outlines the full implementation plan across four phases, introducing a new `edit` fidelity mode that carries verbatim method bodies for byte-exact `replace_in_file` operations.

## The Six Gaps

1. **Real method body content at high fidelity** — No fidelity setting returns literal statement-level source for methods with actual logic; only signatures, imports, and coarse flow-shape flags.
2. **Parameter validation instead of silent no-ops** — Passing `fidelity="full"` silently falls back to default instead of returning an error with valid values.
3. **Genuine edit fidelity with byte-exact guarantees** — IR-reconstructed text is not guaranteed identical to source; the edit tool needs exact matches.
4. **Consistent `workspaceRoot` support** — `provide_code_context` accepts an override but `compress_code_context` and others do not.
5. **Self-reporting skeleton vs. full-content** — No signal tells the agent whether it received structural-only or body-inclusive content.
6. **Graceful degradation tied to index state** — `graph_search` surfaces "still indexing" but `provide_code_context` gives no equivalent signal when it falls back to signature-only output.

## Fidelity Design

| Mode | What Claude sees | Method bodies byte-exact? | Doc byte-exact? | Savings |
|------|------------------|:---:|:---:|:---:|
| `low` | Structural skeleton (thin) | ❌ | ❌ | ~85% |
| `medium` | Structural skeleton (balanced) | ❌ | ❌ | ~70-80% |
| `high` | Structural skeleton (max detail) + control-flow/dataflow metadata | ❌ | ❌ | ~50-60% |
| **`edit`** (NEW) | Structural skeleton **+ verbatim method bodies** via `CoreOp::Body` | ✅ | ❌ | ~40-60% |
| **`verbatim`** (NEW) | Full raw source | ✅ | ✅ | 0% |

Only `edit` triggers `CoreOp::Body` emission. Bodies are expensive (real tokens), so they only appear when the agent actively signals editing via `intent="edit"` or `fidelity="edit"`.

## Phase 1: Core Fidelity & IR Foundation

**Goal:** Add the new fidelity variants and the `CoreOp::Body` instruction that carries verbatim method text.

### 1.1 `src/compression/fidelity.rs`
- Add `Fidelity::Edit` and `Fidelity::Verbatim` variants
- Update `Fidelity::parse` to accept `"edit"` and `"verbatim"`
- Update `FidelityParseError` display to list all valid values
- Update `Serialize`/`Deserialize` impls

### 1.2 `src/ir/opcodes.rs`
- Add `CoreOp::Body(String, String)` — `(method_id, verbatim_text)`
- Add to `opcode_name()` → `"BODY"`
- Add to `arity()` → `Some(3)`
- Add to `Display` impl
- Add `BODY` to wire-format tuple conversion

### 1.3 `src/ir/compiler.rs`
- When `fidelity == Fidelity::Edit`, capture the raw method body text from the tree-sitter `method.root` capture
- Emit `CoreOp::Body(method_id, raw_body)` immediately after `DefMethod`
- The body text is the **verbatim source slice** — no transformation

### 1.4 `src/ir/hierarchical.rs`
- Add `body: Option<String>` to `MethodNode` (serde `skip_serializing_if = "Option::is_none"`)
- Populate from `CoreOp::Body` in `ir_to_hierarchical`
- Emit `CoreOp::Body` in `hierarchical_to_ir`
- **Also fix Gap 1**: stop discarding `DataFlow`/`ControlFlow`/`SideEffect`/`ExecutionContext` ops — add them as optional fields on `MethodNode`

### 1.5 `src/ir/render_llm.rs`
- At `Fidelity::Edit`: render method bodies verbatim after the method signature line
- At `Fidelity::High`: render ControlFlow/DataFlow metadata as inline markers
- At `Low`/`Medium`: current behavior unchanged

### 1.6 `src/ir/delta.rs`
- Verify `CoreOp::Body` participates correctly in delta computation
- Add tests

## Phase 2: Heuristics & Config

**Goal:** Wire the new fidelity modes into the decision engine and configuration.

### 2.1 `src/config.rs`
- Change `default_sd_edit()` from `Fidelity::Low` → `Fidelity::Edit`
- Add `auto_edit_mode: bool` to `HeuristicsConfig` (default `true`)
- Add `edit_auto_classifications: Vec<String>` — file classes that auto-select Edit

### 2.2 `src/mcp/heuristics.rs`
- **Gap 2 fix**: In `resolve_fidelity`, when an explicit `fidelity` arg is provided but fails to parse, return an error instead of silently falling back
- `intent="edit"` → `Fidelity::Edit` (via `smart_defaults.edit`)
- Auto-classification: when `auto_edit_mode` is on and no explicit intent/fidelity, `Service` and `Implementation` classes map to `Fidelity::Edit`
- `Fidelity::Verbatim` passes through as explicit override only

### 2.3 `src/mcp/tools.rs`
- **Gap 4 fix**: Add `"workspaceRoot"` to schemas for `compress_code_context`, `diff_code_context`, `delta_text_context`, `restore_context`, `compress_workspace`
- Update `fidelity` descriptions to include `"edit"` and `"verbatim"`
- Update `intent` description for `provide_code_context` to explain edit mode
- Add `enum` constraints to fidelity fields where feasible

## Phase 3: MCP Response Contract & LLM Instructions

**Goal:** Make the tool self-reporting and teach Claude when/how to use edit mode.

### 3.1 `src/mcp/tool_handlers/core.rs`
- **Gap 5 fix**: Add `content_kind` to all `provide_code_context` responses
- **Gap 3 fix**: Add `byte_exact` field
- **Gap 6 fix**: Add `degradation` field
- Wire the `CompileError` reason into `degradation.ir_compiler` when falling back to legacy compression

### 3.2 `src/mcp/prompts.rs`
- Add "Edit Mode" section to `SYSTEM_PROMPT`

### 3.3 `src/mcp/tool_handlers/core.rs` — `handle_compress_code_context`
- Add `workspaceRoot` parameter extraction
- Add `content_kind` and `byte_exact` to response

## Phase 4: Tests & Verification

### 4.1 New test files
- `src/tests/compression/fidelity.rs` — extend for `Edit`/`Verbatim` parse
- `src/tests/ir/opcodes.rs` — `Body` op arity/display
- `src/tests/ir/hierarchical.rs` — `MethodNode.body` round-trip
- `src/tests/ir/render_llm.rs` — edit mode rendering, high fidelity control-flow
- `src/tests/mcp/heuristics.rs` — invalid fidelity error, auto-edit classification
- `src/tests/mcp/tools.rs` — schema workspaceRoot presence
- `src/tests/mcp/tool_handlers.rs` — response contract fields

### 4.2 Verification
- `cargo clippy --all-targets -- -D warnings` — zero warnings
- `cargo test --workspace --all-targets --all-features` — all passing
- Manual smoke test: `provide_code_context` with `intent="edit"` on a `.cs` file returns verbatim bodies

## Dependency Graph

```
Phase 1 (fidelity + IR)  →  Phase 2 (heuristics + config)  →  Phase 3 (MCP + prompts)
                                                                    ↓
                                                              Phase 4 (tests)
```

## Gap Coverage Matrix

| Gap | Phase | Primary Files |
|-----|:-----:|:-------------:|
| 1 — Real method bodies at high fidelity | 1 | `hierarchical.rs`, `render_llm.rs` |
| 2 — Parameter validation | 2 | `heuristics.rs`, `tools.rs` |
| 3 — Edit fidelity with byte-exact guarantees | 1+3 | `fidelity.rs`, `opcodes.rs`, `core.rs` |
| 4 — Consistent workspaceRoot | 2 | `tools.rs` |
| 5 — Self-reporting skeleton vs full | 3 | `core.rs` |
| 6 — Graceful degradation | 3 | `core.rs`, `state.rs` |