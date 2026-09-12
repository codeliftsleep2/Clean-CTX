---

## [0.4.9] - 2026-08-28
### Fixed

- Edit-fidelity `raw_passthrough` no longer prevents subsequent `apply_edit`
  operations by failing to establish tracked IR state.
- `§PATHMAP` footers for single-file context responses are now scoped to
  aliases actually required by that response instead of serializing the
  entire session-wide path dictionary.
- Working-tree `diff_commits` now includes non-ignored untracked files
  without requiring them to be staged or added with `git add -N`.
- Fixed pre-existing `clippy::manual_range_contains` warning in
  `src/tests/mcp/token_economics.rs`.

### Added

- **`diff_commits` working-tree diff now includes non-ignored untracked files.**
  When `toRef` is omitted, `collect_changed_files()` additionally runs
  `git ls-files --others --exclude-standard` and treats discovered paths as
  `FileChange::Added`, entering the existing diff pipeline unchanged. Ignored
  files remain excluded. No index mutation, no staging, no `git add -N`
  required. (`src/gitdiff/workspace.rs`)

### Tests

- **Workspace-level untracked-file tests:** Discovery in working-tree diff,
  exclusion from commit-to-commit diff, coexistence with tracked modifications,
  .gitignore exclusion, no-tracked-changes edge case.
- **Engine-level untracked-file tests:** Untracked `.ts` file produces AST
  skeleton in manifest; untracked non-compressible file falls back to line-count
  delta; ignored files excluded from manifest; mixed tracked + untracked changes
  counted correctly.
- Replaced the old `git add -N` dependency test
  (`gitdiff_workspace_working_tree_added_file_read_from_disk`) with a no-staging
  untracked variant.
- **Regression test for Edit-fidelity raw-passthrough:**
  `apply_edit_works_after_raw_passthrough_on_small_file` proves small Edit files
  remain editable after the token-economics gate returns raw passthrough.
  Applies to both compressed and raw-passthrough paths.
- **CBM E2E test correction:**
  `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` now calls
  `ensure_indexed()` before `bridge.search()` to respect the architectural
  layering — the MCP handler owns the freshness precondition, not the low-level
  `search()` primitive. No production CBM code was modified.
  (`src/tests/cbm/e2e.rs`)
- **PATHMAP scoping tests:** 6 unit tests in `src/tests/dictionary/path.rs` and 3
  integration tests in `src/tests/mcp/tool_handlers.rs` verify scoped footers,
  no cross-request leakage, and full-footer retention.

### Documentation

- Removed the `git add -N` limitation from `docs/DIFF_COMMITS_GUIDE.md`.
- Documented the new untracked-file behavior in `docs/DIFF_COMMITS_GUIDE.md`.

---


