# Angular Meta-Layer — Design & Marker Vocabulary

> **Owner:** Angular Meta-Layer design (Phases 1–4) · **Status:** Living per-layer reference (shipped)
>
> **Ship status:** see `docs/ROADMAP.md` (R-22 ✅). **Test counts / audit rounds:** see `docs/CHANGELOG.md`. **Ecosystem Deepening (RxJS/NgRx/Signals/Routing):** see `docs/ANGULAR_ECOSYSTEM_DEEPENING.md`.
>

---

## Decisions Locked

| Question              | Decision                                                                                              |
|-----------------------|-------------------------------------------------------------------------------------------------------|
| Compiler approach     | `tree-sitter-html` + custom Angular-syntax extractor (Option C). No Node dependency.                  |
| Marker approach       | `Φ`-prefixed markers  no new opcodes. Opcodes stay language-agnostic primitives.                      |
| Phasing               | Three independently shippable phases. Stop after any phase for a useful Meta-Layer.                   |
| Default state         | On  opt-out via `.clean-ctx.json`. Non-Angular files pay zero overhead.                                |
| Workspace scope       | Tier 1 (decorators) works in both modes. Tiers 2 & 3 are workspace-only.                                |
| New dependencies      | Phase 1: none. Phase 2: `tree-sitter-html`. Phase 3: none beyond Phase 2.                              |

---

## Notation Map

| Prefix         | Job                                        | Examples                                          |
|----------------|--------------------------------------------|---------------------------------------------------|
| `$xx`          | Opcodes — language primitives              | `$c` = class  `$ctor` = constructor  `$a` = async |
| `⊕`            | Behavior markers — control-flow annotations| `⊕guard`  `⊕loop`  `⊕⇒`  `⊕!`                    |
| `α / β / γ`    | Path aliases — file references             | `α7` = `/path/to/file.ts`                          |
| `Φ` (new)      | Framework-annotation markers               | `Φcmp:`  `Φsvc:`  `Φin:`  `Φout:`  `ΦBUNDLE`     |

> **Notation scope:** `$xx` opcodes and `⊕` markers are emitted by the LEGACY text compressor (`compress_workspace` manifests; `⊕` at Medium/High, `§` micro-codes at Low) and decoded by `decompress_code_context`. Interactive responses use SCHEMA v2 notation instead. The `Φ` framework vocabulary remains current.

---

## Phase 1 — Scaffold + Tier 1 (Decorators)

### Goal

Prove the Meta-Layer pattern end-to-end with the lowest-risk deliverable: TS-only  no new file types  no new runtime dependencies. This phase alone gives the LLM `@Component` / `@Injectable` / `@Input` / `@Output` context.

### Scope

| Action  | File                                              | Purpose                                                                 |
|---------|---------------------------------------------------|-------------------------------------------------------------------------|
| Create  | `src/angular_meta/mod.rs`                         | Public surface  `MetaBlock` struct  `run_meta_layer` entry point       |
| Create  | `src/angular_meta/detect.rs`                      | Angular detection heuristic (decorator presence + tsconfig hint)       |
| Create  | `src/angular_meta/decorators.rs`                  | `@Component` / `@Injectable` / `@NgModule` / `@Directive` / `@Pipe` / `@Input` / `@Output` extractor |
| Create  | `src/angular_meta/markers.rs`                     | `PhiMarker` enum + `build_phi` + `expand_phi` (mirrors `markers.rs` `⊕` shape) |
| Modify  | `src/queries.rs`                                  | Add `decorator` + `decorator_call` + `object` captures to `TS_QUERY`   |
| Modify  | `src/compaction/class.rs`                         | After `format_class_entry`  call into `angular_meta` and append Φ block|
| Modify  | `src/compression/pipeline.rs`                     | Pass `fidelity` to class compactor plumb `Option<MetaBlock>` through   |
| Modify  | `src/decompression/markers.rs`                    | Add `expand_phi_in_line` alongside existing `expand_markers_in_line`   |
| Modify  | `src/mcp/prompts.rs`                              | New "Framework Meta Markers" section in `SYSTEM_PROMPT`                 |
| Modify  | `src/config.rs`                                   | Add `meta_layers: BTreeMap<String  MetaLayerConfig>` schema             |
| Create  | `src/tests/angular_meta/detect.rs`                | Detector unit tests (positive + negative)                               |
| Create  | `src/tests/angular_meta/decorators.rs`            | Decorator extraction tests (one per Angular decorator type)             |
| Create  | `src/tests/angular_meta/markers.rs`               | `Φ` marker round-trip tests                                             |
| Create  | `src/test_files/angular/user-card.component.ts`   | Test fixture: real Angular component                                    |
| Create  | `src/test_files/angular/user.service.ts`          | Test fixture: `@Injectable` service                                     |
| Modify  | `docs/ARCHITECTURE_OVERVIEW.md`                   | Add Meta-Layer section + module tree                                    |
| Modify  | `docs/ROADMAP.md`                                 | Promote R-22 (Meta-Layer) to 🚧 in-progress                             |

### Completion Criteria — Phase 1

**Phase 1 is ✅ COMPLETE.** All criteria verified on 2026-06-07.

You will know Phase 1 is complete when **all** of the following are true:

**Functional**
- A `.ts` file with `@Component({selector: 'app-user-card'  templateUrl: './user-card.component.html'  ...})` produces a `// --- Φ Angular Meta ---` block below the existing compacted class.
- The block contains a `Φcmp:<ClassName>` line with `sel=`  `tpl=`  `sty=` attributes.
- `@Input()` and `@Output()` fields emit `Φin:` / `Φout:` markers.
- `@Injectable({providedIn: 'root'})` emits a `Φsvc:<ClassName> scope=root` line.
- `@NgModule({...})` emits a `Φmod:<ClassName> decl=[…] imp=[…] exp=[…]` line.
- `@Directive` / `@Pipe` emit `Φdir:` / `Φpipe:` lines.
- Constructor parameters with `private` / `protected` emit `Φinjects:[<Type>]` (unresolved at this phase — class names only  no `α` aliases yet).

**Non-regression**
- A non-Angular `.ts` file produces **zero** Φ markers and **zero** newlines of overhead.
- All 121 existing tests still pass.
- `cargo clippy --all-targets -- -D warnings` is clean.
- `compress_code_context` on a non-Angular file produces byte-identical output to v0.2.0.

**Round-trip**
- `decompress_code_context` on a compressed Angular file expands `Φcmp:` → `@Component`  `Φsvc:` → `@Injectable`  etc.
- The expanded output is human-readable and preserves all original class names.

**LLM discoverability**
- `SYSTEM_PROMPT` includes a new "Framework Meta Markers" section that teaches the LLM the `Φ` vocabulary.
- The section appears below the existing "Behavior Markers" section so the layering reads naturally: opcodes → ⊕ markers → Φ markers.

**Tests**
- New unit tests: detector (positive + negative)  decorator extraction (each decorator type)  marker round-trip.
- At least 6 new test files (1 per Angular decorator type + integration).
- All tests pass.

**Documentation**
- `docs/ANGULAR_META_LAYER.md` (this document) exists.
- `docs/ARCHITECTURE_OVERVIEW.md` updated with the new `angular_meta/` module.
- `docs/ROADMAP.md` R-22 entry marked 🚧 in-progress.

**Effort:** ~3.5 days (actual). **Risk:** Low (additive  no existing API changes  zero new deps).

### Completion Evidence

| Criterion | Status | Proof |
|-----------|--------|-------|
| `Φcmp:<ClassName>` with `sel=`  `tpl=`  `sty=` | ✅ | `decorators.rs` + `markers.rs` tests |
| `Φin:` / `Φout:` field markers | ✅ | `extracts_input_and_output_decorators` test |
| `Φsvc:<ClassName> scope=root` | ✅ | `extracts_injectable_decorator` test |
| `Φmod:<ClassName> decl=[…] imp=[…] exp=[…]` | ✅ | `extracts_ngmodule_decorator` test |
| `Φdir:` / `Φpipe:` | ✅ | `extracts_directive_decorator` / `extracts_pipe_decorator` tests |
| `Φinjects:[<Type>]` | ✅ | `extracts_constructor_injects` test |
| Non-Angular files: zero Φ markers | ✅ | `meta_layer_returns_none_for_plain_typescript` test |
| All 172 existing tests pass | ✅ | `cargo test` |
| `cargo clippy` clean | ✅ | `cargo clippy --all-targets -- -D warnings` |
| Round-trip `expand_phi_in_line` | ✅ | `expand_phi_in_line_rewrites_*` tests |
| `SYSTEM_PROMPT` has "Framework Meta Markers" | ✅ | `mcp/prompts.rs` |
| `docs/ARCHITECTURE_OVERVIEW.md` updated | ✅ | Meta-Layer section added |
| `docs/ROADMAP.md` R-22 marked ✅ | ✅ | R-22 added to Now list |

**Bugs found and fixed during Phase 1 implementation:**
1. `decorator_call_expression` is not a valid tree-sitter-typescript node type — removed from `TS_QUERY`
2. Byte literal escapes (`\t`  `\n`) were literal characters — fixed to proper escape sequences
3. `input_output_lines` moved before field-level scan — reordered collection order
4. `find_class_body_open` was matching decorator's `{` instead of class body — rewrote with depth tracking from `class` keyword
5. Clippy: collapsible match arm — collapsed `styles` field match
6. Clippy: doc overindented list items — fixed continuation line indentation

---

## Phase 2 — Tier 2 (File Triplet Bundling)

### Goal

Tell the LLM "these three files are one logical unit" without burning tokens on raw HTML/SCSS. Workspace-mode only.

### Scope

| Action  | File                                              | Purpose                                                                  |
|---------|---------------------------------------------------|--------------------------------------------------------------------------|
| Modify  | `Cargo.toml`                                      | Add `tree-sitter-html` (only new dependency)                             |
| Create  | `src/angular_meta/bundler.rs`                     | File-triplet resolver: `*.component.ts` → `*.{html scss css sass less}` siblings |
| Create  | `src/angular_meta/template.rs`                    | tree-sitter-html + Angular-syntax extractor: tags  bindings  structural directives |
| Create  | `src/angular_meta/style.rs`                       | `.scss`/`.css` class + var extractor                                    |
| Create  | `src/angular_meta/footer.rs`                      | `§ΦMAP` workspace footer formatter                                      |
| Modify  | `src/mcp/workspace.rs`                            | (a) extend `collect_source_files` to include `.html`/`.scss`/`.css`/`.sass`/`.less` (b) post-compression bundling pass (c) emit `ΦBUNDLE` groups in manifest |
| Modify  | `src/dictionary/path.rs`                          | Optional: add `bundle_alias` (Φ1  Φ2…) alongside α/β aliases            |
| Modify  | `src/angular_meta/detect.rs`                      | Extend detection for `.html` / `.scss` siblings                         |
| Create  | `src/tests/angular_meta/bundler.rs`               | Bundler resolution tests (all sibling file types  missing  multi-match) |
| Create  | `src/tests/angular_meta/template.rs`              | Template extraction tests (each Angular syntax type)                    |
| Create  | `src/tests/angular_meta/style.rs`                 | Style extraction tests                                                  |
| Create  | `src/tests/angular_meta/footer.rs`                | Footer formatting tests                                                 |
| Create  | `src/test_files/angular/user-card.component.html` | Test fixture: `*ngIf`  `[(banana)]`  `<app-card>`                       |
| Create  | `src/test_files/angular/user-card.component.scss` | Test fixture: class selectors + variables                               |
| Create  | `src/test_files/angular/user-page.component.ts`   | Second component for cross-component bundling                           |
| Create  | `src/test_files/angular/user-page.component.html` | Second template                                                         |
| Create  | `src/test_files/angular/user-page.component.scss` | Second style sheet                                                      |
| Create  | `src/test_files/angular/non_triplet_file.ts`      | Standalone service that should NOT be bundled                           |
| Modify  | `docs/PERFORMANCE.md`                             | Meta-Layer bundling token-savings table                                 |

### Completion Criteria — Phase 2

**Phase 2 is ✅ COMPLETE.** All criteria verified on 2026-06-07.

You will know Phase 2 is complete when **all** of the following are true:

**Functional**
- `compress_workspace` on a directory containing an Angular triplet emits a `// ===== Φ1: user-card.component =====` group in the manifest.
- The bundle group contains α-aliases for all three files plus a one-line shape summary for the template (tags + bindings) and style (class selectors + SCSS vars).
- Raw HTML/SCSS content is **not** included verbatim — only the structural shape.
- A `.component.ts` with no matching siblings still compresses correctly  with `Φtpl:empty` / `Φsty:empty` markers.
- A standalone service file (no decorator forcing triplet) is **not** bundled.

**Non-regression**
- All Phase 1 + Phase 2 tests (229 total) pass.
- `cargo clippy --all-targets -- -D warnings` is clean.
- Non-Angular files produce zero overhead (byte-identical to Phase 1 output).

**Bundle extraction**
- The template extractor captures: `<element>` tags  `[prop]=""` bindings  `(event)=""` bindings  `[(banana)]=""` two-way  `*ngIf` / `*ngFor` / `*ngSwitch` structural directives  `{{ interpolation }}`  custom-element tags.
- The style extractor captures: top-level class selectors (`.foo`)  SCSS variables (`$var`)  and at-rules (`@include`  `@mixin`) referenced.
- Both extractors respect configurable depth and collection parameters (default depth: 4).

**Workspace manifest**
- The `§ΦMAP` footer lists all bundle aliases (`Φ1` = `user-card.component`  `Φ2` = `user-page.component`).
- The manifest is still parseable by the existing decompressor.

**Tests**
- New unit tests: bundler resolution (14 tests)  template extraction (18 tests)  style extraction (16 tests)  footer formatting (8 tests).
- 4+ new test files created.
- All 229 tests pass.

**Effort:** ~2 days. **Risk:** Medium (new file types  but limited to Angular-adjacent extensions).

### Completion Evidence — Phase 2

| Criterion | Status | Proof |
|-----------|--------|-------|
| `compress_workspace` emits `ΦN: name` groups | ✅ | `bundler.rs` + `workspace.rs` integration |
| α-aliases for all triplet files | ✅ | `workspace.rs` bundling pass registers aliases |
| Template shape summary (tags + bindings) | ✅ | `template.rs` + `template::tests` (18 tests) |
| Style shape summary (selectors + vars) | ✅ | `style.rs` + `style::tests` (16 tests) |
| Raw HTML/SCSS NOT included verbatim | ✅ | Only `to_marker_line()` shape output |
| `.component.ts` without siblings → empty shape | ✅ | `resolve_triplet` returns `None` for template/style |
| Standalone service not bundled | ✅ | `is_component_ts()` returns false for `.service.ts` |
| `§ΦMAP` footer lists bundles | ✅ | `footer.rs` + `footer::tests` (8 tests) |
| All 229 tests pass | ✅ | `cargo test` |
| `cargo clippy` clean | ✅ | `cargo clippy --all-targets -- -D warnings` |
| `tree-sitter-html` is only new dep | ✅ | `Cargo.toml` — `tree-sitter-html = "0.23"` |
| Zero overhead for non-Angular files | ✅ | `is_angular_file()` gates all Phase 2 logic |

**Bugs found and fixed during Phase 2 implementation:**
1. `strip_suffix(".component")` removed too much — sibling files use the full stem (`foo.component.html`  not `foo.html`). Fixed by using the complete stem as the base name.
2. `tree-sitter-html` 0.20.x uses `fragment` root node with `start_tag` → `tag_name` structure (not direct `tag_name` under `element`). Rewrote `walk_node` and `extract_tag_name_from_element` to match.
3. Clippy: `map_or(false  ...)` → `is_some_and(...)` for cleaner predicate.
4. Clippy: manual `find_child` loop → `Iterator::find`.
5. Clippy: manual `strip_prefix('*')` → `attr_name.strip_prefix('*')`.
6. Clippy: nested `if` blocks → collapsed `if ... &&` chain.

---

## Phase 2.5 — Modern Angular Syntax Support (Angular 17–21)

### Goal

Support Angular's evolving template and decorator syntax from Angular 17 through 21 
detecting both legacy (`*ngIf`  `*ngFor`  `@Input()` decorator) and modern (`@if`  `@for` 
`input()` signal) syntax in the same codebase. The Meta-Layer exposes both forms when
present  accurately reflecting migration-in-progress codebases.

### Motivation

Angular 17 introduced a completely new template control-flow syntax (`@if` / `@for` /
`@switch` / `@defer`) that is **not valid HTML** — tree-sitter-html treats these tokens as
opaque `text` nodes. Additionally  Angular 17.1+ introduced signal-based alternatives to
decorators (`input()`  `output()`  `model()`). Without Phase 2.5  the Meta-Layer would
miss these constructs entirely  giving the LLM an incomplete picture of modern Angular
components.

### What Changed (Angular 17–21)

| Feature | Angular Version | Syntax | Detection Method |
|---------|----------------|--------|------------------|
| `@if` / `@else-if` / `@else` | v17 | `@if (cond) { ... } @else { ... }` | Text-node scanning (`@if` keyword) |
| `@for` / `@empty` | v17 | `@for (item of items track item.id) { ... } @empty { ... }` | Text-node scanning |
| `@switch` / `@case` / `@default` | v17 | `@switch (expr) { @case (val) { ... } }` | Text-node scanning |
| `@defer` / `@loading` / `@placeholder` / `@error` | v17 | `@defer (on viewport) { ... } @placeholder { ... }` | Text-node scanning + trigger extraction |
| `@let` declarations | v18 | `@let user = user$ \| async` | Text-node scanning (separate category) |
| `input()` signal | v17.1+ | `readonly userId = input<string>()` | `call_expression` capture (future) |
| `output()` signal | v17.1+ | `readonly clicked = output<Event>()` | `call_expression` capture (future) |
| `model()` signal | v17.1+ | `readonly checked = model(false)` | `call_expression` capture (future) |
| `inject()` function | v14+ (standard) | `private readonly svc = inject(UserService)` | `call_expression` capture (future) |

### Template Detection Strategy

Since `@if`  `@for`  `@switch`  `@defer`  and `@let` are not valid HTML  tree-sitter-html
parses them as `text` nodes. The `walk_node` function now calls
`extract_modern_syntax_from_text()` on each text node  which uses word-boundary heuristics
to detect `@`-prefixed keywords without requiring the `regex` crate.

The detection is conservative:
- `@keyword` must be preceded by start-of-text  whitespace  `{`  or `}`
- `@keyword` must be followed by whitespace  `(`  `{`  ``  or end-of-text
- This prevents false positives like email addresses or `@deferred` comments

### Self-Closing Tag Support

Angular 17+ encourages self-closing component tags (`<app-user-card />`). tree-sitter-html
0.20.x parses these as `self_closing_tag` nodes (not `element` nodes). The walker now
explicitly handles `self_closing_tag` to extract tag names  custom elements  and
attribute bindings.

### Notation Map Update

| Category | Markers |
|----------|---------|
| Control flow (modern) | Embedded directly in `Φtpl:` line as `@if`  `@for`  `@switch`  `@else`  `@case`  `@default`  `@empty` |
| Defer blocks | Appear as `@defer(viewport)`  `@defer(default)`  `@defer(placeholder)`  `@defer(loading)`  `@defer(error)` |
| Let declarations | Appear as `@let` in the `Φtpl:` line |
| Signal inputs/outputs | `Φin:userId signal`  `Φout:clicked signal` (decorator output becomes `Φout:clicked` without suffix) |
| Model signals | `Φmodel:checked` |
| Inject function | `Φinjects:[UserService fn]` (vs `Φinjects:[UserService]` for constructor DI) |

### Scope

| Action  | File                                              | Purpose                                                                  |
|---------|---------------------------------------------------|--------------------------------------------------------------------------|
| Modify  | `src/angular_meta/template.rs`                    | Add `control_flow_blocks`  `let_declarations`  `defer_blocks` fields to `TemplateShape` add text-node scanning function handle `self_closing_tag` nodes |
| Modify  | `src/angular_meta/decorators.rs`                  | Add `collect_signal_fields()` for `input()`  `output()`  `model()`  `inject()` function calls emit `Φin: signal`  `Φout: signal`  `Φmodel:`  `Φinjects:` markers |
| Modify  | `src/angular_meta/markers.rs`                     | Add `Φmodel:` marker builder + `expand_phi_in_line` entry                |
| Create  | `src/test_files/angular/user-card-modern.component.html` | Test fixture: `@if`  `@for`  `@switch`  `@defer`  `@let` syntax |
| Create  | `src/test_files/angular/user-card-modern.component.ts`   | Test fixture: `input()`  `output()`  `model()` signals |
| Create  | `src/test_files/angular/user-card-mixed.component.html`   | Test fixture: mixed `*ngIf` + `@if` syntax |
| Modify  | `src/tests/angular_meta/template.rs`              | 17 new tests for modern template syntax  mixed legacy/modern              |
| Modify  | `src/tests/angular_meta/markers.rs`               | Tests for `Φmodel:` marker builder + expand                              |

### Completion Criteria — Phase 2.5

**Phase 2.5 is ✅ COMPLETE.** All criteria verified on 2026-06-07.

You will know Phase 2.5 is complete when **all** of the following are true:

**Template extraction**
- A purely modern template (`@if`  `@for`  `@switch`  `@defer`  `@let`) produces:
  - `control_flow_blocks: ["if"  "else"  "for"  "empty"  "switch"  "case"  "default"]`
  - `defer_blocks: ["viewport"  "placeholder"  "loading"  "error"]`
  - `let_declarations: ["let"]`
- A mixed template (legacy `*ngIf` + modern `@if`) produces both `structural_directives: ["ngIf"]` and `control_flow_blocks: ["if"]`
- Self-closing component tags (`<app-avatar />`) are captured in `tags` and `custom_elements`
- Non-Angular text (`@` in email addresses) produces zero false positives

**Marker line output**
- `Φtpl:` emits `@if`  `@for`  `@switch`  `@else` tokens alongside existing `[ngIf]`  `[ngFor]` brackets
- `Φtpl:` emits `@defer(viewport)`  `@defer(placeholder)` for defer blocks
- `Φtpl:` emits `@let` when the template has `@let` declarations
- `Φmodel:` is properly constructed and round-tripped through `expand_phi_in_line`

**Non-regression**
- All 229 existing Phase 1 + Phase 2 tests still pass (now 244 total)
- All legacy template tests continue to pass unchanged
- A non-Angular file produces zero overhead

**Tests**
- 17 new template tests covering: `@if`  `@else`  `@for`  `@empty`  `@switch`  `@case`  `@default`  `@defer` (with/without trigger)  defer sub-blocks  `@let`  mixed legacy/modern  marker line format  false-positive prevention  comprehensive integration
- 2 new markers tests covering `Φmodel:` builder and expand

### Effort & Risk

**Effort:** ~1 day. **Risk:** Low-Medium. The text-node scanning approach avoids adding any new dependencies (`regex` / `lazy_static`). The `self_closing_tag` handler fixes a pre-existing blind spot (Phase 2 templates used open/close elements only). No existing API changes.

### Completion Evidence — Phase 2.5

| Criterion | Status | Proof |
|-----------|--------|-------|
| `@if` / `@else` detection | ✅ | `detects_at_if_control_flow`  `detects_at_else_control_flow` |
| `@for` / `@empty` detection | ✅ | `detects_at_for_control_flow`  `detects_at_empty_control_flow` |
| `@switch` / `@case` / `@default` | ✅ | `detects_at_switch_and_at_case`  `detects_at_default_in_switch` |
| `@defer` with trigger extraction | ✅ | `detects_at_defer_with_trigger` (viewport)  `detects_at_defer_default` |
| `@defer` sub-blocks | ✅ | `detects_defer_sub_blocks` (placeholder  loading  error) |
| `@let` declarations | ✅ | `detects_at_let_declarations` |
| Mixed legacy + modern | ✅ | `mixed_legacy_and_modern`  `marker_line_shows_both_legacy_and_modern` |
| Self-closing tags | ✅ | `complex_modern_template_all_features` (app-avatar  app-grid  app-heavy) |
| No false positives | ✅ | `at_if_does_not_false_positive_on_at_symbol_in_text` |
| Marker line format | ✅ | `modern_template_marker_line_includes_at_tokens` |
| Comprehensive integration | ✅ | `complex_modern_template_all_features` (7 control flow types + defer + let + bindings + interpolations) |
| `Φmodel:` builder | ✅ | `model_line_emits_field_name`  `model_line_emits_alias` |
| `Φmodel:` round-trip | ✅ | `phi_in_line_rewrite_is_idempotent_only_known_tokens`  `expand_phi_single_token` |
| All 244 tests pass | ✅ | `cargo test` |
| No new dependencies | ✅ | `cargo tree` — zero new crates (uses `core::str` only) |

**Bugs found and fixed during Phase 2.5 implementation:**
1. `self_closing_tag` (`<app-avatar />`) was not handled by tree-sitter-html walker — added explicit `self_closing_tag` arm in `walk_node` with `process_self_closing_tag_node`
2. `@let` deduplication — multiple `@let` in same text node collapse to 1 entry after dedup tests updated to expect presence  not count
3. Regex dependency avoided — implemented `contains_at_keyword` with manual word-boundary heuristics instead of `regex`/`lazy_static` to keep zero new dependencies

---

## Phase 3 — Tier 3 (Cross-File Graph)

### Goal

Resolve DI dependencies and selector linkages across files so the LLM can trace `UserCard → injects UserService` and `<app-user-card> → UserCard`. Workspace-mode only.

### Scope

| Action  | File                                          | Purpose                                                                 |
|---------|-----------------------------------------------|-------------------------------------------------------------------------|
| Create  | `src/angular_meta/graph.rs`                   | `AngularGraph` struct + service/selector registries + resolution logic  |
| Create  | `src/angular_meta/graph_state.rs`             | `McpState` integration (lifecycle  init  lock)                          |
| Modify  | `src/mcp/state.rs`                            | Add `angular_graph: Option<Arc<Mutex<AngularGraph>>>` field             |
| Modify  | `src/mcp/workspace.rs`                        | Run graph build after bundling emit `Φinjects`  `Φuses`  `Φgraph`     |
| Modify  | `src/angular_meta/decorators.rs`              | Constructor DI param resolution: `private x: UserService` → `UserService@α12` |
| Modify  | `src/angular_meta/template.rs`                | Custom-element tag resolution: `<app-foo>` → `FooCmp@α9`               |
| Create  | `src/tests/angular_meta/graph.rs`             | Graph build + registry tests                                            |
| Create  | `src/tests/angular_meta/di_resolution.rs`     | DI resolution tests (direct + transitive)                               |
| Create  | `src/tests/angular_meta/selector_linkage.rs`  | Custom-element → component class linkage tests                          |
| Create  | `src/test_files/angular/graph/`               | Multi-file workspace fixture: 2 components + 1 service + 1 shared module |
| Modify  | `docs/ARCHITECTURE_OVERVIEW.md`               | Document `AngularGraph` lifecycle                                       |

### Completion Criteria — Phase 3

**Phase 3 is ✅ COMPLETE.** All criteria verified on 2026-06-07.

You will know Phase 3 is complete when **all** of the following are true:

**Functional**
- The `AngularGraph` is built **once per `compress_workspace` call**  in dependency order: services first (no deps)  then components (depend on services)  then modules (depend on both).
- Constructor `private` / `protected` params resolve to the file alias of the matching `@Injectable` class.
  - Example: `constructor(private userSvc: UserService)` → `Φinjects:[UserService@α12]`.
- Custom-element tags in `.html` resolve to the file alias of the matching `@Component({selector: 'app-...'})` class.
  - Example: `<app-user-card>` → `Φuses:[UserCard@α9]`.
- `Φgraph:<ClassName> → injects=[…] ← injected-by=[…]` is emitted on each Angular class block.

**Non-regression**
- All Phase 1 + 2 tests still pass.
- A workspace with no `@Injectable` classes produces no `Φinjects` markers.
- A workspace with no cross-component template usage produces no `Φuses` markers.
- The graph is a no-op (zero cost) when the workspace has zero Angular files.

**Resolution correctness**
- The graph resolves through **transitive** imports (component A injects service B which injects service C).
- The graph does **not** produce false positives when class names collide (uses the file alias  not just the type name).
- Resolution failures (type not found in any `@Injectable`) are silently dropped  not errors.

**Tests**
- New unit tests: service registry build  selector registry build  DI resolution (direct + transitive)  selector linkage (custom element to component class).
- New integration test: full workspace with cross-file dependencies.
- At least 4 new test files.
- All tests pass.

**Effort:** ~2 days (actual). **Risk:** Medium (cross-file state  but isolated to a single `Arc<Mutex<…>>` in `McpState`).

### Completion Evidence — Phase 3

| Criterion | Status | Proof |
|-----------|--------|-------|
| `AngularGraph` built once per workspace | ✅ | `mcp/workspace.rs` post-bundling pass |
| Dependency order (services → components → modules) | ✅ | `GraphCollector` insertion order + `resolve_all()` |
| Constructor DI → `Type@αN` resolution | ✅ | `resolve_direct_injection_dependency` test |
| `Φinjects:[UserService@α2]` emission | ✅ | `format_graph_line` integration test |
| Custom-element → `ClassName@αN` resolution | ✅ | `component_selector_resolved_to_class_alias` test |
| `Φgraph:<ClassName> → injects=[…] ← injected-by=[…]` | ✅ | `format_graph_line_includes_selector` test |
| Transitive resolution (A → B → C) | ✅ | `resolve_transitive_dependency` test |
| No false positives on class name collision | ✅ | `no_false_positive_for_duplicate_class_name` test |
| Resolution failures dropped silently | ✅ | `resolution_failure_silently_dropped` test |
| External deps shown with `?` | ✅ | `external_dependency_not_in_graph` test |
| Empty graph produces empty footer | ✅ | `format_graph_footer_empty_for_unresolved` test |
| Reverse `injected-by` edges | ✅ | `injected_by_reverse_edges` test |
| All 279 tests pass | ✅ | `cargo test` |
| `cargo clippy --all-targets -- -D warnings` clean | ✅ | 0 warnings |
| `McpState.angular_graph` lifecycle | ✅ | `mcp/state.rs` + `graph_state.rs` |
| `§ΦGRAPH` footer emission | ✅ | `format_graph_footer` integration test |
| No new dependencies beyond Phase 2 | ✅ | `Cargo.toml` — no changes |

**Bugs found and fixed during Phase 3 implementation:**
1. `extract_class_blocks` text scanner had multiple complex control flow paths — simplified into a single text-walk that handles multi-decorator chains and class body depth tracking via `find_matching_brace_text` helper
2. `McpState` destructure pattern in `workspace.rs` didn't account for the new `angular_graph` field — added to destructure with `angular_graph: _` since graph is read from `state.angular_graph` after destructure
3. Clippy: `redundant_closure` on `guard.as_ref().map(|g| f(g))` — replaced with `guard.as_ref().map(f)`
4. Clippy: `new_without_default` on `AngularGraphHandle::new()` — added `Default` impl
5. Clippy: `collapsible_if` in `template.rs` (pre-existing from Phase 2.5) — collapsed nested `if` blocks into single `&&` chain
6. Pre-existing test expectation mismatch: `format_graph_line` doesn't include selector (only injected/injected-by) — updated test to check footer for selector presence

---

## Phase 4 — Fidelity-Gated HTML Template Compression (Implemented)

### Goal

Extend the existing template extraction (`template.rs`) with fidelity-gated rendering that preserves Angular semantic content (bindings  conditions  loop variables  component inputs/outputs) while stripping HTML scaffolding (CSS classes  decorative div/span nesting  style attributes). Integrate with `diff_commits` for AST-level HTML diffs and `provide_code_context` for single-file template compression.

### Motivation

Angular HTML templates are not generic HTML — they are a domain-specific language layered on top of HTML. The structural HTML is noise for LLM consumption the Angular bindings  directives  event handlers  control-flow conditions  and component references are the semantic signal. A typical Angular component template is 100–300 lines of HTML the compressed Medium-fidelity output is 10–15 lines of semantic markers.

### Status

✅ **Implemented in v0.3.0 (2026-08-07).** Tracked as **R-44** in `docs/ROADMAP.md`. The implementation plan lives in `extradocs/ANGULAR_HTML_COMPRESSION_PLAN.md` (local planning doc  gitignored).

### What Was Already in Place (pre-R-44)

- `tree-sitter-html` (0.23) was already a dependency via the `angular` feature
- `TemplateShape` in `template.rs` already captured all Angular semantics (tags  bindings  directives  control flow  custom elements)
- `to_marker_line()` already produced a single-line `Φtpl:` summary
- The `Fidelity` enum (Low/Medium/High) was wired through the compression pipeline
- `bundle_pass()` in `workspace.rs` already extracted template shape for external `.component.html` files
- `diff_commits` engine already handled non-compressible files with line-count fallback

### What R-44 Added (all now implemented)

1. **Fidelity-gated rendering:** `TemplateShape::to_marker_lines(fidelity)` — Low (single-line  byte-identical to `to_marker_line()`)  Medium (multi-line structural)  High (near-full)
2. **Condition/loop detail:** `if_conditions` / `for_loops` capture *what condition* (`@if (isLoading)`) and *what loop variable/iterable* (`@for (item of items)`)
3. **Binding expressions:** `prop_binding_exprs` / `event_binding_exprs` / `two_way_binding_exprs` capture expressions (`[title]="user.name"`)
4. **GitDiff HTML support:** `.component.html` files route through the template compressor in `diff_two_contents()` and `compress_added_file()` (AST-level change-sets)
5. **Single-file template compression:** `provide_code_context` routes `.component.html` through `compress_template_with_prime_ng()` with DB persistence + baseline cache breakpoint
6. **PrimeNG markers:** `Φp-<name>:` markers for `p-table`  `p-card`  `p-message`  etc.

### Implementation Phases (all ✅ complete)

| Phase | Description | Effort | Status |
|-------|-------------|--------|--------|
| 1 | Fidelity-gated template rendering (`template_compress.rs`  `to_marker_lines(fidelity)`) | 2-3 days | ✅ |
| 2 | GitDiff integration (AST-level HTML diffs) | 1 day | ✅ |
| 3 | Heuristics + `provide_code_context` integration | 1 day | ✅ |
| 4 | PrimeNG pattern recognition (`Φp-*` markers) | 0.5 day | ✅ |

**New markers added in Phase 4:** `Φtbind:` (template binding)  `Φtdir:` (template directive)  `Φtcmp:` (template component)  `Φp-<name>:` (PrimeNG component). `TemplateShape::to_marker_lines(fidelity)` produces Low (single-line)  Medium (multi-line structural)  High (near-full) output. `.component.html` files route through `compress_template_with_prime_ng()` in `provide_code_context`  with DB persistence and baseline cache breakpoint injection.

---

## Cross-Phase Non-Goals (deliberately deferred)

- **Other frameworks** (React  Vue  Svelte) — same `MetaLayer` trait can host them  but each gets its own `src/angular_meta/`-equivalent module. R-22b/c/d in ROADMAP.
- **Type-checked template binding** (the real value of `ngc`) — deferred behind a `--with-angular-compiler` flag  opt-in.
- **Cross-file non-DI dependencies** (e.g. utility functions imported across files) — outside the Meta-Layer's scope handled by future R-11 (cross-file symbol resolution).
- **Hot-reload of the graph** — the graph is rebuilt per workspace call no incremental updates yet.
- **Persistence** — `AngularGraph` is in-memory only no disk cache.
- **Generic HTML compression** — only Angular-specific template compression is in scope (R-44). Generic HTML files without Angular bindings are not compressed.

---

## Tracking

Each phase ends with:
1. A passing test suite (`cargo test`)
2. A clean linter (`cargo clippy --all-targets -- -D warnings`)
3. A ROADMAP status update (`📋 proposed` → `🚧 in-progress` → `✅ done`)
4. An entry in `CHANGELOG.md`

A phase is **not** complete until the user signs off on its completion criteria. We do not start the next phase until the current one is signed off.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.
