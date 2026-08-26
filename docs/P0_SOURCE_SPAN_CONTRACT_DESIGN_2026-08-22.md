# P0 Design: Canonical Decorator/Annotation/Attribute-Inclusive Class Source Contract

**Date:** 2026-08-22
**Status:** ✅ IMPLEMENTED All recommendations shipped (commits `c422961`, `0552e8a`)
**Audience:** Historical record design preserved for reference; implementation is the source of truth

---

## 1. Executive Summary

The primary LLM-facing IR path and the secondary text path both feed the
meta-layer registry a **compacted class name** instead of the full class
source text. The meta-layer extractors (`decorators::extract_decorators`,
`annotations::extract_annotations`, `dotnet_meta::aspnet::extract_aspnet`,
etc.) require the **full class source** leading `@` decorators / `@`
annotations / `[` attributes, the declaration head, and the class body to
detect framework semantics. As a result:

| Path | Angular (TS) | Spring (Java) | .NET (C#) |
|---|---|---|---|
| IR path (`MetaLayerPass`) | WORKS (via class_source_from_capture) | WORKS (via class_source_from_capture) | WORKS (via class_source_from_capture) |
| Text path (compress) | WORKS (class_source_from_capture) | WORKS (via class_source_from_capture) | WORKS (via class_source_from_capture) |

The text path has an **Angular-only** fix via
`decorator_inclusive_class_text` (compression/pipeline.rs:793-814) that
delegates to `find_decorator_inclusive_start` (src/meta_util.rs:635-693).
That helper only understands the TypeScript `@Name(...)` call shape, so
Java (`@RestController`, no parens) and C# (`[ApiController]`, no `@`) both
return `None` and fall back to the compacted name.

The fix is to establish **one canonical class-source contract** shared by
every path. The canonical definition:

```
class_source       = source[span_start .. class_capture_end]
class_capture_end  = cap.start_byte + cap.raw_text.len()
```

where `span_start` is the leading decorator/annotation/attribute byte, or the
declaration keyword byte when no annotation group precedes it (non-decorated
classes `→`. full backward compatibility).

On the IR path the class source must be derived from the **existing capture
identity** `PassContext.captures: Vec<CapEntry>` (pipeline.rs:95-96) which
is already the canonical capture identity produced by the capture pipeline.
This design does **not** introduce a parallel source vector, does **not**
extend `SymbolInfo`, and does **not** change `CoreOp::DefClass`.

---

## 2. Production-Path Trace

### 2.1 IR path (primary LLM-facing: provide / compress / delta)

```
run_capture_pipeline
    ↓
Vec<CapEntry>
    ↓
PassContext.captures          (C-22: existing identity, now populated)
    ├── CoreIRPass `→`. CoreOp::DefClass(name)
    └── MetaLayerPass `→`. CapEntry `→`. canonical source span `→`. meta-layer registry
```

Detailed call chain:

```
mcp/tool_handlers/core.rs (handle_compress_code_context,
                          handle_provide_code_context, handle_delta)
  `→`. compile_file_ir / compile_file_ir_focused (mcp/tool_helpers.rs:232-358)
      `→`. language layers wired per-extension (tool_helpers.rs:292-306)
      `→`. IRCompiler::compile_focused (ir/compiler.rs:153-172)
          `→`. compile_inner (ir/compiler.rs:189-240)
              `→`. PassContext + PassPipeline::default_production()
                  `→`. CoreIRPass (ir/pipeline.rs:436-678)
                        captures via run_capture_pipeline
                        (compression/capture_pipeline.rs:55-101)
                        each class.root CapEntry: { name, text, raw_text, start_byte }
                          text       = extract_class_name(raw_text)  // "UserCardComponent"
                          raw_text   = class_declaration node        // decorators EXCLUDED
                          start_byte = node.start_byte()
                        state.instructions.push(DefClass(id, cap.text)) // compacted name
                        ...
                        // after the loop (C-22):
                        state.captures = captures;   // MOVE the owned batch no clone
                  `→`. MetaLayerPass (ir/pipeline.rs:723-763)
                        BEFORE: class_names from CoreOp::DefClass(_, name)  // compacted
                        AFTER (C-22):
                          filter state.captures for type-root captures
                          slice state.source[ canonical_class_source(source, cap) ]
                          `→`. registry.run_meta_layers_pipeline(source, slices, ...)
```

**Loss today:** `CoreIRPass` keeps the capture batch in a local
`let captures = ...` (pipeline.rs:457-477); the already-declared
`PassContext.captures: Vec<CapEntry>` (pipeline.rs:95-96) is **never
populated**, so `MetaLayerPass` can only recover the compacted
`DefClass` names.

**Fix (C-22):** the owned batch is MOVED into `state.captures` after the
loop (no clone, no borrow conflict the loop borrows the local while `state`
is mutated, then the move happens once the loop ends). `MetaLayerPass`
derives each canonical class source directly from the `CapEntry` it already
owns: `cap.start_byte`, `cap.raw_text.len()`, and the new
`find_class_source_start(...)` over `state.source`.

### 2.2 Text path (secondary: compress_file / compress_source)

```
compress_file_with_source (compression/pipeline.rs:110-255)
  `→`. run_capture_pipeline (same closure)
  `→`. build_output_lines (compression/pipeline.rs:354-528)
      class.root arm:
        output_lines.push(format_class_entry(&cap.text, ...))      // compacted
        class_captures.push(decorator_inclusive_class_text(src,cap))
          `→`. find_decorator_inclusive_start(...)   // TS-only (@Name(...))
            // Java: `@RestController` `→`. None `→`. falls back to cap.text  ❌
            // C#:   `[ApiController]` `→`. None `→`. falls back to cap.text  ❌
      `→`. registry.run_meta_layers_pipeline(source, class_captures) // TS fixed, others broken
  `→`. Φ blocks appended (lines 212-221, 292-303, 655-665)
```

### 2.3 Streaming path

`compress_file_streaming` (compression/streaming.rs:210-267) calls the SAME
`build_output_lines` (line 238), so a single shared fix covers both.

### 2.4 Workspace graph path

`workspace_util::extract_class_blocks` (mcp/workspace_util.rs:205-240) also
delegates to the same canonical helper (`find_decorator_inclusive_start`,
workspace_util.rs:223-226) for Angular graph ingestion. Extending the helper
for Java/C# also improves this consumer.

---

## 3. Where Each Path Loses Source Context

| Path | Loss Point | Feed Today | Result |
|---|---|---|---|
| IR `MetaLayerPass` | ir/pipeline.rs:728-745 `class_names` derived from `DefClass.name`; `state.captures` never populated | `"UserCardComponent"` (compacted) | Angular zero; Spring zero; .NET zero |
| Text (class.root arm) | compression/pipeline.rs:384-396 helper only understands TS `@Name(...)` | full text for TS; `"MyController"` for Java; `"FooController"` for C# | Angular good; Spring/.NET broken |
| Workspace graph | mcp/workspace_util.rs:224 TS-only helper | TS blocks only | Angular-only |

**The fundamental gap:** `PassContext` already declares
`captures: Vec<CapEntry>` (pipeline.rs:95-96) as the canonical capture
identity from the capture pipeline, but `CoreIRPass` keeps the batch in a
local `let captures` (pipeline.rs:457-477) and never populates
`state.captures`. `MetaLayerPass` therefore cannot reach the source span —
it can only recover the compacted name.

---

## 4. Canonical Helper and Required Source Span (Per Language)

### 4.1 Canonical helper (single source of truth)

**Location:** `src/meta_util.rs` the same module that already owns the
TS-only `find_decorator_inclusive_start` (meta_util.rs:635-693). Extend it
in place into a generic, language-agnostic helper:

```rust
/// Returns the start_byte of the decorator/annotation/attribute group that
/// immediately precedes the type declaration at `type_keyword_pos`,
/// or `type_keyword_pos` itself when no group precedes.
pub fn find_class_source_start(source: &str, type_keyword_pos: usize) -> usize;
```

#### Algorithm (backward, string-aware via the existing `find_matching_brace`)

1. `i = type_keyword_pos`.
2. Walk back over whitespace and comments.
3. If the char before is `)`, match the balanced paren group to the `@Name(`
   the full `@Name(...)` call belongs to the type (TS, and Java
   `@RequestMapping(value=...)`).
4. Else if the char before is `]`, match the balanced bracket group to its
   matching `[` one C# attribute group (may be multi-line).
5. Else if the char before is an alpha/underscore, walk back and check the
   identifier is a modifier keyword (`public` / `private` / `protected` /
   `export` / `abstract` / `static` / `sealed` / `partial` / `internal` /
   `final`); if so, repeat from step 2 for the token before the modifier. If
   the reached token is NOT an annotation (`@`/`[`), no group precedes `→`.
   return the original `type_keyword_pos` (the fallback unchanged behavior
   for non-decorated classes). Use the `modifiers` lists in
   `compaction::modifiers` as the authoritative keyword set.
6. Once an `@` / `[` is found, continue backward over BOTH annotations and
   modifiers until the previous non-whitespace token is not a continuation —
   stacked C# attributes (`[A]\n[B]\npublic class`) and Java annotations.
7. Return the final `start`.

The end of the span is always `end = cap.start_byte + cap.raw_text.len()`.

The canonical slice helper (used by EVERY consumer):

```rust
/// Canonical class-source text for a type capture.
/// Locates the type keyword inside `cap.raw_text`, maps it to an absolute
/// byte position, calls `find_class_source_start`, and returns
/// `&source[span_start .. cap.start_byte + cap.raw_text.len()]`.
pub fn class_source_from_capture(source: &str, cap: &CapEntry) -> &str;
```

The slice includes the declaration keyword through the closing `}` because
the meta-layer extractors scan the class body for method/field-level
decorators (`decorators.rs:133-157`, `annotations.rs:126-155`,
`general.rs:235-266`).

### 4.2 Required span per language

| Language | Leading-node shape | Required span_start | Example |
|---|---|---|---|
| TS/Angular | `(decorator)` sibling of `class_declaration` always a call | first `@` | `@Component({...})\nexport class UserComponent {` |
| Java/Spring | `@RestController`, `@Service`, `@RequestMapping("/api")` bare or applied; may be multiple, span lines | first `@` | `@RestController\npublic class UserController extends Base {` |
| C#/.NET | `[ApiController]`, `[Route("...")]`, `[Authorize]`, `[HttpGet]` | first `[` until `]` (matching bracket) | `[ApiController]\n[Route("api/foo")]\npublic class FooController : ControllerBase {` |

Contract:

```
canonical_class_source(source, cap):
    type_pos = cap.start_byte + (index of the type keyword in cap.raw_text)
    start    = find_class_source_start(source, type_pos)
    end      = cap.start_byte + cap.raw_text.len()
    return source[start..end]
```

If no annotation/attribute group precedes, `start` = the declaration-keyword
position identical to today's fallback for non-decorated classes `→`. no
behavior change.

---

## 5. C-22 Canonical Implementation Plan (design only not implemented)

Consumers:

| Consumer | Today | With design |
|---|---|---|
| `compression/pipeline.rs::build_output_lines` class.root arm (L384-399) | private `decorator_inclusive_class_text` (TS-only) | canonical `class_source_text(source, &cap)` (trilingual) |
| `ir/pipeline.rs::CoreIRPass` (L436-678) | keeps batch in a local; `state.captures` never populated | after the loop: `state.captures = captures;` (MOVE same identity, no clone) |
| `ir/pipeline.rs::MetaLayerPass` (L728-746) | `CoreOp::DefClass.name` (compacted) | filters `state.captures` for type roots; calls `class_source_text` `→`. registry |
| `mcp/workspace_util.rs::extract_class_blocks` (L205-239) | TS-only `find_decorator_inclusive_start` | same new helper (trilingual) |

The registry API `run_meta_layers_pipeline(source, class_captures: &[String],
…)` stays **identical** only the caller-side text changes from names to
full source.

### 5.1 Change list (design)

**A. `src/meta_util.rs`**
- Extend `find_decorator_inclusive_start` (now a thin wrapper) OR replace it
  with `find_class_source_start` + `class_source_text<'a>(source, &CapEntry)`.
- Keep the current TS path behavior byte-for-byte for decorated TS —
  existing Angular tests stay green (e.g.
  `src/tests/compression/pipeline.rs::compress_text_emits_angular_component_markers`).

**B. `src/ir/pipeline.rs` reuse the existing capture identity (C-22)**
- No new `PassContext` fields. The batch remains a local during the capture
  loop (the loop borrows it while `state` is mutated). When the loop ends,
  MOVE it into the existing owned field no clone, no parallel vector, no
  borrow split:

  ```rust
  for cap in &captures {
      // unchanged existing loop body: DefClass emission, language-layer
      // ops, control-flow flag accumulation
  }
  state.flush_method_flags();

  // C-22: persist the canonical capture identity for MetaLayerPass
  state.captures = captures;
  ```

- `MetaLayerPass` replaces its `DefClass.name` extraction (L728-746) with a
  span derivation directly from the persisted captures:

  ```rust
  let class_captures: Vec<String> = state
      .captures
      .iter()
      .filter(|cap| {
          matches!(
              cap.name.as_str(),
              "class.root" | "interface.root" | "struct.root" | "enum.root"
                | "trait.root" | "record.root" | "impl.root"
          )
      })
      .map(|cap| {
          // canonical helper: type keyword `→`. find_class_source_start `→`. slice
          class_source_from_capture(&state.source, cap).to_string()
      })
      .collect();
  ```

  Ordering is identical: the batch was sorted by `start_byte` in
  `run_capture_pipeline`, and `DefClass` ops are appended in that same
  capture order so the filtered slice order matches the instruction order.

**C. `src/compression/pipeline.rs`**
- Replace the private `decorator_inclusive_class_text` (L793-814) call in
  `build_output_lines` with the canonical `class_source_from_capture`.
- Delete the private helper (only duplicate TS-only special case —
  the single-source contract eliminates it).

**D. `src/mcp/workspace_util.rs`**
- `extract_class_blocks` switch from `find_decorator_inclusive_start` to
  the new trilingual helper. Preserves TS block extraction behavior.

**E. Tests** (per project rules `→`. `src/tests/`)
- `src/tests/meta_util/class_source_span.rs` (new): TS/Java/C#,
  bare annotations, multi-line attributes, modifiers, non-decorated.
- `src/tests/compression/pipeline.rs`: add
  `compress_text_csharp_emits_dotnet_meta` and
  `compress_text_java_emits_spring_annotations` production-path, NOT
  hand-constructed CapEntry (matches the Angular regression-test style
  at lines 497-524).
- `src/tests/ir/pipeline.rs`: IR-path marker tests + the new C-22 regression
  (see §8).

---

## 6. Impact on MetaLayerPass / CompiledIR / CoreOp::DefClass / CapEntry

| Artifact | Impact |
|---|---|
| `MetaLayerPass` | MUST change feed from `state.captures` (`Vec<CapEntry>`) instead of `CoreOp::DefClass.name`. |
| `PassContext` | No new fields. Existing `captures: Vec<CapEntry>` (pipeline.rs:95-96) becomes POPULATED (was always empty) via a single move after the loop. |
| `CompiledIR` | ZERO no field, no wire change. Markers stay `TypeAlias` rows inside `instructions`. |
| `CoreOp::DefClass` | ZERO still the compacted name; no span in the opcode. Identities come from `PassContext.captures`. |
| `CapEntry` | ZERO `raw_text` + `start_byte` already carry identity + source location; we merely PRESERVE them through the pipeline. |
| `registry.rs::run_meta_layers_pipeline` | ZERO signature/contract unchanged; only the caller supplies full slices now. |
| `compiler.rs` / `tool_helpers.rs` / `core.rs` | ZERO all internal to the pipeline. |
| Symbol table | ZERO not extended; no `SymbolInfo` change. |
| Performance | One O(L) backward scan per class; slices borrowed (no extra allocation of new Vectors); no re-parse; the batch move is O(1). |

---

## 7. Behavior Preservation & Non-Decorated Classes

Non-decorated classes must remain byte-identical:

| Case | Before (today) | After (canonical) |
|---|---|---|
| TS `export class Foo {}` | `cap.text` = `"Foo"` | helper returns `class_pos` `→`. `"Foo"` unchanged |
| Java `class Foo {}` | `"Foo"` | `class_pos` `→`. `"Foo"` unchanged |
| C# `public class Foo {}` | `"Foo"` | `class_pos` `→`. `"Foo"` (fallback) unchanged |
| C# `[ApiController] public class Foo : Base {}` | `"Foo"` (dead no marker) | annotation/attribute-inclusive slice `→`. `Φaspnet` new signal |
| TS `@Component(...)` | decorator slice (works) | same (decorator-only change) |

Rule: modifiers between the annotation group and the `class` keyword remain
**inside** the slice; modifiers with NO annotation group stay **outside**
(identical to today). The annotation group (`@` / `[`) is the span start; the
`class` keyword is never the span start when an annotation group exists.

The IR instructions themselves are untouched only the meta-layer FEED
changes.

---

## 8. Regression Tests (full-production path)

1. IR C#: `compile_file_ir` on an ASP.NET controller
   (`[ApiController] [Route("api/foo")] public class FooController : ControllerBase`)
   `→`. `CompiledIR.instructions` contains the ASP.NET `TypeAlias` marker row.
2. IR Java: `@RestController` `→`. Spring `TypeAlias`.
3. IR Angular: the existing `@Component` fixture driven through
   `compile_file_ir` `→`. the Angular marker is now present (was dead).
4. Text C#: `compress_text` / `compress_file` `→`. `.NET` Φ block.
5. Text Java: `compress_text` `→`. `Φrest`.
6. Non-decorated at IR+TEXT (plain TS/Java/C#) `→`. no new markers; output
   byte-identical to baseline.
7. Render E2E: `handle_provide_code_context` at High `→`. SCHEMA v2 render
   contains the meta block.
8. Stream parity: `compress_file_streaming` on the C#/Java fixtures.
9. Cache-hit: at Low/Medium a cache-HIT response isn't changed by the new Φ
   blocks (meta pipeline only runs on the miss path).
10. Scope guard: `.rs` / `.java` stay absent from `COMPRESSIBLE_EXTENSIONS`
    in this P0.
11. **C-22 identity regression:** after `CoreIRPass` runs on a fixture,
    `state.captures` equals the capture batch produced by
    `run_capture_pipeline` (asserts the identity is preserved the move,
    not a re-derivation); and `MetaLayerPass` derives spans matching
    `source[span_start..cap.start_byte+cap.raw_text.len()]` for every
    type-root capture.

All tests under `src/tests/` per the existing `#[path]` modules.

---

## 9. Alternatives Considered / Risks

### Alternative A Feed `cap.raw_text` directly to the meta-layer
- `raw_text` does NOT include the leading annotation group (tree-sitter
  sibling nodes). Even if a grammar embedded them the contract would become
  language-dependent. **Rejected.**

### Alternative B Query-level annotation captures
- The TS query already has `(decorator) @decorator.root` (queries.rs:43).
  Adding `(annotation) @annotation.root` / `(attribute_list) @attribute.root`
  and joining post-capture changes the query contract for all 3 languages,
  needs join logic in 3 places, and shifts parse risk. **Not selected.**

### Alternative C Re-parse the AST inside the meta-pass
- Inefficient, duplicates the grammar walk, re-validates parser errors.
  **No.**

### Alternative D Parallel `PassContext.class_sources: Vec<String>`
- A new append-only vector aligned with DefClass order. Would introduce a
  NEW parallel-vector invariant, **not** reuse the existing capture identity.
  Rejected under C-22: the capture already contains the only
  requirement identity and source location.

### Alternative E Extend the symbol table / `SymbolInfo` with a source span
- Carries a transient span into a long-lived identity table, and still needs
  an ordered enumeration when feeding `&[String]` to the registry. Rejected.

### Alternative F Encode the span into `CoreOp::DefClass`
- Changes the IR wire encoding for a pipeline-transient value. Rejected
  (and separately disallowed by the user's doctrine).

### Risks
- **C# attribute span**: `[ApiController]` literal `[`/`]` backward scan
  with `find_matching_brace` (string-aware, bracket-depth). Robust to
  multi-line `[ApiController,\n Route(..)]`.
- **Java multi-line annotations**: `@RequestMapping(...)` with newline args —
  handled by the balanced-paren step (string-aware).
- **Java member annotations**: guard against a stray `)` from a *method* —
  only descend a balanced paren group directly attached to an `@` identifier.
- **Modifier chain**: only walk known modifier keywords; stop otherwise;
  confirm against `compaction::modifiers`.
- **Byte-exactness**: non-annotated classes keep the class-keyword fallback;
  the IR instructions are untouched; only the meta-layer feed slice changes
  for annotated classes. Existing Angular text fixtures stay green.

---

## 10. New Architectural Invariant: C-22

**C-22 Meta-layer source context MUST be derived from the canonical
`CapEntry` capture identity.**

Meta-layer source context must be derived from the canonical `CapEntry`
capture identity already established by the capture pipeline.
`PassContext.captures` is the source of truth for associating class
declarations with their source spans.

Do not:

- maintain a parallel source vector (`Vec<String>` class_sources),
- encode transient source spans into `CoreOp::DefClass`,
- or extend `SymbolInfo` / the symbol table for transient spans.

Prefer: `CoreIRPass` MOVES the batch into the existing `state.captures`
field; `MetaLayerPass` filters the type-root captures and derives the span
from `cap.start_byte` + `cap.raw_text.len()` + canonical
`find_class_source_start(...)`.

**The capture already contains the identity; preserve and reuse it rather
than creating a second identity mechanism.**

---

## 11. Recommended Next Steps

1. Implement `find_class_source_start` + `class_source_text` (+
   `ClassSourceSpan` if desired) and unit tests in `src/meta_util.rs`
   (TS/Java/C#, attribute groups, modifiers, non-decorated).
2. Wire the IR path: `CoreIRPass` ends with `state.captures = captures;`;
   `MetaLayerPass` derives `class_captures` from `state.captures` (C-22);
   remove the `DefClass.name` filter.
3. Update the text/streaming/workspace consumers to the shared helper and
   remove the private TS-ONLY `decorator_inclusive_class_text`.
4. Add the production-path regression tests (incl. C-22 identity regression).
5. Full verification: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --workspace`.

---

This P0 design satisfies:

- one canonical class-source contract IR and text paths share `class_source_text`;
- reuses the existing capture identity `PassContext.captures`, NOT a parallel
  vector (C-22);
- preserved `CompiledIR` / `CoreOp::DefClass` / symbol-table / registry API;
- non-decorated behavior unchanged;
- no inference-layer / CBM involvement; and
- Angular / Spring / .NET semantics restored on BOTH the IR path (via
  `state.captures`) and the text path (via the shared trilingual helper).

<!-- End of design doc -->

---

## 12. Implementation Status

All 5 recommended steps from Section 11 have been implemented:

| Step | Implementation | Commit |
|------|---------------|--------|
| 1. `find_class_source_start` + `class_source_text` + unit tests in `src/meta_util.rs` (TS/Java/C#) | `class_source_from_capture()` in `src/meta_util.rs` with trilingual backward scan | `c422961` |
| 2. Wire IR path: `CoreIRPass` ends with `state.captures = captures`; `MetaLayerPass` derives `class_captures` from `state.captures` (C-22) | `MetaLayerPass::run()` in `src/ir/pipeline.rs:735-758` filters type-root captures from `state.captures` | `c422961` |
| 3. Update text/streaming/workspace consumers to shared helper; remove private TS-only `decorator_inclusive_class_text` | `LayerRegistry::run_meta_layers_pipeline()` receives `class_captures: &[String]` directly via `MetaLayer::enrich()` | `0552e8a` |
| 4. Production-path regression tests (including C-22 identity regression) | `class_source_from_capture_c22_identity` + multi-class cross-contamination tests (9 tests across Angular/Spring/.NET at Low/Medium/High) | `0552e8a`, `8869e68` |
| 5. Full verification | `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace` — all green | `0552e8a` |

**Architectural invariant C-22** is formally documented in `docs/ARCHITECTURAL_INVARIANTS.md` and enforced by the production pipeline.
