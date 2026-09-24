# Live Discovery Registry (`docs/agent/DISCOVERY_REGISTRY.md`)

**Purpose:** Record discoveries made in the **real-world discovery environment**
(Claude + Clean-CTX operating against large real repositories) and their
resolution, so field findings become institutional engineering knowledge rather
than disappearing into a development conversation.

This is NOT a changelog (see `docs/CHANGELOG.md`) and NOT release accounting
(see `docs/agent/releases.md`). It exists to close the two-environment gap:

```text
REAL-WORLD DISCOVERY (Claude + real repo)
      ↓
root-cause investigation
      ↓
classify (Protocol | Semantic | Emergent)
      ↓
can it be distilled deterministically?  → local regression
      ↓                                        (cheap Rust test)
not reproducible at small scale
      ↓
retain as a live acceptance scenario → documented record here
```

## Format

One entry per significant discovery, newest first. An entry is closed when a
local regression covers it, a live scenario has been re-verified, or the
behavior is superseded.

## Entry Template

| Field | Value |
|-------|-------|
| **Discovered** | YYYY-MM-DD |
| **Environment** | Claude + Clean-CTX vX.Y.Z |
| **Repository/context** | Description (approximate size, languages, frameworks) |
| **Symptom** | What was observed |
| **Root cause** | Precise technical cause |
| **Classification** | Protocol / Semantic / Emergent |
| **Reproducible locally?** | Yes / No |
| **Local regression** | Test path(s) or N/A |
| **Live scenario required?** | Yes / No |
| **Architectural invariant** | INV-XXX or N/A |
| **Status** | Open / Fixed / Verified |

---

## DIS-2026-020: Model-Visible Content Is Assembled from the Codec, Not a Presentation

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-24 |
| **Environment** | Controlled laboratory (this repository): source audit + measurement harness |
| **Repository/context** | Clean-CTX itself — `src/mcp/tool_handlers/core/content.rs` and its 17 caller sites |
| **Symptom** | Every content-producing handler (`compress_code_context`, `provide_code_context`, `control_full_delta`, `apply_delta`, `restore_context`, `replay_history`, persistence) assembles the model-visible `content` from the CONTROL-FULL codec (`compact_a::render_file_context`, A2), shipping the codec's preamble, grammar legend, envelope schema id and `§BODIES` framing into the model's context. `render_hierarchical_for_llm` (SCHEMA v5), the LLM-facing presentation renderer, has zero production callers. |
| **Root cause** | The feature branch replaced the presentation renderer with the codec, conflating the reversible wire (CTX-001) with the model-visible presentation (ARCH-003). The codec's decode side has no production caller (`decode_cold` / `decode_declarations` / `decode_facts` reference only one another), so its legend is required by nothing in the protocol while being paid in every prompt. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — dispatch any content handler and read `result.content[0].text`. |
| **Local regression** | `src/tests/mcp/presentation_boundary.rs` (content is not the codec document; carries no decoder legend / envelope schema id / body framing; typed owner + method identity survive). |
| **Live scenario required?** | No — the defect is the production assembly path, not scale-dependent. |
| **Architectural invariant** | ARCH-003 (presentation) vs CTX-001 (reversible codec), now mechanically enforced. |
| **Status** | Open — decision recorded, fix in progress |

**Decision (2026-09-24):** `content` becomes the presentation renderer
(`render_hierarchical_for_llm`, SCHEMA v5) on every path; the codec stays
code-side (`result.ir` + persistence). Option A now, Option C (a purpose-built
presentation) follows; Option B (codec minus its legend) was rejected because a
positional grammar without its interpretive key is undecodable, not presentable.
See `verification/context-compression/compact-a/PRESENTATION_BOUNDARY_PLAN.md` §6.


## DIS-2026-019: `provide_code_context` Fabricated Method Identities for C# Generic and Tuple-Returning Declarations

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-17 |
| **Environment** | Live MCP session against real C# LINQ-shaped sources (client not recorded) + local reproduction in the controlled laboratory |
| **Repository/context** | Real-world C# declarations: an extension method with two method type parameters whose parameter type nests a generic argument list (`Expression<Func<TFirst, TSecond>>`), plus two unrelated methods returning named tuples, in a `static` class with no base list. |
| **Symptom** | `provide_code_context` returned structurally valid but factually incorrect method signatures, and the compressed schema hid it. `Pair<TFirst, TSecond>` rendered as the method identity `M TSecond>` at every fidelity while single-type-parameter methods in the same source were correct; `public static (int alpha, int beta) GetPair(int[] values)` rendered with name `static`, parameters `int alpha, int beta` and return type `GetPair(int[] values)` — the fields shifted — whereupon two unrelated tuple-returning methods shared the fabricated identity and rendered as `static(+2)`, a false overload group. The same class-level output also acquired a fabricated base-class line `X source.OrderByDescending(keySelector);` although the class declares no base list. |

| **Root cause** | Two token-position rules living at the flatten-then-reparse boundary between `compaction::extract_method_sig` and the IR compiler, each defeated by a legitimate declaration shape. (1) The declared NAME was derived as "the last whitespace token before the declaration's parameter list", which is only accidentally the name: a generic method's type-parameter list itself contains `, `, so the token sequence of `Pair<TFirst, TSecond>(` ends in `TSecond>` and the name became a type parameter at every fidelity; the identical rule in `compact_method_low`, `compact_method_medium` and `diff::keys::method_key` propagated the corrupted identity into the compressed label and the diff grouping key. (2) The PARAMETER LIST was located as "the first depth-0 balanced `(...)` group preceded by an identifier or `>`", and `static` is an identifier, so a parenthesized RETURN type — the C# tuple return — was accepted as the parameter list; every later field then derived from the wrong side of the wrong boundary (name = the prefix's last token `static`, parameters = the tuple members, return type = the real name plus its parameter list). Because `CoreOp::DefMethod` is the canonical identity consumed by the hierarchical wire form, `render_llm` and its overload grouping, `UnitTable`, the `builtin` / `Method` registration occurrences and the caller side of every `Calls` edge, the corruption was never rendering-only — but it was produced ONCE, upstream, so both consumers were corrected by one fix with no projection or renderer change (the renderer is a pure projection and was proven innocent, including by the Java control below). A third defect in the same capture-boundary family has a separate mechanism: `CSharpLayer::extract_class_relationships` scanned the whole class declaration node (head **and** body) for the first `:`, so a ternary in a member body became a base class. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic fixtures; no scale dependency. |
| **Local regression** | `src/tests/compaction/signature.rs` (shared structural boundary, plus the Low/Medium label contract for C#, TypeScript and Rust); `src/tests/ir/method_signature_shape.rs` (RED-SIG1–RED-SIG12 through the production pipeline at Low/Medium/High); `src/tests/ir/signature_cross_language.rs` (TypeScript function and method, Rust `impl` `fn`, and the Java control); `src/tests/mcp/provider_code_context_signature.rs` (the real dispatch path at low/medium/high plus edit fidelity with `focusMethods`, each case asserting `content_kind != "raw_passthrough"`); `src/tests/ir/semantic_projection.rs` (registration occurrences and the `Calls` caller identity at the projection boundary). |
| **Live scenario required?** | Already exercised and worth re-running: the maintainer's run of `target/tmp/signature_live_acceptance.mjs` against the rebuilt binary printed the corrected identities at all three fidelities with every §Symptom value absent; the driver's own too-coarse probe was corrected afterwards and that re-run is not yet recorded. |
| **Architectural invariant** | N/A — no catalogued invariant covers declaration-identity derivation (`ARCH-002` and `IRPAT-001` govern other boundaries). The rule is enforced structurally instead: one shared extractor (`src/compaction/signature.rs`), consumed by the label, IR and diff paths, plus its tracked regressions. |
| **Status** | Fixed (local regressions green; live acceptance exercised the corrected identities — the corrected probe's re-run not yet recorded) |

**Fix (2026-09-17):** `src/compaction/signature.rs` (new) now owns declaration identity: the NAME is the identifier that OWNS the parameter list (balanced angle-bracket scanning, `.`-qualified chains kept whole); a parenthesized RETURN TYPE is recognised structurally, because the declared name and then the declaration's own parameter list follow the group, so it can never be selected as the parameter list; the RETURN TYPE is the trailing TYPE expression of a return-type-first prefix, so modifiers and the declaration's own type-parameter list cannot leak into it; and formal PARAMETERS split at structural depth, so a comma nested inside a generic argument list cannot inflate them. `compaction::method` (Low/Medium labels, `find_method_params`, `extract_param_names`), `ir::pipeline/signature.rs::parse_method_sig` and `diff::keys::method_key` consume that one implementation, and `CSharpLayer::extract_class_relationships` now reads only the attribute-stripped declaration head. Cross-language outcome: the shared rule had also affected **TypeScript** and **Rust** (both corrected and regression-tested); **Java** is structurally immune because its method type-parameter list precedes the return type — the control that isolated the defect to the shared flattening rather than the renderer.

**Distillation note:** neither rule was a C# accident — both were shared-boundary assumptions about flattened text, which is why the fix belongs at the shared boundary and why the audit crossed languages. Neither defect needed scale: each is a one-line declaration shape, so both distilled into cheap tracked regressions on the first attempt.

**Adjacent, unfixed finding (deliberately not part of this fix):** `CoreOp::Flags` has **two** producers writing separate ops for the same method id — the language layer's declaration modifiers (`STATIC`, `ASYNC`, `PRIVATE`, …) and the accumulated control-flow flags (`IF`, `LOOP`, `RET`, `THROW`) flushed at the end of the declaration — while `src/ir/hierarchical/encode.rs` keeps only the LAST op (`methods[mi].flags = Some(flags.clone())`). Consequence: a method whose body contains control flow renders `fl:RET` and its declaration modifiers are never rendered at all. This is pre-existing, language-agnostic, unrelated to the identity defect, and NOT fixed here: correcting it means either merging the two flag families at encode (which changes the inputs `CompressingPatternRecognizer` matches on — `FLAGS(ASYNC)`, `FLAGS(OVERRIDE)`) or splitting the opcode, i.e. a `SCHEMA v2` / pattern-recognition decision that needs its own approval. The new IR test instrumentation unions a method's flag ops, so an assertion about either family stays meaningful meanwhile.

**Minimal trigger:**
```csharp
using System.Linq;
using System.Linq.Expressions;

public static class QueryablePairExtensions
{
    public static IOrderedQueryable<TFirst> Pair<TFirst, TSecond>(
        this IQueryable<TFirst> source,
        Expression<Func<TFirst, TSecond>> keySelector,
        ListSortDirection direction)
    {
        return source.OrderByDescending(keySelector);
    }

    public static (int alpha, int beta) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }

    public static (string name, int count) Tenth(string[] names)
    {
        return (names[0], names.Length);
    }

    public static int Pick(int[] values, bool ascending)
    {
        // The ternary `:` is what a full-node relationship scan mistook for a
        // base-class separator, fabricating an `extends` edge.
        return values.Length == 0 ? 0 : values.OrderByDescending(v => v).First();
    }
}
```

---

## DIS-2026-018: Workspace-Scoped Edge Queries Returned Another Repository's Occurrences

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-16 |
| **Environment** | Claude + Clean-CTX (live multi-repository session; native `Calls` already confirmed working) |
| **Repository/context** | A wanted C# repository queried through `workspace_query` while an unrelated repository was already compiled into the same session `WorkspaceIndex`. |
| **Symptom** | `workspace_query(type="reverse_edges", domain="builtin", entity_type="Method", name="OrderBy")` returned the wanted repository's real `OrderBy(argc=2)` callers **plus three real `OrderBy(argc=2)` callers from the unrelated repository**. Every returned fact was genuine; the failure was that none of those three belonged to the queried workspace. |
| **Root cause** | `handle_forward_edges` / `handle_reverse_edges` called `WorkspaceIndex::forward_edges_by_identity` / `reverse_edges_by_identity`, which select the entire adjacency bucket across the whole session and return only `&SemanticEdge` — the stored occurrence's `asserting_file` provenance was discarded before the response, and no workspace-root filter existed anywhere on the query path. Semantic identity `(domain, entity_type, name)` is file-free by design (Model C), so the identity itself could never separate the repositories. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic fixtures; no scale dependency. |
| **Local regression** | `src/tests/mcp/workspace_query_scope.rs` (RED-SCOPE1–RED-SCOPE8: two-repository isolation, additional-root inclusion, third-repository exclusion, identical subject identity, identical name+arity, forward edges, non-Calls relation, repository-name prefix safety) and `src/tests/mcp/workspace_query_scope_provenance.rs` (RED-SCOPE9 canonicalization parity, RED-SCOPE10 removal/recompile, the object-provenance control, and root-less-query preservation). |
| **Live scenario required?** | Yes — re-run the `OrderBy` query against the wanted repository (only its own callers), then query the other repository explicitly (its three callers appear there), proving the facts were filtered rather than deleted. |
| **Architectural invariant** | Proposed `WSC-004` (pending maintainer approval). `IDX-002` is unchanged: the index still stores and can return every occurrence. |
| **Status** | Fixed locally; live re-verification pending |

## DIS-2026-017: Broad `search_graph` Searches Skipped C# Caller Verification Entirely

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-16 |
| **Environment** | Claude + Clean-CTX (live C# workspace with a custom `OrderBy` extension overload) |
| **Repository/context** | Real C# workspace (size not recorded) queried through `cbm_proxy`; the workspace contains a custom `OrderBy` extension method that overload-conflicts with the framework `OrderBy` |
| **Symptom** | `search_graph(name_pattern = "^OrderBy$")` (one result) returned verified caller evidence (`raw_in_degree`, `verified_in_degree`, `in_degree_evidence`, `clean_ctx_caller_verification`) for the custom overload, while `search_graph(name_pattern = "OrderBy")` (four results) returned the same symbol with **no** per-result verification fields at all — the custom overload surfaced CBM's raw, known-incorrect `in_degree` with nothing marking it as unverified. |
| **Root cause** | `src/cbm/caller_verify_proxy.rs` derived one verification target for the whole response and gated it on `results.len() == 1` (`(results.len() == 1).then(|| results[0]["qualified_name"]…)`). With more than one result the request was built with `target: None` and `raw_candidates: 0`, so `verify_request_sources` returned an unverifiable summary and `annotate_surface_counts` skipped the per-result annotation branch entirely: verification was never attempted, and no result carried a status saying so. The arity algorithm itself was correct — only the orchestration was single-result. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic per-result orchestration fixtures with the shared verifier injected as a canned closure (no CBM needed). |
| **Local regression** | `src/tests/cbm/caller_verify_search.rs` (`RED-SG1`–`RED-SG8`: single-result control, one-eligible-among-many, several verifiable methods, verified+ambiguous, verified+unverifiable, the unanchored `OrderBy` reproduction, anchored/unanchored parity for the same symbol, and the no-bare-raw-`in_degree` invariant) plus `src/tests/cbm/caller_verify_proxy.rs` (single-target request/count pins). |
| **Live scenario required?** | Yes — re-run the confirmed field case: `name_pattern = "OrderBy"` must attach the same verified caller count to the custom overload as `name_pattern = "^OrderBy$"`. |
| **Architectural invariant** | `CBM-VERIFY-001` |
| **Status** | Fixed (live re-verification pending) |

**Resolution:** caller verification for `search_graph` is now per result. `src/cbm/caller_verify_search.rs` plans one verification target per result carrying `in_degree` and orchestrates one `VerificationRequest` per eligible result through the unchanged shared verifier; every `in_degree` result is annotated with its own truth (`raw_in_degree`, `verified_in_degree`, `in_degree_evidence`, `in_degree_resolution`, plus `in_degree_reason` whenever it is not verified), and the top-level `clean_ctx_caller_verification` block keeps its existing field set while adding per-response disposition counters. Eligible results are limited to callable C# symbols (non-callable labels, non-C# targets, and duplicated identities are annotated instead of guessed), so no candidate narrowing, caching, or arity semantics changed, and `trace_path` / `query_graph` keep their single-target behavior.

---

## DIS-2026-016: WorkspaceIndex Collapsed Same-Name Entity Edges Across Files

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-15 |
| **Environment** | Claude + Clean-CTX (live multi-project Angular workspace) |
| **Repository/context** | Real multi-project Angular workspace; two unrelated classes sharing the semantic name `LoadingComponent` (one per project), each with its own constructor-injected services and its own selector |
| **Symptom** | `find_entities("LoadingComponent")` correctly returned both file occurrences and the unique `HasSelector` edges survived, but `forward_edges` returned only one project's `Injects` edges (the other project's overlapping injections disappeared), and `reverse_edges` undercounted the real consumers of the shared services. |
| **Root cause** | `WorkspaceIndex::EdgeKey` identified an edge by `(relation, subject identity, object identity)` only. Both files asserted the same triple, so the second `add_edges` call hit `edge_set.insert == false` and `continue`d before recording `file_edges`, `forward`, or `reverse` — its evidence never entered the index. `remove_file` then deleted the single shared `edge_set` key and pruned the adjacency indexes by entity-reference provenance, so removing or recompiling one file also destroyed the other file's equivalent evidence. The defect lived in generic edge storage, not in Angular extraction. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic `WorkspaceIndex` fixtures; no scale dependency. |
| **Local regression** | `src/tests/workspace/index_edge_occurrence.rs` (`RED-E1`–`RED-E5`) and `src/tests/workspace/index_edge_lifecycle.rs` (`RED-E6`–`RED-E11`, including the three-project collision, compile-order independence, and a dotnet-shaped cross-domain case); `src/tests/workspace/index.rs` (`same_edge_inserted_twice_is_indexed_once`, `edge_occurrence_dedup_preserves_cross_file_evidence`, `counters_reflect_insertion_and_dedup`); `src/tests/workspace/index_queries.rs` (`resolve_selector_unique_despite_cross_file_duplicate_selector`); `src/tests/workspace/index_performance.rs` (file-local removal work unchanged). |
| **Live scenario required?** | Yes — re-run the multi-project case: both `LoadingComponent` occurrences keep their complete `Injects` evidence through `forward_edges`, shared-service `reverse_edges` counts both consumers with their file provenance, and removing or recompiling one project leaves the other project's overlapping edges intact. |
| **Architectural invariant** | `IDX-002` |
| **Status** | Fixed (live re-verification pending) |

**Resolution:** edge identity is now occurrence-aware. `EdgeKey` in `src/workspace/index/edges.rs` adds the asserting source occurrence to the semantic triple, so a repeated extraction within one file still dedups while the same triple asserted by a different file is retained as its own evidence record; forward/reverse now store `StoredEdge { asserting_file, edge }`. `remove_file` (`src/workspace/index/remove.rs`) is driven solely by `file_edges[file]` and drops exactly the occurrences that file asserted, so removal work stays proportional to the affected file (no whole-index scan). Semantic entity identity is unchanged — `EntityKey = (domain, entity_type, name)` (Model C) — and no semantic layer, `SemanticRelation`, or query contract was modified.

---

## DIS-2026-015: Angular `Injects` Edges Missed Bare Typed Constructor Parameters

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-15 |
| **Environment** | Claude + Clean-CTX v0.6.4 (live Angular workspace) |
| **Repository/context** | Real Angular workspace; single-service consumer measurement (workspace size and non-Angular languages not recorded) |
| **Symptom** | `workspace_query(type="reverse_edges", domain="angular")` returned 33 consumers of one service, while independent source inspection found 42 real constructor-injected consumers. All 9 missing consumers (≈21 %) used bare typed constructor parameters without a parameter-property modifier. |
| **Root cause** | `src/angular_meta/decorators.rs::extract_constructor_injects` gated every parameter on a literal modifier prefix (`private ` / `protected ` / `public ` / `readonly private ` / `readonly protected ` / `readonly public `) *before* splitting the `:` type. `constructor(foo: FooService) {}` and `constructor(readonly foo: FooService) {}` were skipped outright, and `constructor(@Inject(TOKEN) api: ApiClient) {}` was skipped because the modifier test ran before the type was reached. That `injects` list is the sole input to `SemanticRelation::Injects`, so the undercount propagated unchanged into `WorkspaceIndex` and the query layer. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic extraction fixture; no scale dependency. |
| **Local regression** | `src/tests/angular_meta/constructor_injects.rs` (21 extraction-level cases: `RED-DI1`–`RED-DI10` plus parity, decorator-preservation, eligibility, guard, and `Φinjects:` projection cases) and `src/tests/angular_meta/constructor_di_edges.rs` (15 production-path cases through `IRCompiler` → `MetaLayerPass` → `AngularMetaLayer` → `class_to_semantic_edges` → `WorkspaceIndex`, including `RED-W1`–`RED-W3`). |
| **Live scenario required?** | Yes — re-run the real service-consumer reverse-edge measurement and confirm the 9 bare-parameter consumers appear while the previously reported 33 remain, with no duplicates and no unrelated plain TypeScript classes. |
| **Architectural invariant** | `ANG-DI-001` |
| **Status** | Fixed (live re-verification pending — the corrected live count has not yet been measured) |

**Resolution:** constructor-injection extraction now lives in `src/angular_meta/constructor_injects.rs` and is modifier-independent: parameter-property modifiers are stripped as whole words rather than required, parameter decorators are skipped so the declared type stays reachable, and the parse tolerates multiline layouts, trailing commas, and line breaks between decorator / modifier / name / type. One unified extraction feeds both `class_to_semantic_edges` and the `Φinjects:` marker, so no second recognition path can emit a duplicate edge; `WorkspaceIndex` and the query layer were deliberately left untouched (no downstream workaround). Injection identity (parameter type), type eligibility, and the Angular class-level gate are unchanged, so ordinary TypeScript constructors still produce no Angular edges.

---

## DIS-2026-014: CBM Inbound Caller Evidence Was Accepted Without C# Arity Verification

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-15 |
| **Environment** | Clean-CTX v0.6.4 `cbm_proxy` inbound-caller path (recorded from commit `dc5dff1`; live environment details were not recorded in the commit) |
| **Repository/context** | C# repository indexed by CBM and queried through the `cbm_proxy` inbound-caller path (size not recorded) |
| **Symptom** | Not recorded in the commit — the shipped change is a correctness hardening of the inbound caller evidence surfaced through `cbm_proxy`. |
| **Root cause** | Caller evidence produced by CBM was consumed without checking the candidate C# source's structural argument shape (arity / overload compatibility), so a call site that cannot match the queried declaration's parameter shape could still be surfaced as a verified dependency. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — structural verification over candidate C# source with deterministic statuses. |
| **Local regression** | `src/tests/cbm/caller_verify.rs` (verification statuses, arity rejection, ambiguity, unverifiable evidence). |
| **Live scenario required?** | Not recorded in the commit; a live C# workspace re-query is advisable to confirm the reported rejection/ambiguity counts. |
| **Architectural invariant** | N/A — CBM remains the candidate provider; Clean-CTX source parsing remains the semantic authority. |
| **Status** | Fixed (live context and re-verification not recorded) |

**Resolution:** `src/cbm/caller_verify.rs` verifies CBM-derived inbound caller candidates against candidate C# source structure and classifies evidence as `VerifiedCompatible`, `RejectedArityMismatch`, `Ambiguous`, or `Unverifiable`; `src/cbm/caller_verify_proxy.rs` applies it inside `cbm_proxy` after CBM returns candidate evidence and before the response is compressed, recovering candidate file paths only from supplementary CBM queries. Verification annotates the response with deterministic counts and never inserts a relationship into `WorkspaceIndex`.

---

## DIS-2026-013: Angular Reactive Forms Reconstruction Duplicated Artifacts and Dropped Nested Groups

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-14 (recorded from commit `01eb3e4` and its own 0.6.4 changelog entry; no live measurement was recorded) |
| **Environment** | Clean-CTX v0.6.4 Angular Reactive Forms meta-layer |
| **Repository/context** | Angular repository using `FormBuilder` / `FormGroup` construction (live context and size not recorded in the commit) |
| **Symptom** | Not recorded in the commit — the shipped regression class was duplicate logical artifacts for repeated construction of the same form/control/array, and loss of recursive structure for nested `FormGroup` fields. |
| **Root cause** | Reactive-form reconstruction emitted one artifact per construction site instead of merging compatible structural evidence, and nested group traversal did not recurse into descendant groups/controls/arrays. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes — deterministic structure fixture suite. |
| **Local regression** | `src/tests/angular_meta/reactive_forms_structure.rs` with `src/angular_meta/reactive_forms_normalize.rs`. |
| **Live scenario required?** | Not recorded; the commit's own 0.6.4 changelog entry is the acceptance record for that release. |
| **Architectural invariant** | N/A |
| **Status** | Fixed (documented in the 0.6.4 changelog section) |

**Provenance:** this entry is included for discovery-log completeness of the commits shipped between the 0.6.4 changelog update and v0.6.5. Unlike `DIS-2026-009`–`DIS-2026-012` and `DIS-2026-015` it was not captured from a recorded live scenario, so its symptom/context fields state only what the commit documents. The fix merges compatible structural evidence in stable source order, preserves recursive nested group/control/array structure with descendant validators, and keeps lower fidelities compact without false control markers.

---

## DIS-2026-012: Live CBM Tests Serialized Bodies but Retained Multiple Subprocesses

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-12 |
| **Environment** | Clean-CTX v0.6.4 live CBM E2E tests in the complete test binary |
| **Repository/context** | Process-scoped shared live-CBM fixture plus multi-root and edit/reindex fixture tests. |
| **Symptom** | Live CBM tests passed alone but intermittently failed with pipe/proxy errors when run in the complete test group. |
| **Root cause** | `#[serial(cbm_live)]` serialized test bodies, but two fixture tests constructed private CBM-enabled `McpState` values while the static shared state retained its subprocess for the entire test process. After consolidating the subprocess, eight duplicate and unasserted preflight calls in the multi-root test could trip the now-shared circuit before fixture indexing; fallible assertions made while holding `graph_bridge` then poisoned the mutex and cascaded into the next test. |
| **Classification** | Emergent |
| **Reproducible locally?** | Yes — source/lifecycle audit; grouped failure was intermittent. |
| **Local regression** | `src/tests/cbm/e2e.rs` shared-state identity test plus the live tests in `e2e_multiroot.rs` and `e2e_reindex.rs`, which now acquire only `shared_live_state()` and use panic-safe workspace restoration. |
| **Live scenario required?** | Yes — run the complete live CBM E2E group repeatedly in one test process and confirm the proxy, multi-root, and edit/reindex tests remain green. |
| **Architectural invariant** | At most one live E2E `McpState` → `GraphBridge` → CBM subprocess at a time; a degraded instance is dropped and reaped before a serial successor launches its replacement. |
| **Status** | Verified |

**Resolution:** the generated fixture is now a configured additional root owned
by the shared live state. Both fixture tests index and query that root through
the shared bridge instead of constructing private CBM-enabled states. Duplicate
unasserted preflight calls were removed, fallible setup results are asserted only
after releasing the graph mutex, and a scoped workspace guard restores the prior
active root during normal return and panic unwinding. A degraded shared process
is dropped before its serial successor starts a replacement. The oversized E2E
file was decomposed along test boundaries.

**Verified 2026-09-12:** the complete `cbm::tests::e2e` group passed with
`--all-features`, including the proxy, multi-root, and edit/reindex scenarios.

---

## DIS-2026-011: Filesystem Hydration Had No Safe Default Traversal Scope

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-12 |
| **Environment** | Clean-CTX v0.6.3 with CBM disabled against a real multi-root workspace |
| **Repository/context** | Normal source roots containing dependency, VCS metadata, build-output, generated, and cache trees; default empty `exclude_patterns`. |
| **Symptom** | A `workspace_query reverse_edges` filesystem fallback recursively scanned supported-extension files in heavyweight irrelevant trees and did not return within a practical period, requiring manual interruption. |
| **Root cause** | The dedicated `WalkDir` fallback pruned entries only through user-configured `is_excluded`; default configuration supplies no patterns. |
| **Classification** | Emergent |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_7.rs` RED-S1 through RED-S9. Test-only counters prove 2,048 supported files under `node_modules` are neither considered nor read while legitimate primary/additional-root source remains searchable. |
| **Live scenario required?** | Yes — with CBM disabled and empty exclusions, repeat `reverse_edges` plus another hydration query against the real multi-root workspace and inspect provider/completion/candidate metadata. |
| **Architectural invariant** | WSC-001/WSC-002 |
| **Status** | Fixed locally; live acceptance pending |

**Resolution:** filesystem candidate discovery now prunes an exact, always-on
set of conventional metadata/dependency/build/cache directories before walking
their contents, then applies configured exclusions additively. The provider
contract, trusted candidate-path boundary, exhaustive processing semantics,
symlink behavior, and partial status on traversal/read failure are unchanged.
No numeric work budget or timeout was introduced.

---

## DIS-2026-010: workspace_query Hydration Silently Required CBM

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-12 |
| **Environment** | Clean-CTX without an installed or usable CBM bridge |
| **Repository/context** | Fresh session with relevant source files under the primary workspace root or configured `additional_roots`, but not yet compiled into WorkspaceIndex. |
| **Symptom** | Hydration-eligible queries ran but discovered and compiled zero candidates whenever CBM was absent. The response was indistinguishable from a successful workspace search that genuinely found nothing. |
| **Root cause** | Candidate discovery returned an empty vector immediately when `graph_bridge` was `None`; no CBM-independent provider existed and the metadata described counts rather than discovery-provider coverage. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_6.rs` RED-F1 through RED-F10 (no-CBM primary/additional-root hydration, reverse consumers, authority isolation, truthful zero results, healthy-CBM preservation, degraded fallback, exhaustive candidates, deduplication, and already-indexed exclusion). |
| **Live scenario required?** | Yes — disable CBM and run `find_entities`, `forward_edges`, and `reverse_edges` without manually compiling targets; repeat for an additional-root-only entity and inspect discovery metadata. |
| **Architectural invariant** | WSC-001/WSC-002 |
| **Status** | Fixed locally; live acceptance pending |

**Resolution:** healthy CBM remains the preferred candidate provider. A missing,
unavailable, failed, or root-incomplete CBM discovery automatically falls back
for the affected configured roots to deterministic filesystem enumeration.
Only extensions accepted by Clean-CTX's language registry are read, configured
exclusions are honored, and literal query-name matches contribute paths only.
Every candidate still crosses trusted-path validation and the existing
Clean-CTX compilation pipeline. Provider, completion, and fallback metadata
distinguish a successful empty scan from discovery that could not run.

---

## DIS-2026-009: Five-File Hydration Cap Silently Truncated Semantic Results

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-12 |
| **Environment** | Clean-CTX deterministic hydration regressions, distilled from the real `reverse_edges` scenario with approximately 13 inbound consumers |
| **Repository/context** | Hydration-eligible `workspace_query` operations whose relevant semantic evidence spans more than five candidate files, including configured multi-root workspaces. |
| **Symptom** | Candidate discovery could return every relevant file, but hydration compiled only the first five normalized paths and reran the WorkspaceIndex query as though the resulting semantic subset represented the query. For `reverse_edges`, a target with more than five consumers therefore returned at most five authoritative edges without identifying the result as truncated. |
| **Root cause** | A defensive automatic-work bound (`HYDRATION_MAX_CANDIDATES = 5`) was applied inside semantic candidate selection. Because the response had no semantic truncation contract, the resource policy silently changed query meaning instead of merely controlling execution. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_5.rs` RED-C1 through RED-C6; superseded cap assertions RED-12, RED-17, and RED-27 now protect exhaustive processing after deduplication and already-indexed exclusion. |
| **Live scenario required?** | Yes — rerun `workspace_query reverse_edges` against a target with more than five inbound consumers and confirm all Clean-CTX-authoritative consumers are returned and `candidates_compiled` is not capped at five. |
| **Architectural invariant** | WSC-001/WSC-002 |
| **Status** | Fixed locally; live acceptance pending |

**Resolution:** hydration still performs exactly one discovery/compile/rerun cycle,
but candidate selection now normalizes paths, deduplicates them, excludes files
already represented in WorkspaceIndex, sorts deterministically, and returns the
entire remaining set. Every selected path still crosses trusted-root validation
and the normal Clean-CTX compilation boundary. No replacement numeric cap was
introduced. Any future resource guard that prevents exhaustive candidate
processing must expose explicit partial/truncated coverage.

---

## DIS-2026-008: reverse_edges Hydration Compiled the Target Instead of Its Consumers

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-11 |
| **Environment** | Claude + Clean-CTX v0.6.3 field testing (`workspace_query reverse_edges` across configured roots) |
| **Repository/context** | Multi-project workspace with a target service declaration in one file and inbound consumers in separate files. |
| **Symptom** | Hydration discovered and compiled one candidate—the target entity's declaration file—but `reverse_edges` remained empty even though direct CBM graph querying showed inbound consumers. |
| **Root cause** | Every hydration-eligible query used declaration/name search. That is correct when the target's declaration emits the desired evidence, but reverse semantic edges are emitted while compiling caller/referrer files. Compiling only the target declaration cannot populate those inbound WorkspaceIndex edges. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_4.rs` RED-23 through RED-28; `src/tests/cbm/project_search.rs::inbound_reference_query_is_path_only_exact_and_escaped` and `explicit_project_inbound_error_preserves_active_project`. |
| **Live scenario required?** | Yes—repeat the original additional-root `reverse_edges` query and confirm consumer candidates compile and produce authoritative WorkspaceIndex edges. |
| **Architectural invariant** | WSC-002 |
| **Status** | Fixed |

**Resolution:** bounded hydration now selects discovery by query semantics.
`find_entities`, `forward_edges`, and `transitive_dependencies` retain name-based
declaration discovery. `reverse_edges` performs a project-explicit inbound graph
query for the existing simple target name and projects caller file paths only;
known definition-only `DEFINES` and `DEFINES_METHOD` relationships are excluded.
Each path still crosses the existing trusted-root and Clean-CTX compilation
boundary. CBM relationship type, direction, identity, and cardinality never enter
WorkspaceIndex. Multi-project merging, deterministic ordering, the global
five-file cap, one-cycle hydration, advisory readiness, and coverage metadata are
unchanged.

**Superseded 2026-09-12 by DIS-2026-009:** the five-file cap was removed because
it could silently return incomplete semantic results. The remaining one-cycle,
authority, ordering, readiness, and metadata behavior is unchanged.

---

## DIS-2026-007: Session-Local CBM Readiness Suppressed Queryable Persisted Graphs

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-11 |
| **Environment** | Claude + Clean-CTX v0.6.3 field testing (`workspace_query` against a configured additional root) |
| **Repository/context** | Primary workspace plus an additional repository whose persisted CBM graph was immediately queryable through a project-explicit `cbm_proxy` call in the same process. |
| **Symptom** | Repeated hydration returned `hydration_attempted: true`, zero discovered/compiled candidates, and zero semantic results even though direct project-explicit CBM search returned the additional-root entity. |
| **Root cause** | Registration and enumeration were correct. Hydration reached the additional project, but required `ensure_indexed_for(project)` to return `Ready` before calling `search_in_project`. Bridge-local `StillIndexing`/`NotStarted` or `Failed` bookkeeping therefore suppressed a query against an already-usable persisted CBM graph. The local readiness map describes this process's indexing activity; it is not evidence that persisted graph data is absent or unusable. |
| **Classification** | Emergent |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/cbm/project_search.rs::additional_root_registration_matches_proxy_resolution_and_enumeration`; `src/tests/mcp/workspace_query_3.rs` RED-19 through RED-22 (queryable graph under `StillIndexing`/failed readiness, actual search-failure isolation, and deterministic bounded project coverage metadata). |
| **Live scenario required?** | Yes — rerun the original additional-root-only query and confirm the candidate is searched/compiled while `project_coverage` reports the observed local readiness. |
| **Architectural invariant** | WSC-001/WSC-002 |
| **Status** | Fixed |

**Resolution:** hydration treats bridge-local readiness as diagnostic state, not
search authority. Any available bridge attempts the existing project-explicit
search for `Ready`, `StillIndexing`/`NotStarted`, and locally failed projects.
Only actual CBM unavailability skips search; an actual project search failure is
reported locally and does not prevent other configured projects from being
searched. Bounded `project_coverage` metadata makes searched, search-failed, and
skipped projects visible without exposing CBM payloads or changing semantic
authority.

---

## DIS-2026-006: workspace_query Hydration Searched Only the Active CBM Project

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-11 |
| **Environment** | Claude + Clean-CTX v0.6.3 field testing (`workspace_query` against a configured multi-root workspace) |
| **Repository/context** | Primary workspace plus an already-indexed repository configured through `additional_roots`; the requested entity existed only in the additional repository. |
| **Symptom** | Hydration reported `hydration_attempted: true` but `candidates_discovered: 0` when the entity existed only in an additional root. Project-aware CBM tools could find the entity in that repository. |
| **Root cause** | `discover_candidate_paths` called `GraphBridge::search(query_name)`, which resolved its project through `self.project_str()` and therefore searched only the bridge's single active project. Hydration neither enumerated the existing configured project↔root map nor anchored project-relative `GraphNode.file` values to the root that produced them. Its validation call also supplied an empty `additional_roots` list. |
| **Classification** | Emergent |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_3.rs` — RED-15 additional-root-only discovery, RED-16 active-project preservation across success/partial-failure/empty/error paths, RED-17 merged-pool global cap/dedup/index exclusion/deterministic order, RED-18 per-project failure isolation; `src/tests/cbm/project_search.rs` — configured-project enumeration and explicit-search state preservation. |
| **Live scenario required?** | Yes — repeat `workspace_query` against the original configured primary + additional-root workspace and confirm the additional-root-only entity is returned. |
| **Architectural invariant** | WSC-002 (CBM discovers candidate file identities across configured projects; Clean-CTX alone determines WorkspaceIndex semantics) |
| **Status** | Fixed |

**Resolution:** hydration now enumerates the bridge's existing primary-root +
`additional_roots` project map and searches every configured project explicitly,
without calling `set_project`. Each relative CBM path is joined to the canonical
root mapped to the project that returned it. The merged pool is normalized,
deduplicated, filtered against already-indexed files, sorted once, and capped at
five files total before every selected path passes the unchanged trusted-root
validation and Clean-CTX compilation boundary. A failure in one project is local
to that project; coverage remains partial and the original query still reruns once.

**Superseded 2026-09-12 by DIS-2026-009:** cross-project candidate merging remains
unchanged, but the five-file truncation step was removed. Every unique,
previously-unindexed discovered candidate is now processed.

---

## DIS-2026-002: IR Consumptive Pattern Compression Orphaning Method Identities

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-03 |
| **Environment** | Claude + Clean-CTX IR pipeline (production `PassPipeline::default_production()`) |
| **Repository/context** | Real Angular workspace; a component constructor calling `.subscribe()` (DI param + RxJS callbacks in body). |
| **Symptom** | IR compilation failed with `[E007] DATAFLOW references unknown method 'M16'; [E003] FLAGS references unknown method 'M16'` — the compile itself returned `Err`, not merely corrupt output. |
| **Root cause** | The consumptive CTOR pattern compressed `DefMethod(M)` into `Pattern(CTOR, ..., M)` while leaving surviving `DataFlow(M, ...`/`Flags(M, ...)` ops with no valid owner. The validator registers method identities ONLY from `DefMethod`; `PatternOp` has no payload slot for DataFlow / SideEffect / ExecutionContext / ControlFlow facts. The first invalid state was created by CTOR pattern compression — NOT by TypeScript extraction (the pre-pattern stream was fully valid)and NOT by validation. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/ir/regression_ctor_pattern_orphan.rs` (covers CTOR, EMPTY_CTOR, and the original orphan scenario) |
| **Live scenario required?** | No |
| **Architectural invariant** | IR identity-preservation invariant for consumptive pattern transformations |
| **Status** | Fixed |

**Distillation note:** nested-callback depth alone was ruled out; the four initially suspected AST candidates (bare arrow parameter, nested plain callback, optional chaining, destructuring) were all ruled out;and the TypeScript constructor/arrow capture-kind coverage gap remains a **separate** issue, NOT causal. The fix deliberately declines CTOR / EMPTY_CTOR compression when an unrepresentable M-reference exists; healthy CTOR compression remains intact when only representable trailing `Flags` are present.

**Architectural invariant (IR pattern-transformation layer, framework/language-agnostic):**

> A consumptive IR pattern must never consume a `DefMethod(M)` while leaving behind surviving IR operations that reference `M` without preserving a valid representation/ownership relationship. If a consumptive pattern cannot represent an M-referencing operation (`DataFlow`, `SideEffect`, `ExecutionContext`, `ControlFlow`) within the resulting pattern representation, it must decline compression rather than consume the `DefMethod` and orphan the reference.



Evidence (minimal trigger: constructor with ≥1 DI parameter + `.subscribe()` in its body):

```text
Pre-pattern (valid):                   After faulty CTOR compression (invalid):
  DefMethod(M)                          Pattern(CTOR,, M)
  Param(M, ...)                         DataFlow(M,, ...)
  Return(M, ...)                       Flags(M,, ...)
  Flags(M,, ...)
  DataFlow(M,, ...)
  Flags(M,, ...)

Result: [E007] DATAFLOW references unknown method M; [E003] FLAGS references unknown method M
```
---

## DIS-2026-005: workspace_query Coverage/Hydration Cardinality Coupling

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-11 |
| **Environment** | Claude + Clean-CTX field testing (`workspace_query` against real repositories) |
| **Repository/context** | Large real workspace; WorkspaceIndex populated demand-side only by the files compiled so far in the session. |
| **Symptom** | `workspace_query` coverage/hydration behavior was coupled to result cardinality: `initial count == 0` was treated as the hydration trigger and `initial count > 0` was treated as sufficient coverage. A partial index could report confidently wrong coverage in both directions — a fresh session (empty index) and a partially-warmed session (non-zero but incomplete index) both failed to discover relevant files that original CBM could resolve. Additionally, the hydration design risked CBM graph semantics (relationship counts/types/direction) leaking into `workspace_query` results rather than CBM merely suggesting which files Clean-CTX should compile. |
| **Root cause** | Hydration eligibility had been defined from result cardinality instead of from coverage capability and query type. A partial `WorkspaceIndex` is the expected steady state (session-scoped, demand-populated), so cardinality is never evidence of either "empty meaning needs hydration" or "non-empty meaning complete". Separately, the CBM→hydration boundary must be deliberately narrowed: CBM supplies candidate file paths only (`GraphNode.file`), and every other CBM graph semantic is discarded before compilation. Only Clean-CTX semantic edges produced by compiling accepted candidates may enter `WorkspaceIndex`. |
| **Classification** | Emergent |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query_2.rs` — RED-9 partial-nonzero hydration (non-zero initial result remains hydration-eligible; authoritative initial result survives), RED-10 fresh-index hydration (empty index is hydration-eligible), RED-11 CBM authority/cardinality isolation (CBM relationship count never determines results), RED-12 hard bound (max 5 previously-unindexed candidates, deterministic lexical order, one hydration pass, no second cycle), RED-13 candidate-with-no-matching-relation (zero fabricated relationships), RED-14 per-query-type eligibility classification (`find_entities`/`forward_edges`/`reverse_edges`/`transitive_dependencies` eligible; `entities_in_file`/`has_cycle` never hydrate). |
| **Live scenario required?** | Yes — bounded CBM candidate discovery with a live CBM binary against real workspaces (does CBM resolve useful candidate files for the requested entity? do accepted candidates pass `resolve_file_path_checked`?). |
| **Architectural invariant** | WSC-001 (authoritative facts do not imply authoritative coverage), WSC-002 (CBM discovers candidates; Clean-CTX alone determines WorkspaceIndex semantics) — `docs/ARCHITECTURAL_INVARIANTS.md` |
| **Status** | Fixed |

**Distillation note:** the corrected flow is
`initial WorkspaceIndex query → hydration eligibility (query-type identity, NOT cardinality) →
bounded CBM candidate-file discovery → extract candidate paths ONLY → discard CBM graph semantics →
dedup / exclude already-indexed / deterministic order / max 5 → resolve_file_path_checked →
compile_file_ir_focused → Clean-CTX semantic extraction → semantic_edges → WorkspaceIndex →
rerun ORIGINAL WorkspaceIndex query exactly once → final result`.
Truthful hydration metadata (`hydration_attempted`, `candidates_discovered`, `candidates_compiled`)
is returned without implying completeness. If more valid candidates exist than the cap permits,
coverage necessarily remains partial.

**Superseded 2026-09-12 by DIS-2026-009:** the `max 5` selection step was an
incorrect semantic truncation mechanism. Selection is now exhaustive after
normalization, deduplication, and already-indexed exclusion; no replacement
numeric cap exists.

**Test injection seam:** `TEST_HYDRATION_CANDIDATES` (cfg(test)-only static, following the
`TEST_INJECTED_IR_FAILURE` pattern) injects ONLY candidate file paths — never semantic edges,
entities, precompiled IR, or results. Injected paths flow through the full production hydration
path, so the regressions prove CBM discovers where Clean-CTX should look while Clean-CTX alone
determines what semantic facts exist.

---

## DIS-2026-004: IR Nested-Type Ownership Reparents Enclosing-Class Methods + C# Static-Flag Contamination

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-08 |
| **Environment** | Claude + Clean-CTX v0.6.1 (production `provide_code_context` → `apply_edit`) |
| **Repository/context** | Real C# repository; a non-static service class containing an instance constructor, instance methods, and a nested public enum with 2+ members (registered as a scoped DI service elsewhere). |
| **Symptom** | `apply_edit` with `replace_body` reported `unit not found: OrderService.SomeMethod` for a method that `provide_code_context(fidelity="edit")` confirmed existed and was byte-identical. The same `ClassName.MethodName` convention succeeded moments earlier against a plain static helper class and a plain test class. `provide_code_context` also rendered `cl: EXPORT STATIC` for the non-static class and showed the nested enum's members as stray class-level fields outside the enum boundary. |
| **Root cause** | Two independent IR defects, both in the production `CoreIRPass` / `CSharpLayer` path. (1) `CoreIRPass` used a single `current_class` slot with no scope restoration — a nested `enum.root`/`class.root` capture overwrote it, reparenting every member lexically after the nested type to that type; `UnitTable::materialize` then keyed the method under the nested type's name, so `UnitTable::resolve` returned `NotFound`. (2) `CSharpLayer::extract_class_flags`/`extract_method_flags` scanned the FULL declaration node (head + body) with `contains("static")` substring matching, so any `static` token in a body, comment, or string contaminated the enclosing class/method flags. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/ir/pipeline.rs` (nested-enum / nested-class span-containment ownership, enum members stay inside the enum); `src/tests/ir/layers/mod.rs` (C# static-flag isolation + legitimate static class/method recognition); `src/tests/edit/spans.rs` (`apply_edit` end-to-end identity + render check); `src/tests/ir/rust_integration.rs` (struct-following-impl methods attach and emit Flags) |
| **Live scenario required?** | No |
| **Architectural invariant** | Structural type invariant (a declaration's modifiers reflect only its own declaration head, never body tokens) + nested-type boundary invariant (members of a nested type stay inside it and never leak into the enclosing type) |
| **Status** | Fixed |

**Fix (2026-09-08):** `PassContext` now carries a span-keyed `TypeScope` stack in `src/ir/pipeline.rs`: type roots push `[start_byte, end_byte)` scopes, member captures refresh ownership to the innermost scope containing them (mirroring the proven `diff/builder.rs` containment contract), and `impl.root` reuses the struct's `class_id` (no duplicate `DefClass`) so methods attach and emit their Flags. C# flag extraction in `src/ir/layers/csharp.rs` now inspects only the declaration head with word-boundary token matching via `has_head_modifier`; legitimate `public static class` and static methods remain flagged. C# `enum.root` naming routed through `extract_class_name` instead of the Rust-only `pub`-stripper.

**Distillation note:** The live trigger shape (non-static class + nested enum + method after the enum) was distilled to cheap deterministic local fixtures. A separate process finding: the `rust_integration` suite is `#[cfg(feature = "rust")]` and never ran under default features, so the defect was only exercisable in CI's `--all-features` build.

**Minimal trigger:**
```csharp
public class OrderService
{
    public enum OrderStatus { Pending, Shipped }   // nested type

    public void MethodAfterEnum() { }              // reparented to OrderStatus → "unit not found"
}
```

---

## DIS-2026-003: IR CTOR Compression Orphaning with Empty Constructor + Method-Scoped References

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-04 |
| **Environment** | Clean-CTX IR pipeline (production `PassPipeline::default_production()`) |
| **Repository/context** | Angular component with empty constructor using a private parameter property (`constructor(private store: Store) {}`) and Store operations in a separate method (`ngOnInit`). |
| **Symptom** | IR compilation failed with `[E003] FLAGS references unknown method 'M5'`. |
| **Root cause** | The consumptive CTOR pattern compressed the empty constructor's `DefMethod(M)` into `Pattern(CTOR, ..., M)` while leaving surviving IR operations that reference a different method (`ngOnInit`, M5) without a valid owner. The existing CTOR orphan-prevention fix (DIS-2026-002) covers the case where unrepresentable M-referencing operations exist within the constructor body, but not the case where the constructor is empty and the references belong to a separate method. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/ir/regression_ctor_pattern_orphan.rs::edit_fidelity_param_property_ctor_does_not_orphan_flags` (compiler-level regression at Fidelity::Edit); `src/tests/mcp/workspace_query.rs::builtin_decorated_class_and_ngrx_semantic_names_production_path` (production-path fixture restored with the empty constructor) |
| **Live scenario required?** | No |
| **Architectural invariant** | Same as DIS-2026-002 (IRPAT-001): IR identity-preservation for consumptive pattern transformations |
| **Status** | Fixed |

**Fix (2026-09-04):** `op_is_unrepresentable_method_ref` in `src/ir/patterns.rs` now treats `Body(M, ...)` as an unrepresentable M-referencing operation, alongside `DataFlow`, `SideEffect`, `ExecutionContext`, and `ControlFlow`. At Edit fidelity the TS language layer emits `Body(M)` between a constructor's `Return(M)` and its trailing `Flags(M, ["PRIVATE"])` (parameter property); the `Body` op breaks the wrapper's adjacent trailing-Flags run, so `Flags(M)` would survive as an orphan (E003) if compression consumed `DefMethod(M)`. The extended guard makes the CTOR/EMPTY_CTOR patterns decline compression for that region, preserving the full valid sequence. The production-path fixture (empty constructor restored) and the new compiler-level Edit-fidelity regression both pass.

**Distillation note:** This defect predates the selector-representation fix and is unrelated to it. The existing CTOR orphan-prevention fix (DIS-2026-002) covers the constructor-body case but NOT this shape. Separate IR/compiler defect.

**Minimal trigger:**
```typescript
@Component({ selector: 'widget-shell' })
export class ShellComponent {
  constructor(private store: Store) {}  // EMPTY + private parameter property
  ngOnInit() {
    this.store.pipe(select('panelState'));  // Store ops in SEPARATE method
    this.store.dispatch({ type: TOGGLE_PANEL });
  }
}
```

---

**Resolved (2026-09-04):** Production fix implemented in `src/ir/patterns.rs`. The earlier fixture workaround (removing the empty constructor) is no longer needed — and has been reverted so the production-path test exercises the exact DIS-2026-003 scenario again.
---
## DIS-2026-001: workspace_query MCP Response Envelope Bypass

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-02 |
| **Environment** | Claude + Clean-CTX (post-0.5.0 structured-output migration) |
| **Repository/context** | Large heterogeneous real workspace |
| **Symptom** | Every `workspace_query` response carried bare domain fields (`entities`, `edges`, `count`, ...) directly under JSON-RPC `result` with NO MCP `content` channel — a schema-validating MCP client had nothing renderable to display. |
| **Root cause** | All six `workspace_query` sub-handlers built bare `result` objects, bypassing the canonical MCP `CallToolResult` envelope (`content` + optional `structuredContent`/`_meta`) established by the 0.5.0 migration. The handler was introduced after the migration (commit `2d18377`, 2026-09-01) with no contract audit gate for new tools; its tests validated the ad-hoc result shape rather than the envelope. |
| **Classification** | Protocol |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query.rs` (all operations validated via `assert_valid_mcp_envelope` + `structuredContent`); `src/tests/mcp/phase3_contract.rs` (shared envelope helpers); `src/tests/mcp/envelope_contract.rs` (complete remaining produced-tool coverage) |
| **Live scenario required?** | No |
| **Architectural invariant** | MCP-001 (docs/ARCHITECTURAL_INVARIANTS.md) |
| **Status** | Fixed |

**Distillation note:** the root cause was NOT workspace complexity. It was an
MCP response-contract violation, reproducible with the existing sample corpus —
no large fixture was required. This is the canonical example of a **Protocol**
class discovery: field evidence surfaced it; a cheap deterministic Rust
regression now protects it permanently.
