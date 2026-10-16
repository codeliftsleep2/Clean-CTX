# Semantic Query Performance Architecture Implementation Campaign — 2026-10-16

**Agent:** Agent MaxHeadRoom  
**Campaign status:** Phases 1–3 GREEN; Phase 4 cross-request reuse investigation complete — DEFER

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

**STATUS:** GREEN — user reported all focused Phase 3 gates passing

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

## Phase 4 — Compatible-Evidence Identity for Cross-Request Catalog Reuse

**STATUS:** INVESTIGATION COMPLETE — DEFER; no production cache implemented

### Decision and Remaining Work

Cross-request reuse is technically possible, but the repository does not own a
single identity proving the complete evidence compatibility required by either
catalog. Implementing a cache now would create a new durable validity contract
across source, publication, dependency, discovery, and reconciliation
lifecycles. That risk is not justified by measured post-Phase-3 payoff.

**RECOMMENDATION: DEFER.** Keep request-local reuse. Measure representative
cross-request workloads first; if residual cost is material, introduce an
observational domain evidence identity before introducing a cache.

Phase 2 counters prove constructor, admitted-source, and template work. Phase 3
reduced equivalent C# batch work from four builds/parses to one, and Angular to
one build per required `discovery_complete` interpretation. After a relevant
in-request mutation it rebuilds once and then reuses. Because the context dies
with the response, two compatible requests still build twice. No representative
cross-request frequency or latency payoff has been measured, and the former
within-request multiplication has already been removed.

### Existing Generations and Authorities

There is no general `WorkspaceIndex` generation. The index owns entities,
declarations, edges, file projections, and file-local semantic coverage without
one mutation sequence for the aggregate state.

`HydrationDiscoveryCache` generations are per provider/canonical root. They
prove only that a discovery key completed under the current discovery
invalidation generation. They do not identify source bytes, projections,
dependencies, reconciliation, or interpretation. Equal generations do not
prove equal semantic truth; different generations do not prove catalog inputs
changed.

`SemanticCoverage` proves exact owner source hash, minimum fidelity, and current
hashes for recorded external dependencies for one published projection. It is
not a workspace-set identity and is not exposed as an enumerable catalog
fingerprint. `SemanticProducerIdentity` and configuration identities prove
whether unchanged inputs have compatible producers/configuration; they do not
prove unchanged inputs. Durable file versions and response completeness
certificates are also local authorities. None may serve alone as a cache key.

### C# Compatible-Evidence Dependencies

The C# catalog enumerates scoped `.cs` entries in `file_map`, obtains cached
source snapshots, and parses declarations, bases, and references.

| Dependency | Classification |
| --- | --- |
| Normalized semantic scope/root set | Necessary; determines admission and ambiguity. |
| Sorted canonical admitted C# file set | Necessary; entry/exit changes resolution. |
| Current source hash for every admitted file | Necessary; catalog facts come from these bytes. |
| C# grammar/query/catalog producer identity | Necessary; binds interpretation. |
| Interpretation-changing configuration | Necessary cache partition. |
| Fidelity/semantic coverage | Live use gate; not sufficient content identity. |
| Dependency currentness | Live use gate; owner hash cannot prove another file current. |
| Declaration/relationship authority | Consumer dependency; must remain live rather than frozen into catalog authority. |
| Reconciliation state | Required live gate for consumers of reconciled calls. |
| Discovery completeness | Consumer interpretation/use gate, not parsed content. |

Owner currentness is derivable from source hashes. Dependency currentness,
publication success, reconciliation, and discovery completeness are not.
Computing the sorted file/hash set is precise but preserves traversal and
hashing cost and still needs all live authority gates.

### Angular Compatible-Evidence Dependencies

Angular construction enumerates components and `HasTemplate` ownership, reads
declaring TypeScript for inline templates, reads external templates, extracts
template facts, and resolves selectors against index evidence.

| Dependency | Classification |
| --- | --- |
| Normalized semantic scope/root set | Necessary. |
| Component declaration identity and provenance | Necessary; name is not identity. |
| Current `HasTemplate` ownership | Necessary. |
| Declaring component source hash | Necessary for decorator/inline content. |
| External template canonical path and current hash | Necessary; independent of owner source. |
| Selector/declaration evidence set | Necessary for endpoint resolution. |
| Angular/TypeScript/template producer and config identity | Necessary. |
| Dependency currentness | Necessary. |
| `discovery_complete` interpretation | Necessary partition/use gate. |

Inline content is derivable from declaring-source hash plus producer identity;
external template state is not. Angular therefore cannot safely share C#
validity semantics merely because both could use the same cache container.

### Candidate Designs

1. **Exact compatible-evidence fingerprint:** hash normalized scope, sorted
   domain evidence records, source/dependency hashes, producers, and
   interpretation keys. Precise and preserves reuse across unrelated mutation,
   but fingerprint construction may approach catalog cost, especially for
   Angular selector/template enumeration.
2. **Global semantic mutation generation plus scope/producers:** cheap lookup,
   but safe only after every relevant mutation is centralized. It over-invalidates
   and still needs special handling for external files and partial publication.
3. **Domain evidence epochs:** separate C# and Angular generations with hooks
   for source, publication, removal, restore, reconciliation, dependency, and
   template changes. Better precision, but creates a cross-cutting authority
   contract vulnerable to a missed mutation path.
4. **Hybrid epoch plus exact verification:** strongest eventual design, but the
   most machinery and duplicates some work.

A future bounded cache owner, immutable `Arc` entries, concurrency, metrics,
and eviction mechanics may be shared. Evidence extraction, keys, invalidation,
and authority use gates must be domain-specific.

### Required Failure Matrix

| Failure case | Exact fingerprint | Fully audited domain epoch | Existing discovery generation |
| --- | --- | --- | --- |
| Owner contents change | Hash invalidates | Mutation must advance | Not guaranteed |
| File enters/leaves scope | Set/scope invalidates | Membership must advance | Not guaranteed |
| Dependency-only change | Dependency hash invalidates | Dependency hook required | No |
| External Angular template changes | Template hash invalidates | External-source hook required | No |
| Partial discovery becomes complete | Partition/gate changes | Separate discovery gate required | Partial signal only |
| Producer/grammar/config changes | Identity invalidates | Producer partition required | No |
| Fidelity/coverage changes | Authority gate changes | Publication hook required | No |
| Failed/partial acquisition | Cannot install complete entry | Must not publish reusable epoch | No |
| Publication leaves dependency stale | Dependency check rejects | Epoch alone is insufficient | No |
| Similar warm states via different histories | Safe only if normalized evidence matches | Safe only with total hook coverage | Unsafe |
| Unrelated mutation | Reuses | Conservatively invalidates | May invalidate accidentally |

“Fully audited” is a prerequisite, not a fact established by the current code.

### Future Structural Verification

Use tracked tests and Phase 2 work counts, never timing:

1. request A builds once; compatible request B adds zero builds with identical
   ordered results;
2. owner mutation, scope entry/exit, dependency-only mutation, external-template
   mutation, fidelity transition, producer/config partition, and
   partial-to-complete discovery each cause their required rebuild/partition;
3. failed/partial acquisition never installs reusable authority;
4. publication with stale dependencies rejects reuse;
5. an unrelated mutation documents the selected precision policy;
6. cold, warm, reversed-history, provider-state, and batch variants retain
   identical truth and completeness;
7. concurrency proves single construction or benign duplicate construction
   without publishing incompatible state.

The decisive cross-request sequence is `build -> reuse -> relevant mutation ->
reject -> rebuild -> reuse`, with separate assertions for work counts and
semantic equivalence.

### Risk and Conditional Smallest Slice

Risk is high. A cross-request cache makes persistent warm state depend on
correct invalidation across source caching, publication/removal, durable
restore, refresh, external dependencies, reconciliation, discovery, and
producer versions. A missed edge can make warm truth differ from cold truth.

If later measurement justifies proceeding, the smallest safe slice is **C#
only and observational**:

1. derive an immutable identity from normalized scope, sorted admitted
   canonical file/source hashes, and C# producer/config identity;
2. retain live fidelity, dependency, reconciliation, and discovery checks as
   use gates; the identity must not authorize them;
3. expose test-only comparisons and counters without retaining catalogs;
4. prove all C# mutation/history controls;
5. only then add a bounded session-owned C# catalog cache.

Angular remains deferred until external-template and selector-evidence identity
can be produced completely and cheaply. Do not add a global `WorkspaceIndex`
generation merely to make caching convenient.

No production or test code changed. No Cargo command, test, build, check,
Clippy, binary, MCP, CBM, benchmark, or live verification was run. Phase 4 ends
at this design freeze and does not reopen the GREEN Phase 1–3 contracts.

## Next Target Selection — EF/C# Lookup Work Measurement

**STATUS:** MEASUREMENT GREEN; REQUEST-LOCAL EF REUSE IMPLEMENTED / NOT RUN

### Phase 4 Decision Freeze

Cross-request semantic catalog caching remains **DEFERRED** until representative
measurement proves residual reconstruction cost material. No global
`WorkspaceIndex` generation, persistent cache, or new invalidation authority may
be introduced for convenience. Re-entry remains C#-only observational identity
first, exactly as specified above. Angular remains deferred.

### Current Performance Topology

After Phases 1–3, production work separates as follows:

1. source acquisition and hydration establish authoritative candidates;
2. canonical compilation/publication occurs only when evidence is not current
   and adequate;
3. one required C# reconciliation runs after a publishing hydration cycle;
4. one request-local C# catalog parses each admitted C# source, then is reused
   until relevant publication invalidates it;
5. one request-local Angular catalog exists per required completeness
   interpretation and evidence interval;
6. consumers still perform linear resolution over the immutable C# catalog;
7. every applicable EF preparation independently constructs `EfCoreCatalog`
   from that shared C# catalog.

Static query compilation and within-request C#/Angular reconstruction
amplification are amortized. Consumer lookup and EF derivation are not.

### Remaining C# Provenance/Reference Work

The current C# catalog stores declarations and base references in vectors.
Repeated operations remain:

- `recognizes_edge` and `resolve_edge` scan all references for
  `(file, owner spelling, written type)`;
- `base_reference` scans references for file, owner, namespace, and nesting;
- `resolve_declaration` and `resolve` scan declarations for every candidate
  qualified path and arity;
- unique-name/file helpers scan declarations and intentionally reject multiple
  matches;
- `written_references_to` scans every reference and may resolve each against
  declaration scans;
- dependency graph construction performs per-edge reference lookup,
  declaration resolution, and a linear declaration-position lookup.

Phase 3 eliminates repeated catalog construction but does not amortize these
scans inside a build or across consumers. A safe mechanical index must use
existing provenance, not invent ownership. Candidate keys include canonical
file plus namespace/nesting owner path, written/reference shape, and declaration
path plus generic arity. Values must preserve ambiguity sets; a lookup may
accelerate the existing “zero, unique, or ambiguous” decision but may not choose
among duplicates. `(file, owner name)` alone remains insufficient.

### Remaining EF Multiplication

`EfCoreCatalog::build` remains uncached request-local derived work. Forward and
reverse EF preparations each invoke it from the shared C# catalog; equivalent
batch items and the second preparation pass can therefore repeat the entire EF
derivation even though C# parsing is shared.

The current crossed collections remain:

- configuration class declarations × all references (`D × R`) to locate
  `IEntityTypeConfiguration<T>`;
- candidate context declarations × all references (`D × R`) to establish
  `DbContext` ancestry;
- model/configuration occurrences × references (`O × R`) to recover an owner
  reference context;
- each resolved written type × declarations/candidate paths (`O × D`);
- assembly markers × configuration mappings (`A × K`), including repeated
  nearest-project discovery/comparison;
- EF model contexts are reparsed, and written type probes construct parsers and
  trees independently.

These are scans over already-derived request-local facts. Mechanical indexes in
the owning C# catalog could serve EF and non-EF consumers without becoming a
new semantic authority. However, the repository currently counts catalog
builds/parses, not EF builds, scan candidates, type probes, or project lookups.

### Reconstruction and Batch Audit

Remaining required parsing is now distinguishable:

- canonical acquisition parses when current adequate evidence is absent;
- one C# catalog parse per admitted source/evidence interval remains necessary
  for query-time declarations/references;
- EF reparses each selected context source for `OnModelCreating` and parses
  synthetic written-type probes;
- Angular processes templates once per required request-local interpretation;
- dynamic IR/diff/edit Tree-sitter queries still compile dynamically by design.

No current evidence shows accidental duplicate C# or Angular construction after
the Phase 3 lifecycle controls. Required partial/complete Angular partitions and
mutation-driven rebuilds are not defects. The remaining proven batch candidate
is repeated `EfCoreCatalog` derivation per applicable item/pass; existing
instrumentation cannot yet state its exact work multiplier.

Parser pooling remains **DEFERRED**. Higher-level reuse removed most parser
construction, while EF-specific residual parsing has not been structurally
counted. Pooling would save setup, not tree construction or scan complexity.

Parse-tree caching remains **DEFERRED**. It would add source-hash, grammar,
lifetime, memory, eviction, and concurrency authority while duplicating parts
of canonical and derived state. The nearer problem is repeated EF derivation
and linear lookup over an already-owned immutable catalog.

### Measurement Gap and Selected Campaign

The next action is exactly **MEASURE FIRST**. Add test-only observational
counters at the EF/C# work boundaries for:

- `EfCoreCatalog` constructions;
- configuration declarations examined and reference comparisons;
- context declarations examined and reference comparisons;
- model occurrences and owner-reference comparisons;
- declaration candidates examined during written-type resolution;
- synthetic written-type parses;
- assembly-marker/configuration comparisons and project-root lookups;
- optionally, shared C# lookup comparisons used outside EF, separated by lookup
  kind.

Counters remain thread-local test instrumentation. They must not enter cache
keys, production responses, semantic decisions, or authority.

### Ranked Candidates

| Rank | Candidate | Classification | Proof/amplification | Risk and scope |
| --- | --- | --- | --- | --- |
| 1 | Measure repeated EF derivation and lookup comparisons | **MEASURE FIRST** | Static `D×R`, `O×R`, `O×D`, `A×K` loops remain; batch multiplier not counted | Test-only, deterministic, low risk |
| 2 | C# catalog-owned provenance/declaration indexes consumed by EF | **PROCEED only after counts** | Repeated vector scans remain and scale with `R`/`D` | Mechanical if ambiguity sets are preserved; moderate identity risk |
| 3 | Request-local `EfCoreCatalog` reuse | **MEASURE FIRST** | Build is invoked per applicable preparation | Same evidence lifecycle as C# catalog, but reduction unknown |
| 4 | Other C# consumer lookup indexes | **MEASURE FIRST** | Linear scans are visible but frequency is uncounted | Low authority risk with exact provenance keys |
| 5 | Canonical/derived parse elimination | **DEFER** | Remaining parses mix required canonical work and EF-local work | Boundary must be measured first |
| 6 | Parser pooling | **DEFER** | Setup cost not measured material | Saves setup only |
| 7 | Parse-tree caching | **DEFER** | No post-Phase-3 materiality proof | High lifetime/memory/validity cost |
| 8 | Cross-request catalog caching | **DEFER/FROZEN** | Residual cost not measured | Highest authority risk |
| 9 | Static-query/catalog reconstruction defects from Phases 1–3 | **OBSOLETE / ALREADY AMORTIZED** | GREEN structural controls | Reopen only on contradictory evidence |

### Structural RED/GREEN Strategy

Prepare one focused EF fixture with controlled cardinalities: multiple ordinary
classes, contexts, configuration declarations, model occurrences, and an
assembly marker. Exercise equivalent forward/reverse EF items in one batch.

The characterization RED must assert current exact structural counts and
identical semantic results across items/passes. It should expose both:

```text
shared C# catalog builds = 1
EF catalog builds > 1
reference/declaration comparisons grow by the crossed collection sizes
```

The later production GREEN, under separate authorization, should preserve
edges, provenance, ambiguity refusal, ordering, and completeness while proving
the selected redundant counts collapse. If indexes are selected, comparisons
must scale with matched provenance buckets rather than global `R`/`D`. If
request-local EF reuse is selected, equivalent preparations build once per
compatible evidence interval and mutation forces rebuild then reuse.

### Frozen Semantic Invariants

- An index accelerates retrieval of established facts; it never decides their
  meaning or authority.
- Name is not identity. Proximity is not ownership.
- Duplicate declaration/reference matches remain ambiguous unless existing
  provenance already establishes uniqueness.
- Warm state and query order cannot change truth, completeness, provenance, or
  ordering.
- Fidelity, dependency currentness, discovery, publication, and reconciliation
  remain live authority gates.
- Optimization eliminates redundant work, never required work.

### Commands Requiring Separate Authorization

Test-only instrumentation and the structural characterization are now prepared
in the existing performance-work-count module. No Cargo command was run. The
focused user-owned command is:

```text
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::equivalent_ef_batch_items_reuse_one_request_local_catalog" -- --exact --nocapture
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::ef_catalog_build_reuse_invalidate_rebuild_reuse_sequence" -- --exact --nocapture
```

Any adjacent suite, build, Clippy invocation, benchmark, binary, MCP/CBM run, or
release gate requires its own authorization. No production optimization begins
until the structural characterization is GREEN and selects the smallest proven
removal boundary.

The user reported the unchanged structural characterization GREEN, proving one
shared C# catalog but four repeated EF constructions. The narrow repair now
stores one `Arc<EfCoreCatalog>` beside the C# catalog in `PreparationContext`.
Forward and reverse EF consumers obtain it through that context. Existing
derived-catalog invalidation clears both values after publication or a possibly
partial acquisition failure. No value survives the request.

Expected GREEN work counts are `C# builds = 1`, `EF builds = 1`, and
`EF reuses = 3` for two equivalent items across both preparation passes. The mutation
control proves `build -> reuse -> invalidate -> rebuild -> reuse`, with two EF
builds, six reuses, one invalidation, and identical results before and after an
unrelated authoritative publication. EF interpretation, scans inside the one
required build, provenance, ambiguity, fidelity, discovery, and completeness
are unchanged.

Both focused request-local EF reuse regressions were reported GREEN. The
post-reuse single-build ledger was: three configuration declarations and five
configuration/reference comparisons; three context declarations and six
context/reference comparisons; one model occurrence; two owner/reference
comparisons; and two synthetic type parses.

A scaled characterization is now prepared, not run. It compares four and
sixteen `IEntityTypeConfiguration<T>` mappings under one assembly-scanning
context. Both cases must retain one C# build and one EF build. Reference scan
comparisons must grow by more than eight times when cardinality grows four
times, while assembly/configuration comparisons grow exactly four times and
synthetic parses/project lookups grow by the twelve added mappings. This
separates residual multiplicative scans from
required linear semantic work without authorizing an index.

```text
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::ef_reference_scans_scale_multiplicatively_after_catalog_reuse" -- --exact --nocapture
```

## Request-Local C# Reference Candidate Index for EF Lookups

**STATUS:** IMPLEMENTED / NOT RUN

The GREEN scaled characterization proved the residual global-reference
amplification: four-to-sixteen configurations increased configuration/reference
comparisons `35 -> 425` and context/reference comparisons `45 -> 561`, while
the enclosing C# and EF catalogs each built once.

The C# catalog now owns a mechanical index from exact existing `(file, owner
spelling)` fields to ordered positions in its existing reference vector. It is
built once after source admission, has the same lifetime as the request-local
C# catalog, and rebuilds naturally with that catalog after invalidation. It
does not store answers, declaration identities, ownership decisions, namespace
inference, nesting inference, or framework classification.

Only the three measured EF consumers use the candidate buckets:

- configuration discovery obtains the file/owner bucket and retains the full
  `IEntityTypeConfiguration<T>` framework/shape check;
- context discovery obtains the file/owner bucket and retains the full
  `DbContext` framework/shape check;
- written-class owner context obtains the same ordered bucket and retains the
  existing first-candidate behavior and subsequent declaration resolution.

Empty buckets yield no candidates, matching the prior global scan. Bucket
construction walks the reference vector in existing order, appends every
position, and performs no sorting or deduplication. A dedicated negative
regression supplies the same owner spelling in multiple files and multiple
same-file candidates with distinct namespace/nesting provenance. It proves the
same-file bucket returns all candidates in original order, preserves their
provenance, excludes other file/owner buckets, and returns empty for missing
keys. The bucket is a candidate prefilter, not authority.

Test-only counters now report index builds, key count, total indexed references,
and maximum bucket size. For the scaled fixtures the mathematically expected
post-index counts are:

| Work | 4 configurations | 16 configurations |
| --- | ---: | ---: |
| C# catalog builds | 1 | 1 |
| EF catalog builds | 1 | 1 |
| candidate-index builds | 1 | 1 |
| index keys / indexed references | 5 / 5 | 17 / 17 |
| maximum bucket | 1 | 1 |
| configuration/reference comparisons | 5 | 17 |
| context/reference comparisons | 5 | 17 |
| owner/reference comparisons | 1 | 1 |
| assembly/configuration comparisons | 4 | 16 |
| synthetic parses | 5 | 17 |
| project-root lookups | 5 | 17 |

Thus the phase targets only global reference comparisons. Assembly scanning,
synthetic parsing, project lookup, declaration resolution, ambiguity,
provenance, fidelity, discovery, reconciliation, and completion are unchanged.
No state crosses requests and no new invalidation protocol exists.

Focused commands requiring authorization:

```text
cargo test --all-features "mcp::tool_handlers::query::csharp_resolution::tests_reference_candidates::same_file_owner_bucket_preserves_all_candidates_and_original_order" -- --exact --nocapture
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::ef_reference_scans_scale_multiplicatively_after_catalog_reuse" -- --exact --nocapture
```

No Cargo command, build, Clippy, binary, MCP, CBM, benchmark, or live
verification was run. Do not select the next optimization until the observed
post-index ledger is available.
