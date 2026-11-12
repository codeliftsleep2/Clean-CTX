# Invariant family: projection-fidelity

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### ARCH-003 Canonical IR and LLM Projection Are Separate Contracts

| Property | Value |
|----------|-------|
| **Intent** | Internal correctness structure must not force a verbose model-facing representation, and token optimization must never weaken canonical guarantees. |
| **Invariant** | Canonical IR is explicit, typed, ordered, identity-bearing, occurrence-preserving, and remains the complete semantic authority. LLM text is a separate compact projection: it may abbreviate while preserving complete meaning, or it may omit bounded semantic categories only when the response explicitly declares that loss in language-scoped, machine-readable degradation metadata. Undeclared semantic loss is forbidden. Exact representations retain their existing exactness guarantees. The model-visible `content` preserves names, typed ownership, signatures, collapsed modifiers, extends/implements, imports, source-written import aliases, type aliases, and exact bodies when requested except for categories explicitly named by such degradation metadata; code-side machinery and workspace-served facts remain available through the reversible codec and `workspace_query`. Import rows render as `$ module [named]`, or `$ [named]` when the canonical module field is empty; removing the generated identity must never leave a phantom column or remove a source-written alias. A structured file presentation identifies its request-scoped file boundary as `// αN`; the exact source path appears once in the trailing `§PATHMAP`, which is authoritative for resolving that alias. `provide_code_context` always returns a complete current presentation under its declared projection contract (or an explicitly selected complete raw representation); it never substitutes a code-side delta receipt. Structured sequence deltas are requested explicitly through `delta_code_context`. |
| **Enforcement** | Distinct canonical types/wire paths and `render_hierarchical_for_llm`; semantic-family renderer contracts under `src/tests/ir/**`; import-projection contracts in `src/tests/ir/render_llm.rs`; `src/tests/mcp/presentation_footer.rs` pins the alias-only file boundary and single authoritative path mapping; registered provider-boundary regression `src/tests/mcp/provide_complete_context.rs`. |
| **Authority** | `src/ir/opcodes.rs`, `src/ir/opcodes/semantic.rs`, `src/ir/hierarchical.rs`, `src/ir/render_llm.rs`, `src/mcp/tool_handlers/core/content.rs`, `src/tests/ir/render_llm.rs`, `src/tests/ir/control_summaries.rs`, `src/tests/ir/pattern_facts.rs`, `src/tests/ir/side_effects.rs`, `src/tests/mcp/presentation_footer.rs`, `src/tests/mcp/tool_helpers.rs` |
| **Type** | STRUCTURAL and ENFORCED |
| **Gate** | Rust compiler and `cargo test --all-features` |

**Deferred:** After canonical IR repairs, audit token efficiency for LLM views that require whole method bodies. This is presentation work and must preserve complete meaning.

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 101-115, original section `ARCH-003`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `2c94033859d6d676382b1ba2f5b70dc8402f64149fef3a68242cb7d1d691ef3e`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: ARCH-003
kind: INVARIANT
status: IMPLEMENTED
title: ARCH-003 Canonical IR and LLM Projection Are Separate Contracts
track: FOUNDATION
artifact: docs/invariants/projection-fidelity.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### CONTENT-001 Visible Content Metadata Describes Only the Visible Text

| Property | Value |
|----------|-------|
| **Intent** | A consumer must never infer byte-exact source regions from fidelity or from an auxiliary code-side payload when those regions are absent from the visible `content`. |
| **Invariant** | `content_kind` and `byte_exact` describe the text in `content`, not the requested fidelity, canonical IR, or delta payload. Structural SCHEMA-vNext reports `skeleton`/`[]`; complete Edit bodies report `skeleton_with_verbatim_bodies`/`["method_bodies"]`; partial focused coverage reports `skeleton_with_focused_verbatim_bodies`/`["focused_method_bodies"]`; Edit with no bodies reports `skeleton`/`[]`; whole-document source reports `verbatim_document` or `raw_passthrough` with `["document"]`; a visible delta acknowledgement reports `delta_summary`/`[]`. Delta operations remain exclusively code-side auxiliary data and never become an LLM responsibility: the explicit `delta_code_context` result carries `result.delta` while `content` contains only its acknowledgement. `provide_code_context` never carries a delta payload or acknowledgement. |
| **Enforcement** | The typed `ContentKind` enum is the wire-value authority. `contract_fields_for_hierarchy` derives regenerated Edit coverage from the actual hierarchy instead of fidelity alone. `src/tests/mcp/tool_handlers_contract.rs` pins every wire string; `src/tests/mcp/content_kind_lifecycle.rs` crosses explicit delta generation, baseline/cache, apply, restart, restore, replay, full/focused/empty Edit coverage, and provider raw fallback while asserting that structured delta data remains auxiliary. `src/tests/mcp/provide_complete_context.rs` pins the complete-provider boundary. |
| **Authority** | `src/mcp/tool_handlers/core/common.rs`, `src/mcp/tool_handlers/core/{delta,provide,delta_apply,restore}.rs`, `src/mcp/tool_handlers/persistence/mod.rs`, `src/mcp/prompts.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 116-128, original section `CONTENT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `38174e27272babff97582df4f81459a967f25b1b42339fa8e37db99d7bc6b3ef`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: CONTENT-001
kind: INVARIANT
status: IMPLEMENTED
title: CONTENT-001 Visible Content Metadata Describes Only the Visible Text
track: FOUNDATION
artifact: docs/invariants/projection-fidelity.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### WSC-006 File-Local Semantic Coverage Is Fidelity- and Source-Aware

| Property | Value |
|----------|-------|
| **Intent** | `entities_in_file` must answer an explicit authorized file in one call without confusing an uncompiled file, stale projection, or insufficient semantic fidelity with a genuine empty result. |
| **Invariant** | WorkspaceIndex owns semantic coverage beside the file occurrences it qualifies: canonical file identity, normalized semantic fidelity, and source hash. A projection is reusable only when its hash matches current source and its fidelity is at least the requested semantic level. Query Edit/Verbatim normalize to High; an actual Edit/Verbatim compilation does not claim High completeness because extractors are not globally monotonic. Missing, stale, or insufficient coverage compiles a read-only candidate and atomically replaces the file's entities, edges, and coverage, including an empty edge set. Removal clears coverage with occurrences. Query-only compilation creates no session alias, rendered context, persistence record, or compression-statistics claim. |
| **Enforcement** | `src/workspace/index/coverage.rs`; `src/workspace/index/remove.rs`; `src/mcp/tool_handlers/query/entities.rs`; `src/mcp/tool_handlers/core/provide/evaluate.rs`; `src/tests/mcp/workspace_query_entities_auto_compile.rs` (one-call first touch, invalid fidelity, schema, High→Low reuse without recompilation, Low→Medium .NET action upgrade, source invalidation, empty replacement, no context/alias publication, and provide→query reuse). WSC-004 suites retain path/root/withinPath authority. |
| **Authority** | `WorkspaceIndex::has_current_semantic_projection`, `WorkspaceIndex::replace_semantic_projection`, `SemanticFidelity` |
| **Type** | ENFORCED (state ownership + test) |
| **Gate** | `cargo test --all-features tests_entities_auto_compile` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 597-609, original section `WSC-006`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `287c9784fe15de90f1b57ed6dae995324cfe39f1ab751a3a782bf92cc89682fb`. Original text remains in Git history; this file is the sole current normative owner of this section.

### CTX-001 Canonical Semantics and Representation Authorities Stay Distinct

| Property | Value |
|----------|-------|
| **Intent** | Token savings and representation changes must never blur semantic correctness, physical persistence, auxiliary wire views, and model-facing presentation. |
| **Invariant** | Canonical `CompiledIR` plus semantic-edge state preserves explicit typed identity, ownership, occurrence order/grouping/duplicates, unresolved written call names and written-arity/spread evidence, injection facts, edge relation/layer/file provenance, and exact bodies/spans when compiled. Normalized `CONTROL-FULL` is the regenerated semantic correctness oracle, not stored text and not the model-visible presentation. Physical binary `0x04` plus the aligned semantic-edge snapshot is the durable baseline authority; checked `dv:2` history is incremental canonical transport. Reduced `result.ir` is non-reversible auxiliary output. Superseded COMPACT-A1/A3 code is retired; A2 `pretty_text` remains non-authoritative compatibility/diagnostic data and is never used to restore canonical state. `focusMethods` resolves typed owners into canonical method IDs before bodies are filtered: bare selectors spanning owners and duplicate qualified owners are errors; a same-owner overload family selects every canonical occurrence. Verbatim remains the explicit whole-document source mode. |
| **Enforcement** | `src/ir/control_full.rs` and `src/ir/focus.rs`; exact-oracle and focus regressions under `src/tests/ir/control_full.rs` and `src/tests/ir/focus.rs`; registered MCP dispatch regressions under `src/tests/mcp/control_full_content.rs`; language call regressions under `src/tests/ir/calls_{typescript,java,csharp,rust}.rs`; injection rendering regressions under `src/tests/ir/render_llm_meta.rs` and `src/tests/mcp/tool_handlers_render.rs`. |
| **Authority** | `src/ir/{compiler,control_full,binary_wire}.rs`, `src/ir/focus.rs`, `src/mcp/sqlite_store/{replay,semantic_state}.rs`, `src/mcp/tool_handlers/core/content.rs`, `docs/architecture/{BINARY_V04_CONTRACT,LLM_CONTEXT_COMPRESSION_RESEARCH}.md` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 662-674, original section `CTX-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `d5b96af35d23d07f36ed1fa939bb743931aa5a496cabb0ddb550ba9408714d73`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: CTX-001
kind: INVARIANT
status: IMPLEMENTED
title: CTX-001 Canonical Semantics and Representation Authorities Stay Distinct
track: FOUNDATION
artifact: docs/invariants/projection-fidelity.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### PROJ-001 Acquisition Strength Does Not Upgrade Publication Fidelity

**Status:** ENFORCED

Reconciliation may acquire stronger internal evidence than the requested
presentation, but Low/Medium/High output retains its requested bounded contract.
Omitted supported semantic categories are declared; Edit and Verbatim exactness
remain separate guarantees.

**Enforcement:** context projection metadata and framework bounded projections.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 863-873, original section `PROJ-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `465f6cac5504aad2e457f9a2ada5668e60ad0cdbaf516e33c78363787c7c2f4b`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: PROJ-001
kind: INVARIANT
status: IMPLEMENTED
title: PROJ-001 Acquisition Strength Does Not Upgrade Publication Fidelity
track: FOUNDATION
artifact: docs/invariants/projection-fidelity.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

```architecture-graph
node: WSC-006
kind: INVARIANT
status: IMPLEMENTED
title: WSC-006 File-Local Semantic Coverage Is Fidelity- and Source-Aware
track: REPOSITORY
artifact: docs/invariants/projection-fidelity.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
