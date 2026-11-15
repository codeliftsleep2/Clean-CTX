# ANG-R1 exact baseline

Agent: **Agent MaxHeadRoom**. Investigation evidence, 2026-11-14.

## Repository and execution identity

| Item | Observed value |
| --- | --- |
| Workspace | `C:\Users\MNasty\Desktop\RustContextLayerAI` |
| Branch | `(fix)Performance-Regression-Issues` |
| Entry HEAD | `d28570f115e68da767b82ec4e128f280c5d17752` |
| Origin | `https://github.com/codeliftsleep2/Clean-CTX.git` |
| Remote branch SHA | Same exact HEAD, verified with `git -c http.sslBackend=schannel ls-remote` |
| Entry working tree | Clean; no reset/rebase/stash/new branch |
| Delivered B11 baseline | Exactly current entry HEAD; no subsequent commits |
| Existing production binary | `target/debug/clean-ctx.exe` |
| Binary SHA-256 | `4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f` |
| Profile | `.clean-ctx/pcg-documents/profile.json` |
| Profile SHA-256 | `0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9` |
| Production document DB | `.clean-ctx/pcg-documents/documents.sqlite` |
| Launch directory | `.clean-ctx/pcg-documents/authoring` |
| Knowledge generation | `4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3` |
| Angular semantic / marker generations | **78 / 6**, `src/layers/meta/mod.rs` |
| Canonical pipeline generation | **1**, `src/ir/pipeline.rs` |

The existing isolated launch configuration disables optional CBM, proxy auto-start,
source persistence and source cache. Every knowledge request explicitly passes the
actual workspace. The profile retains `document_roots:["docs"]`, 2,048 files,
512 nodes, 2,048 edges and 33,554,432 observed bytes. Its legacy token-budget field
is not an architectural knowledge admission/completeness criterion. No profile,
database identity or user's normal workspace configuration was changed.

The B11 production build receipt reused the successful build after input/hash
verification; its captured source HEAD was `5d406e63` with the declared B11 working
changes. That historical build receipt is not a claim of an ANG-R1 build or a
clean-tree build at that older HEAD. ANG-R1 directly verified the executable hash.
Current Cargo tests build/use a test executable, separately from this stdio binary.

## Relevant chronology and source changes

No commits follow the delivered B11 baseline. The relevant comparison is against
the earlier accepted Angular implementation, not an imagined post-B11 change:

| Commit | Relevant effect | What it does not establish |
| --- | --- | --- |
| `4e375dfc` | Retains exact neutral resolve mapping authority | Public keys or resolver eligibility |
| `ddfa6c56` | Generation-78 exact written resolver membership and call-local admission batching | General Angular migration or runtime semantics |
| `7d23808d` | Ordered index membership companions | Declaration membership or all #174 work resolved |
| `a6a41b42`, `7afaf72c` | Bounded C# selection/dependency retrieval corrections | Angular witness/snapshot performance fixed |
| `ca51811c`, `442f795f` | Compression token recount and hierarchy placement corrections | Resolver performance acceptance |
| `1bf14c70`, `faa35ece`, `f8dbea51` | Structural-edit and diff ownership/selection corrections | Permission to change Angular public identity |
| `1771b837`, `4e264a65`, `feb3d0ef`, `db6ce3f7` | Encoder, demand-manifest, diff-source and delayed-authority ownership changes | A new Angular semantic generation |
| `8ce0a686` | Later deferred-work checkpoint, including scoped performance outcomes | Acceptance of every optimization candidate |
| `df4d0414` | Type-query namespace correction and Angular module query/currentness repair | Public resolver key registration |
| `d28570f1` | B11 governed admission/protected authority correction | Fresh Angular executable certification |

Direct diff `ddfa6c56..HEAD` shows route_objects and durable_semantics changes in
the examined Angular path are test-only work observations. Witness::current keeps
the original observations while adding counters. Native `namespace.rs` and
`type_queries.rs` have real later type-query behavior changes; index coverage has
real module currentness integration changes. These require scoped current evidence,
even though the resolver projection itself is unchanged.

## Actual production path

1. Registered public context/query acquisition reaches the existing hydration or
   explicit compilation path. `semantic_compilation/compile.rs::compile_source_artifact`
   captures `TypeScriptAuthority::capture_with_demands` using authorized roots,
   actual project and `module_metadata::DECLARATION_FIELDS`/bootstrap demanded APIs.
2. `capture.rs` and `declaration_facets` retain source/hash/declaration, separate
   type/value references, demanded syntax, mappings, package exports and witnesses.
   `DECLARATION_FIELDS` explicitly contains `resolve`.
3. `IRCompiler::compile_focused` runs the default `PassPipeline`; its real
   `MetaLayerPass::run` passes the compilation-local authority and invocation view
   to `LayerRegistry::evaluate_meta_layers_context`. This is not a standalone detector.
4. Angular combined evaluation reaches
   `layers/meta/angular_semantic.rs::extract_non_testing_edges`. The routing stage
   suppresses owned weak facts by source-object spans, then invokes canonical
   `route_resolvers::project` alongside independently owned routing families.
5. `SemanticCompilationArtifact::from_compiler` retains neutral evidence and the
   semantic result; `take_retained_authority` transfers the typed payload with the
   normal compatible projection. Canonical-only decomposition is not semantic publication.
6. `RetainedAuthority`, `SemanticSnapshot`, `DurableSemanticSnapshot`, compatibility
   validation and `sqlite_store/semantic_state.rs` carry/validate that payload in
   the existing source-owned session and SQLite lifecycle. `SemanticSnapshot::is_current`
   checks source/witness authority, relation replay and ordinary dependencies.
7. `index/coverage.rs::replace_semantic_projection_with_retained_authority` validates
   witnesses and relation evidence before source-owned index replacement. Edge keys
   preserve private canonical endpoints; existing public domain/type/name queries
   still address occurrence buckets, not a single canonical declaration selector.
8. `workspace_query` forward/reverse execution exposes `ResolvedBy` and declaration
   provenance through the existing public shape. This does not expose mapping keys.

This is a static production-path trace. Current tracked tests and production stdio
knowledge calls have distinct verification scopes; neither alone proves current
end-to-end Angular binary behavior through every acquisition/edit/restore path.

Concrete retained-transfer call sites were inspected in
`tool_handlers/core/provide/evaluate.rs`, `core/compress.rs`, `core/delta.rs`,
`tool_handlers/edit.rs`, `hydration/publication.rs` and `query/entities.rs`.
They take typed authority before decomposing the artifact. Hydration's
`commit_candidate` validates expected source/authority/dependencies through the
existing publication ticket before `replace_semantic_projection_with_retained_authority`.
`SqliteStore::save_context_with_retained_semantics` feeds the existing atomic
transaction. `query/edges.rs` prepares forward/reverse occurrence queries and
only the exact modern bootstrap forward identity adds conservative bootstrap
owner hydration. This confirms the bounded legacy discovery caveat in the matrix;
it does not certify every path's current execution.

## Historical applicability

The generation-78 binary receipt records 319 checks, 26 scenarios and three finished
sessions against binary `6d537af84b83afce6c832a9502273b461f20480afc2958760a3ea2a14da22cd7`.
Its 1,224 recorded source inputs now have **101 differing hashes**, no inferred
semantic regression. B4b's earlier comparison recorded 68 differences; that was a
different checkpoint and is preserved. Core route_resolvers, relation_evidence and
resolve_mappings source remain byte-identical to the accepted implementation.
The refreshed input comparison is retained in `target/ang-r1/generation78-drift.json`.
New/unlisted files are outside that old inventory: equality of listed files would
not certify their absence or correctness.
