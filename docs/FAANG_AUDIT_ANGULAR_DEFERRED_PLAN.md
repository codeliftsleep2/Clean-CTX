# Clean-CTX — Angular Meta-Layer Deferred-Work Plan

**Source:** `docs/FAANG_AUDIT_ANGULAR.md` (2026-06-07)
**Audit state at planning time:** 20 of 23 findings fixed · 283/283 tests pass · 0 clippy warnings
**Deferred findings:** 11 (F-ANG-05, 06, 07, 08, 09, 11, 12, 13, 15, 16, 20)
**This plan:** 9 of 11 fixed across 4 tracks; 2 punted with rationale

**Status (updated 2026-06-08):** Track A ✅ **complete** (2026-06-07). Track B ✅ **complete** (2026-06-08). Track C ✅ **complete** (2026-06-08). Track D ✅ **complete** (2026-06-08). 10 of 11 deferred findings resolved (F-ANG-03, 05, 06, 07, 08, 09, 12, 13, 15, 20). 301/301 tests pass, 0 clippy warnings. Punted: F-ANG-11, F-ANG-16.

---

## Executive Summary

The 11 deferred findings cluster into 4 natural refactor groups. Tracks are ordered by **risk-reduction-per-engineering-hour** and have a hard dependency only between Track A and Track D (Track D uses helpers that Track A promotes to `pub(crate)`).

| Track | Findings | Effort | Risk | Status |
|-------|----------|--------|------|--------|
| **A** | Honest types for string walkers (F-ANG-07/08/09/12/13) | 0.5 d | Low | ✅ **Complete (2026-06-07)** — 5 findings fixed, 8 new tests, 291/291 pass |
| **B** | `AngularGraph` typestate (F-ANG-05) | 0.5 d | Medium | ✅ **Complete (2026-06-08)** — F-ANG-05 fixed, 2 new tests, 293/293 pass |
| **C** | `Φ` marker grammar centralisation (F-ANG-06) | 1.0 d | Low | ✅ **Complete (2026-06-08)** — F-ANG-06 fixed, 3 new tests, 296/296 pass |
| **D** | God-function split (F-ANG-15) + `extract_class_blocks` rewrite (F-ANG-03) + insertion-order iteration (F-ANG-20) | 1.5 d | Medium | ✅ **Complete (2026-06-08)** — 3 findings fixed, 5 new tests, 301/301 pass |
| **Total** | **10 findings fixed** | **3.5 d** | — | 10/10 done |
| — | F-ANG-11 (deferred: syscall is cheap) | — | — | ⏭ Skip |
| — | F-ANG-16 (deferred: rayon + tree-sitter `Send`) | — | — | ⏭ Follow-up PR |

---

## Track A — Honest types for the string walkers (✅ Complete)

**Findings:** F-ANG-07, F-ANG-08, F-ANG-09, F-ANG-12, F-ANG-13
**Files touched:** `src/angular_meta/decorators.rs`
**Effort:** 0.5 day
**Risk:** Low — pure refactor; existing 162 angular tests already exercise the truncated-input paths
**Completion date:** 2026-06-07

### What changed

| Finding | Function | Before | After |
|---|---|---|---|
| F-ANG-08 | `find_matching_brace` | `-> usize` (returned `len-1` on no-match) | `-> Option<usize>` (`None` on no-match), promoted to `pub(crate)` for Track D |
| F-ANG-09 | `consume_call_expression` | `-> (usize, String)` (sliced to end of text on EOF) | `-> Option<(usize, String)>` (`None` on unterminated) |
| F-ANG-12 | `find_class_head_end` | `-> usize` (returned `raw.len()` on no-match) | `-> Option<usize>` (`None` on no-match) |
| F-ANG-13 | `extract_class_name` | `-> String` (returned `"(anonymous)"` literal) | `-> Option<String>` (`None` on no-name; callers substitute `?`) |
| F-ANG-07 | `find_class_body_open` | already used `?`; promoted to `pub(crate)` for Track D | unchanged control flow, now exported for reuse |

### Caller updates (11 sites)

- **`extract_decorators`** — `?` for `find_class_head_end`, `.unwrap_or_else("?")` for class name, combined `if` chains with `&& let` for `find_matching_brace` (collapsed nested if).
- **`extract_graph_entries`** — same treatment as `extract_decorators`.
- **`collect_signal_fields`** — `consume_call_expression` returns `.map(|(_,arg)| arg).unwrap_or_default()`.
- **`collect_decorators`** / **`collect_field_decorators`** — `if let Some((consumed, arg_str)) = ... else { i += 1; }` to avoid infinite loops on unterminated decorator calls.
- **`extract_constructor_injects`** — `.map(|(_,p)| p).unwrap_or_default()` for unterminated constructor.

### New tests added (8)

- `find_matching_brace_returns_none_on_unclosed_body` (F-ANG-08)
- `consume_call_expression_returns_none_on_unterminated_call` (F-ANG-09)
- `find_class_head_end_returns_none_on_no_class_keyword` (F-ANG-12)
- `extract_class_name_returns_none_for_anonymous_class` (F-ANG-13)
- `find_class_body_open_returns_none_when_no_class_keyword` (F-ANG-07)
- `extract_decorators_substitutes_question_mark_for_anonymous_class` (F-ANG-13 e2e)
- `extract_decorators_returns_none_for_input_without_class_keyword` (F-ANG-12 e2e)
- `extract_decorators_handles_unterminated_decorator_call` (F-ANG-08/09 e2e, wrapped in `catch_unwind`)

### Process notes

- Initial implementation used `let-else` for the early returns. Clippy's `question_mark` lint flagged 3 cases where the enclosing function returns `Option` (prefers `?` over `let-else` when both apply). Reverted those; the behavioural change is identical.
- Clippy's `collapsible_if` lint flagged 2 nested `if let` blocks. Combined them with `&& let` chains (the new `let-chains` syntax, stable since Rust 1.88).
- Test count: 283 → **291** (8 new).

### Acceptance

- ✅ All 162 pre-existing angular tests still pass
- ✅ 8 new tests pass
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --all-targets` clean
- ✅ Behavior on success paths is byte-identical (no callers were observable-changed)
- ✅ `find_matching_brace` and `find_class_body_open` are now `pub(crate)` — **Track D unblocked**

---

## Track B — `AngularGraph` typestate (F-ANG-05)

**Findings:** F-ANG-05
**Files touched:** `src/angular_meta/graph.rs`, possibly `src/angular_meta/graph_state.rs`
**Effort:** 0.5 day
**Risk:** Medium — touches the `GraphCollector` API (1 call site in `src/mcp/workspace.rs`)

### Why this track

The `resolved: bool` flag in `graph.rs:84` is a real footgun:
- `register_class` (line 110) silently resets it to `false` after a `resolve_all` (line 152) was called.
- Nothing at the type level prevents a caller from registering *after* `resolve_all`, leaving the `injected_by` edges stale.
- The audit correctly identifies this as the only `🔴` (red) deferred finding.

Splitting into a builder + a resolved form makes the bug unrepresentable.

### Concrete changes

1. **New type** in `graph.rs`:
   ```rust
   pub struct AngularGraphBuilder {
       classes: HashMap<String, ClassEntry>,
       selectors: HashMap<String, String>,
   }
   impl AngularGraphBuilder {
       pub fn new() -> Self { ... }
       pub fn register_class(&mut self, ...) { ... } // moves out of AngularGraph
   }
   ```
2. **`AngularGraph::new()` becomes private** (or `pub(crate)`) — the only way to get an `AngularGraph` is via `AngularGraphBuilder::build(self) -> AngularGraph`, which takes `self` by value and calls `resolve_all` internally.
3. **`AngularGraph` public surface shrinks:**
   - **Drop:** `register_class` (moved), `resolve_all` (private step inside `build`).
   - **Keep:** `format_graph_line`, `format_graph_footer`, `class_names_by_kind`, `get_class`, `all_classes`, `resolve_inject_type`, `resolve_selector`, `is_resolved` (always `true` after `.build()` — can be deleted or kept for symmetry).
4. **`GraphCollector::build_graph`** (line 400) becomes:
   ```rust
   pub fn build_graph(&self) -> AngularGraph {
       let mut builder = AngularGraphBuilder::new();
       for entry in &self.entries {
           builder.register_class(...);
       }
       builder.build()
   }
   ```
5. **`AngularGraphHandle` (in `graph_state.rs`)** is unchanged — it wraps an `AngularGraph` (the resolved form), not a builder.

### New tests (≥2)

- `builder_consumes_self` — `AngularGraphBuilder::build(self)` moves; you cannot go back and call `register_class`.
- `graph_has_no_register_method` — compile-time check (or doc-test) that demonstrates `AngularGraph` has no `register_class` method.

**Acceptance:** all 162 angular tests pass; 2 new tests pass; `cargo doc` clean; `cargo clippy --all-targets -- -D warnings` clean.

---

## Track C — `Φ` marker grammar centralisation (F-ANG-06)

**Findings:** F-ANG-06
**Files touched:** `src/angular_meta/markers.rs`, all 9 call sites of the `build_*` functions
**Effort:** 1.0 day
**Risk:** Low — pure refactor; behavior is byte-identical

### Why this track

There are **9 hand-formatted marker builders** in `markers.rs:45-156`:
`build_component_line`, `build_service_line`, `build_module_line`, `build_directive_line`, `build_pipe_line`, `build_input_line`, `build_output_line`, `build_model_line`, `build_injects_line`.

Adding a new marker requires updating:
- the builder function,
- the `expand_phi_in_line` table (line 185),
- the `expand_phi` table (line 211).

That's three places to keep in lockstep. Centralising into a `PhiLine` trait + per-variant impls collapses it to one.

### Concrete changes

1. **Define an enum of marker kinds** (the single source of truth for the marker vocabulary):
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   pub enum PhiLineKind {
       Component, Service, Module, Directive, Pipe,
       Input, Output, Model, Injects,
       Graph, Bundle, Map,
   }
   ```
2. **Define a `PhiLine` trait:**
   ```rust
   pub trait PhiLine {
       fn kind(&self) -> PhiLineKind;
       fn render(&self) -> String;
   }
   ```
3. **One impl per marker type** (`ComponentLine`, `ServiceLine`, …). Each impl owns its own formatting.
4. **Centralised `static PHI_VOCAB: phf::Map<&'static str, &'static str>`** (using the `phf` crate) for the 13 expansion pairs. The two duplicate tables in `expand_phi_in_line` (line 185) and `expand_phi` (line 211) collapse to one source. If adding a `phf` dependency is undesirable, a plain `match` is fine.
5. **Keep the free-function builders as thin wrappers** for back-compat (or update the 9 call sites; the audit notes 9 call sites are well-tested, so the refactor is mechanical).

### New tests (≥3)

- `phi_line_kind_uniqueness` — every `PhiLineKind` variant has exactly one renderer.
- `phi_vocab_is_bijective` — the expansion table is a bijection (no two markers expand to the same string).
- `phi_line_round_trip` — for each impl, `render() → expand_phi_in_line` is lossless.

**Acceptance:** all 162 angular tests pass; 3 new tests pass; adding a new marker is now a 1-step change (add a `PhiLineKind` variant + an impl) instead of a 3-step change.

**Completion date:** 2026-06-08

### What changed

| Component | Before | After |
|---|---|---|
| Marker vocabulary | Scattered across 3 tables | Single source of truth: `PhiLineKind` enum |
| Marker rendering | Inline in each `build_*` function | `PhiLine` trait + 9 struct impls |
| `expand_phi_in_line` | 14-entry pair table | Generic loop over `PhiLineKind::all_in_expand_order()` |
| `expand_phi` | 14-arm match block | `PhiLineKind::from_token().map(expansion)` |
| `build_*` functions | Full formatting logic | Thin wrappers: create struct + `.render()` |
| New marker cost | 3 steps | 1 step (add variant + impl) |

### New tests added (3)

- `phi_line_kind_uniqueness` -- all 14 variants have unique prefixes and expansions
- `phi_vocab_is_bijective` -- expansion table is a bijection
- `phi_line_round_trip` -- render then expand is lossless for all 9 builder impls

### Process notes

- `phf` crate rejected as unnecessary dependency; plain match suffices for 14 entries.
- All 9 existing marker tests pass unmodified, confirming byte-identical output.
- Test count: 293 to **296** (3 new).

### Acceptance

- All 293 pre-existing tests still pass (296 total)
- 3 new structural tests pass
- `cargo clippy --all-targets -- -D warnings` clean (0 warnings)
- Behavior on all success paths is byte-identical
- Adding a new marker is now a 1-step change instead of 3

---

## Track D — God-function split + `extract_class_blocks` rewrite + insertion-order iteration

**Findings:** F-ANG-15, F-ANG-03 (re-included from Phase 1), F-ANG-20
**Files touched:** `src/mcp/workspace.rs`, `src/angular_meta/graph.rs`
**Effort:** 1.5 days
**Risk:** Medium — 270-line function split, but each resulting function is unit-testable in isolation

### Why this track

`compress_workspace_dir` (line 54) mixes four distinct concerns:
1. File collection + exclusion
2. Per-file compression
3. File-triplet bundling + `§ΦMAP` emission
4. Cross-file graph + `§ΦGRAPH` emission

The audit's F-ANG-03 partial fix (caching file content) was deferred because the full refactor needs `find_class_body_open` / `find_matching_brace` to be `pub(crate)` (which **Track A has now provided**).

### Concrete changes

1. **`compress_workspace_dir`** (line 54) becomes a 30-line orchestrator:
   ```rust
   pub(crate) fn compress_workspace_dir(
       dir_path: &str,
       fidelity: Fidelity,
       state: &mut McpState,
   ) -> Result<WorkspaceResult, Box<dyn std::error::Error>> {
       let manifest = format_manifest_header(dir_path, fidelity, state);
       let context = compress_pass(dir_path, fidelity, state, &mut manifest)?;
       let footer_builder = bundle_pass(&context, state, &mut manifest);
       let graph = graph_pass(&context, state, &mut manifest);
       let angular_graph = state.angular_graph.set(graph.clone());
       format_manifest_footer(state, footer_builder, graph, manifest, excluded, errors)
   }
   ```
2. **`compress_pass`** (~60 lines) — file collection + exclusion + per-file compression + manifest emission for the per-file section.
3. **`bundle_pass`** (~50 lines) — file-triplet resolution + `ΦBUNDLE` + `§ΦMAP` emission. Returns a `FooterBuilder`.
4. **`graph_pass`** (~50 lines) — graph build + `§ΦGRAPH` emission. Returns an `AngularGraph`. **This is where F-ANG-03 lives.**
5. **F-ANG-03 — replace `extract_class_blocks`** (line 379-515) with a thin driver that delegates to the Track-A-promoted `decorators::find_class_body_open` + `decorators::find_matching_brace`. The duplicated state machine in `workspace.rs` (line 386-522) is deleted in favour of:
   ```rust
   fn extract_class_blocks(source: &str) -> Vec<String> {
       let mut blocks = Vec::new();
       let mut cursor = 0;
       while let Some(class_pos) = find_next_class_keyword(&source[cursor..]) {
           let abs = cursor + class_pos;
           if let Some(open) = find_class_body_open(&source[abs..]) {
               let abs_open = abs + open;
               if let Some(close) = find_matching_brace(source, abs_open) {
                   let block_start = find_block_start(&source[..abs]);
                   blocks.push(source[block_start..=close].to_string());
                   cursor = close + 1;
                   continue;
               }
           }
           cursor = abs + 6;
       }
       blocks
   }
   ```
6. **F-ANG-20 — insertion-order iteration in `all_classes`** (line 274-278):
   ```rust
   // before
   pub fn all_classes(&self) -> Vec<&ClassEntry> {
       let mut sorted: Vec<&ClassEntry> = self.classes.values().collect();
       sorted.sort_by(|a, b| a.class_name.cmp(&b.class_name));
       sorted
   }
   // after
   pub fn all_classes(&self) -> Vec<&ClassEntry> {
       self.classes.values().collect()
   }
   ```
   Note: this changes observable behavior — the order in the `§ΦGRAPH` footer will be insertion order, not sorted. The audit's existing test for "current behavior" needs to be updated to assert insertion order. This is a deliberate behavior change to match the doc-comment claim at line 273.

### New tests (≥4)

- `compress_pass_emits_per_file_section`
- `bundle_pass_emits_phi_bundle_and_footer`
- `graph_pass_emits_phi_graph_section`
- `extract_class_blocks_uses_decorators_helpers` — assert that a malformed class body (no closing brace) does not panic.
- `all_classes_returns_insertion_order`

**Acceptance:** all 162 angular tests pass + 4 new tests pass; `compress_workspace_dir` ≤ 30 lines; `extract_class_blocks` ≤ 20 lines.

---

## Skipped / punted findings

### F-ANG-11 — `resolve_triplet` calls `is_file()` twice per extension

**Decision:** ⏭ **Skip** — the audit itself notes: "the `is_file()` syscall is cheap (single `stat(2)`); batching would not yield meaningful speedup." The optimization is not worth touching `bundler.rs` for. No PR.

### F-ANG-16 — Parallelize workspace reads with rayon

**Decision:** ⏭ ** Pundle workspace reads with rayon

**Decision:** Punt to separate PR. The audit ties this to F-20 in the main audit. The blocker is tree-sitter's Parser not being Send by default; the fix requires a per-thread parser pool. The audit estimates this as a complex refactor that does not belong in a small follow-up. Recommended as a follow-up PR after Track D lands, with the god-function split making the parallelism surface much easier to reason about (the compress_pass boundary becomes the natural place to introduce par_iter).

---

## Suggested execution order

```
Week 1, Day 1 morning:   Track A (0.5d)  COMPLETE
Week 1, Day 1 afternoon: Track B (0.5d)  in parallel with Track C
Week 1, Day 2:           Track C (1.0d)  independent of A
Week 1, Day 3 + Day 4:   Track D (1.5d)  depends on A (now unblocked)
```

Each track is shippable as a single PR. Updated PR sequence (Track A already shipped):

1. PR #1 - Track A: "Honor None returns in decorators.rs string walkers" - SHIPPED 2026-06-07
2. PR #2 - Track B: "Typestate for AngularGraph (split builder/resolved)"
3. PR #3 - Track C: "Centralise the Phi marker grammar into a PhiLine trait"
4. PR #4 - Track D: "Decompose compress_workspace_dir and rewrite extract_class_blocks"

This ordering puts the smallest, most independent refactor (Track A) first, leaves room to reorder B and C independently, and saves the biggest, riskiest refactor (Track D) for last, when the type-safety and grammar-centralisation work is already merged.

---

## Acceptance gates (per track)

| Track | Gate | Verification command | Status |
|-------|------|----------------------|--------|
| A | cargo test lib >= 168 pass; 0 clippy warnings | cargo test --lib && cargo clippy --all-targets -- -D warnings | 291/291 pass, 0 warnings |
| B | AngularGraph typestate split; register_class no longer compiles | cargo test --lib && cargo doc --no-deps | ✅ 293/293 pass, 0 warnings |
| C | PhiLine trait; 1-step marker adds | cargo test --lib | ✅ 296/296 pass, 0 warnings |
| D | compress_workspace_dir decomposed; extract_class_blocks delegates to Track A helpers | cargo test --lib && cargo clippy --all-targets -- -D warnings | ✅ 301/301 pass, 0 warnings |

Final state after all 4 tracks: 10 of 11 deferred findings fixed (91%), 283 → 301 tests pass (+18), 0 clippy warnings, 2 findings deferred with documented rationale (F-ANG-11, F-ANG-16).

---

## Closing Notes

The deferred work is genuinely small in absolute terms: 3.5 engineer-days for 9 findings, all mechanical and well-scoped. The 2 punted findings (F-ANG-11, F-ANG-16) are explicitly not worth the engineering cost:

- F-ANG-11: the audit itself documents the syscall as cheap.
- F-ANG-16: blocked on tree-sitter's !Send Parser; a real fix needs a per-thread parser pool that the audit correctly identifies as out-of-scope for a small follow-up.

The plan respects the existing audit methodology (risk-reduction-per-engineering-hour, single-PR-per-track) and produces a clear before/after for every deferred finding.

**Track A landed cleanly on 2026-06-07** with all 5 of its targeted findings resolved, 8 new tests, 0 clippy warnings, and find_class_body_open / find_matching_brace promoted to pub(crate) so Track D's extract_class_blocks rewrite can proceed as planned.

- Plan authored 2026-06-07. Source: docs/FAANG_AUDIT_ANGULAR.md. Track A completion recorded 2026-06-07. Track B completion recorded 2026-06-08.

