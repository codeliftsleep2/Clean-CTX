# Semantic Query Performance Architecture Investigation — 2026-10-16

**Investigator:** Agent MaxHeadRoom  
**Verdict:** PERFORMANCE ARCHITECTURE DEFECTS PROVEN  
**Scope:** Static investigation and design freeze. No optimization implementation or runtime execution was authorized.

## Executive Verdict

Clean-CTX repeatedly reconstructs equivalent request-local semantic machinery from unchanged authoritative source. The defects are independent of the repaired hydration/reconciliation multiplier:

1. C# and Angular query refinement rebuild whole scoped catalogs for individual query preparations, including the batch second preparation pass.
2. C# catalog construction creates and configures one parser and reparses each admitted indexed C# source on every build.
3. Angular template catalogs reread and reparse each linked template on every build.
4. static Angular, .NET, Spring, and canonical capture queries are compiled repeatedly.
5. C# and EF Core refinement repeatedly linearly scan declarations and references; several EF Core paths are genuinely `declarations × references` or `occurrences × references/declarations`.

The facts prove opportunities for reuse, but they do not yet prove that a bare WorkspaceIndex generation number would be a sufficient validity key. Catalog truth also depends on current source snapshots, admitted scope, dependency currentness, discovery completeness, feature/producer compatibility, and—in Angular template refinement—external template source. The first safe architectural step is static query reuse. Generation-scoped catalog reuse requires an explicit compatible-evidence identity or epoch contract rather than cache validity inferred from population.

## Finding Matrix

| Finding | Proven? | Production path | Complexity | Authority risk | Recommended boundary |
|---|---:|---|---|---|---|
| Repeated C# catalog construction | Yes | `entities`, `edges`, `graph`, testing/EF/controller refinement → `csharp_resolution::Catalog::build` | Each build visits `C` admitted C# files and parses `Σ bytes(C)` | Stale declaration/reference authority | Compatible scoped evidence generation |
| Repeated Angular catalog construction | Yes | forward/reverse/template selector refinement → `angular_templates::Catalog::build` | `components + template links + template bytes + selector resolutions` per build | External/inline template changes and incomplete discovery | Scoped evidence generation plus template source hashes and discovery-complete state |
| Repeated parser creation | Yes | C# catalog per file; Angular templates; EF model parsing/type probes; standalone framework detectors; shared capture pipeline | At least one parser setup per observation, often per file | Pooling must not retain source/tree state incorrectly | Prefer eliminating parses; consider thread-local parser reuse last |
| Repeated unchanged-source parsing | Yes | C# catalog builds and Angular catalog builds over unchanged cached source; batch pass 2 repeats preparation | `build count × unchanged bytes` | Parse cache identity must include canonical file, hash, language/grammar | Prefer generation-scoped derived catalogs or canonical evidence reuse |
| Repeated static tree-sitter query compilation | Yes | `capture_pipeline`, Angular/.NET/Spring standalone detection | Once per capture/detection call; .NET compiles two static queries | Shared object thread-safety and grammar-version identity | Language/producer-lifetime compiled immutable query, pending local trait verification |
| EF Core quadratic scans | Yes | `EfCoreCatalog::build`, `configuration_entities`, `is_ef_db_context`, written-type resolution | `D×R`, plus `O×R`, `O×D`, `A×K` terms | Fast keys must not manufacture declaration identity | Catalog-owned provenance-preserving indexes |
| Repeated linear reference lookup | Yes | edge recognition/resolution, base lookup, dependency and EF refinement | Repeated `O(R)` scans, sometimes nested under declarations/occurrences | `(file, owner spelling)` alone can conflate nested/ambiguous declarations | Index by existing source/declaration provenance; retain ambiguity sets |
| Angular equivalents | Yes | template catalog and standalone detectors; capture pipeline | Repeated template parses and static query compilation | Template source lies outside declaring-file ownership | Shared cache mechanics only; Angular-specific dependency key |

`C` = admitted C# files, `D` = declarations, `R` = collected references, `O` = model/configuration occurrences, `A` = assembly markers, `K` = configuration mappings.

## Production Call-Path Evidence

### C# catalog

`Catalog::build(state, index, scope)` iterates every scoped `.cs` key in `WorkspaceIndex::file_map`, fetches `cached_source_snapshot`, calls `add_file`, constructs a new `tree_sitter::Parser`, sets the C# language, parses the entire source, and reconstructs declaration/reference vectors. The catalog is not retained.

Static call sites include:

- controller ancestry loops and final controller projection in `query/entities.rs`;
- forward and reverse classification, ASP.NET, testing, and EF Core branches in `query/edges.rs`;
- transitive dependency and cycle preparation in `query/graph.rs`;
- nested construction by `.NET testing` and `EfCoreCatalog`.

The heterogeneous batch intentionally prepares index-backed items twice. Request-local catalog values are captured inside each `PreparedQuery`, so equivalent items and pass 2 build separate catalogs even when the evidence generation has not changed.

### Angular catalog

`angular_templates::Catalog::build` enumerates component identities and `HasTemplate` links, rereads inline or external template source, calls `template_ordered::extract`, resolves selector candidates, and reconstructs edges, diagnostics, and source status. It is invoked independently by applicable forward/reverse queries and selector hydration. No shared catalog exists across equivalent items or the second preparation pass.

### Static query compilation

Repository production `Query::new` sites include:

- `compression/capture_pipeline.rs`: compiles the supplied query string after every fresh parse. Most production callers supply language constants, although the public helper accepts arbitrary query text.
- `angular_meta/detect.rs`: one language-scoped static decorator query per standalone detection.
- `dotnet_meta/detect.rs`: two language-scoped static queries per eligible standalone detection.
- `spring_meta/detect.rs`: one language-scoped static annotation query per standalone detection.
- `edit/locate.rs`: accepts a query argument and is not presumed static.

Compilation-pipeline detectors already have `*_with_regions` paths that avoid an extra framework-detection parse. That proves parse reuse is already an intended architecture in the canonical pipeline, but standalone and query-refinement paths remain separate.

## Cost Model

### Cold controller batch

For three `find_entities` controller items and one `entities_in_file`:

- candidate publications: dependent on discovery; the repaired hydration cycle reconciles once per cycle containing published C# candidates;
- C# reconciliation: each invocation performs two admitted-C# compilation passes;
- controller refinement: ancestry traversal can build one C# catalog per ancestry step plus a final catalog per item;
- batch pass 2 repeats indexed item preparation and its catalogs;
- one C# catalog build creates `C` parsers and performs `C` full-source parses.

The structural catalog cost is approximately `B × C parses`, where `B` is the number of controller/catalog build sites reached across both preparation passes. This is separate from the repaired publication multiplier.

### Warm equivalent controller batch

Candidate publication and reconciliation fall to zero when source/fidelity/dependencies remain current. Catalog cost does not: equivalent controller refinement still rebuilds and reparses scoped C# source in both preparation passes. Warm acquisition therefore improves one cost layer while leaving generation-stable derived computation cold.

### EF Core relationship query

An EF forward `DbContext` or reverse `Entity` query builds a C# catalog and then `EfCoreCatalog`:

- `configuration_entities`: every class declaration searches the reference vector (`D×R` worst case);
- DbContext selection: each class declaration invokes `is_ef_db_context`, which scans references (`D×R`);
- each model occurrence may scan references to find owner context (`O×R`), parse a written type with a new parser, then scan declarations for candidate paths (`O×D`);
- assembly marker projection scans configuration mappings (`A×K`) and performs filesystem project discovery.

The reported quadratic defect is therefore proven with respect to declaration/reference cardinalities, not merely inferred from visual nesting.

### Angular framework query

An applicable Angular edge query builds a catalog by walking components and template links, rereading and parsing each linked template, then resolving every selector occurrence against the WorkspaceIndex. Equivalent items and pass 2 reconstruct this result. Standalone Angular detection additionally creates a parser, parses source, and compiles the same decorator query per call.

## Cache and Index Design

### 1. Static compiled tree-sitter queries

**KEY:** language/grammar identity + static producer/query identity.  
**VALUE:** compiled immutable `tree_sitter::Query` or deterministic compile failure.  
**DEPENDENCIES:** linked grammar version, feature configuration, query constant.  
**VALIDITY:** process lifetime for a fixed binary.  
**INVALIDATION:** process restart/binary replacement.  
**CONCURRENCY:** verify the pinned crate's `Send`/`Sync` traits before selecting process-global sharing; thread-local lazy cells are the fallback that does not require cross-thread sharing. `QueryCursor` remains per observation.  
**AUTHORITY:** execution machinery only; never semantic authority.  
**MEMORY:** one small compiled query per static producer/language. No eviction required.  

The generic capture API must distinguish registered static query identities from genuinely dynamic query strings; caching arbitrary strings globally would create unbounded cardinality.

### 2. C# derived catalog

**KEY:** compatible semantic-evidence identity + normalized WorkspaceScope.  
**VALUE:** immutable declarations, references, and provenance-preserving lookup indexes.  
**DEPENDENCIES:** admitted canonical C# file set; source hash of every admitted file; producer/grammar compatibility; scope roots/narrowing; relevant semantic dependencies.  
**VALIDITY:** only while all dependency fingerprints match the current compatible source/evidence generation.  
**INVALIDATION:** publication, edit, refresh, restore/replay, deletion, root/configuration change, producer compatibility change, or dependency staleness affecting the admitted set.  
**CONCURRENCY:** immutable `Arc` value; serialized generation-aware construction or duplicate-build-and-publish with exact key comparison. Never hold WorkspaceIndex/source locks while parsing.  
**AUTHORITY:** reconstructable derived view; WorkspaceIndex and current source remain authority. Cold and warm results must be identical.  
**MEMORY:** proportional to scoped declarations/references plus indexes; initially one current entry per normalized scope, with bounded eviction if arbitrary narrow scopes are admitted.  

A current explicit WorkspaceIndex generation token was not found at the query boundary. Source-generation strings exist for file completeness, and discovery has its own generation, but neither alone covers catalog dependencies. Therefore a persistent/session catalog keyed only by “index generation” is **not yet safe**. A compatible-evidence epoch/fingerprint contract must be designed first, or reuse must remain request-local with explicit invalidation after acquisition changes.

### 3. Angular template catalog

**KEY:** compatible evidence identity + normalized scope + discovery-complete state.  
**VALUE:** immutable selector occurrence/refinement edges, diagnostics, and source status.  
**DEPENDENCIES:** Component/HasTemplate evidence; admitted component files; hashes of external template files or declaring source for inline templates; selector declaration evidence; producer support.  
**VALIDITY/INVALIDATION:** all listed dependencies current; invalidate on source/selector/template publication, refresh, deletion, scope or discovery-completeness transition.  
**CONCURRENCY/AUTHORITY:** same derived immutable mechanics as C#, with Angular-owned key construction and semantics.  
**MEMORY:** proportional to template occurrences and diagnostics; reconstructable and in-memory only.  

C# and Angular may share cache mechanics, but not dependency inference or semantic authority.

### 4. Reference lookup indexes

**KEY:** derive from existing provenance: canonical file identity, namespace/nesting path, owner declaration identity where available, and reference shape/occurrence discriminator as required.  
**VALUE:** slices/indices of existing `BaseReference` occurrences; declaration-path/arity indexes separately map authorized declarations.  
**DEPENDENCIES/LIFETIME:** identical to the owning C# catalog.  
**AUTHORITY:** indexes preserve candidate sets and ambiguity; they never convert spelling into identity.  
**MEMORY:** `O(R + D)` additional keys/indices, reconstructable.  

The hypothesized `(file, owner name)` key is insufficient because nested or duplicated owner spellings can exist. The current `BaseReference` retains file, namespace, nesting, owner, imports, and type shape, which can support a provenance grouping, but exact owner arity/identity requirements need a targeted design audit before implementation.

### 5. Parse reuse

Parse-tree caching is not the first recommendation. A tree key would require canonical file identity, source hash, language/grammar version, memory ownership, eviction, and thread-safety; templates and synthesized type probes need distinct identities. Higher-level C#/Angular catalog reuse eliminates more work with fewer competing representations. Canonical IR reuse should be evaluated before a general parse-tree cache because it already participates in authority publication.

Parser pooling alone saves allocation/language setup but not parse cost. Reconsider it only after static query reuse, catalog reuse, reference indexing, and redundant parse elimination are structurally measured.

## Execution-Control Findings

### Host cancellation

The stdio reader parses each JSON-RPC line and enqueues an owned handler closure. `router::dispatch` recognizes initialize, tools, prompts, and tools/call only. No cancellation notification is routed; dispatcher work items carry no cancellation token; batch preparation, hydration, compilation, reconciliation, refinement, serialization, and response emission accept none. Host cancellation information is therefore absent at ingress or lost before dispatch.

Required future boundary: request identity → cancellation token → cooperative checks at phase and file-loop boundaries → safe unwind. Publication already commits candidate state transactionally; cancellation policy must specify whether previously committed valid publications remain warm state while discovery completion and final response do not claim success.

### Request timeout policy

No request execution timeout exists. Dispatcher `send_timeout` bounds queue insertion, `slow_request_threshold` observes latency, and shutdown timeouts concern lifecycle—not handler cancellation. A default execution deadline is an externally observable policy and remains unapproved.

### Partial-result budgets

Do not silently truncate. Budget exhaustion removes authority to claim completeness. Existing exact/file-certified families should fail unless a separately approved opt-in partial contract defines requested, examined, and omitted scope. Existing explicitly lower-bound indexed-evidence families may describe existing coverage, but that does not automatically authorize new time/file truncation. This requires an architectural decision: **YES**.

### Progress reporting

The server can emit JSON-RPC/MCP notifications at the protocol level, but no request-scoped progress-token routing or semantic work ledger is present in the handler chain. Implement progress only after protocol capability negotiation and request-token ownership are proven. Report phase/work units (acquisition candidates, files examined, reconciliation phase), never fabricated percentages. Progress is observability, not cancellation or completeness.

## Candidate Invariant Decisions

- **Generation reuse:** validate with the refined wording: equivalent authoritative evidence under the same complete dependency fingerprint should not reconstruct equivalent derived semantic state.
- **Static query compilation:** validated for registered static queries, subject to pinned-library concurrency verification.
- **Reference indexing:** validated when keys preserve source/declaration provenance and ambiguity.
- **Acquisition/reconciliation:** already repaired and verified separately.
- **Truth preservation:** validated; every cache is reconstructable derived state and cannot establish authority.

## Recommended Implementation Order

1. **Static query reuse:** small, process-local multiplicative reduction; first verify pinned `Query` concurrency traits and split static identities from dynamic capture/edit queries.
2. **Test-only work counters:** catalog builds, files parsed per catalog, static query compilations, template parses, and EF reference/declaration scan iterations.
3. **Request-local catalog reuse:** share an immutable catalog among equivalent prepared items only after acquisition is complete; explicitly invalidate/rebuild if the request changes evidence. This proves composition before session caching.
4. **Compatible-evidence identity/epoch design:** define all mutation/invalidation points and dependency fingerprints. This is the prerequisite for safe session generation-scoped catalogs.
5. **C# and Angular generation-scoped catalogs:** separate semantic owners over shared cache mechanics.
6. **C# provenance lookup indexes:** eliminate proven repeated `R` and `D` scans while preserving ambiguity.
7. **EF Core index consumption:** replace `D×R`/`O×R` scans and structurally assert bounded lookups.
8. **Canonical/derived evidence reuse audit:** eliminate remaining unchanged-source reparses; only then evaluate parse-tree caching.
9. **Parser pooling measurement:** implement only if structural counters show material residual parser-setup cost.
10. **Cancellation, timeout, partial results, and progress:** separate approved architecture work, not a cache patch.

## Regression Strategy

Add test-only counters, not production telemetry frameworks:

- C# catalog builds and C# files parsed per build;
- Angular catalog builds and template sources parsed;
- static query compilations by registered query identity;
- parser creation/source parse counts at relevant shared boundaries;
- C# reference and declaration candidates examined;
- EF configuration, context, occurrence, and assembly-mapping scans.

Required structural regressions:

1. Equivalent batch items over unchanged post-acquisition evidence construct one request-local C# catalog per normalized scope, not one per item/pass.
2. Warm equivalent request over an approved generation cache returns byte/semantic-equivalent truth and performs zero catalog reconstruction.
3. Source edit, refresh, deletion, scope change, dependency change, and producer-version change invalidate the appropriate derived catalog.
4. Static registered query compilation is once per approved lifetime while cursors remain per observation.
5. EF relationship results remain identical while reference lookup candidates scale with matched provenance buckets rather than all `R` references.
6. Angular external and inline template edits invalidate catalog reuse; query order, batching, eviction, and restart do not change truth.
7. Cold and warm semantic results and declared completeness remain equal across cache eviction and process restart.

## Runtime Evidence Still Required

No Cargo, binaries, MCP servers, CBM, benchmarks, or live verification were run for this investigation. After each separately approved implementation phase, use focused structural tests first, then the authoritative repository gate. Performance timings may supplement but never replace work-count assertions.

