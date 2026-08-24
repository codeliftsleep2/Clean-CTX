# Clean-CTX — Troubleshooting Guide

> **Owner:** Problem-solving + error codes + diagnostic commands · **Status:** Living reference

**Last updated:** 2026-08-24

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
- For very large files (>=10 MB), the server will also return a `FileTooLarge` error — split the file or use `compress_workspace` instead

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

### Workspace compression is slow on large repos

**Symptom:** `compress_workspace` takes >30 seconds on a repository with 5,000+ files.

**Cause:** Tree-sitter parser instantiation per file. The workspace walker is currently single-threaded.

**Mitigations:**
1. Use `exclude_patterns` in `.clean-ctx.json` to skip `node_modules`, `dist`, `build/`, etc.
2. Compress only the subdirectories you need (pass a more specific `directoryPath`)
3. Avoid symlink loops — the walker handles them, but extra canonicalization adds overhead

**Planned improvement:** Streaming workspace walk (F-19) and rayon parallelization (F-20) will address this in a future release.

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
1. Call `list_projects` to list the exact IDs CBM knows (it is project-independent and always works)
2. Pass that exact slug via `parameters.project`, or pass the repository path via `arguments.workspaceRoot` / `arguments.project` and let Clean-CTX resolve it to the canonical slug

**Note — two kinds of proxy calls:**
- **Project-independent** (`list_projects`, `get_cbm_status`): need no project, never gated on indexing state.
- **Project-targeted** (`search_graph`, `query_graph`, `trace_path`, `get_architecture`): need a project. The built-in wrappers resolve the active workspace root automatically; raw `cbm_proxy` calls without an explicit project are forwarded unchanged and CBM rejects them with the error above.

---

### Port conflict when running locally

**Symptom:** Error like "address in use" or "port already bound".

**Cause:** Clean-CTX uses stdio only — there is no port, no HTTP server, and no network listener. If you see a port-related error, another process is interfering.

**Fix:** Ensure you are running `clean-ctx.exe` directly (not through a wrapper that adds a network layer). The binary should receive JSON-RPC on stdin and write responses to stdout only.

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
2. Check the [Changelog](CHANGELOG.md) for known edge cases and their fixes
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
