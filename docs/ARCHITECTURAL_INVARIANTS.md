# Clean-CTX — Architectural Invariants

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

### WIRE-001 — Canonical IR Serialization Stability

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must survive serialization/deserialization without semantic or structural loss. |
| **Invariant** | Encoding and decoding a valid `CompiledIR` preserves its canonical instruction stream across all supported wire formats. |
| **Enforcement** | Property tests (100 random seeds) for named wire, binary wire, hierarchical wire, and compact delta formats. All 20 `CoreOp` variants are covered. Determinism and double-encode stability are also verified. |
| **Authority** | `src/tests/ir/round_trip.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### VALID-001 — IR Structural Validity

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must not contain invalid references or structurally inconsistent instructions. |
| **Invariant** | Valid IR passes `DefaultValidator` without E001–E010 violations. Invalid IR (dangling references, orphaned methods, inconsistent effect/context annotations) is detected. |
| **Enforcement** | `DefaultValidator` implementing `IRValidator` trait. 10 unit tests (one per rule) plus edge-case tests for empty IR and error display. |
| **Authority** | `src/ir/validator.rs` (rules), `src/tests/ir/validator.rs` (tests) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### DELTA-001 — Delta Correctness

| Property | Value |
|----------|-------|
| **Intent** | Applying a computed delta between two `CompiledIR` states must reproduce the intended semantics of the target state. |
| **Invariant** | `DeltaComputer::compute(baseline, current)` produces a `Some(delta)` when the two IRs differ, and `None` when they are identical. The delta correctly identifies additions, modifications, and deletions. The delta preserves version chain (`from` / `to`). |
| **Enforcement** | Unit tests covering: add detection, removal detection, modification detection (renamed methods, changed types), identical-IR returns None, version chain correctness, JSON serialization with `+`/`~`/`-` keys, and edge cases (empty IRs, different files, duplicate keys). |
| **Authority** | `src/tests/ir/delta.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

### ARCH-001 — Inference State Is Ephemeral

| Property | Value |
|----------|-------|
| **Intent** | Inference-layer state must not become part of canonical serialized IR. |
| **Invariant** | `InferenceLayer` is structurally separate from `CompiledIR`. There is no canonical serialization path that includes `InferenceLayer` data. |
| **Enforcement** | Rust type system. `CompiledIR` has no field of type `InferenceLayer`. Serialization functions (`ir_to_wire`, `encode`, `ir_to_string_table_wire`, etc.) operate on `CompiledIR` only and cannot access `InferenceLayer`. Violating this invariant requires deliberately changing type signatures. |
| **Authority** | `src/ir/compiler.rs` (`CompiledIR`), `src/ir/inference_layer.rs` (`InferenceLayer`), `src/ir/wire.rs`, `src/ir/binary_wire.rs` (serialization) |
| **Type** | STRUCTURAL (type system) |
| **Gate** | Rust compiler |

---

### ARCH-002 — Language-Agnostic Canonical IR Boundary

| Property | Value |
|----------|-------|
| **Intent** | Language-specific layers and meta-layers ultimately produce canonical `CompiledIR` / `CoreOp` representations, enabling common architectural invariants to apply regardless of which source language produced the IR. |
| **Invariant** | All language layers (TypeScript, C#, Rust, Java) emit `CoreOp` instructions compatible with the canonical instruction stream. Meta-layers enrich the compressed output but participate in the same `CompiledIR` representation. |
| **Enforcement** | Language conformance tests compile source to `CompiledIR` and verify expected `CoreOp` structure. The validator and round-trip tests operate on the resulting `CompiledIR` without language-specific knowledge. Feature gates (`#[cfg(feature = "...")]`) ensure language layers are compiled only when the corresponding tree-sitter grammar is available. |
| **Authority** | `src/tests/ir/compiler.rs` (TypeScript conformance), `src/tests/ir/rust_integration.rs` (Rust conformance), `src/tests/ir/layers_integration.rs` (C# + layers), `src/layers/registry.rs` (feature-gated registration) |
| **Type** | ENFORCED (test architecture) |
| **Gate** | `cargo test --all-features` |

---

### PIPELINE-001 — Compilation Pipeline Ordering

| Property | Value |
|----------|-------|
| **Intent** | Compilation stages must execute in a known architectural order. |
| **Invariant** | The production `PassPipeline` must register passes in the required order: `CoreIRPass` → `LanguageLayerPass` → `MetaLayerPass` → `PatternRecognitionPass` → `AliasResolutionPass` → `ValidationPass`. This ordering reflects the data and semantic dependencies between stages. |
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

---

## Architectural Debt

### ARCH-DEBT-001 — PassPipeline Migration (RESOLVED)

| Property | Value |
|----------|-------|
| **Description** | The `PassPipeline` migration from the monolithic `IRCompiler::compile_inner()` has been completed. `PassPipeline` is now the active production compilation path. |
| **Resolution** | `IRCompiler::compile_inner()` is now an orchestration boundary that constructs a `PassContext`, configures the `PassPipeline`, and delegates compilation to `PassPipeline::run()`. Individual compilation stages are implemented in their corresponding `IRPass` implementations in `src/ir/pipeline.rs`. |
| **Production pipeline order** | `CoreIRPass` → `LanguageLayerPass` → `MetaLayerPass` → `PatternRecognitionPass` → `AliasResolutionPass` → `ValidationPass` |
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