# Semantic Query Performance Architecture Implementation Campaign — 2026-10-16

**Agent:** Agent MaxHeadRoom  
**Campaign status:** Phases 1–2 GREEN; Phase 3 request-local reuse implemented, focused verification pending

## Phase 1 — Static Tree-Sitter Query Reuse

**STATUS:** PHASE 1 BOUNDED GREEN; REPOSITORY-WIDE FINAL GATE NOT RUN

### Proven Defect

Every production `Query::new` call currently compiles a tree-sitter query for that observation. Several inputs are binary-owned constants whose definitions cannot change during the process lifetime. Other call paths accept or construct query text dynamically. A safe repair must reuse only explicitly registered static definitions and must not create an arbitrary-string cache.

### Exact Production Inventory

| Site | Input origin | Classification | Phase-1 disposition |
|---|---|---|---|
| `compression/capture_pipeline.rs` | Generic `&str` accepted by `run_capture_pipeline*` | Mixed API | Preserve dynamic entry point; add an explicit registered-static entry point |
| `compression/pipeline.rs` callers | `language_for_extension` returns one of `TS_QUERY`, `CS_QUERY`, `RS_QUERY`, `JAVA_QUERY` | `REGISTERED_STATIC` / `LANGUAGE_STATIC` | Route through registered identities |
| `angular_meta/template_ownership.rs` | `TS_QUERY` | `REGISTERED_STATIC` | Route through registered TypeScript identity |
| `mcp/tool_handlers/query/angular_templates.rs` | `TS_QUERY` | `REGISTERED_STATIC` | Route through registered TypeScript identity |
| `ir/invocation_evidence/collect.rs` | `TS_CALL_QUERY` | `REGISTERED_STATIC` | Route through registered TypeScript-call identity |
| `ir/pipeline/core.rs` | Runtime-composed capture query, conditionally appending Angular/NGRX fragments | `DYNAMIC_REQUEST` | Retain dynamic compilation |
| `diff/builder.rs` | Runtime-composed comparison query, conditionally appending C# evidence | `DYNAMIC_REQUEST` | Retain dynamic compilation |
| `angular_meta/detect.rs` | `ANGULAR_DECORATOR_QUERY` | `FRAMEWORK_STATIC` | Register Angular detector identity |
| `dotnet_meta/detect.rs` | `CS_ATTRIBUTE_QUERY` | `FRAMEWORK_STATIC` | Register .NET attribute identity |
| `dotnet_meta/detect.rs` | `CS_BASE_CLASS_QUERY` | `FRAMEWORK_STATIC` | Register .NET base-class identity |
| `spring_meta/detect.rs` | `SPRING_ANNOTATION_QUERY` | `FRAMEWORK_STATIC` | Register Spring detector identity |
| `edit/locate.rs` | Caller-supplied edit selector query | `EDIT_DYNAMIC` | Retain independent compilation |

No production `Query::new` site was found whose query text is derived directly from source contents, so the current `DYNAMIC_SOURCE` set is empty. The generic capture API remains capable of receiving arbitrary text and must therefore remain dynamic unless its caller selects a registered identity explicitly.

### Pinned Tree-Sitter Reuse Proof

The lockfile pins `tree-sitter` 0.26.10 (`Cargo.toml` specifies the compatible `0.26` line). The pinned Rust binding establishes:

- `Query` owns a non-null native `TSQuery` pointer plus its Rust-side capture and predicate metadata.
- `Drop for Query` invokes `ts_query_delete`; therefore the compiled object has clear owned lifetime.
- The binding explicitly implements both `Send` and `Sync` for `Query`.
- `QueryCursor` is documented as stateful and is also marked `Send + Sync`, but matching APIs mutably borrow it. Phase 1 will create a fresh cursor per observation and will never share cursor execution state.
- `Query::new(&Language, text)` compiles against the supplied grammar. A registered identity must therefore bind the query constant and its grammar/language constructor as one definition; query text alone is not a valid key.
- For a fixed binary, registered query text, linked grammar implementation, and Cargo feature configuration are process-lifetime constants. Binary replacement or restart naturally discards the compiled values.

This proves process-global immutable `Query` reuse is supported by the pinned binding. It does not authorize sharing parsers, trees, cursors, source buffers, or match iterators.

### Approved Architectural Boundary Proposed

Introduce a closed registered-query identity owned by the capture/query machinery. Each variant binds:

1. a stable semantic identity;
2. its exact static query constant;
3. its exact language/grammar constructor;
4. one process-lifetime lazy compilation result.

The API split should be explicit:

- registered-static execution accepts only the closed identity and obtains the compiled immutable query;
- dynamic execution continues to accept `Language + &str` and compiles per call;
- both paths converge only after compilation on one internal capture-walk accepting `&Query`;
- every observation constructs its own `QueryCursor`.

Do not infer registration by comparing strings or pointers. Do not expose a global `HashMap<String, Query>`. Runtime-composed IR/diff queries and edit selectors remain on the dynamic path.

Framework detectors may consume the same registered-query mechanism, but the mechanism must not infer framework authority. Registration only owns executable syntax machinery.

### Artifact Contract

**ARTIFACT:** registered compiled tree-sitter query  
**CARDINALITY:** one per closed registered identity  
**SIZE DRIVER:** fixed query definition complexity  
**OWNER:** process-local query execution machinery  
**LIFETIME:** process/binary lifetime  
**DEPENDENCIES:** exact static text, exact grammar constructor, linked grammar/tree-sitter versions, Cargo feature availability  
**INVALIDATION:** process restart or binary replacement  
**EVICTION:** none; bounded closed set  
**CONCURRENCY:** immutable shared `Query`; observation-local `QueryCursor`  
**RECONSTRUCTABLE:** yes  
**PERSISTENT?:** no

### Authority Analysis

The compiled query is execution machinery, not semantic evidence. Reuse changes neither source, parse tree, capture processing, ordering, completeness, nor framework authority. The static and dynamic paths must execute the same internal walk so that compilation lifetime is the only behavioral difference.

Compile failure must remain deterministic. The lazy cell should retain a failure for a registered binary-owned definition as well as a success, because neither its text nor grammar can change inside that process. The externally observed error/fallback behavior at each caller must remain unchanged.

### Structural RED Design

Add the smallest `#[cfg(test)]` instrumentation at the registered compilation boundary:

- increment by registered identity only when the lazy initializer actually invokes `Query::new`;
- do not increment on cache reads;
- keep dynamic-compilation observation separate so arbitrary dynamic strings never populate registered state;
- expose no production telemetry or resettable production state.

Use a test-owned cache instance, or a test-only registered identity whose process-global cell cannot have been initialized by unrelated tests, so the assertion is deterministic under parallel test execution. Do not rely on resetting a production `OnceLock`.

The focused regression must prove in one test:

1. `N` observations through the same registered identity yield identical captures and exactly one registered compilation;
2. each observation creates/uses a fresh cursor (proved through a test-only cursor-construction count at the common execution boundary, expected `N`);
3. two dynamic invocations retain two compilations and do not change the registered count;
4. registered and dynamic execution of the same language/query/source produce structurally identical captures.

The pre-repair RED expectation is registered compilation count `1`; existing behavior should observe `N`. The exact unchanged test becomes the GREEN proof after implementation.

### Proposed Bounded Phase-1 Change

1. Add a small registered-query module under `compression` containing the closed identities, language/query binding, bounded lazy cells, and test-only initialization counters.
2. Refactor `capture_pipeline` to one internal parsed-tree capture walk over `&Query`, preserving its current public dynamic functions.
3. Add registered-static capture entry points and migrate only proven constant callers.
4. Route Angular, .NET, and Spring standalone detectors through registered identities while retaining their current failure behavior and observation-local cursors.
5. Leave runtime-composed IR/diff queries and `edit/locate.rs` unchanged on dynamic compilation.
6. Add one tracked structural regression under `src/tests/**` and wire it through the existing `#[path]` convention.
7. Do not introduce catalog caching, parser pooling, parse-tree caching, or production telemetry.

Before production edits, verify target file line counts. `capture_pipeline.rs` is currently 227 lines and `compression/mod.rs` is 53 lines, so the proposed work does not activate oversized legacy-file decomposition.

### RED / Green / Commands

**RED:** observed by the user with the focused regression unchanged at its structural assertion: compilation count `3` versus expected `1`; cursor and semantic-equivalence assertions passed before the failing compilation assertion. The repair routes the observations through an explicit test-only registered identity rather than inferring registration from arbitrary query text.  
**GREEN:** the user ran the focused regression successfully. Its first GREEN reported one `dead_code` warning for the legacy `run_capture_pipeline` projection after all production callers moved to registered execution. The projection was restricted to `cfg(test)`, matching its remaining consumers instead of suppressing the warning, and the user confirmed the unchanged focused regression then passed warning-free. Registered language/call capture queries and framework detector queries use bounded process-lifetime `OnceLock<Result<Query, QueryError>>` ownership. Dynamic capture and edit queries still compile per invocation. Query cursors remain observation-local.  
**ADJACENT GREEN:** the user reported all requested adjacent suites green: `compression::capture_pipeline::tests`, `angular_meta::detect::tests`, `dotnet_meta::detect::tests`, and `spring_meta::tests::detect_tests`. These are Rust test results, distinct from any live harness.  
**COMMANDS RUN:** bounded repository searches and local source reads only. No Cargo command, test, build, check, Clippy, binary, MCP server, or benchmark was run.  
**FILES CHANGED:** registered capture-query machinery and counters in `src/compression/capture_pipeline.rs`; registered extension mapping in `src/compression/language.rs`; proven static capture callers; Angular/.NET/Spring detector query ownership; tracked regression; this report.  
**BOUNDED GREEN COMMAND REQUIRING AUTHORIZATION:** `cargo test -p clean-ctx --all-features compression::capture_pipeline::tests::registered_static_query_compiles_once_across_observations -- --exact`  
**REMAINING RISKS:** the authoritative repository-wide final gate has not run, so Phase 1 is not release-certified. No live MCP verification was required for this internal execution-machinery lifetime change.  
**NEXT PHASE:** stop at the Phase 1 boundary. Phase 2 structural work instrumentation requires separate authorization; alternatively run the authoritative final gate if release certification is desired before continuing the campaign.

## Phase 2 — Structural Work-Count Instrumentation

**STATUS:** GREEN — user reported both focused characterizations passing

The current branch still reconstructs C# and Angular derived query catalogs.
Phase 2 adds a test-only, thread-local work ledger at the constructors and
successful source-processing boundaries. It does not enter `McpState`, the
WorkspaceIndex, production caches, completeness, or query results.

Observed work units are:

- C# catalog constructions;
- admitted C# files considered by a construction;
- C# source snapshots successfully parsed by a construction;
- Angular catalog constructions;
- Angular component identities considered by a construction;
- successfully acquired Angular template sources processed, split into inline
  and external sources.

The prepared C# characterization uses two equivalent indexed batch items over
one current High-fidelity source. Current orchestration prepares each indexed
item before and after shared acquisition, so the expected current baseline is
four catalog constructions and four parses with identical item results.

The prepared Angular characterization explicitly builds the current ephemeral
catalog twice over one inline and one external template. The executed RED
corrected the static assumption about component enumeration: each build sees
the two declaration identities plus the two `HasSelector` endpoint identities,
because all four are indexed as `angular/Component`. The characterized current
baseline is therefore two catalog constructions, eight component-identity
considerations, and four template-source processing operations, split evenly
between inline and external sources, with identical derived results.

These are characterization assertions for the uncached implementation, not a
future cache contract. No production optimization, cache, evidence generation,
parser pool, parse-tree cache, lookup index, or authority change is included.
The user-run Angular characterization first corrected the component count from
four to eight, then both focused Phase 2 characterizations passed. The agent
ran no Cargo command or runtime verification while preparing Phase 2.

## Phase 3 — Request-Local Semantic Catalog Reuse

**STATUS:** IMPLEMENTED / NOT RUN

`PreparationContext`, already owned by exactly one `workspace_query` request,
now owns request-local `Arc` values for the C# catalog and the two Angular
catalog interpretations (`discovery_complete = false | true`). The request has
one shared validated `WorkspaceScope`, so scope does not require a second cache
key. The context is dropped after the single query or batch response.

Validity follows existing mutation boundaries rather than a new evidence epoch:

- hydration that publishes candidates invalidates all derived catalogs after
  its required reconciliation;
- hydration error invalidates conservatively because partial publication may
  have preceded the failure;
- successful hydration with zero publications preserves compatible catalogs;
- direct `entities_in_file` or `declarations_in_file` publication invalidates
  immediately before semantic projection replacement;
- hydration cache hits perform no acquisition and preserve derived views.

All C# consumers in entity, edge, graph, .NET testing, and EF preparation obtain
the shared request-local C# catalog. Angular selector hydration and final edge
preparation obtain the appropriate request-local Angular catalog. The catalogs
remain reconstructable derived machinery and never establish source authority,
completion, fidelity, or dependency validity.

Expected structural changes from the Phase 2 baseline are:

- two equivalent C# batch items across both preparation passes: catalog builds
  `4 -> 1`, files considered `4 -> 1`, files parsed `4 -> 1`;
- two equivalent Angular batch items reuse one incomplete-discovery and one
  complete-discovery catalog while retaining identical results;
- direct C# publication between equivalent preparations forces reconstruction:
  the structural sequence is one build, one reuse, one non-empty invalidation,
  one rebuild, and one reuse, followed by reuse during second-pass preparation;
  the two builds process one file before publication and two afterward.

The direct two-build Angular characterization remains intentionally unchanged:
it calls the constructor twice outside `PreparationContext` and therefore does
not claim request-local reuse. No reuse survives a request, and no session cache,
evidence epoch, compatibility fingerprint, parser pool, parse-tree cache, or
lookup index was introduced.
