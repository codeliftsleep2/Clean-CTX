# Invariant family: meta-evidence

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### C-22 — Meta-Layer Source Context from Canonical Capture Identity

| Property | Value |
|----------|-------|
| **Intent** | Meta-layer source context MUST be derived from the canonical `CapEntry` capture identity — NOT from the compacted `CoreOp::DefClass.name`. |
| **Invariant** | `MetaLayerPass` derives each class capture's canonical source span from `PassContext.captures` (the persisted capture identity). `class_source_from_capture()` produces the decorator/annotation/attribute-inclusive class text (TS `@Name(...)`, Java `@Name`, C# `[Name]`). The compilation-scoped `MetaLayerContext` carries those spans and the existing canonical source path into one combined layer evaluation without changing either identity's authority. No `DefClass.name` round-trip is permitted. Non-decorated classes use the declaration-keyword byte as fallback (backward compatible). |
| **Enforcement** | `class_source_from_capture_c22_identity` asserts reconstruction from source + `CapEntry`. `MetaLayerPass::run()` in `src/ir/pipeline/meta_layer.rs` filters type-root captures and constructs the shared context. Multi-class cross-contamination tests verify per-class isolation; Angular testing regressions verify canonical-path delivery and backward-compatible path-independent output. |
| **Authority** | `src/meta_util.rs` (`class_source_from_capture`), `src/layers/meta/context.rs` (`MetaLayerContext`), `src/ir/pipeline/meta_layer.rs` (`MetaLayerPass::run`), `src/layers/registry.rs` (combined context dispatch), `src/tests/meta_util.rs` (C-22 identity test), `src/tests/compression/pipeline.rs` (multi-class tests), `src/tests/ir/pipeline_meta_layer.rs` (canonical-path integration) |
| **Type** | ENFORCED (test + structural) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 160-172, original section `C-22`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `66e7e2eb59b6ece429b289c83d67350fc428807c59afd652fd95ffb4c0cedac4`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: INV.C22
kind: INVARIANT
status: IMPLEMENTED
title: C-22 — Meta-Layer Source Context from Canonical Capture Identity
track: FOUNDATION
artifact: docs/invariants/meta-evidence.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### META-001 Framework Annotations Retain Declaration-Local Ownership and Intelligible Values

| Property | Value |
|----------|-------|
| **Intent** | Framework enrichment must add compact, useful facts without losing which declaration supplied them or turning incidental syntax into plausible-looking annotations. |
| **Invariant** | A framework annotation is derived only from evidence inside its owning declaration or statement. An annotation that represents a method-local construct identifies that containing method; an assignment-owned construct uses only the assignment's local left-hand declaration. Field/declaration annotations contain genuine declaration identities, never parameter types, default values, punctuation fragments, or text borrowed from an earlier declaration. Projection and rendering preserve these values but do not guess or repair ownership after extraction. |
| **Enforcement** | RxJS extraction separates observable/subject declarations from pipe/combinator extraction. Observable-field recognition rejects method parameter lists and return annotations. Pipe ownership examines only the current assignment statement or resolves the actual enclosing TypeScript method. Unit regressions pin both rules, and the registered MCP regression verifies the complete `provide_code_context` → meta-layer → `CoreOp::TypeAlias` → hierarchy → SCHEMA-vNext lifecycle. |
| **Authority** | `src/angular_meta/rx/extract.rs`, `src/angular_meta/rx/pipes.rs`, `src/ir/pipeline/meta_layer.rs`, `src/tests/angular_meta/rx.rs`, `src/tests/mcp/rxjs_meta_presentation.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |
| **Relationship to C-22** | C-22 establishes the canonical type-level source span delivered to a meta-layer. META-001 governs declaration- and statement-level ownership inside that span; receiving the correct class source does not authorize whole-prefix evidence borrowing within the class. |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 173-186, original section `META-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `7543e6a4e1427eb0328c0720775e572daf754d5093e5bf76ceadab9e95a75bde`. Original text remains in Git history; this file is the sole current normative owner of this section.

### META-002 Compilation-Scoped Single Evaluation

| Property | Value |
|----------|-------|
| **Intent** | Framework enrichment must reuse immutable evidence already owned by the current compilation instead of reparsing the file or independently deriving the marker and semantic projections. |
| **Invariant** | `MetaLayerPass` constructs one `MetaLayerContext`, including one shared `LexicalRegions` index, and sends it through one registry evaluation. Each applicable registered layer makes one applicability decision and returns markers plus semantic edges in one `MetaLayerEvaluation`. Angular and .NET production applicability reuse compilation-scoped evidence and do not invoke their standalone AST detector; standalone detector entry points retain their established behavior for non-pipeline callers. The text compression pipeline may retain its text-oriented API, but it does not create a second compilation-context route. |
| **Enforcement** | The combined registry regressions pin one applicability check and one evaluation hook per layer. Production-pipeline regressions pin one registry evaluation, one lexical index, one framework detection pass, and zero detection-only AST parses. Framework-specific work counters pin shared Angular testing/NgRx/routing shapes and shared .NET ASP.NET/EF Core analyses where both projections consume the same facts. |
| **Authority** | `src/layers/meta/context.rs`, `src/layers/meta/mod.rs`, `src/layers/registry.rs`, `src/ir/pipeline/meta_layer.rs`, `src/tests/layers/registry.rs`, `src/tests/ir/pipeline_meta_layer.rs` |
| **Type** | STRUCTURAL + ENFORCED (test) |
| **Gate** | Focused meta-layer regressions locally; complete `cargo test --all-features` in CI |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 187-199, original section `META-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `9a3f6affb9441b3889003488ff7d2003f87c41797504d89762a6044472723082`. Original text remains in Git history; this file is the sole current normative owner of this section.
