---

## [0.4.7] - 2026-08-28 - Automatic CBM Reindex after apply_edit & Schema Documentation

### Added

- **Automatic CBM reindex after `apply_edit`.** After a successful filesystem mutation, Clean-CTX now synchronously performs a CBM `fast` `index_repository` when CBM is available. The reindex runs after the commit lock is released, so it never blocks the filesystem mutation. The edit remains successful even if reindexing fails (failure is logged as a warning only). See `src/cbm/bridge.rs` (`reindex_for_file`), `src/mcp/tool_handlers/edit.rs` (post-commit reindex wiring). (`src/cbm/bridge.rs`, `src/mcp/tool_handlers/edit.rs`)

- **`list_projects` MCP tool.** Lists all CBM-indexed projects with their canonical identity, path, and status. Project-independent — no project parameter required. This is the authoritative mechanism for discovering the CBM project slug. (`src/cbm/tools.rs`, `src/mcp/tools.rs`)

- **`index_repository` MCP tool.** Triggers CBM to index (or reindex) a repository. Accepts `repo_path` (required) and `mode` (`"fast"` for normal refresh, `"full"` for rebuild/recovery). Agents should not normally need this after `apply_edit` (automatic reindex covers it), but it is available for external edits performed outside Clean-CTX. (`src/cbm/tools.rs`, `src/mcp/tools.rs`)

### Changed

- **CBM schema updated.** The `cbm_proxy` tool's `cbm_tool` parameter now documents six agent-facing CBM operations: `search_graph`, `query_graph`, `trace_path`, `get_architecture`, `list_projects`, `index_repository`. The `get_symbol_importance` and `get_dead_code` exclusions are preserved. (`src/cbm/tools.rs`)

- **Agent documentation refreshed.** `docs/agent/tooling.md` now includes: `list_projects`/`index_repository` in the CBM tool table, a project discovery workflow, index repository modes, automatic reindex after `apply_edit` with workflow diagrams, and the external-edit limitation. Proxy aliases (`graph_search` → `search_graph`, etc.) are documented as compatibility aliases, not preferred names. (`docs/agent/tooling.md`)

- **Integration rules updated.** `docs/CLAUDE_INTEGRATION_RULES.md` (RULE 2) lists all six forbidden direct calls, their corresponding `cbm_proxy` equivalents, and the Quick Reference includes automatic reindex and external-edit guidance. (`docs/CLAUDE_INTEGRATION_RULES.md`)

- **Architecture overview clarified.** `docs/ARCHITECTURE_OVERVIEW.md` now explicitly distinguishes the six agent-facing CBM operations from internal bridge helpers (`get_symbol_importance_mut`, `get_blast_radius`, `get_dead_code`, `get_call_edges`). (`docs/ARCHITECTURE_OVERVIEW.md`)

### Tests

- 4 regression tests for `reindex_for_file` behavior (CBM unavailable, extra-root resolution, fallback to active root, cache invalidation). (`src/tests/cbm/regression.rs`)
- 1 end-to-end test `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` proving the real production `handle_apply_edit` path triggers CBM reindexing. (`src/tests/cbm/e2e.rs`)
- Schema parity test `p3_21_tool_names_match_tool_list_and_registry` updated to include `list_projects` and `index_repository`. (`src/tests/mcp/tools.rs`)

### Verification

- CBM regression: 54 passed, 0 failed.
- CBM E2E: 19 passed, 0 failed.
- apply_edit: 3 passed, 0 failed, 3 pre-existing ignored.
- `cargo check --lib` clean.
- `cargo clippy --lib -- -D warnings` zero warnings.

---

### Fixed

- **A C# constructor initializer clause was mis-selected as the parameter group.** `find_method_params` returned the LAST balanced depth-0 paren group, so for `Greeter(string prefix) : base(prefix)` — and every `: base(...)` / `: this(...)` constructor — the initializer's OWN parentheses won the selection: Low rendered the constructor as `base(prefix)`, Medium welded the clause onto the label as a fake return-type annotation after space-collapse (`Greeter(string prefix):base(prefix)`), `method_key` grouped every base-initializing constructor under `base` (distinct overloads merged into one diff entry), the IR compiler named the constructor `base` and fed the initializer argument through as a parameter, the diff body fingerprint keyed off the initializer's own parens (initializer-only edits were invisible to the differ), and the class primary-constructor peel guard peeled a base call's `(Value)` because its own tail was "empty". The parameter list is now the FIRST balanced depth-0 paren group ANCHORED TO THE DECLARED NAME — its opening `(` immediately preceded (after whitespace) by `>` (a generic-type close) or by an identifier that is neither `base` nor `this` — scanned literal-aware via the shared `skip_quoted_literal` so a default value like `void M(string s = "a (", int n)` cannot break it (`src/compaction/method.rs`). New shared `strip_base_initializer_clause` drops the `: base(...)` / `: this(...)` call-site clause from the header before Medium compaction and before `parse_method_sig` name/params/return-type derivation (`src/compaction/method.rs`, `src/ir/pipeline.rs`).

- **Behavior preservation.** Tuple-return signatures still select the method's own group (the tuple's top-level `(` is preceded by `<` and skipped); every non-constructor signature is byte-identical; High-fidelity output remains byte-exact including the full initializer clause.

- **Deliberate Medium-tier revision.** The ff2a29a Medium expectation was revised by design: the compacted label now DROPS the initializer clause (call-site metadata, not a return annotation) instead of carrying the interpolation holes, while High keeps the byte-exact header. The ff2a29a truncation fix itself is untouched.

### Tests

- Name-anchored selection RED→GREEN: base-initializer (`Greeter(string prefix) : base(prefix)` → `(string prefix)`), this-initializer, literal-paren default value (`void M(string s = "a (", int n)` → full group), generic-`>` anchor (control), generic-`new()` constraint exclusion, and the existing tuple-return test unchanged.
- Tier and consumer regressions RED→GREEN: Low label `Greeter(prefix)`, Medium label `Greeter(string prefix)`, `method_key("Greeter(string prefix) : base(prefix)")` → `Greeter`, IR identity (constructor compiles named `Greeter` with `string prefix` parameter and void return), class peel guard leaves `Example(string Value) : Base(Value)` untouched, and the body-fingerprint test fails on initializer-only edits.
- Revised ff2a29a pair: unit `high_fidelity_base_initializer_interpolation_keeps_full_header` (High byte-exact including the clause; Medium = bare declaration) and e2e `gitdiff_ctor_base_initializer_interpolation_not_truncated` (bare label `ExampleException(string value,object context)`; `{value}`/`{context}` absent; body statements absent).

### Verification

- compaction 95 passed; diff 80 passed (2 feature-ignored); gitdiff 42 passed; ir::compiler 20 passed; ir::pipeline 12 passed; `cargo test encoding` 6 passed; `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` (default and `--features rust`) 0 warnings; UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake).

---

