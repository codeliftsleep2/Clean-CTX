# Invariant family: historical-debt

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### ARCH-DEBT-001 PassPipeline Migration (RESOLVED)

| Property | Value |
|----------|-------|
| **Description** | The `PassPipeline` migration from the monolithic `IRCompiler::compile_inner()` has been completed. `PassPipeline` is now the active production compilation path. |
| **Resolution** | `IRCompiler::compile_inner()` is an orchestration boundary that constructs a `PassContext`, configures the `PassPipeline`, and delegates compilation to `PassPipeline::run()`. Shared pipeline context and ordering remain in `src/ir/pipeline.rs`; core capture, meta-layer orchestration, and post-meta passes live in cohesive child modules under `src/ir/pipeline/`. |
| **Production pipeline order** | `CoreIRPass` `→`. `LanguageLayerPass` `→`. `MetaLayerPass` `→`. `PatternRecognitionPass` `→`. `AliasResolutionPass` `→`. `ValidationPass` |
| **Optional passes** | `ExecutionSemanticsPass`, `ProgramGraphPass`, `InferenceLayerPass` remain outside the default production pipeline. |
| **See also** | `src/ir/pipeline.rs`, `src/ir/pipeline/core.rs`, `src/ir/pipeline/meta_layer.rs`, `src/ir/pipeline/post.rs`, `src/ir/compiler.rs`, `docs/ARCHITECTURAL_INVARIANTS.md` (PIPELINE-001) |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 1072-1083, original section `ARCH-DEBT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `66d96b93e5b5edd622499263234cf9d4f85bd28dab4978c8fa9a0929ee58bb33`. Original text remains in Git history; this file is the sole current normative owner of this section.
