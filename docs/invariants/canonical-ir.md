<!-- pcg:family-standing:begin -->
# Invariant family: canonical-ir

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:WIRE-001:begin -->
### WIRE-001 Canonical IR Serialization Stability

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must survive serialization/deserialization without semantic or structural loss. |
| **Invariant** | Encoding and decoding a valid `CompiledIR` preserves its canonical instruction stream across all supported wire formats. |
<!-- pcg:WIRE-001-verification:begin -->
| **Enforcement** | The current canonical enum has 30 CoreOp variants; physical Binary 0x04 has 31 opcode values because exact and spread Call use distinct opcodes. The dedicated binary_wire_v04 fixture covers all 30 variants and both Call forms with complete-value, order and duplicate assertions. The randomized named/binary generators cover 25 variants over 100 seeds, not the full current enum. Other legacy round-trip tests have format-specific and sometimes partial oracles; their existence does not prove complete all-format coverage. Determinism and double-encode assertions remain tracked. See the B0/B1 C13 source comparison for the exact coverage boundary; no test was rerun by this documentation migration. |
| **Authority** | `src/tests/ir/round_trip.rs`, `src/tests/ir/round_trip_randomized.rs`, `src/tests/ir/binary_wire_v04.rs`; current enum `src/ir/opcodes.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |
<!-- pcg:WIRE-001-verification:end -->

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 36-48, original section `WIRE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `916c7501d349592d39cd6a30f914e5143dabf83f9641756417233fe9ac5750d1`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: WIRE-001
kind: INVARIANT
status: IMPLEMENTED
title: WIRE-001 Canonical IR Serialization Stability
track: FOUNDATION
artifact: docs/invariants/canonical-ir.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#WIRE-001
  - qualification self#family-standing
  - verification_obligation self#WIRE-001-verification
```

<!-- pcg:WIRE-001:end -->
<!-- pcg:VALID-001:begin -->
### VALID-001 IR Structural Validity

| Property | Value |
|----------|-------|
| **Intent** | Canonical IR must not contain invalid references or structurally inconsistent instructions. |
| **Invariant** | Valid IR passes `DefaultValidator` without E001–E011 violations. Invalid IR (dangling references, orphaned methods/callers, inconsistent effect/context annotations) is detected. |
<!-- pcg:VALID-001-verification:begin -->
| **Enforcement** | `DefaultValidator` implementing `IRValidator`; tracked tests cover every rule, including E011 native-call caller ownership, plus empty-IR and error-display boundaries. |
| **Authority** | `src/ir/validator.rs` (rules), `src/tests/ir/validator.rs` (tests) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |
<!-- pcg:VALID-001-verification:end -->

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 49-61, original section `VALID-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `7ada783762c3d3d95c21b134ecc9845ea9f3f4033328fd0fd89a880ddcba7957`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: VALID-001
kind: INVARIANT
status: IMPLEMENTED
title: VALID-001 IR Structural Validity
track: FOUNDATION
artifact: docs/invariants/canonical-ir.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#VALID-001
  - qualification self#family-standing
  - verification_obligation self#VALID-001-verification
```

<!-- pcg:VALID-001:end -->
<!-- pcg:ARCH-001:begin -->
### ARCH-001 Inference State Is Ephemeral

| Property | Value |
|----------|-------|
| **Intent** | Inference-layer state must not become part of canonical serialized IR. |
| **Invariant** | `InferenceLayer` is structurally separate from `CompiledIR`. There is no canonical serialization path that includes `InferenceLayer` data. |
<!-- pcg:ARCH-001-verification:begin -->
| **Enforcement** | Rust type system. `CompiledIR` has no field of type `InferenceLayer`. Serialization functions (`ir_to_wire`, `encode`, `ir_to_string_table_wire`, etc.) operate on `CompiledIR` only and cannot access `InferenceLayer`. Violating this invariant requires deliberately changing type signatures. |
| **Authority** | `src/ir/compiler.rs` (`CompiledIR`), `src/ir/inference_layer.rs` (`InferenceLayer`), `src/ir/wire.rs`, `src/ir/binary_wire.rs` (serialization) |
| **Type** | STRUCTURAL (type system) |
| **Gate** | Rust compiler |
<!-- pcg:ARCH-001-verification:end -->

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 75-87, original section `ARCH-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `ed936b4563c4a2452ae22ef2136fa8bca62db048e84661b1cf492642fe01471a`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: ARCH-001
kind: INVARIANT
status: IMPLEMENTED
title: ARCH-001 Inference State Is Ephemeral
track: FOUNDATION
artifact: docs/invariants/canonical-ir.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#ARCH-001
  - qualification self#family-standing
  - verification_obligation self#ARCH-001-verification
```

<!-- pcg:ARCH-001:end -->
<!-- pcg:ARCH-002:begin -->
### ARCH-002 Language-Agnostic Canonical IR Boundary

| Property | Value |
|----------|-------|
| **Intent** | Language-specific layers and meta-layers ultimately produce canonical `CompiledIR` / `CoreOp` representations, enabling common architectural invariants to apply regardless of which source language produced the IR. |
| **Invariant** | All language layers (TypeScript, C#, Rust, Java) emit `CoreOp` instructions compatible with the canonical instruction stream. Meta-layers enrich the compressed output but participate in the same `CompiledIR` representation. |
<!-- pcg:ARCH-002-verification:begin -->
| **Enforcement** | Language conformance tests compile source to `CompiledIR` and verify expected `CoreOp` structure. The validator and round-trip tests operate on the resulting `CompiledIR` without language-specific knowledge. Feature gates (`#[cfg(feature = "...")]`) ensure language layers are compiled only when the corresponding tree-sitter grammar is available. |
| **Authority** | `src/tests/ir/compiler.rs` (TypeScript conformance), `src/tests/ir/rust_integration.rs` (Rust conformance), `src/tests/ir/layers_integration.rs` (C# + layers), `src/layers/registry.rs` (feature-gated registration) |
| **Type** | ENFORCED (test architecture) |
| **Gate** | `cargo test --all-features` |
<!-- pcg:ARCH-002-verification:end -->

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 88-100, original section `ARCH-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `8a04810ca9448f2c59b62a28422604a34be787dc40b332c0694cbf5103e9f6d3`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: ARCH-002
kind: INVARIANT
status: IMPLEMENTED
title: ARCH-002 Language-Agnostic Canonical IR Boundary
track: FOUNDATION
artifact: docs/invariants/canonical-ir.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#ARCH-002
  - qualification self#family-standing
  - verification_obligation self#ARCH-002-verification
```

<!-- pcg:ARCH-002:end -->
<!-- pcg:PIPELINE-001:begin -->
### PIPELINE-001 Compilation Pipeline Ordering

| Property | Value |
|----------|-------|
| **Intent** | Compilation stages must execute in a known architectural order. |
| **Invariant** | The production `PassPipeline` must register passes in the required order: `CoreIRPass` `→`. `LanguageLayerPass` `→`. `MetaLayerPass` `→`. `PatternRecognitionPass` `→`. `AliasResolutionPass` `→`. `ValidationPass`. This ordering reflects the data and semantic dependencies between stages. |
<!-- pcg:PIPELINE-001-verification:begin -->
| **Enforcement** | `production_pipeline_preserves_architectural_order` test in `src/tests/ir/pipeline.rs` asserts the exact pass sequence via `PassPipeline::pass_names()`. |
| **Authority** | `src/ir/pipeline.rs` (`PassPipeline::default_production()`), `src/tests/ir/pipeline.rs` (ordering test) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |
<!-- pcg:PIPELINE-001-verification:end -->

<!-- pcg:PIPELINE-001-rationale:begin -->
**Rationale for ordering:**
- **CoreIR** must precede **Language Finalize**: The core capture/emission phase must process all captures before language-layer finalization occurs.
- **Language Finalize** must precede **Meta Layer**: Meta layers depend on the instruction stream after language-layer processing.
- **Meta Layer** must precede **Pattern Recognition**: Pattern recognition operates on the complete instruction stream including meta-layer output.
- **Pattern Recognition** must precede **Alias Resolution**: Alias resolution must see all relevant `Extends`/`Implements` instructions after pattern processing.
- **Alias Resolution** must precede **Validation**: Validation must inspect the final canonical instruction stream after all transformations.

<!-- pcg:PIPELINE-001-rationale:end -->
Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 142-159, original section `PIPELINE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `d3d7271332738cd76c6edfbd7b47c7e5557e4eae01e0919cf2a5516a6cc76c3f`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: PIPELINE-001
kind: INVARIANT
status: IMPLEMENTED
title: PIPELINE-001 Compilation Pipeline Ordering
track: FOUNDATION
artifact: docs/invariants/canonical-ir.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#PIPELINE-001
  - qualification self#family-standing
  - verification_obligation self#PIPELINE-001-verification
  - rationale self#PIPELINE-001-rationale
```
<!-- pcg:PIPELINE-001:end -->
