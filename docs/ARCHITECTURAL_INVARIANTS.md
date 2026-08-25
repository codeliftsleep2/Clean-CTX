# Clean-CTX Architectural Invariants

**Purpose:** Make Clean-CTX's important architectural decisions visible, identify how they are currently enforced (type system, compiler, tests, or convention), and establish a pattern for future architectural governance.

**Audience:** Developers contributing to the Clean-CTX codebase.

---

## Status Classifications

| Status | Meaning |
|--------|---------|
| **ENFORCED** | Actively enforced by tests or tooling. Failure blocks CI. |
| **STRUCTURAL** | Enforced by Rust's type system or compiler. Violation requires changing type signatures. |
| **DOCUMENTED** | Architectural convention currently not machine-enforced. Violation is possible but should trigger a design discussion. |
| **DEFERRED** | Important architectural decision intentionally postponed. |
| **PROPOSED** | Under consideration but not yet accepted. |
| **RESOLVED** | Previously documented architectural debt that has been completed. |

## Architectural Gate

The architectural gate is the existing CI pipeline:

```
cargo test
+
cargo clippy --all-targets -- -D warnings
```

No separate executable, trait, registry, or framework is used. Each invariant below identifies which part of the gate enforces it.

---

## Invariant Catalog

### WIRE-001 Canonical IR Serialization Stability

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must survive serialization/deserialization without semantic or structural loss. |
| **Invariant** | Encoding and decoding a valid `CompiledIR` preserves its canonical instruction stream across all supported wire formats. |
| **Enforcement** | Property tests (100 random seeds) for named wire, binary wire, hierarchical wire, and compact delta formats. All 20 `CoreOp` variants are covered. Determinism and double-encode stability are also verified. |
| **Authority** | `src/tests/ir/round_trip.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### VALID-001 IR Structural Validity

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must not contain invalid references or structurally inconsistent instructions. |
| **Invariant** | Valid IR passes `DefaultValidator` without E001–E010 violations. Invalid IR (dangling references, orphaned methods, inconsistent effect/context annotations) is detected. |
| **Enforcement** | `DefaultValidator` implementing `IRValidator` trait. 10 unit tests (one per rule) plus edge-case tests for empty IR and error display. |
| **Authority** | `src/ir/validator.rs` (rules), `src/tests/ir/validator.rs` (tests) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### DELTA-001 Delta Correctness

| Property | Value |
|----------|-------|
| **Intent** | Applying a computed delta between two `CompiledIR` states must reproduce the intended semantics of the target state. |
| **Invariant** | `DeltaComputer::compute(baseline, current)` produces a `Some(delta)` when the two IRs differ, and `None` when they are identical. The delta correctly identifies additions, modifications, and deletions. The delta preserves version chain (`from` / `to`). |
| **Enforcement** | Unit tests covering: add detection, removal detection, modification detection (renamed methods, changed types), identical-IR returns None, version chain correctness, JSON serialization with `+`/`~`/`-` keys, and edge cases (empty IRs, different files, duplicate keys). |
| **Authority** | `src/tests/ir/delta.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### ARCH-001 Inference State Is Ephemeral

| Property | Value |
|----------|-------|
| **Intent** | Inference-layer state must not become part of canonical serialized IR. |
| **Invariant** | `InferenceLayer` is structurally separate from `CompiledIR`. There is no canonical serialization path that includes `InferenceLayer` data. |
| **Enforcement** | Rust type system. `CompiledIR` has no field of type `InferenceLayer`. Serialization functions (`ir_to_wire`, `encode`, `ir_to_string_table_wire`, etc.) operate on `CompiledIR` only and cannot access `InferenceLayer`. Violating this invariant requires deliberately changing type signatures. |
| **Authority** | `src/ir/compiler.rs` (`CompiledIR`), `src/ir/inference_layer.rs` (`InferenceLayer`), `src/ir/wire.rs`, `src/ir/binary_wire.rs` (serialization) |
| **Type** | STRUCTURAL (type system) |
| **Gate** | Rust compiler |

---

### ARCH-002 Language-Agnostic Canonical IR Boundary

| Property | Value |
|----------|-------|
| **Intent** | Language-specific layers and meta-layers ultimately produce canonical `CompiledIR` / `CoreOp` representations, enabling common architectural invariants to apply regardless of which source language produced the IR. |
| **Invariant** | All language layers (TypeScript, C#, Rust, Java) emit `CoreOp` instructions compatible with the canonical instruction stream. Meta-layers enrich the compressed output but participate in the same `CompiledIR` representation. |
| **Enforcement** | Language conformance tests compile source to `CompiledIR` and verify expected `CoreOp` structure. The validator and round-trip tests operate on the resulting `CompiledIR` without language-specific knowledge. Feature gates (`#[cfg(feature = "...")]`) ensure language layers are compiled only when the corresponding tree-sitter grammar is available. |
| **Authority** | `src/tests/ir/compiler.rs` (TypeScript conformance), `src/tests/ir/rust_integration.rs` (Rust conformance), `src/tests/ir/layers_integration.rs` (C# + layers), `src/layers/registry.rs` (feature-gated registration) |
| **Type** | ENFORCED (test architecture) |
| **Gate** | `cargo test --all-features` |

---

### PIPELINE-001 Compilation Pipeline Ordering

| Property | Value |
|----------|-------|
| **Intent** | Compilation stages must execute in a known architectural order. |
| **Invariant** | The production `PassPipeline` must register passes in the required order: `CoreIRPass` `→`. `LanguageLayerPass` `→`. `MetaLayerPass` `→`. `PatternRecognitionPass` `→`. `AliasResolutionPass` `→`. `ValidationPass`. This ordering reflects the data and semantic dependencies between stages. |
| **Enforcement** | `production_pipeline_preserves_architectural_order` test in `src/tests/ir/pipeline.rs` asserts the exact pass sequence via `PassPipeline::pass_names()`. |
| **Authority** | `src/ir/pipeline.rs` (`PassPipeline::default_production()`), `src/tests/ir/pipeline.rs` (ordering test) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

**Rationale for ordering:**
- **CoreIR** must precede **Language Finalize**: The core capture/emission phase must process all captures before language-layer finalization occurs.
- **Language Finalize** must precede **Meta Layer**: Meta layers depend on the instruction stream after language-layer processing.
- **Meta Layer** must precede **Pattern Recognition**: Pattern recognition operates on the complete instruction stream including meta-layer output.
- **Pattern Recognition** must precede **Alias Resolution**: Alias resolution must see all relevant `Extends`/`Implements` instructions after pattern processing.
- **Alias Resolution** must precede **Validation**: Validation must inspect the final canonical instruction stream after all transformations.

### C-22 — Meta-Layer Source Context from Canonical Capture Identity

| Property | Value |
|----------|-------|
| **Intent** | Meta-layer source context MUST be derived from the canonical `CapEntry` capture identity — NOT from the compacted `CoreOp::DefClass.name`. |
| **Invariant** | `MetaLayerPass` derives each class capture's canonical source span from `PassContext.captures` (the persisted capture identity). `class_source_from_capture()` produces the decorator/annotation/attribute-inclusive class text (TS `@Name(...)`, Java `@Name`, C# `[Name]`). The `MetaLayer::enrich()` trait receives `class_captures: &[String]` directly — no `DefClass.name` round-trip. Non-decorated classes use the declaration-keyword byte as fallback (backward compatible). |
| **Enforcement** | `class_source_from_capture_c22_identity` test asserts `class_source_from_capture` reconstructs the capture from source + `CapEntry`. `MetaLayerPass::run()` in `src/ir/pipeline.rs` filters type-root captures from `state.captures`. Multi-class cross-contamination tests (9 tests across Angular/Spring/.NET at Low/Medium/High) verify per-class isolation — a class's `@Component`/`@RestController`/`[ApiController]` marker never leaks to sibling classes. |
| **Authority** | `src/meta_util.rs` (`class_source_from_capture`), `src/ir/pipeline.rs` (`MetaLayerPass::run`), `src/layers/registry.rs` (`run_meta_layers_pipeline`), `src/tests/meta_util.rs` (C-22 identity test), `src/tests/compression/pipeline.rs` (multi-class tests) |
| **Type** | ENFORCED (test + structural) |
| **Gate** | `cargo test` |

---

### CBM-ID-001 Canonical CBM Project Identity & Multi-Root Lifecycle

| Property | Value |
|----------|-------|
| **Intent** | Every CBM interaction — indexing, readiness, querying, proxy routing, and cache partitioning — must address the project CBM actually indexed, regardless of how many repos are configured. |
| **Invariant** | **Never derive or invent a CBM project identifier independently of the canonical-root mapping.** CBM's canonical project slug is the single source of identity for indexing, readiness, querying, proxy routing, and cache partitioning. Specifically: (1) A CBM project identity is the slug derived from the canonical repo path (`cbm_project_slug()`), never a directory basename. (2) Every configured root (primary + `additional_roots`) maps to its own CBM project ID via the bridge's two-way identity map (`project_ids` / `project_paths`). (3) One CBM subprocess serves all configured roots. (4) Indexing begins asynchronously at bridge construction for every root (`start_indexing_roots()`). (5) Indexing/readiness state is tracked independently per CBM project; untracked projects pass through as ready rather than dead-ending in a permanent gate. (6) Graph queries and `cbm_proxy` resolve targets through the root/project mapping (`resolve_project_id`) and never invent a dirname-based identity. (7) Project-independent CBM tools (e.g. `list_projects`) bypass the indexing gate entirely. (8) The verified CBM 0.8.1 wire contract is preserved: `index_repository(repo_path, mode)` takes no project parameter — CBM derives the ID from the canonical path. |
| **Enforcement** | Regression tests covering: slug fidelity against live-captured CBM responses; per-root registration for primary + additional roots; dirname/path overrides canonicalizing instead of diverging; per-project readiness isolation with untracked pass-through; single-root backward compatibility; proxy gate scoping (project-less calls skip the gate). |
| **Authority** | `src/cbm/bridge.rs` (`cbm_project_slug`, `try_create_with_roots`, `resolve_project_id`, `ensure_indexed_for`), `src/cbm/proxy.rs` (`resolve_proxy_target_project`), `src/tests/cbm/regression.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

### CBM-E-001 Explicit CBM Error Propagation

| Property | Value |
|----------|-------|
| **Intent** | CBM unavailability or failure must never masquerade as legitimate empty graph data. |
| **Invariant** | Every graph-intelligence bridge method returns `Result<_, CbmError>`. `Ok(empty)` is reserved for valid zero-result queries; any CBM-reported tool failure (`result.isError` envelope), transport fault, timeout, or open circuit surfaces as `Err(CbmError)`. Downstream consumers (intelligence layer, inference pass, MCP handlers) propagate or explicitly handle `Err`; none may convert it into empty success data. The pipeline-level failure policy is fixed: log loudly and continue without enrichment - CBM is strictly additive to the IR. |
| **Enforcement** | `check_soft_error()` maps isError envelopes to `CbmError::ToolError` in the parsed transport path before callers observe them; deterministic fixtures pin the envelope shape; live probes assert `Err` on unknown projects vs `Ok(empty)` for valid no-result queries. |
| **Authority** | `src/cbm/client.rs` (`CbmError::ToolError`, `check_soft_error`), `src/cbm/bridge.rs` (Result signatures), `src/ir/inference_layer.rs`, `src/ir/pipeline.rs`, `src/tests/cbm/graph_intel.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

### CBM-WIRE-001 Verified CBM `trace_path` Wire Contract

| Property | Value |
|----------|-------|
| **Intent** | The typed `graph_trace` path must consume what CBM actually emits on the wire — never a presumed shape — so real call-relationship data reaches agents instead of silently collapsing to zero results while the raw proxy path works. |
| **Invariant** | **`inner["edges"]` is NOT a valid CBM 0.8.1 `trace_path` response shape and must never be assumed** by any parser, wrapper, or fixture. The verified contract (verbatim live captures from a fresh subprocess, 2026-08-24): relationships arrive as directional `callers` / `callees` ARRAYS whose entries carry exactly `name`, `qualified_name`, and `hop` (a JSON number); an empty half is a real empty array, never a missing key and never an `edges` key. Specifically: (1) **Directionality** — every `callers[i]` calls the traced function, and the traced function calls every `callees[i]`; normalized edges always orient caller → callee regardless of which array produced them. (2) **Canonical identity** — edge endpoints are `qualified_name` with fallback to the bare `name` (`map_search_result` precedent); `hop` is dropped: a `GraphEdge` represents a relationship, not traversal metadata. Note the raw wire key is `name` — `nm` exists only in Clean-CTX's own compressed proxy view. (3) **Boundary matching** — the API accepts bare names while the wire carries qualified names, so a target matches a canonical endpoint when the endpoint EQUALS the target (fully qualified form) OR its FINAL DOT SEGMENT equals the bare target; a bare `to` name with qualified wire endpoints MUST retain the edge; partial/multi-segment targets match nothing. `__file__` module pseudo-node callers are genuine relationships and pass through untouched. (4) **Both directions work** — outbound-reachable pairs resolve on the FIRST attempt (pre-fix behavior preserved byte-for-byte); inbound-only relationships are discovered through a SINGLE inbound fallback taken only when the outbound attempt succeeds but yields no edge touching the target; errors are never swapped for the other direction. (5) **Depth honesty** — depth>1 responses are FLAT hop-tagged BFS discoveries with NO parent linkage; only `hop: 1` entries convert into edges; `hop > 1` entries are unattributable and must NEVER become invented edges (they remain available via `cbm_proxy`). (6) **Ordering & dedup** — CBM emission order is preserved; ONLY exact duplicate edges may collapse, preserving first-seen order; repeated nodes are not relationships and are never merged away. (7) **Result semantics** — a valid query yielding no relationships remains `Ok(empty)`; any CBM soft error (`result.isError` envelope, e.g. `"error": "function not found"`) propagates as an explicit `Err(CbmError::ToolError)` BEFORE parsing — failure is never a valid empty result (normative generalization: CBM-E-001). |
| **Enforcement** | Deterministic pins against verbatim raw captures (`TRACE_INBOUND_WIRE_CAPTURE`, `TRACE_OUTBOUND_WIRE_CAPTURE`, `TRACE_BOTH_WIRE_CAPTURE`, `TRACE_DEEP_OUTBOUND_WIRE_CAPTURE`, `TRACE_NOT_FOUND_RESULT_ENVELOPE`): directional synthesis, hop-1-only conversion (the depth-3 capture proves flat semantics and cross-hop node repeats), qualified→name fallback, exact-duplicate dedupe ordering, absent-array leniency, not-found → `ToolError` gate, boundary predicate strictness (exact/final-segment retained; partial/multi-segment rejected; `regression_bare_to_name_with_qualified_endpoint_is_retained`). Four fresh-process `serial(cbm_live)` probes over a SYNTHETIC temp-dir fixture repo (caller → callee is the only relationship; nothing derives from this repository): typed `graph_query` CALLS rows, two-endpoint outbound preservation, single-endpoint inbound discovery, and THE regression — two-endpoint inbound-only discovery via the fallback. Finalization evidence: `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` zero warnings; `cargo test --workspace --all-targets --all-features` 2,497 passed / 0 failed / 5 ignored including `e2e_cbm_multiroot_multilingual_integration` green against live CBM 0.8.1 (commit `193f885`). |
| **Authority** | `src/cbm/client.rs` (`extract_trace_edges`, `trace_entry_endpoint`, `trace_entry_is_direct`), `src/cbm/bridge.rs` (`GraphBridge::trace_path` direction determination, `edge_touches_target`, `filter_trace_edges`), `src/tests/cbm/trace_wire.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

### CBM-WIRE-002 Verified CBM `query_graph` Wire Contract & Strict Positional Edge Extraction

| Property | Value |
|----------|-------|
| **Intent** | The typed `graph_query` path must surface relationship data that CBM actually returns instead of collapsing it into duplicated column-0 nodes with a permanently empty edge list. |
| **Invariant** | CBM answers `query_graph` with `{columns, rows, total}` where `columns` echo the projection expressions verbatim (`"a.name"`, `"type(r)"`), cells are JSON strings, undirected `-[r]-` patterns are supported, and one result set may mix every relationship type (DEFINES / DECORATES / USAGE / CALLS — captured live 2026-08-24). Conversion follows the STRICT POSITIONAL convention: exactly three cells per row, uniform across ALL rows ⇒ one `GraphEdge{from=row[0], label=row[1], to=row[2]}` per row, no synthesized nodes; EVERY other projection shape (0/1/2/4+ cells, mixed arities, empty) keeps the legacy column-0 node mapping with no edges. Deliberately excluded from this contract (separate findings): node deduplication, file-path population, endpoint normalization — duplicates pass through untouched; qualified/bare M-01 matching does NOT apply here because graph_query performs no target filtering. Cache compatibility: populated edges reuse the existing serialized `edges` key — no key versioning. |
| **Enforcement** | Verbatim raw-row captures pin the conversion deterministically (directed CALLS bare names, undirected mixed types, qualified endpoints, two-column node-only control); synthetic policy pins cover non-triple arity fallback, uniform-arity requirement (one deviant row downgrades the whole set), empty results, and rider-out duplication pass-through. Two fresh-process `serial(cbm_live)` probes over the SYNTHETIC temp-dir fixture repo prove typed edges end-to-end (CALLS projections surface one edge per row with endpoint cells passed through exactly as projected — bare under `.name`, qualified under `.qualified_name` — and no invented nodes) plus preserved node-only behavior. Full finalization gate green (fmt clean; clippy `-D warnings` zero; workspace tests incl. live CBM probes + multilingual fixture). |
| **Authority** | `src/cbm/bridge.rs` (`convert_query_rows`, `GraphBridge::query_graph`), `src/tests/cbm/query_wire.rs`, captures archived under `target/tmp/gq_raw_out.txt` + `gq_v6_out.txt` (session artifacts) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

## Architectural Debt

### ARCH-DEBT-001 PassPipeline Migration (RESOLVED)

| Property | Value |
|----------|-------|
| **Description** | The `PassPipeline` migration from the monolithic `IRCompiler::compile_inner()` has been completed. `PassPipeline` is now the active production compilation path. |
| **Resolution** | `IRCompiler::compile_inner()` is now an orchestration boundary that constructs a `PassContext`, configures the `PassPipeline`, and delegates compilation to `PassPipeline::run()`. Individual compilation stages are implemented in their corresponding `IRPass` implementations in `src/ir/pipeline.rs`. |
| **Production pipeline order** | `CoreIRPass` `→`. `LanguageLayerPass` `→`. `MetaLayerPass` `→`. `PatternRecognitionPass` `→`. `AliasResolutionPass` `→`. `ValidationPass` |
| **Optional passes** | `ExecutionSemanticsPass`, `ProgramGraphPass`, `InferenceLayerPass` remain outside the default production pipeline. |
| **See also** | `src/ir/pipeline.rs`, `src/ir/compiler.rs`, `docs/ARCHITECTURAL_INVARIANTS.md` (PIPELINE-001) |

---

## How to Add a New Invariant

When a new architectural invariant is needed:

1. **First:** Can the Rust type system or compiler enforce it? If yes, do that (classify as STRUCTURAL).
2. **Second:** Does an existing test already cover it? If yes, document it here (classify as ENFORCED).
3. **Third:** Can Clippy or `cargo check` enforce it? If yes, add the appropriate lint (classify as STRUCTURAL).
4. **Only if none of the above suffice:** Add a new `#[test]` function (classify as ENFORCED).

Do not create a fitness-function framework, trait, registry, or gate abstraction. The architectural gate is `cargo test` + `cargo clippy --all-targets -- -D warnings`.

---

## Invariants That Are NOT Documented Here

The following are important architectural properties but are **not** formalized as architectural invariants:

- **Module dependency direction:** Currently enforced by Rust's module and visibility system within a single crate. The existing dependency patterns (MCP → IR, IR → compression, no reverse dependencies) are healthy but not independently tested. If a dependency becomes important enough to require hard enforcement, the appropriate mechanism is splitting into separate crates.
- **Meta-layer additivity:** Meta-layers currently append to compressed output rather than modifying it. However, the `MetaLayer::enrich()` trait signature permits modification, and "additivity" has not been established as a formal architectural contract. This is a candidate for future formalization if the contract is explicitly defined.
- **Meta-layer per-class source isolation:** Previously an uncovered concern — each meta-layer could accidentally inspect neighboring type declarations or whole-file text when trying to extract framework annotations. This is now **formally covered by C-22** (see above). The canonical capture path (`PassContext.captures` → `class_source_from_capture()` → `MetaLayer::enrich(class_captures)`) ensures that a meta-layer receives only the exact source span belonging to the type it is enriching. Multi-class cross-contamination tests (9 tests across Angular/Spring/.NET at all three fidelity levels) enforce this structurally: a class's `@Component` / `@RestController` / `[ApiController]` marker never leaks to sibling classes.