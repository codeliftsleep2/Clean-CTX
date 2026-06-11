# Token Efficiency Audit

> **Date:** 2026-06-10
> **Scope:** Full `provide_code_context` flow — heuristics, compression, delta transport, persistence, and workspace mode.
> **Trigger:** Post-fix review following the delta baseline key mismatch bug (`α1` alias vs raw path).

---

## Finding 1: Underutilized `source_cache`

### Impact: High · Effort: Medium

`McpState::read_source()` (implemented in `src/mcp/state.rs`) provides a shared source-content cache keyed by canonicalized path, but **only two call sites in `workspace.rs` use it** (template `.html` and style `.scss` shape extraction). All other code paths call `std::fs::read_to_string()` directly.

### Affected call sites

| Location | File | Line(s) | Reads per call |
|---|---|---|---|
| Heuristics source counting | `tools.rs` | 991 | 1× |
| `compress_file` / `compress_text_body` | `tools.rs` | 1023, 1098, and `pipeline.rs` | 1× each |
| `compile_file_ir` | `tools.rs` | 1484 | 1× |
| `compress_pass_with_global_symbols` | `workspace.rs` | 264 | 1× per file in workspace |
| `diff_code_context_handler` | `tools.rs` | 1571 | 1× (already hashed) |

Each `provide_code_context` call performs **3-4 redundant file reads** when the source has not changed. On delta calls, the file is read at least twice (once for `compress_text_body`, once for `compile_file_ir`).

### Fix: Rewire all call sites to `state.read_source()`

1. **`handle_provide_code_context`** (line 991): replace `std::fs::read_to_string` with `state.read_source()`
2. **`compile_file_ir`** (line 1484): accept `&mut McpState`-borrowed source, or call `state.read_source()` internally
3. **`compress_text_body`** (line 815): accept pre-read source string instead of re-reading
4. **`compress_pass_with_global_symbols`** (workspace.rs:264): use `state.read_source()`

**Gain:** 2-3 redundant reads eliminated per call → faster response times, reduced disk I/O. For workspace mode, up to 5000 files × 2 redundant reads = 10,000 fewer I/O ops.

---

## Finding 2: Double IR compile in delta transport path

### Impact: Medium · Effort: Low

In `handle_provide_code_context`'s `DeltaTransport` branch (lines 1114-1125 of `tools.rs`), the file is compiled to IR **twice**:

```rust
// Line 1114 — First IR compile (updates state.ir_context)
if let Ok(ir) = compile_file_ir(&resolved_path, decision.fidelity, state) {
    state.ir_context.load_ir(ir.clone());
}

// ... (delta computation between)

// Line 1125 — Second IR compile (for persistence only!)
let persist_ir = compile_file_ir(&resolved_path, decision.fidelity, state);
if let Some(store) = &mut state.persistence_store {
    if let Ok(ir) = persist_ir {
        let ir_binary = crate::ir::binary_wire::encode(&ir);
        // ... save to DB
    }
}
```

The second compilation re-reads from disk and re-parses the same file. The first compilation's result is discarded after `load_ir`.

### Fix: Reuse the first compilation result

The `ir` variable from line 1114 is an owned `CompiledIR` already available before the persistence block. The persistence block needs a separate borrow scope to avoid conflicting with `&mut state`:

```rust
// First compile
let compiled_ir = if let Ok(ir) = compile_file_ir(&resolved_path, decision.fidelity, state) {
    state.ir_context.load_ir(ir.clone());
    Some(ir)
} else {
    None
};

// ... delta computation ...

// Persistence block — use compiled_ir instead of re-compiling
if let Some(store) = &mut state.persistence_store {
    if let Some(ir) = &compiled_ir {
        let ir_binary = crate::ir::binary_wire::encode(ir);
        // ... save
    }
}
```

**Gain:** 1 full IR compilation + 1 file read eliminated per delta call.

---

## Finding 3: Path resolution inconsistency

### Impact: Medium · Effort: Low

The `restore_context` and `delta_code_context` handlers use different path values for different purposes within the same function:

**`handle_restore_context` (~line 1219):**
- Exclusion check uses `resolved_path` (from `resolve_file_path(file_path_str, None)`)
- Dict alias and `compress_file` use **raw** `file_path_str`

If a relative path is passed, these diverge. The dict alias gets created under the raw relative path while the exclusion check canonicalizes it — same class of mismatch as the delta baseline bug.

**`handle_delta_code_context` (~line 604):**
- No path resolution at all — no `resolve_file_path`, no workspaceRoot support

### Fix: Canonicalize early, use consistently

1. In both handlers, resolve the path once at the top via `resolve_file_path(file_path_str, workspace_root)` (or `None`)
2. Use the resolved path for: exclusion check, dict alias, `compress_file`, `compile_file_ir`
3. `handle_delta_code_context` should accept optional `workspaceRoot` in its schema (backward-compatible addition)

---

## Finding 4: Fragile ownership of `source` in delta path

### Impact: Low · Effort: Low

In the delta branch (lines 1118-1150), `source` is an owned `String` captured before any mutable borrows. After `state.text_delta.compute_and_store(&path_alias, body_lines)` borrows `state` mutably, `source` is still used (implicitly) for persistence hashing. This works because `source` is not behind a reference, but if anyone later changes it to `state.read_source()` (which returns `Arc<String>`), the code would fail to compile with a double-borrow error.

### Fix: Hash source before the delta computation

```rust
let source_hash = format!("{:x}", sha2::Sha256::digest(source.as_bytes()));
// ... then do delta computation ...
// ... persistence uses source_hash instead of recomputed hash
```

This is a defensive change that enables Finding 1's migration to `state.read_source()`.

---

## Priority Order

| # | Finding | Impact | Effort | Recommended order |
|---|---|---|---|---|
| 1 | Underutilized `source_cache` | High | Medium | **1st** — largest token/I/O savings |
| 4 | Fragile source ownership | Low | Low | **2nd** — prerequisite for Finding 1 |
| 2 | Double IR compile | Medium | Low | **3rd** — straightforward fix |
| 3 | Path resolution inconsistency | Medium | Low | **4th** — correctness, not performance |

---

## Test Plan

After each fix:

1. **Finding 4** — Existing compile_file_ir tests should pass unchanged (no functional change)
2. **Finding 1** — Verify `provide_code_context` still produces identical output for same input. Verify `source_cache` hit count increases via instrumentation
3. **Finding 2** — Verify delta path persistence still saves correct IR binary. Verify no regression in `handle_restore_context`
4. **Finding 3** — Pass relative paths to `restore_context` and `delta_code_context` and verify they resolve correctly. Verify `handle_delta_code_context` still works without workspaceRoot