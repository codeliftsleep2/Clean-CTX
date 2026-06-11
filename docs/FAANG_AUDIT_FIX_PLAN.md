# FAANG Audit Remediation Plan — Phase 2

## Overview

**Date:** 2026-06-11
**Context:** Following the completion of Rust language support (Phase 1: L-01 through L-06), a second FAANG audit was performed. This document outlines the issues found and a phased plan to resolve them.

**Reference:** `docs/FAANG_AUDIT_RUST_SUPPORT.md` (Phase 1 report)

---

## Current State

- 866 tests pass (0 failed)
- All Phase 1 deferred items (L-03 through L-06) resolved
- Rust token tracking is live via SessionStats pipeline
- 3 dead code warnings on compilation

---

## Issues Found

### P1-CRITICAL: `provide_code_context` fails on `.rs` files

```
Error: MCP error -32603: Unsupported file extension: .rs
```

**Root cause:** The MCP server binary may not have `tree-sitter-rust` linked, or the error is propagating incorrectly. The text pipeline (`compress_file` → `language_for_extension("rs")`) returns a valid `Some(...)`, so the error is likely in the MCP transport or JSON-RPC layer.

**Files affected:**
- `src/mcp/tools.rs` — `handle_provide_code_context` → `compress_file` → pipeline
- `Cargo.toml` — verify `tree-sitter-rust` is a dependency

**Verification:** `cargo run` with a `.rs` file path should succeed.

---

### P2: 3 items of dead code (compiler warnings)

1. **`MODIFIERS_LOW_RS`** — `src/compaction/modifiers.rs:58` — defined but never used
2. **`MODIFIERS_MEDIUM_RS`** — `src/compaction/modifiers.rs:67` — defined but never used
3. **`RustLayer::extract_visibility()`** — `src/ir/layers/rust.rs:82` — defined but never called

**Fix:** Remove unused constants. Either remove `extract_visibility()` or wire it into the RustLayer `process_capture` path.

---

### P3: `extract_cfg()` defined but never wired

`RustLayer::extract_cfg(source, type_start)` → `src/ir/layers/rust.rs:208`

- Has unit tests in `src/tests/ir/layers/rust.rs`
- **No call site** in the IR compiler (`src/ir/compiler.rs`) or text pipeline (`src/compression/pipeline.rs`, `src/compaction/class.rs`)

**Impact:** `#[cfg(feature = "...")]` attributes on Rust types are silently dropped. The LLM never sees platform/feature gating.

**Fix:** Wire `extract_cfg()` into the `struct.root`/`enum.root`/`trait.root` handler in `compiler.rs` (emit as a class-level flag or separate op).

---

### P4: `extract_generic_params()` defined but never called

`RustLayer::extract_generic_params(text)` → `src/ir/layers/rust.rs:226`

- Has unit tests
- **No call site** in any pipeline code

**Impact:** Generic parameters (`MyStruct<T, U>`) are stripped by `extract_rust_struct_name` before they can be captured. The LLM never sees `T: Clone` or `T, U` constraints.

**Fix:** Call `extract_generic_params()` from the `struct.root`/`enum.root`/`trait.root` handler in `compiler.rs`. Emit generic params as a class-level annotation or separate instruction. Or integrate into `extract_rust_struct_name` for Medium/High fidelity.

---

### P5: Text pipeline vs IR pipeline inconsistency for `impl.root`

| Pipeline | `impl.root` behavior |
|----------|---------------------|
| Text (`build_output_lines`) | Creates a class entry, increments `class_count`, calls `format_rust_type_entry` |
| IR (`compiler.rs`) | Does NOT create `DefClass` — only calls language layers |

**Impact:** A standalone inherent impl with no preceding struct/enum/trait definition (e.g., `impl ForeignType { fn helper(&self) {} }`) appears in text output but disappears from IR output.

**Fix:** Either:
- (a) Make `impl.root` in the IR compiler emit a `DefClass` for the self-type if no current class exists, OR
- (b) Document this as intentional (IR pipelines require a preceding struct/enum/trait).

---

### P6: `diff_code_context` fallback has Rust blind spot

`src/diff/builder.rs` lines 37-44:

```rust
let (other_lang, other_query) = if query_string == queries::TS_QUERY {
    (tree_sitter_c_sharp::language(), queries::CS_QUERY)
} else {
    (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
};
```

**Impact:** When `detect_language` picks Rust but yields no captures (e.g., a Rust file with only comments and whitespace), the fallback tries TS → C# but never Rust. It can never fall back *to* Rust.

**Fix:** Add a Rust fallback branch. The current ternary only handles TS ↔ C#. Rust needs `RS_QUERY` as a third option:

```
First choice → Rust → fallback → TS
                         → C#
```

---

### P7: Trait/type generics lost in `extract_rust_struct_name`

`src/compaction/class.rs` lines 163-178 — for `impl Trait for Type`:

```rust
let trait_name = trait_part
    .split_whitespace()
    .next()  // loses "<T>" 
```

**Impact:** `impl<T> Repository<T> for PostgresRepo` produces `PostgresRepo:Repository` instead of `PostgresRepo:Repository<T>`. At Medium/High fidelity this is a data loss.

**Fix:** For Medium/High fidelity, preserve the generic portion by extracting `<...>` alongside the trait name.

---

## Remediation Phases

### Phase A: Dead Code & Cleanup (P2)

**Effort:** 15 min
**Risk:** None

1. Remove `MODIFIERS_LOW_RS` and `MODIFIERS_MEDIUM_RS` from `src/compaction/modifiers.rs`
2. Wire `extract_visibility()` into `RustLayer::process_capture` for `struct.root`/`enum.root`/`trait.root` captures, or remove it

---

### Phase B: Wire Unused Functions into Pipeline (P3 + P4)

**Effort:** 1-2 hours
**Risk:** Low — adding new IR ops is backward-compatible

1. **P3 (cfg):** In `src/ir/compiler.rs`, in the `struct.root` | `enum.root` | `trait.root` handler (lines 217-244), call `RustLayer::extract_cfg(source, class_start_position)` and emit the cfg predicate as a `ClassFlags` entry or new op
2. **P4 (generics):** In the same handler, call `RustLayer::extract_generic_params(cap.text)` and store the generic string in `LayerContext` for downstream emission
3. Add integration tests for both

---

### Phase C: Text/IR Pipeline Consistency (P5)

**Effort:** 1-2 hours
**Risk:** Medium — may change behavior for existing IR consumers

1. In `src/ir/compiler.rs` `impl.root` handler (line 247), if `current_class` is `None`:
   - Extract the self-type from the impl head
   - Emit `DefClass(self_type_id, self_type_name)` 
   - Set `current_class` to the new ID
2. Update `extract_rust_struct_name` to handle the result properly

---

### Phase D: diff_code_context Fallback (P6)

**Effort:** 30 min
**Risk:** Low

1. In `src/diff/builder.rs`, add a three-way fallback: Rust → TS → C#
2. Track which parsers have been tried to avoid infinite loops

---

### Phase E: Generic/Where Fidelity Preservation (P7)

**Effort:** 1-2 hours
**Risk:** Low — improves fidelity, doesn't change Low behavior

1. Update `extract_rust_struct_name` to accept a `Fidelity` parameter
2. At Medium/High, preserve `<>` from both trait and self-type names
3. At Low, strip as before

---

### Phase F: Fix `provide_code_context` for `.rs` files (P1)

**Effort:** 2-4 hours (investigation + fix)
**Risk:** Low, but root cause is unknown

1. Reproduce with `cargo run` and a `.rs` file
2. Check `Cargo.toml` for `tree-sitter-rust` dependency
3. Check MCP tool description handling
4. Fix the error and add regression proof

---

## Verification

After each phase:
```
cargo test 2>&1 | tail -5
```

Expected: 866+ passing tests (new tests added per phase), 0 warnings on the targeted items.

After Phase F: Manual MCP-level test with `provide_code_context` on a `.rs` file succeeds.

---

## Priority Order

```
Phase A (dead code — quick win)
  ↓
Phase B (wire cfg + generics — feature completion)
  ↓
Phase E (generic fidelity — improves existing output)
  ↓
Phase D (diff fallback — edge case fix)
  ↓
Phase C (pipeline consistency — behavior change, needs care)
  ↓
Phase F (MCP tool fix — investigation required)