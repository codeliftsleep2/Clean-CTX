# Clean-CTX — Troubleshooting Guide

> **Owner:** Problem-solving + error codes + diagnostic commands · **Status:** Living reference

**Last updated:** 2026-09-27

---

## Common Issues & Resolutions

### Server fails to start with "Failed to load cl100k BPE data"

**Symptom:**
```
[clean-ctx] fatal: Failed to load cl100k BPE data: ...
```

**Cause:** The BPE (Byte-Pair Encoding) data file for the cl100k tokenizer (used by GPT-4 / Claude) is embedded in the `tiktoken-rs` crate at compile time. This error means the embedded data is corrupted or the binary was built with an incompatible version.

**Fix:**
1. Rebuild with `cargo build --release` to regenerate the binary with fresh BPE data
2. If the issue persists, run `cargo update` to get the latest `tiktoken-rs` patch, then rebuild
3. Check file system permissions — the binary needs read access to its own mapped memory (no external files needed)

---

### "Request too large" error

**Symptom:**
```json
{
    "error": {
        "code": -32600,
        "message": "Request too large (limit: 16777216 bytes)"
    }
}
```

**Cause:** You sent a JSON-RPC request line larger than 16 MB. This is usually caused by:
- Base64-encoding a very large file into the `filePath` parameter
- Accidentally piping a large file directly into stdin

**Fix:**
- Pass file paths, not file contents, to `compress_code_context`
- For files above `resource_limits.max_file_size_bytes`, narrow or split the
  source; do not rely on a removed workspace-compression tool.

---

### "unknown fidelity '...' " error

**Symptom:**
```json
{
    "error": {
        "code": -32602,
        "message": "unknown fidelity 'hihg' (expected 'low', 'medium', or 'high')"
    }
}
```

**Cause:** A typo in the `fidelity` parameter. Only `"low"`, `"medium"`, and `"high"` are accepted (case-insensitive, so `"HIGH"` works but `"hihg"` does not).

**Fix:** Correct the `fidelity` value in your tool call.

---

### File not compressed / "language not supported"

**Symptom:** `language_for_extension` returns `None`, or the output is empty.

**Cause:** Clean-CTX currently supports `.ts`, `.js`, and `.cs` files only. Other extensions are rejected.

**Temporary workaround:**
- Rename the file to a supported extension (not recommended — parsing will likely fail)
- Or extend the tool: see [`DEVELOPER_DOCUMENTATION.md`](DEVELOPER_DOCUMENTATION.md) for adding a new language

---

### `diff_code_context` always returns "no baseline stored"

**Symptom:** Every call returns `"No baseline stored — use the tool twice to see changes"`

**Cause 1:** The session state is reset between MCP server connections. Each connection gets a fresh `LocalStateCache`.

**Fix:** Call `diff_code_context` twice within the same session.

**Cause 2:** The file path changed between calls. The cache key includes the absolute path, so `/src/file.ts` and `C:\project\src\file.ts` are different keys.

**Fix:** Use consistent, absolute paths for all calls.

---

### Historical internal workspace compression is slow on large repos

This section applies only to internal/legacy workspace-compression code. There
is no public `compress_workspace` MCP tool in the current surface.

**Symptom:** `compress_workspace` takes >30 seconds on a repository with 5,000+ files.

**Cause:** Tree-sitter parser instantiation per file. The workspace walker is currently single-threaded.

**Mitigations:**
1. Use `exclude_patterns` in `.clean-ctx.json` to skip `node_modules`, `dist`, `build/`, etc.
2. Compress only the subdirectories you need (pass a more specific `directoryPath`)
3. Avoid symlink loops — the walker handles them, but extra canonicalization adds overhead

For current broad discovery, use CBM graph tools; request file context only for
the returned authoritative paths.

---

### Symlink loop error in workspace scan

**Symptom:** Workspace scan hangs or takes excessively long.

**Cause:** Circular symlinks in the directory tree.

**Fix:** Clean-CTX 0.1.0 includes symlink-loop detection (F-17) via canonical-path tracking with a `MAX_WALK_DEPTH` of 32. If you encounter a loop, verify that the detection is working correctly:

```bash
# Test symlink loop detection
cargo test collect_source_files_survives_symlink_loop
```

If the test passes, the loop detection is functioning. The server will not crash, but may process fewer files than expected.

---

### Binary crashes with SIGABRT / "assertion failed"

**Symptom:** The MCP server process terminates unexpectedly without a JSON-RPC error response.

**Checklist:**
1. Are you using the correct version of the binary? Run `clean-ctx --version` (or check the binary's build date)
2. Does the startup log appear? If not, the server failed before the BPE init check
3. Are you piping binary data into stdin? The JSON-RPC parser expects UTF-8 only
4. Is the binary from the correct Rust toolchain? Edition 2024 requires Rust 1.85+

**If the crash is reproducible:**
```bash
# Run with RUST_BACKTRACE=1 to get a stack trace
RUST_BACKTRACE=1 clean-ctx.exe < test_input.json
```

Report the crash with the stack trace and reproduction steps.

---

### Config changes not taking effect

**Symptom:** You edited `.clean-ctx.json` but the server still uses the old settings.

**Cause:** The config is loaded once at server startup and cached in a `OnceLock`. The server has no file-watch hot-reload.

**Fix:** Restart the MCP server (restart your IDE or the MCP host process).

---

### Graph queries return "project not found or not indexed"

**Symptom:** Raw `cbm_proxy` calls fail with:

```text
e:project not found or not indexed
hint:Use list_projects to see all indexed projects, then pass the project name.
```

**Cause:** The `project` value doesn't match CBM's canonical project ID. CBM derives project IDs from the **canonical repository path**, never the directory name. `RustContextLayerAI` (a directory basename) is **not** a valid project ID — the real ID for `C:\Users\MNasty\Desktop\RustContextLayerAI` is the canonical slug:

```text
C:/Users/MNasty/Desktop/RustContextLayerAI  →  C-Users-MNasty-Desktop-RustContextLayerAI
```

**Fix:**
1. Call `list_projects` when you need to inspect the exact IDs CBM knows.
2. For structured wrappers, pass the repository path or canonical project ID;
   their resolver normalizes the target. For `cbm_proxy`, `project` resolution
   is scoped to that call and does not change the bridge's active project.

**Note — two kinds of proxy calls:**
- **Project-independent** (`list_projects`, `get_cbm_status`): need no project, never gated on indexing state.
- **Project-targeted** (`search_graph`, `query_graph`, `trace_path`,
  `get_architecture`): need a project. Structured wrappers may select the
  bridge's active project. `cbm_proxy` resolves an explicit path/slug for one
  call only and does not mutate that active selection.

---

### Port conflict when running locally

**Symptom:** Error like "address in use" or "port already bound".

**Cause:** The core MCP server uses stdio, but the optional
`clean-ctx-proxy` is an HTTP listener (port 8787 by default). A port conflict
usually means a proxy instance already owns the configured port.

**Fix:** If you need only MCP, run `clean-ctx` with proxy auto-start disabled.
If you need the proxy, inspect the configured `proxy.port`; Clean-CTX may adopt
an already healthy proxy rather than spawn a second child.

---

## Diagnostic Commands

```bash
# Verify the binary starts correctly (ctrl-c after startup message)
echo '{}' | clean-ctx.exe

# Check the Rust version
rustc --version

# Verify all tests pass
cargo test

# Run the linter
cargo clippy --all-targets -- -D warnings

# Check for outdated dependencies
cargo outdated

# Check for security advisories
cargo audit

# Verify symlink loop detection
cargo test collect_source_files_survives_symlink_loop

# Verify file size guard
cargo test compress_file_rejects_file_larger_than_max

# Verify fidelity validation
cargo test parse_typo_rejected
```

---

## Getting Help

If none of the above resolves your issue:

1. Check the [Architecture Overview](ARCHITECTURE_OVERVIEW.md) for system design context
2. Check the [Changelog](changelogs/CHANGELOG.md) for known edge cases and their fixes
3. Check the [Developer Documentation](DEVELOPER_DOCUMENTATION.md) for build and test instructions
4. Open an issue with:
   - Binary version (build date or commit hash)
   - Operating system and Rust version
   - Full error output (including any `RUST_BACKTRACE`)
   - Steps to reproduce
   - Input data (redacted if necessary)

---

### CBM Cypher aggregation limitations

**Symptom:** CBM 0.8.1's Cypher engine does not support aggregation functions like COUNT, GROUP BY, SUM, or AVG. Queries using these functions return empty or error results.

**Cause:** CBM uses a limited Cypher subset for graph queries. Aggregation is a Neo4j Cypher feature not present in CBM 0.8.1.

**Resolution:** This is an upstream CBM limitation and not a Clean-CTX bug. Clean-CTX does not require aggregation — all production queries filter by specific node properties and return individual rows. If you need summary data, paginate through results client-side.

**Also applies to:** MATCH (n) RETURN n, count(*) (and similar aggregate patterns) — use RETURN n LIMIT N instead.

### Blast radius / caller lists look wrong or too large

**Symptom:** `get_blast_radius` (or a raw CALLS query) returns callers for the entire project instead of one symbol's callers.

**Cause:** Historical bug - the Cypher filtered on an undeclared variable (`m.name`). CBM does not reject invalid WHERE clauses; it fail-opens and returns every matching row. Fixed in the 2026-08-24 audit (`f.name`); a live regression test pins the result to the exact ground-truth caller set.

### Dead code misses class methods

**Symptom:** Dead-code output lists only free functions; dead class methods never appear.

**Cause:** Pre-audit implementation scanned only `:Function` nodes. The current implementation scans `Function` AND `Method` labels and merges results - covered by a live set-equality test in `src/tests/cbm/graph_intel.rs`.

### DATAFLOW queries return nothing

**Symptom:** Cypher over `DATAFLOW` edges yields empty results.

**Cause:** Expected behavior - CBM 0.8.1 has no DATAFLOW edge type, and its USAGE/WRITES edges are not read-aware equivalents. Clean-CTX ships no dataflow enrichment; local program-graph `DataFlowRead`/`DataFlowWrite` edges are independent of CBM. A guard test fails automatically if a future CBM introduces DATAFLOW edges.

### .razor symbols missing from graph queries

**Symptom:** Searches for Razor components/views return nothing even though the repository contains `.razor` files.

**Cause:** Verified upstream limitation - CBM 0.8.1 creates no Razor nodes. There is nothing for Clean-CTX to configure; track upstream.
