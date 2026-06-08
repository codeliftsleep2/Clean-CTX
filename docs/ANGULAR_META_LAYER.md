# Angular Meta-Layer Plan

> **Status:** 🚧 Phase 1 ✅ · **Last updated:** 2026-06-07
>
> **Core principle:** The Meta-Layer is **purely additive** — it never modifies the existing TS compaction output. It only appends a `Φ` block below the existing compacted class. Existing users see no change; Angular users get enriched output.

---

## Decisions Locked

| Question              | Decision                                                                                              |
|-----------------------|-------------------------------------------------------------------------------------------------------|
| Compiler approach     | `tree-sitter-html` + custom Angular-syntax extractor (Option C). No Node dependency.                  |
| Marker approach       | `Φ`-prefixed markers, no new opcodes. Opcodes stay language-agnostic primitives.                      |
| Phasing               | Three independently shippable phases. Stop after any phase for a useful Meta-Layer.                   |
| Default state         | On, opt-out via `.clean-ctx.json`. Non-Angular files pay zero overhead.                                |
| Workspace scope       | Tier 1 (decorators) works in both modes. Tiers 2 & 3 are workspace-only.                                |
| New dependencies      | Phase 1: none. Phase 2: `tree-sitter-html`. Phase 3: none beyond Phase 2.                              |

---

## Notation Map

| Prefix         | Job                                        | Examples                                          |
|----------------|--------------------------------------------|---------------------------------------------------|
| `$xx`          | Opcodes — language primitives              | `$c` = class, `$ctor` = constructor, `$a` = async |
| `⊕`            | Behavior markers — control-flow annotations| `⊕guard`, `⊕loop`, `⊕⇒`, `⊕!`                    |
| `α / β / γ`    | Path aliases — file references             | `α7` = `/path/to/file.ts`                          |
| `Φ` (new)      | Framework-annotation markers               | `Φcmp:`, `Φsvc:`, `Φin:`, `Φout:`, `ΦBUNDLE`     |

---

## Phase 1 — Scaffold + Tier 1 (Decorators)

### Goal

Prove the Meta-Layer pattern end-to-end with the lowest-risk deliverable: TS-only, no new file types, no new runtime dependencies. This phase alone gives the LLM `@Component` / `@Injectable` / `@Input` / `@Output` context.

### Scope

| Action  | File                                              | Purpose                                                                 |
|---------|---------------------------------------------------|-------------------------------------------------------------------------|
| Create  | `src/angular_meta/mod.rs`                         | Public surface, `MetaBlock` struct, `run_meta_layer` entry point       |
| Create  | `src/angular_meta/detect.rs`                      | Angular detection heuristic (decorator presence + tsconfig hint)       |
| Create  | `src/angular_meta/decorators.rs`                  | `@Component` / `@Injectable` / `@NgModule` / `@Directive` / `@Pipe` / `@Input` / `@Output` extractor |
| Create  | `src/angular_meta/markers.rs`                     | `PhiMarker` enum + `build_phi` + `expand_phi` (mirrors `markers.rs` `⊕` shape) |
| Modify  | `src/queries.rs`                                  | Add `decorator` + `decorator_call` + `object` captures to `TS_QUERY`   |
| Modify  | `src/compaction/class.rs`                         | After `format_class_entry`, call into `angular_meta` and append Φ block|
| Modify  | `src/compression/pipeline.rs`                     | Pass `fidelity` to class compactor; plumb `Option<MetaBlock>` through   |
| Modify  | `src/decompression/markers.rs`                    | Add `expand_phi_in_line` alongside existing `expand_markers_in_line`   |
| Modify  | `src/mcp/prompts.rs`                              | New "Framework Meta Markers" section in `SYSTEM_PROMPT`                 |
| Modify  | `src/config.rs`                                   | Add `meta_layers: BTreeMap<String, MetaLayerConfig>` schema             |
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
- A `.ts` file with `@Component({selector: 'app-user-card', templateUrl: './user-card.component.html', ...})` produces a `// --- Φ Angular Meta ---` block below the existing compacted class.
- The block contains a `Φcmp:<ClassName>` line with `sel=`, `tpl=`, `sty=` attributes.
- `@Input()` and `@Output()` fields emit `Φin:` / `Φout:` markers.
- `@Injectable({providedIn: 'root'})` emits a `Φsvc:<ClassName> scope=root` line.
- `@NgModule({...})` emits a `Φmod:<ClassName> decl=[…] imp=[…] exp=[…]` line.
- `@Directive` / `@Pipe` emit `Φdir:` / `Φpipe:` lines.
- Constructor parameters with `private` / `protected` emit `Φinjects:[<Type>]` (unresolved at this phase — class names only, no `α` aliases yet).

**Non-regression**
- A non-Angular `.ts` file produces **zero** Φ markers and **zero** newlines of overhead.
- All 121 existing tests still pass.
- `cargo clippy --all-targets -- -D warnings` is clean.
- `compress_code_context` on a non-Angular file produces byte-identical output to v0.2.0.

**Round-trip**
- `decompress_code_context` on a compressed Angular file expands `Φcmp:` → `@Component`, `Φsvc:` → `@Injectable`, etc.
- The expanded output is human-readable and preserves all original class names.

**LLM discoverability**
- `SYSTEM_PROMPT` includes a new "Framework Meta Markers" section that teaches the LLM the `Φ` vocabulary.
- The section appears below the existing "Behavior Markers" section so the layering reads naturally: opcodes → ⊕ markers → Φ markers.

**Tests**
- New unit tests: detector (positive + negative), decorator extraction (each decorator type), marker round-trip.
- At least 6 new test files (1 per Angular decorator type + integration).
- All tests pass.

**Documentation**
- `docs/ANGULAR_META_LAYER.md` (this document) exists.
- `docs/ARCHITECTURE_OVERVIEW.md` updated with the new `angular_meta/` module.
- `docs/ROADMAP.md` R-22 entry marked 🚧 in-progress.

**Effort:** ~3.5 days (actual). **Risk:** Low (additive, no existing API changes, zero new deps).

### Completion Evidence

| Criterion | Status | Proof |
|-----------|--------|-------|
| `Φcmp:<ClassName>` with `sel=`, `tpl=`, `sty=` | ✅ | `decorators.rs` + `markers.rs` tests |
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
2. Byte literal escapes (`\t`, `\n`) were literal characters — fixed to proper escape sequences
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
| Create  | `src/angular_meta/bundler.rs`                     | File-triplet resolver: `*.component.ts` → `*.{html,scss,css,sass,less}` siblings |
| Create  | `src/angular_meta/template.rs`                    | tree-sitter-html + Angular-syntax extractor: tags, bindings, structural directives |
| Create  | `src/angular_meta/style.rs`                       | `.scss`/`.css` class + var extractor                                    |
| Create  | `src/angular_meta/footer.rs`                      | `§ΦMAP` workspace footer formatter                                      |
| Modify  | `src/mcp/workspace.rs`                            | (a) extend `collect_source_files` to include `.html`/`.scss`/`.css`/`.sass`/`.less`; (b) post-compression bundling pass; (c) emit `ΦBUNDLE` groups in manifest |
| Modify  | `src/dictionary/path.rs`                          | Optional: add `bundle_alias` (Φ1, Φ2…) alongside α/β aliases            |
| Modify  | `src/angular_meta/detect.rs`                      | Extend detection for `.html` / `.scss` siblings                         |
| Create  | `src/tests/angular_meta/bundler.rs`               | Bundler resolution tests (all sibling file types, missing, multi-match) |
| Create  | `src/tests/angular_meta/template.rs`              | Template extraction tests (each Angular syntax type)                    |
| Create  | `src/tests/angular_meta/style.rs`                 | Style extraction tests                                                  |
| Create  | `src/tests/angular_meta/footer.rs`                | Footer formatting tests                                                 |
| Create  | `src/test_files/angular/user-card.component.html` | Test fixture: `*ngIf`, `[(banana)]`, `<app-card>`                       |
| Create  | `src/test_files/angular/user-card.component.scss` | Test fixture: class selectors + variables                               |
| Create  | `src/test_files/angular/user-page.component.ts`   | Second component for cross-component bundling                           |
| Create  | `src/test_files/angular/user-page.component.html` | Second template                                                         |
| Create  | `src/test_files/angular/user-page.component.scss` | Second style sheet                                                      |
| Create  | `src/test_files/angular/non_triplet_file.ts`      | Standalone service that should NOT be bundled                           |
| Modify  | `docs/PERFORMANCE.md`                             | Meta-Layer bundling token-savings table                                 |

### Completion Criteria — Phase 2

You will know Phase 2 is complete when **all** of the following are true:

**Functional**
- `compress_workspace` on a directory containing an Angular triplet emits a `// ===== Φ1: user-card.component =====` group in the manifest.
- The bundle group contains α-aliases for all three files plus a one-line shape summary for the template (tags + bindings) and style (class selectors + SCSS vars).
- Raw HTML/SCSS content is **not** included verbatim — only the structural shape.
- A `.component.ts` with no matching siblings still compresses correctly, with `Φtpl:external` / `Φsty:external` markers.
- A standalone service file (no decorator forcing triplet) is **not** bundled.

**Non-regression**
- All Phase 1 tests still pass.
- A workspace with no `.html`/`.scss` files produces byte-identical output to Phase 1.
- A workspace with `.html`/`.scss` files that are **not** Angular-related (e.g. raw `index.html`) is not affected.

**Bundle extraction**
- The template extractor captures: `<element>` tags, `[prop]=""` bindings, `(event)=""` bindings, `[(banana)]=""` two-way, `*ngIf` / `*ngFor` / `*ngSwitch` structural directives, `{{ interpolation }}`, custom-element tags.
- The style extractor captures: top-level class selectors (`.foo`), SCSS variables (`$var`), and at-rules (`@include`, `@mixin`) referenced.
- Both extractors respect `config.meta_layers.angular.template_depth` (default 2) and `style_collect` (default `["class", "var"]`).

**Workspace manifest**
- The `§ΦMAP` footer lists all bundle aliases (`Φ1` = `user-card.component`, `Φ2` = `user-page.component`).
- The manifest is still parseable by the existing decompressor.

**Tests**
- New unit tests: bundler resolution, template extraction, style extraction, footer formatting.
- At least 4 new test files.
- All tests pass.

**Effort:** ~2 days. **Risk:** Medium (new file types, but limited to Angular-adjacent extensions).

---

## Phase 3 — Tier 3 (Cross-File Graph)

### Goal

Resolve DI dependencies and selector linkages across files so the LLM can trace `UserCard → injects UserService` and `<app-user-card> → UserCard`. Workspace-mode only.

### Scope

| Action  | File                                          | Purpose                                                                 |
|---------|-----------------------------------------------|-------------------------------------------------------------------------|
| Create  | `src/angular_meta/graph.rs`                   | `AngularGraph` struct + service/selector registries + resolution logic  |
| Create  | `src/angular_meta/graph_state.rs`             | `McpState` integration (lifecycle, init, lock)                          |
| Modify  | `src/mcp/state.rs`                            | Add `angular_graph: Option<Arc<Mutex<AngularGraph>>>` field             |
| Modify  | `src/mcp/workspace.rs`                        | Run graph build after bundling; emit `Φinjects`, `Φuses`, `Φgraph`     |
| Modify  | `src/angular_meta/decorators.rs`              | Constructor DI param resolution: `private x: UserService` → `UserService@α12` |
| Modify  | `src/angular_meta/template.rs`                | Custom-element tag resolution: `<app-foo>` → `FooCmp@α9`               |
| Create  | `src/tests/angular_meta/graph.rs`             | Graph build + registry tests                                            |
| Create  | `src/tests/angular_meta/di_resolution.rs`     | DI resolution tests (direct + transitive)                               |
| Create  | `src/tests/angular_meta/selector_linkage.rs`  | Custom-element → component class linkage tests                          |
| Create  | `src/test_files/angular/graph/`               | Multi-file workspace fixture: 2 components + 1 service + 1 shared module |
| Modify  | `docs/ARCHITECTURE_OVERVIEW.md`               | Document `AngularGraph` lifecycle                                       |

### Completion Criteria — Phase 3

You will know Phase 3 is complete when **all** of the following are true:

**Functional**
- The `AngularGraph` is built **once per `compress_workspace` call**, in dependency order: services first (no deps), then components (depend on services), then modules (depend on both).
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
- The graph does **not** produce false positives when class names collide (uses the file alias, not just the type name).
- Resolution failures (type not found in any `@Injectable`) are silently dropped, not errors.

**Tests**
- New unit tests: service registry build, selector registry build, DI resolution (direct + transitive), selector linkage (custom element to component class).
- New integration test: full workspace with cross-file dependencies.
- At least 4 new test files.
- All tests pass.

**Effort:** ~2 days. **Risk:** Medium (cross-file state, but isolated to a single `Arc<Mutex<…>>` in `McpState`).

---

## Cross-Phase Non-Goals (deliberately deferred)

- **Other frameworks** (React, Vue, Svelte) — same `MetaLayer` trait can host them, but each gets its own `src/angular_meta/`-equivalent module. R-22b/c/d in ROADMAP.
- **Type-checked template binding** (the real value of `ngc`) — deferred behind a `--with-angular-compiler` flag, opt-in.
- **Cross-file non-DI dependencies** (e.g. utility functions imported across files) — outside the Meta-Layer's scope; handled by future R-11 (cross-file symbol resolution).
- **Hot-reload of the graph** — the graph is rebuilt per workspace call; no incremental updates yet.
- **Persistence** — `AngularGraph` is in-memory only; no disk cache.

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
