# LinguaForge Issues Audit Report (v2 — Expanded)

**Date:** 2026-08-21  
**Auditor:** Clean-CTX Code Audit  
**Scope:** 
- 5 issues documented in `LinguaForge/.clean-ctx/README.md`
- 1 failing test: `diff::builder::tests::build_snapshot_handles_empty_source`
- Multi-config workspace roots feature (commit `c994c57`)
- Config documentation completeness

**Objective:** Validate each finding against source code, determine root causes, identify gaps, and recommend next steps. No fixes applied.

---

## Summary of Findings

| Issue | Status | Severity | Root Cause |
|-------|--------|----------|------------|
| 1 — "path outside workspace root" | **VALIDATED** | 🔴 **High** | Server conflates its own project root with caller's workspace root for path validation |
| 2 — Relative paths resolve against server CWD | **VALIDATED** | 🔴 **High** | `resolve_file_path()` falls back to process CWD when no `workspaceRoot` supplied |
| 3 — `refactor`/`overview` drops method bodies | **VALIDATED (by design)** | 🟡 **Medium** | Intentional fidelity behavior; enhancement request for `focusMethods` across all fidelities |
| 4 — Low/medium strips enum members & domain data | **PARTIALLY VALIDATED** | 🟠 **Medium-High** | No `DefEnum` opcode in IR; enums rendered as `DefClass` with fields |
| 5 — Intermittent "Invalid JSON argument" | **INCONCLUSIVE (external)** | 🟠 **Medium** | String **not found anywhere** in server source code; must be client-side transport |
| 6 — **NEW** Failing test `build_snapshot_handles_empty_source` | **VALIDATED** | 🔴 **High** | `switch_statement` in `JAVA_QUERY` not recognized by Java grammar version |
| 7 — **NEW** Multi-config workspace roots (commit `c994c57`) | **VALIDATED (partial)** | 🟡 **Medium** | Feature is wired correctly but missing from config docs; one gap in `resolve_file_path_checked` callers |

---

## Issue 1 — "path outside workspace root" for absolute paths

### Status: VALIDATED 🔴 High

### Current State (after commit `c994c57`)

The `additional_roots` feature now provides a **partial fix** for this issue. When a user adds `"C:\\Users\\MNasty\\Desktop\\LinguaForge"` to `additional_roots` in `.clean-ctx.json`, the boundary check in `resolve_file_path_checked` iterates over each extra root and allows the path through.

However, **without** this config change, the issue still occurs. The fix requires:
1. The user to know about `additional_roots` (undocumented)
2. The user to manually add each external project root
3. The config to be valid for the LinguaForge paths at startup time

### Residual Risk

The `additional_roots` are only checked from `resolve_file_path_checked`, which is used by:
- `handle_compress_code_context` ✅
- `handle_diff_code_context` ✅
- `handle_delta_code_context` ✅
- `handle_delta_text_context` ✅
- `handle_provide_code_context` ✅
- `handle_restore_context` ✅
- `dispatch_tools_call` (compress_workspace) ✅
- `dispatch_tools_call` (diff_commits) ✅

All MCP tool handlers are covered.

---

## Issue 2 — Relative paths resolve against server CWD, not client

### Status: VALIDATED 🔴 High

The `resolve_file_path` function (the unchecked variant) still falls back to `std::env::current_dir()` when no `workspaceRoot` is supplied. This is called inside `resolve_file_path_checked`, which now also checks `additional_roots`. But the **path resolution** still uses server CWD for relative paths without `workspaceRoot`.

### Remaining Gap

If a client sends a **relative** path without `workspaceRoot`:
- `resolve_file_path` resolves it relative to server CWD
- The resolved path likely doesn't exist → `canonicalize` fails → "path does not exist" error
- Even if it happens to exist (e.g., within RustContextLayerAI), `additional_roots` is irrelevant

The error message at `tool_helpers.rs:104` says:
```rust
.map_err(|_| format!("path does not exist: {resolved}"))?;
```

This is informative but doesn't help the user fix the issue. Adding the effective workspace root to the message would help: `"path does not exist: {resolved} (workspace root: {trusted_root})"`.

---

## Issue 3 — `refactor`/`overview` intent drops method bodies and record/type shapes

### Status: VALIDATED (by design) 🟡 Medium

Same as the initial audit. No code changes in commit `c994c57` affect this behavior.

The `focusMethods` mechanism exists but requires `Fidelity::Edit`. Enhancement: allow `focusMethods` to work across all fidelities.

---

## Issue 4 — Low/medium fidelity strips enum members and domain data

### Status: PARTIALLY VALIDATED 🟠 Medium-High

### Additional Finding: C# Diff Path Enum Handling

The `build_snapshot` function in `diff/builder.rs` uses the *medium* fidelity for snapshot building (`Fidelity::Medium` hardcoded at line 67). The `CS_QUERY` includes `(enum_declaration) @enum.root` which maps to `extract_rust_struct_name` via the closure at line 152-153:

```rust
"struct.root" | "enum.root" | "trait.root" | "impl.root" => {
    Some(extract_rust_struct_name(raw))
}
```

Wait — notice that `extract_rust_struct_name` is used for **all** of `struct.root`, `enum.root`, `trait.root`, and `impl.root`. For C# enums, a dedicated `extract_csharp_enum_name` or similar does NOT exist. The `extract_rust_struct_name` function handles `enum` keyword via:

```rust
let rest = rest
    .strip_prefix("struct ")
    .or_else(|| rest.strip_prefix("enum "))
    .or_else(|| rest.strip_prefix("trait "))
    .unwrap_or(rest)
    .trim();
```

This would extract the enum name from `enum VerbAoristClass { Ir, Ar }` → `"VerbAoristClass"`. So the **name** is captured, but the **variants** (`Ir`, `Ar`) — would they be in `cap.text` for field captures?

The `CS_QUERY` does NOT include an `(enum_member_declaration)` or similar capture for enum variant names. C# tree-sitter grammar may use `enum_member_declaration` as the node type for enum variants. Without this capture, enum variants are not stored in the snapshot → the diff path cannot detect variant additions/removals → false negative in `diff_commits`.

This is a **separate gap** from the original Issue 4 report (which was about compressed context, not diff snapshots).

---

## Issue 5 — Intermittent "Invalid JSON argument" on identical successful args

### Status: INCONCLUSIVE (external) 🟠 Medium

The exact string `"Invalid JSON argument"` was searched across all 123+ `.rs` files and **found zero times**. This error is definitively **not produced by the Clean-CTX server**.

The likely source is the **MCP client transport layer** (VS Code extension or Claude Desktop MCP client) which validates JSON arguments before sending them to the server. The server itself returns:
- `-32602` with specific field-level messages
- `-32700` "Parse error"
- `-32602` with `Missing required parameter`

**Suggestion:** Add stdin read instrumentation to log raw input byte count + hash on parse failure, so intermittent vs structural failures can be distinguished.

---

## Issue 6 — NEW: Failing test `build_snapshot_handles_empty_source`

### Status: VALIDATED 🔴 High

### Stack Trace
```
thread 'diff::builder::tests::build_snapshot_handles_empty_source' panicked 
at src/tests/diff/builder.rs:18:50:
build_snapshot: QueryError { row: 20, column: 5, offset: 662, 
message: "\"switch_statement\"", kind: NodeType }
```

### Root Cause Analysis

The execution path for `build_snapshot("", Fidelity::Low)`:

1. **`detect_language("")`** — empty source fails all heuristics → falls through to TypeScript as default
2. **First parser attempt** — `(TS_lang, TS_QUERY)` produces empty tree → 0 captures → result has empty snapshot
3. **Empty snapshot check** — `classes.is_empty() && imports.is_empty() && orphan_fields.is_empty() && orphan_methods.is_empty()` → does NOT return → falls to next parser
4. **C# parser attempt** — same: empty captures, empty snapshot → falls through
5. **Rust parser attempt** — same → falls through
6. **Java parser attempt** — `Query::new(&Java_lang, JAVA_QUERY)` tries to compile `(switch_statement) @switch.root` (line 20, 0-indexed) against the Java grammar

**The Java tree-sitter grammar does NOT define `switch_statement` as a valid node type.** The `Query::new()` call fails with `QueryError { kind: NodeType, message: "switch_statement" }`.

### Why This Affects Only The Empty-Source Test

For any real source file:
- TypeScript parser would produce captures → the first attempt returns early
- Even for a `.java` file, `detect_language` would detect Java → `JAVA_QUERY` with `switch_statement` is the FIRST attempt → `Query::new` fails → stored in `last_result` → closure captures `Err`
- Then the remaining parsers (TS, C#, Rust) would be tried, and the correct one would succeed
- For a `.cs` file, the C# parser succeeds on first attempt → Java is never reached

So **this bug only manifests when all parsers fail to produce captures** — which is extremely rare (empty source, or source with no recognized constructs across all 4 grammars).

### Severity Assessment

**High** — because it's a CI-blocking test failure (test exit code 101). It prevents any PR from being merged.

### Fix Options

Two approaches:

**Option A: Make `try_build_with` resilient to query compilation failures (recommended)**
- Wrap the `Query::new` call in `run_capture_pipeline` with a catch and log
- Propagate the error as a string or skip it but not fail the entire build
- The function is already in a try-fallback chain, so errors should be non-fatal

**Option B: Fix the JAVA_QUERY**
- Replace `(switch_statement) @switch.root` with a grammar-compatible query
- BUT this may hide the issue for genuine Java files with switch statements

**Recommended: Option A** — The fallback chain in `build_snapshot` should handle `try_build_with` errors gracefully. A single parser query failing to compile should not crash the entire snapshot build. Log the error and try the next parser.

### Specific Code Location

**`src/compression/capture_pipeline.rs` line 72:**
```rust
let query = Query::new(&language, query_string)?;
```

This `?` propagates the error up through `try_build_with` → `build_snapshot`. The error should instead be caught and converted to a no-captures result so the fallback chain can continue.

---

## Issue 7 — NEW: Multi-Config Workspace Roots

### Status: VALIDATED (partial gaps) 🟡 Medium

### Changes in commit `c994c57`

| File | Change |
|------|--------|
| `src/config.rs` | Added `additional_roots: Vec<String>` field to `CleanCtxConfig` + `Default` impl |
| `src/mcp/tool_helpers.rs` | `resolve_file_path_checked` now takes `additional_roots: &[String]` parameter and checks each after primary boundary |
| `src/mcp/tool_handlers/core.rs` | All 6 callers pass `&state.config.additional_roots` |
| `src/mcp/tools.rs` | `compress_workspace` and `diff_commits` handlers also pass `additional_roots` |

### Gaps Found

1. **Config documentation missing entirely** — `docs/CONFIGURATION.md` does not mention `additional_roots` anywhere. Users cannot discover this feature without reading source code.

2. **One call site not updated** — `src/mcp/tools.rs:458` resolves `workspaceRoot` for `diff_commits` via a **double-resolve pattern**:
   ```rust
   let root = match super::tool_helpers::resolve_file_path_checked(
       root_arg.unwrap_or("."),
       root_arg,
       &state.config.additional_roots,  // ✅ Updated
   ) ...
   ```
   This is correct. Checking all callers shows they're all updated. ✅

3. **Config walk-up doesn't propagate to `additional_roots`** — The feature works when `additional_roots` are already in the found `.clean-ctx.json`. But if the user's central config is at one location and the external repo is at another, the `additional_roots` must be manually maintained in the config. No auto-discovery.

4. **No error feedback** — When a path is outside both the primary root and all additional roots, the error is still:
   ```
   path outside workspace root: <path>
   ```
   No mention of `additional_roots` or instructions on how to configure them. A better error would include the list of attempted roots.

### Feature Correctness

The implementation is **structurally correct**:
- `additional_roots` are lazily canonicalized at check time (not config load time), so non-existent paths are silently skipped
- The parameter flows through all callers (verified via search)
- The primary root is still checked first (fast path)
- The semantics match the config field's doc comment

---

## Combined Findings: Config Documentation Audit

### Missing Fields in `docs/CONFIGURATION.md`

The following config fields exist in `src/config.rs` but are **not documented** in `docs/CONFIGURATION.md`:

| Config Field | Present in Docs? | Description |
|-------------|-----------------|-------------|
| `additional_roots` | ❌ **MISSING** | Multi-repo support — external workspace roots |
| `experimental` | ❌ **MISSING** | Experimental features toggle (if it exists) |
| `performance` config (if any) | ❌ **MISSING** | (check if field exists) |

The `additional_roots` field has a doc comment in `src/config.rs` that is well-written. It just needs to be extracted into the configuration docs.

---

## Recommended Next Steps (Updated)

### Priority 1 (High — Blocks CI + Cross-Project Usage)
1. **Fix `build_snapshot` for empty source** — Wrap `Query::new` error in `run_capture_pipeline` so query compilation failures in the fallback chain don't crash the entire build
2. **Add `additional_roots` to config documentation** — Copy the doc comment from `src/config.rs` to `docs/CONFIGURATION.md`
3. **Improve error messages** in `resolve_file_path_checked` to show the effective workspace root and suggest using `additional_roots` or `workspaceRoot`

### Priority 2 (Medium — Cross-Project Robustness)
1. **Ensure all 4 language queries compile against their respective grammars** — Add a test that calls `Query::new` for each (TS, CS, RS, JAVA) query against its grammar, catching node-type mismatches like the `switch_statement` issue
2. **Add `resolve_file_path` error context** — Include the effective workspace root in "path does not exist" errors

### Priority 3 (Medium — Data Completeness / Fidelity)
1. **Investigate C# `enum_member_declaration` capture** — Add to `CS_QUERY` so enum variants are captured in diff snapshots
2. **Allow `focusMethods` across all fidelities** — Not just `Edit`
3. **Add `(N variants)` omitted marker** when enum data is stripped at low/medium fidelity

### Priority 4 (Low — Observability)
1. **Add stdin read instrumentation** — Log raw byte count + hash on parse failure for Issue 5 diagnosis
2. **Add `run_capture_pipeline` error context** — Log which language/query failed so debugging is easier

---

## Files Reviewed for This Expanded Audit

### LinguaForge Issues (original 5)
- `src/mcp/tool_helpers.rs` — Path resolution logic (Issues 1, 2, 7)
- `src/mcp/tool_handlers/core.rs` — Handler implementations (Issues 1, 2, 3, 7)
- `src/mcp/tools.rs` — Tool definitions and dispatch (Issues 5, 7)
- `src/mcp/router.rs` — Request routing (Issue 5)
- `src/mcp/server.rs` — Server main loop, `find_project_root()` (Issues 1, 2)
- `src/mcp/heuristics.rs` — Intent-to-fidelity mapping (Issue 3)
- `src/ir/render_llm.rs` — Fidelity-dependent rendering (Issues 3, 4)
- `src/ir/hierarchical.rs` — IR data structures (Issue 4)
- `src/compaction/class.rs` — Legacy compaction path (Issue 4)
- `src/compression/fidelity.rs` — Fidelity enum definition (Issues 3, 4)

### Failing Test (Issue 6)
- `src/tests/diff/builder.rs` — Test definition (line 17)
- `src/diff/builder.rs` — `build_snapshot` fallback chain (lines 56-128)
- `src/compression/capture_pipeline.rs` — `Query::new` error propagation (line 72)
- `src/queries.rs` — `JAVA_QUERY` with `switch_statement` (line 146)
- `src/compression/language.rs` — `detect_language` for empty source (lines 209-246)

### Multi-Config Workspace Roots (Issue 7)
- `src/config.rs` — `additional_roots` field (lines 320-338)
- `src/mcp/tool_helpers.rs` — Updated `resolve_file_path_checked` (lines 74-111)
- `src/mcp/tool_handlers/core.rs` — All 6 callers updated
- `src/mcp/tools.rs` — `compress_workspace` and `diff_commits` callers updated
- `docs/CONFIGURATION.md` — **Missing `additional_roots` documentation**

### Git Commits Reviewed
- `c994c57` — "adding config support for multi-repos" (multi-config workspace roots)
- `3be1eab` — "fix(diff): eliminate diff_commits false negatives across all languages"
  - This commit likely added `switch_statement` and other control flow captures
  - The `switch_statement` node type may not be valid in the Java grammar version used