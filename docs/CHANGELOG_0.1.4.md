---

## [0.1.4] — 2026-06-08

### Added

#### Track C — Phi Marker Grammar Centralisation (F-ANG-06)
- `src/angular_meta/markers.rs` — `PhiLineKind` enum (single source of truth for all 14 marker kinds), `PhiLine` trait with per-variant struct impls (`ComponentLine`, `ServiceLine`, `ModuleLine`, `DirectiveLine`, `PipeLine`, `InputLine`, `OutputLine`, `ModelLine`, `InjectsLine`), generic `expand_phi_in_line` / `expand_phi` loops replacing 3 scattered tables
- Adding a new marker is now a 1-step change (add `PhiLineKind` variant + impl) instead of 3-step (builder + two tables)
- 3 new structural tests: `phi_line_kind_uniqueness`, `phi_vocab_is_bijective`, `phi_line_round_trip`

#### Track D — God-function split + extract_class_blocks rewrite (F-ANG-15, F-ANG-03, F-ANG-20)
- `src/mcp/workspace.rs` — `compress_workspace_dir` decomposed into 5 focused helpers: `format_manifest_header`, `compress_pass`, `bundle_pass`, `graph_pass`, `format_manifest_footer`, with `PassContext` struct for shared state
- `src/mcp/workspace.rs` — `extract_class_blocks` rewritten from 137-line duplicate state machine to ~20-line driver delegating to Track A-promoted `decorators::find_class_body_open` + `decorators::find_matching_brace`, with new `find_decorator_start` handling `export`/`abstract`/`default`/`declare` modifier keywords
- 5 new tests: `extract_class_blocks_does_not_panic_on_unclosed_body`, `extract_class_blocks_handles_empty_input`, `compress_pass_emits_per_file_section`, `bundle_pass_emits_phi_bundle_and_footer`, `graph_pass_emits_phi_graph_section`

### Fixed
- `src/angular_meta/decorators.rs` — `consume_call_expression` termination bug: after the closing `)` brought depth to 0, the loop continued scanning past the decorator boundary because `depth == 0` was checked at loop top, not immediately after decrement. Fixed by returning immediately when depth reaches 0. This was a pre-existing bug that prevented decorator args from being extracted when the decorator call was followed by class body content.
- `src/mcp/workspace.rs` — `find_decorator_start` now scans backwards through decorator names (`Injectable`, `Component`, etc.) to find `@`, not just checking the character immediately before `(`

### Test count
- 301/301 tests pass (5 new from Tracks C+D)
- 0 clippy warnings (`cargo clippy --all-targets -- -D warnings`)

---

