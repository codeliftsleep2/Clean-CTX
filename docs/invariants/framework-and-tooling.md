<!-- pcg:family-standing:begin -->
# Invariant family: framework-and-tooling

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:EDIT-001:begin -->
### EDIT-001 apply_edit Unit Verification & EOL Preservation

| Property | Value |
|----------|-------|
| **Intent** | `apply_edit`'s optimistic-concurrency verification must be robust to transport-level line-ending normalization while never altering the file's stored ending convention. |
| **Invariant** | EOL representation may differ across transport, but after normalization to the target file's convention, `expectedOldText` must match the tracked target bytes exactly. Incoming replacement/insertion text is adapted to the FILE's measured EOL convention before splicing — endings are never rewritten as a side effect and never mixed. Content differences beyond EOL width remain hard rejections. |
| **Enforcement** | `edit::spans::{crlf_file_accepts_lf_normalized_copy_and_preserves_crlf_on_disk, lf_file_accepts_crlf_padded_copy_and_preserves_lf_on_disk, content_changes_are_still_rejected_regardless_of_eol}` — both transport directions plus a forged-content guard; the acceptance tests were RED pre-fix with exact newline-count deltas. |
| **Authority** | `src/edit/apply.rs` (`verify_expected`, `to_unit_eol`, `unit_is_crlf`) |
| **Type** | ENFORCED (test) |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 317-328, original section `EDIT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `f74f0baac8a95b393f94c249993dd7a0630bb8aa7ef6240ce41ad782a0d8c4d5`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:EDIT-001:end -->
<!-- pcg:MCP-001:begin -->
### MCP-001 MCP Result Envelope Conformance

| Property | Value |
|----------|-------|
| **Intent** | Every externally exposed Clean-CTX MCP tool result MUST conform to the canonical MCP `CallToolResult` envelope so schema-validating MCP clients receive a renderable `content` channel. Domain-specific fields MUST NOT be emitted directly at the MCP result level. |
| **Invariant** | The JSON-RPC `result` object of any successful tool call carries a non-empty `content` array (`[{type:"text", text:"..."}]`) as the human/model-readable representation. Machine-readable payloads, when applicable, live in `structuredContent` and are described by a declared `outputSchema` in `tools/list`. Metadata, when applicable, lives in `_meta`. No ad-hoc domain fields are permitted directly under `result`. `structuredContent`/`outputSchema`/`_meta` remain optional — the invariant is about valid canonical result structure, not about forcing every tool into structured output. Result-level failures use `isError: true` (errors that use the JSON-RPC `error` object are outside this envelope). |
| **Enforcement** | Shared `crate::tests::assert_valid_mcp_envelope` applied at the dispatched-handler/wire boundary: CBM graph tools (`src/tests/cbm/handlers.rs`), `apply_edit` (`src/tests/mcp/apply_edit.rs`), all seven `workspace_query` operations (`src/tests/mcp/workspace_query.rs`, `src/tests/mcp/workspace_query_calls_in_file.rs`), comprehensive `outputSchema`↔`structuredContent` consistency (`src/tests/cbm/handlers.rs`, `src/tests/mcp/apply_edit.rs`, `src/tests/mcp/workspace_query.rs`, `src/tests/mcp/workspace_query_calls_in_file.rs`), Phase-3 migrated tools (`src/tests/mcp/phase3_contract.rs`), and the complete dispatched-handler coverage suite for remaining result-producing tools (`src/tests/mcp/envelope_contract.rs`). `compress_code_context`/`delta_code_context` remain the sole documented exceptions (R-46, deferred to 0.6.0). |
| **Authority** | Reference implementation: `src/cbm/handlers.rs` (`handle_graph_search` — `content` + `structuredContent` + declared `outputSchema`). Shared helper: `src/tests/mod.rs::assert_valid_mcp_envelope`. |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 329-341, original section `MCP-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `d9bbd21c900fbeed15d97633e406c4ae057ee3420b4f2da72116d743f5b59ccf`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:MCP-001:end -->
<!-- pcg:MCP-002:begin -->
### MCP-002 Focused Context Requests Have One Explicit Fidelity Meaning

| Property | Value |
|----------|-------|
| **Intent** | A caller that requests selected method bodies must never receive a successful structural response in which that selection was silently ignored, nor have an explicit non-Edit choice silently overridden. |
| **Invariant** | `provide_code_context.focusMethods` is Edit-only targeting. A non-empty focus with neither `fidelity` nor `intent` implies Edit. Explicit Edit fidelity or intent accepts focus. Any explicit non-Edit fidelity or intent conflicts and returns `-32602` before file IO or session mutation. An empty focus retains its specialized “Edit structure with no bodies” meaning and therefore requires explicit Edit fidelity or intent. Focus resolution and ambiguity rules remain governed by CTX-001 after this request-boundary validation. |
| **Enforcement** | `src/mcp/tool_handlers/core/provide/evaluate.rs`; `src/tests/mcp/focus_fidelity_contract.rs` (implicit Edit, explicit-fidelity conflict, explicit-intent conflict, and empty implicit-focus rejection); existing canonical focus tests under `src/tests/ir/focus.rs` and `src/tests/mcp/control_full_content.rs`. |
| **Authority** | `provide::evaluate::evaluate` request validation followed by `heuristics::decide` and canonical focus resolution |
| **Type** | ENFORCED (request boundary + test) |
| **Gate** | `cargo test --all-features focus_fidelity_contract_tests` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 342-354, original section `MCP-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `dbc90a831a26827753658dbd664a624ac8c8f9703ea9ca1b77229f393f90c1db`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:MCP-002:end -->
<!-- pcg:MCP-003:begin -->
### MCP-003 Batched Context Reads Reuse One Authoritative File Evaluator

| Property | Value |
|----------|-------|
| **Intent** | Reading several independently useful source files in one MCP call must reduce repeated tool invocation, setup, and caller-orchestration overhead without creating a second compilation lifecycle, hiding item failures, weakening exact-text authority, or implying a cross-file transaction. Callers may avoid compatibility mirroring only through an explicit projection suited to a verified host. |
| **Invariant** | `provide_code_context` accepts exactly one of singular `filePath` or batched `files`. A batch contains at most eight ordered items with unique non-empty IDs, shares top-level workspace/tokenizer scope, and evaluates items sequentially through the same typed single-file evaluator used by the legacy form. Successful items retain normal per-file persistence/publication/statistics effects. Batch-only `responseMode` resolves to `mirrored` when omitted, `structured`, or `indexed`, and the response declares `response_mode`. Mirrored places exact text in both each successful item's `content` and a `content_index`-identified top-level block. Structured places it only in the item and emits a routing notice. Indexed places it only in the indexed top-level block. Item failures remain ordered structured errors with no content and never suppress successful siblings. Later items resolving to an already evaluated canonical file fail locally so focus-sensitive state is never order-dependent. The outer cache identity includes the selected mode and covers the ordered successful blocks. An all-failure batch emits one non-authoritative fallback text block solely to preserve the non-empty MCP envelope; no item points to it. Batching grants no rollback, aggregate truncation, concurrency, retry, or cross-file edit authority. |
| **Enforcement** | Structural separation in `src/mcp/tool_handlers/core/provide/{evaluate,batch,outcome}.rs`; public schema in `src/mcp/tool_schemas.rs` and `src/mcp/tools.rs`; tracked dispatch regressions in `src/tests/mcp/provide_code_context_batch.rs` cover request shape, ordered mixed modes, legacy parity, each exact placement contract, isolated failures, duplicate canonical files, all-failure envelope validity, and truthful token accounting. `src/tests/mcp/provide_code_context_response_mode_economics.rs` verifies distinct reusable mode-scoped cache identities and reports 2/4/8-item serialized sizes. |
| **Authority** | `provide::evaluate::evaluate` owns the complete per-file lifecycle; `provide::batch` owns only structural validation, shared-field projection, ordered coordination, and the outer response/cache envelope. |
| **Type** | STRUCTURAL and ENFORCED (test) |
| **Gate** | `cargo test --all-features provide_code_context_batch_tests` and `cargo test --all-features response_modes_have_distinct_reusable_cache_identities_and_measured_sizes -- --nocapture` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 355-366, original section `MCP-003`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `5802127c59ec78c7daee05fcaf70a7315ce007448776b023f9cb993a9ffc3ade`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:MCP-003:end -->
<!-- pcg:SEL-001:begin -->
### SEL-001 Selector-Value Invariant

| Property | Value |
|----------|-------|
| **Intent** | Semantic-edge object names must carry the exact semantic value declared by the source; index/lookup mechanics must never contaminate entity identity. |
| **Invariant** | The `name` field of a `HasSelector` semantic edge's object `EntityRef` MUST contain the exact CSS selector string declared by the source code (`@Component({ selector: '...' })`), without artificial encoding, wrapping, or normalization. Distinct selector forms (`app-widget`, `[app-widget]`, `.app-widget`) are distinct semantic identities. |
| **Enforcement** | `src/tests/angular_meta/semantic.rs::has_selector_preserves_literal_selector_forms` (literal extraction of all three selector forms + pairwise distinctness); `src/tests/workspace/index.rs::resolve_selector_literal_forms_are_distinct` (literal lookup through `WorkspaceIndex`, cross-form isolation). |
| **Authority** | `src/angular_meta/semantic.rs` (`class_to_semantic_edges`), `src/workspace/index.rs` (`resolve_selector`) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 471-483, original section `SEL-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `2c33090e29e539f48127f9345cb48648561ff6b0a280a321f5d2097866d0a9e2`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:SEL-001:end -->
<!-- pcg:INV.ANG_DI_001:begin -->
### ANG-DI-001 Angular Constructor Injection Is Modifier- and Formatting-Independent

| Property | Value |
|----------|-------|
| **Intent** | Angular constructor DI identity is semantic, not syntactic. TypeScript parameter-property modifiers control property generation; they must never gate Angular injection extraction, and source formatting must never alter semantic extraction. |
| **Invariant** | Within a class already recognized as an Angular DI class (a `@Component` / `@Injectable` / `@Directive` / `@Pipe` / `@NgModule` decorator is required before extraction runs — the class-level gate), a **typed** constructor parameter is a constructor injection regardless of whether it also declares a parameter property. `private foo: FooService`, `public foo: FooService`, `protected foo: FooService`, `readonly foo: FooService`, `private readonly foo: FooService`, and `foo: FooService` are equivalent for DI and MUST each contribute exactly one `SemanticRelation::Injects` edge (`angular`, subject class → `angular`/`Service`/`<parameter type>`). Parameter decorators (`@Inject(...)`, `@Optional()`, …) are DI metadata and must not hide the parameter type; the injection identity remains the parameter type. Type-eligibility rules are unchanged (no primitive filter is added or removed, and generic/qualified type names keep the pre-existing leading-identifier reduction). Formatter layout (single-line, multiline, trailing comma, line breaks between decorator / modifier / name / type) MUST NOT change the resulting edges. Ordinary non-Angular TypeScript constructors MUST NOT gain Angular `Injects` edges. Deduplication remains the `WorkspaceIndex` write-boundary contract — the extraction emits one edge per parameter. |
| **Enforcement** | `src/tests/angular_meta/constructor_injects.rs` (RED-DI1…RED-DI10 extraction-level parity, decorator preservation, formatting independence, single-injection-per-parameter, Φinjects projection); `src/tests/angular_meta/constructor_di_edges.rs` (RED-DI1…RED-DI10 + RED-W1/W2/W3 through the production path: `IRCompiler` → `MetaLayerPass` → `AngularMetaLayer` → `SemanticRelation::Injects` → `WorkspaceIndex` forward/reverse queries, plus the non-Angular class guard). |
| **Authority** | `src/angular_meta/constructor_injects.rs` (extraction; single source of truth for the `injects` list consumed by `class_to_semantic_edges` and the `Φinjects:` marker), `src/angular_meta/decorators.rs` (`extract_graph_entries` — the class-level Angular gate), `src/angular_meta/semantic.rs` (`class_to_semantic_edges`) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 649-661, original section `ANG-DI-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `6b5edbd5d3ef9c797e534ce2caa43193b70984efa371bb4f8fedde012b7dc170`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.ANG_DI_001:end -->
<!-- pcg:INV.ANG_TPL_001:begin -->
### ANG-TPL-001 Angular Template Ownership and Identity

| Property | Value |
|----------|-------|
| **Intent** | Component/template linkage must follow structured Angular ownership without filename or basename inference. |
| **Invariant** | Structured Component metadata exclusively authors `HasTemplate`. External Template identity is its canonical physical/semantic-owner path; inline identity is deterministic and Component-scoped. The declaring TypeScript file owns publication and retraction. |
| **Enforcement** | Phase 4 production-path identity, forward/reverse, replacement, transition, deduplication, deletion/restoration, refresh, and generation regressions. |
| **Authority** | `src/angular_meta/semantic.rs`; `src/tests/mcp/workspace_query_angular_templates.rs` |
| **Type** | ENFORCED (structured producer boundary + tests) |
| **Gate** | `cargo test --all-features workspace_query_angular_template_tests` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 947-959, original section `ANG-TPL-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `07957de884fcd33c512058219b4bcecbe48d2d1b3ac0ddc6472e8aac6059aa46`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.ANG_TPL_001:end -->
<!-- pcg:INV.ANG_TPL_002:begin -->
### ANG-TPL-002 Angular Selector Refinement Is Occurrence-Preserving and Ephemeral

| Property | Value |
|----------|-------|
| **Intent** | Repeated selector uses and their controlling conditions must survive query-time resolution without inventing durable cross-file ownership. |
| **Invariant** | Every occurrence-derived `UsesSelector` and `UsesComponent` result carries its Template occurrence evidence: a zero-based half-open UTF-8 byte span and control ancestry. Neutral `UsesSelector` remains present when unique typed refinement is returned. Neither relationship enters WorkspaceIndex, producer generations, or snapshots. |
| **Enforcement** | Ordered-authority extraction plus production MCP forward/reverse, ambiguity, mixed-syntax, replacement, refresh, deletion/restoration, and producer-generation regressions. |
| **Authority** | `src/angular_meta/template_occurrences.rs`; `src/mcp/tool_handlers/query/angular_templates.rs`; `src/tests/mcp/workspace_query_angular_selector_occurrences.rs` |
| **Type** | ENFORCED (request-local type boundary + tests) |
| **Gate** | `cargo test --all-features workspace_query_angular_selector_occurrence_tests` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 960-972, original section `ANG-TPL-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `dbd2741b377818a49e081a1d110e8119223c17c644f3bd761992cfb9aac11067`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.ANG_TPL_002:end -->
<!-- pcg:INV.CSHARP_RESOLUTION_001:begin -->
### CSHARP-RESOLUTION-001 Structured C# Resolution Owns Framework Authority

| Property | Value |
|----------|-------|
| **Intent** | Written C# spelling must not become framework identity. Shared C# mechanics establish neutral identity; framework modules alone assign framework meaning. |
| **Invariant** | Base/type and attribute references used by .NET semantic producers resolve through explicit `Resolved`, `Ambiguous`, `Unsupported`, or `Unresolved` dispositions. Only `Resolved` neutral identities authorize framework roles or relationships. All non-success dispositions fail closed while parser-owned neutral evidence survives. Attribute resolution implements suffix elision without turning short written names into authority. |
| **Supported envelope** | Explicit and `global::` qualification, ordinary imports with proven-candidate collision handling, namespace aliases, type aliases, admitted-source namespace/nesting/name/arity resolution, valid partial-symbol composition, generic/nested identity, and cycle-checked indirect inheritance. An unmodeled imported namespace is not itself evidence of a competing same-name declaration: one proven candidate resolves, multiple proven candidates remain ambiguous, and no proven candidate remains unresolved. Known external identity is bounded to the migrated framework contracts. Arbitrary assemblies/NuGet metadata, implicit SDK evidence absent from admitted source, unproved project-reference legality, and compiler-grade expression/overload binding remain explicit non-success boundaries. |
| **Consumers** | EF Core `DbContext` and configuration identity, AutoMapper `Profile`, SignalR `Hub`/`Hub<T>`, ASP.NET `Controller`/`ControllerBase` plus controller/route/HTTP attributes, and MSTest `TestClassAttribute`. Marker-only attributes and conventions remain rendering concerns unless separately promoted through an approved semantic migration. |
| **Applicability** | Resolved framework identity positively admits .NET semantic evaluation. Textual detection may schedule marker/candidate work but cannot establish semantic truth or veto resolved proof. |
| **Declaration provenance** | Resolve by identity, render by position. Core lowering preserves exact declaration spans with allocated method IDs and emitted names; compilation method facts join through those spans, never by zipping independently produced record sequences. Class generic arity counts only direct type-parameter children associated with their class by one parser query match. Method generic parameters cannot contribute to enclosing class identity. Issues #153/#154 are covered by `csharp_compilation_identity.rs` and `csharp_compilation_identity_preservation.rs` under `src/tests/dotnet_meta/`. |
| **Lifecycle** | Resolution uses the current admitted-source generation and composes partial owners. Generation reconciliation retracts and reprojects affected roles, relationships, and SignalR realtime execution contexts after edits, alias retargeting, ancestry changes, part changes, refresh, or deletion. Independent consumer facts and independent execution contexts are preserved. Resolution partitions do not invent cross-project reference legality. |
| **Provider boundary** | Optional providers supply additive acquisition hints only. Provider readiness, empty/subset results, and query history cannot narrow Clean-authoritative semantic truth or earn authoritative root completion without compatible proof. |
| **Enforcement** | Neutral resolution/result types and the admitted-source symbol index enforce the authority split structurally. R0-R12 tracked resolver, consumer, lifecycle, cross-root, history, and Issue #148 provider-equivalence regressions enforce behavior. R9-R12 live MCP harnesses verify the production boundary separately from CI coverage. |
| **Authority** | `src/ir/csharp_resolution.rs`; `src/ir/csharp_resolution/`; `src/ir/csharp_symbols.rs`; `src/dotnet_meta/semantic.rs`; `src/mcp/csharp_calls/reconciliation.rs`; `src/mcp/semantic_compilation/artifact.rs`; R0-R12 tests under `src/tests/` |
| **Type** | STRUCTURAL and ENFORCED |
| **Gate** | R0-R12 focused regression suites, Issue #148 provider-equivalence suites, then the authoritative Final Verification Gate |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 1024-1042, original section `CSHARP-RESOLUTION-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `e025d90f9edff23f2302dc6b788fb3bfb102f86e0af48bb569dba544b2f8c529`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CSHARP_RESOLUTION_001:end -->
<!-- pcg:DOTNET-001:begin -->
### DOTNET-001 Framework Role Identity Is Independent of Relationship Evidence

| Property | Value |
|----------|-------|
| **Intent** | Clean-CTX durably expresses a proven framework role independently of whether that role currently has relationship payload. |
| **Invariant** | A resolved framework role may register a file-provenanced semantic occurrence independently of `HasEntity`, mappings, hub targets, routes/actions, or tests. Role registration never becomes source declaration authority or graph adjacency; `builtin/Class` remains the C# declaration. A role alone cannot fabricate relationship payload. DbContext and Controller publish explicit role occurrences; relationship-bearing Profile/Hub projections retain their resolved-role authorization even when their current payload is empty. |
| **Lifecycle** | Role and relationship projections use the same admitted-source resolution generation described by CSHARP-RESOLUTION-001. Publication, replacement, refresh, deletion, rename, warm reuse, partial composition, alias/ancestry changes, producer compatibility, and same-name owner identity use `WorkspaceIndex` owner lifecycle and generation reconciliation. .NET semantic producer generation 7 owns the current projection. |
| **Enforcement** | `src/dotnet_meta/semantic.rs` owns framework interpretation; `src/mcp/semantic_compilation/artifact.rs` retires and rebuilds generation-owned facts; `src/workspace/index/edges.rs` normalizes role registrations into non-declaration occurrences. R6-R12 tracked suites pin role/payload independence, fail-closed resolution, lifecycle, cross-consumer isolation, and history/provider convergence. |
| **Authority** | CSHARP-RESOLUTION-001; `src/dotnet_meta/semantic.rs`; `src/mcp/semantic_compilation/artifact.rs`; `src/workspace/index/edges.rs` |
| **Type** | STRUCTURAL and ENFORCED (test) |
| **Gate** | Focused R6-R12 regressions, then the authoritative Final Verification Gate |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 1043-1056, original section `DOTNET-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `8edb8b28a7d8d9be17bd48da3095729f31f91d3e304ee7ac4756afc5e03d7f00`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: EDIT-001
kind: INVARIANT
status: IMPLEMENTED
title: EDIT-001 apply_edit Unit Verification & EOL Preservation
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#EDIT-001
  - qualification self#family-standing
```

```architecture-graph
node: MCP-001
kind: INVARIANT
status: IMPLEMENTED
title: MCP-001 MCP Result Envelope Conformance
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#MCP-001
  - qualification self#family-standing
```

```architecture-graph
node: MCP-002
kind: INVARIANT
status: IMPLEMENTED
title: MCP-002 Focused Context Requests Have One Explicit Fidelity Meaning
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#MCP-002
  - qualification self#family-standing
```

```architecture-graph
node: MCP-003
kind: INVARIANT
status: IMPLEMENTED
title: MCP-003 Batched Context Reads Reuse One Authoritative File Evaluator
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#MCP-003
  - qualification self#family-standing
```

```architecture-graph
node: SEL-001
kind: INVARIANT
status: IMPLEMENTED
title: SEL-001 Selector-Value Invariant
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#SEL-001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CSHARP_RESOLUTION_001
kind: INVARIANT
status: IMPLEMENTED
title: CSHARP-RESOLUTION-001 Structured C# Resolution Owns Framework Authority
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
preserves:
  - AUTH-001
  - WSC-012
  - SEMDEP-001
note: Current source-described contract; no witness execution or independent certification is inferred.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CSHARP_RESOLUTION_001
  - qualification self#family-standing
```

```architecture-graph
node: DOTNET-001
kind: INVARIANT
status: IMPLEMENTED
title: DOTNET-001 Framework Role Identity Is Independent of Relationship Evidence
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
depends_on:
  - INV.CSHARP_RESOLUTION_001
preserves:
  - WSC-011
note: Current source-described contract; no witness execution or independent certification is inferred.
knowledge_scope: invariant
knowledge_content:
  - statement self#DOTNET-001
  - qualification self#family-standing
```

```architecture-graph
node: INV.ANG_DI_001
kind: INVARIANT
status: IMPLEMENTED
title: ANG-DI-001 Angular Constructor Injection Is Modifier- and Formatting-Independent
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
preserves:
  - WSC-012
note: Operative bounded source contract; historical approval and execution remain attributed, not language certification or runtime permission.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.ANG_DI_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.ANG_TPL_001
kind: INVARIANT
status: IMPLEMENTED
title: ANG-TPL-001 Angular Template Ownership and Identity
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
preserves:
  - SEMDEP-001
  - WSC-012
note: Operative bounded source contract; historical approval and execution remain attributed, not language certification or runtime permission.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.ANG_TPL_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.ANG_TPL_002
kind: INVARIANT
status: IMPLEMENTED
title: ANG-TPL-002 Angular Selector Refinement Is Occurrence-Preserving and Ephemeral
track: REPOSITORY
artifact: docs/invariants/framework-and-tooling.md
depends_on:
  - INV.ANG_TPL_001
preserves:
  - REFINE-001
note: Operative bounded source contract; historical approval and execution remain attributed, not language certification or runtime permission.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.ANG_TPL_002
  - qualification self#family-standing
```
<!-- pcg:DOTNET-001:end -->
