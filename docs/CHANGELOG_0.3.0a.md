---

## [0.3.0] — 2026-08-07 — Angular HTML Template Compression

### Added

#### R-44: Angular HTML Template Compression
- New `src/angular_meta/template_compress.rs` module — fidelity-gated Angular template compression entry point:
  - `compress_template(html, fidelity)` — Low → single-line shape summary, Medium → multi-line structural Angular semantics, High → near-full template with HTML scaffolding stripped
  - `compress_template_to_string(html, fidelity)` — joined-string convenience wrapper
  - `compress_template_with_prime_ng(html, fidelity)` — appends PrimeNG `Φp-<name>:` markers
  - `is_prime_ng_component(tag)` / `extract_prime_ng_markers(shape)` — PrimeNG pattern recognition (Phase 4)
- `TemplateShape::to_marker_lines(fidelity)` in `src/angular_meta/template.rs` — fidelity-gated rendering:
  - Low → byte-identical to existing `to_marker_line()` (non-regression)
  - Medium → preserves `@if(cond)`, `@for(var of iter)`, custom elements with binding expressions, structural directives
  - High → preserves all elements, all bindings, all conditions, all event handlers, interpolation count
  - Empty template → `["Φtpl:empty"]` at all fidelities
- `TemplateShape` extended with: `elements` (structured `TemplateElement`), `if_conditions`, `for_loops`, `prop_binding_exprs`, `event_binding_exprs`, `two_way_binding_exprs`
- `TemplateElement` struct with `render()` — compact per-element line with bindings/directives
- `PhiLineKind` extended with `TemplateBinding` (`Φtbind:`), `TemplateDirective` (`Φtdir:`), `TemplateComponent` (`Φtcmp:`) — full vocabulary wiring (marker_prefix, expansion, expand order, token lookup)
- `src/angular_meta/mod.rs` — exports `template_compress` module; `run_meta_layer()` uses `shape.to_marker_lines(fidelity)`

#### R-44 Phase 2: GitDiff Integration
- `src/gitdiff/engine.rs` — `.component.html` files routed through the Angular template compressor:
  - `is_angular_template(path)` — detects `.component.html` (feature-gated)
  - `diff_two_contents()` — modified `.component.html` files produce compressed old/new template change-sets (not line-count deltas)
  - `compress_added_file()` — added `.component.html` files emit compressed template skeleton

#### R-44 Phase 3: Heuristics + provide_code_context Integration
- `src/mcp/heuristics.rs`:
  - `is_angular_template_path()` — detects `.component.html`
  - `classify_file()` — `.component.html` classified as `FileClass::Implementation` with `Fidelity::Medium` default (checked before Config classification)
  - `resolve_fidelity()` — `intent="edit"` on `.component.html` → `Fidelity::High` (template editing trigger)
- `src/mcp/tool_handlers/core.rs` — `handle_provide_code_context()` routes `.component.html` files through `compress_template_with_prime_ng()`:
  - Fidelity resolution: explicit arg > edit-intent High > Medium default
  - Records compression stats to the `angular_template` domain
  - Persists to SQLite DB (`queue_save_context` + `flush_persistence`) so `context_stats` and cross-session dashboards report Angular template savings
  - Injects baseline cache breakpoint

#### R-44 Phase 4: PrimeNG Pattern Recognition
- `Φp-<name>:` markers for PrimeNG components (`p-table`, `p-card`, `p-message`, etc.)
- `is_prime_ng_component()` operates on `custom_elements` (tags containing a hyphen) — safe from `<p>`/`<picture>` false-positives

#### Dashboard: `angular_template` domain
- `src/mcp/session_stats.rs` — `angular_template` domain added to the per-domain breakdown rendering (`Angular Templates: {raw} → {comp} ({savings}%↓)`)

### Fixed
- **Word-boundary bug** in `src/angular_meta/template.rs`: `@if`/`@for` inside string literals (e.g. `{{ "@if (x)" }}`) or identifiers (e.g. `@formatter`) were falsely captured into `if_conditions`/`for_loops`. Gated extraction behind the same `contains_at_keyword` word-boundary check used for `control_flow_blocks`. Added regression tests.
- **Persistence gap** in `src/mcp/tool_handlers/core.rs`: `.component.html` compressions were recorded in-memory but never persisted to the SQLite DB. Added `queue_save_context` + `flush_persistence`.

### Tests
- `src/tests/angular_meta/template_compress.rs` (new) — 17 tests: Low/Medium/High fidelity rendering, PrimeNG detection/markers, empty-template edge cases
- `src/tests/angular_meta/template.rs` — 2 new regression tests for the word-boundary fix
- `src/tests/angular_meta/markers.rs` — `phi_line_kind_uniqueness` updated to 17 variants
- `src/tests/mcp/heuristics.rs` — 3 new tests for `.component.html` classification + edit-intent fidelity
- `src/tests/gitdiff/engine.rs` — 2 new tests for `.component.html` diff change-sets
- `src/tests/mcp/session_stats.rs` — 1 new test for `angular_template` domain rendering

**Verification:** 2,141 tests passing, 0 clippy warnings. Live E2E verified: `provide_code_context` on `.component.html` returns fidelity-gated output with PrimeNG markers + baseline breakpoint. Measured compression: High 47.4% byte / 34.2% token reduction, Medium 32.1% byte reduction.

---

