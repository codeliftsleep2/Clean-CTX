# Issue #146 Cross-Root DbContext Classification Consistency Investigation

**Date:** 2026-10-18  
**Investigator:** Agent MaxHeadRoom  
**Scope:** Static repository investigation only. No Cargo command, build, test,
Clippy, binary, MCP runtime session, or live verification was run.

## 1. Executive verdict

**Verdict: RUNTIME FALSIFICATION REQUIRED.**

**Final recommendation: PROCEED TO BOUNDED RUNTIME FALSIFICATION.**

**Controlled falsification result (user-run, 2026-10-18): PASS.** The focused
two-root regression
`issue_146_equivalent_cross_root_dbcontexts_keep_independent_occurrences` passed.
This is tracked Rust-test evidence reported by the user; Agent MaxHeadRoom did
not run the command. It falsifies a general defect caused solely by equivalent
root placement, requested/additional-root reversal, provider ready versus
unavailable state, cold versus warm execution, same-name occurrence identity,
or public `find_entities` projection. It does not reproduce or explain the
original field roots, whose actual isolated class bodies and publication
receipts remain unavailable.

Static inspection proves an important correction to the issue premise: there
is not one DbContext classification path. Clean-CTX currently has two materially
different EF producers:

1. the durable `.NET` meta-layer producer, which uses the isolated class source,
   recognizes the literal `: DbContext`, and creates
   `dotnet/DbContext/<name>` only as the subject of an emitted `HasEntity`
   relationship for a detected `DbSet<T>`; and
2. the request-local exact EF catalog used by exact DbContext edge queries,
   which consumes structured C# declarations and base references, verifies the
   `Microsoft.EntityFrameworkCore` qualification/import condition, parses
   `OnModelCreating`, and creates request-local `HasEntity` edges.

The observed unfiltered `find_entities` result reads the durable
`WorkspaceIndex` occurrence map. It does not consult the request-local exact EF
catalog. Therefore `public class ContextA : DbContext, IContextA` alone does not
guarantee a durable `dotnet/DbContext/ContextA` occurrence. The durable producer
can emit an EF marker while emitting no semantic edge and hence no query-visible
DbContext occurrence when the isolated class contains no detected `DbSet<T>`.

The field evidence does prove that the two roots' published index projections
differed. It does not expose the upstream observations needed to choose among:

- genuinely different isolated class bodies or EF relationship evidence;
- one file not being acquired or republished at the required generation;
- one producer not running because applicability/configuration differed;
- stale semantic coverage being accepted;
- different fidelity or compilation failure; or
- a source/projection state changed by the Issue #144 incomplete-acquisition
  condition.

It is not possible to name a production repair boundary from static code plus
the reported final entities without guessing. The deterministic registration
boundary rules out name collision as the immediate cause, and root/project
identity does not participate in the durable EF producer. A bounded experiment
must capture the per-file semantic artifact and publication state immediately
before and after `WorkspaceIndex` publication.

**EARLIEST PROVEN DIVERGENCE: the per-owner semantic projection admitted to the
WorkspaceIndex differed—Root B's projection contained at least one real edge
whose endpoint was `dotnet/DbContext/ContextA`, while Root A's current published
projection did not.** Registration after that point is deterministic. The field
record does not establish whether the earlier divergence occurred in source
evidence, acquisition, meta-layer applicability, compilation, cache-currentness,
or publication.

## 2. Current DbContext authority model

### 2.1 Declaration authority remains unchanged

Issue #145's result stands. The source declaration is independently registered
as:

```text
builtin / Class / ContextA
```

through the builtin self-`Defines` carrier. A framework occurrence is not a
declaration and must not rewrite this identity.

### 2.2 Durable framework occurrence authority

`DotNetMetaLayer::evaluate_if_applicable` first requires the layer to be enabled
and `detect::is_dotnet_file_with_regions` to accept the source. Its evaluation
receives per-class source spans produced from parser captures.

For each isolated class, `evaluate_applicable_source` ultimately calls
`dotnet_meta::efcore::analyze_efcore`. That function:

- checks `class_source.contains(": DbContext")`;
- extracts the class name textually;
- extracts `DbSet<T>` members textually;
- renders an EF marker for a recognized class; and
- emits one `HasEntity` edge per distinct detected `DbSet<T>` entity.

Only an emitted edge carries an `EntityRef::new("dotnet", "DbContext", name)`.
No self-`Defines` carrier is emitted for the framework identity. If there are no
DbSets, the marker exists in compressed output but `semantic_edges` is empty;
the durable index gets no dotnet DbContext occurrence from this producer.

This producer is file-local. It does not read a `.csproj`, package references,
the declaration catalog, or imported framework declarations, and returns an
empty `semantic_dependencies` vector.

### 2.3 Request-local exact EF authority

`csharp_resolution::EfCoreCatalog` is a separate, request-local projection used
by exact `forward_edges`/`reverse_edges` requests for
`dotnet/DbContext/<name>`.

It builds a structured C# `Catalog` from admitted source snapshots. A class is
recognized as an EF DbContext only when a captured base reference:

- has terminal name `DbContext` and arity zero; and
- is written as `Microsoft.EntityFrameworkCore.DbContext` (including supported
  global qualification), or is unqualified while the file imports
  `Microsoft.EntityFrameworkCore`.

The catalog then requires supported `OnModelCreating` relationship evidence
before it emits request-local `HasEntity` edges. Project lookup is used only for
`ApplyConfigurationsFromAssembly` ownership. It is not the authority that makes
a class a DbContext.

### 2.4 Consequence

The phrase “DbContext classification” currently conflates:

- a rendered EF marker;
- a durable relationship endpoint visible to unfiltered `find_entities`; and
- an exact request-local EF relationship projection.

These are not interchangeable authorities. Issue #146 concerns the second
because the field symptom is presence/absence of an indexed semantic
occurrence.

## 3. Complete classification call/data path

### 3.1 Source admission and acquisition

`find_entities(name=ContextA)` constructs the effective scope from the requested
workspace root plus configured additional roots. Hydration discovers candidate
paths through CBM when usable and filesystem fallback otherwise. Each candidate
must pass `resolve_file_path_checked`, supported-extension checks, exclusion
policy, and trusted-root admission.

Failure behavior is lower-bound discovery: an undiscovered or failed candidate
is absent, not proven non-DbContext. Discovery completeness metadata is needed
to distinguish those cases.

### 3.2 Canonical owner and source identity

An admitted path becomes `state.semantic_owner_path(validated)`. The compilation
job carries:

- the validated physical file path;
- current source text;
- canonical owner path;
- fidelity;
- file alias/version; and
- the per-file skip set.

Root A and Root B files with the same basename remain different canonical
owners when their canonical physical paths differ.

### 3.3 Parsing and canonical C# representation

The C# compiler parses each file. Core/builtin projections capture the class and
produce `builtin/Class/ContextA`. C# base-list evidence is also captured for the
structured declaration/reference catalog used by request-local resolution.

The `MetaLayerPass` builds each class source using parser-owned capture spans,
constructs `MetaLayerContext`, attaches the structured declaration list for C#,
and evaluates registered meta layers.

### 3.4 Durable .NET producer

`DotNetMetaLayer::evaluate_if_applicable`

```text
layer enabled
  -> .NET file detector accepts source
  -> evaluate_context
  -> evaluate_applicable_source(class_captures, fidelity, config)
  -> analyze_efcore(isolated class source, fidelity)
  -> detect literal ': DbContext'
  -> extract DbSet<T> entries
  -> create HasEntity edges
       subject = dotnet/DbContext/ContextA
       object  = dotnet/Entity/T
```

`MetaLayerPass` attaches the canonical file provenance to endpoints that lack
it and adds the edges to the compilation artifact.

Absence at this stage means only “this producer emitted no durable EF semantic
relationship.” It can mean no matching class syntax, no detected DbSet, layer
not applicable/disabled, insufficient or malformed capture text, or compilation
failure. It is not a structured proof that the class is not a real EF context.

### 3.5 Artifact and publication

`SemanticCompilationArtifact` owns:

- canonical owner;
- source hash;
- fidelity;
- producer-generation fingerprint;
- compiled IR;
- semantic edges and semantic dependencies; and
- the current source snapshot.

Hydration commits an artifact only after publication-ticket and source-hash
validation. `replace_semantic_projection_with_dependencies` removes the old
file-owned projection, adds the new edges, and records coverage.

### 3.6 WorkspaceIndex occurrence registration

For every real semantic edge, `WorkspaceIndex::add_edges` attaches missing file
provenance and registers subject and object before adjacency deduplication.
Occurrence identity is:

```text
(domain, entity_type, name, file)
```

Thus a published `HasEntity` edge for each root necessarily creates two
file-distinct `dotnet/DbContext/ContextA` occurrences. Same names cannot collapse
across different canonical files.

### 3.7 Query visibility

Unfiltered `find_entities` calls `find_entities_by_name[_in_scope]` and serializes
the stored `EntityRef` values directly. No DbContext refinement or relabeling
occurs in this handler. If Root A's in-scope occurrence is missing, its current
index projection has no edge endpoint with that identity.

## 4. Root A versus Root B comparison

| Stage | Root A | Root B | Proven from field report? |
|---|---|---|---|
| Source admitted sometime | Yes: builtin Class visible | Yes: builtin Class visible | Yes |
| Current source generation | Unknown | Unknown | No |
| Isolated class bytes equivalent | Claimed structurally; bytes unavailable | Claimed structurally; bytes unavailable | No |
| .NET detector accepted current compilation | Unknown | At least one relevant projection existed | No direct proof |
| Durable EF producer ran | Unknown | Unknown; endpoint proves an edge was published, not which generation produced it | No |
| DbSet relationship emitted | No current indexed endpoint | At least one endpoint-producing edge existed | Only at index boundary |
| Structured EF import authority | Unknown | Unknown | No |
| Project metadata equivalent | Unknown | Unknown | No |
| Projection current for source/dependencies | Unknown | Unknown | No |
| `dotnet/DbContext` occurrence visible | No | Yes | Yes |

The two builtin declarations prove that both owners were compiled/published at
some point. They do not prove the same fidelity, source generation, producer
generation, meta-layer configuration, or EF relationship payload.

The controlled two-root fixture now proves the symmetric control: when both
owners contain equivalent relationship-producing EF evidence, both independently
publish file-distinct `dotnet/DbContext/ContextA` occurrences across root order,
provider state, and cold/warm execution. The original field asymmetry therefore
requires an input or lifecycle distinction not present in that control.

## 5. Earliest proven divergence

**EARLIEST PROVEN DIVERGENCE: semantic-edge projection at the publication/index
boundary.**

The proof is structural:

1. `find_entities` does not manufacture DbContext results.
2. `WorkspaceIndex::add_edges` registers every real edge endpoint before edge
   deduplication.
3. file provenance participates in occurrence identity.
4. therefore Root B's result requires a published edge carrying
   `dotnet/DbContext/ContextA` for Root B;
5. Root A's absent result means no such Root A endpoint survives in its current
   admitted projection.

Nothing in the report distinguishes these upstream alternatives:

```text
candidate missing/failed
  OR accepted stale currentness
  OR .NET layer skipped/disabled
  OR different isolated class/body evidence
  OR producer emitted no relationship
  OR publication failed/superseded
```

Static code cannot lawfully select one without per-root evidence. That is why
the verdict is runtime falsification required rather than a weaker speculative
acquisition or cache diagnosis.

## 6. Root and project resolution analysis

### 6.1 Effective roots

Hydration's `configured_roots` places the request's `workspaceRoot` first and
then configured `additional_roots`, followed by path-based deduplication.
Admission accepts a candidate under either the requested root or an additional
root. Scoping later filters occurrences by canonical file provenance.

Requested and additional roots differ in discovery/project orchestration but do
not change the durable `.NET` producer's inputs once the file is compiled.

### 6.2 Nested roots

Root matching/invalidation generally selects by canonical containment, with
some paths explicitly choosing the longest containing root. Nested roots can
therefore affect discovery scope, cache invalidation, or which provider project
is queried. They cannot directly change `analyze_efcore`, which receives only
the isolated class text and fidelity.

### 6.3 Project lookup

The exact EF catalog's `ProjectLookup` walks from a source file's parent upward
to the nearest directory containing exactly one `.csproj`. Zero projects yields
no project; more than one in the same directory is ambiguous and yields none.
Results are cached by full file string within one catalog build.

This lookup is used for assembly-wide configuration discovery. It is not used
by `is_ef_db_context`, and it is not used at all by the durable meta producer.
Package-reference contents are not inspected by either path shown here.

### 6.4 Import/framework authority

For the exact catalog, an unqualified `DbContext` requires an exact
`Microsoft.EntityFrameworkCore` import. Fully qualified framework spelling is
also accepted. For the durable producer, the literal `: DbContext` alone starts
marker analysis; imports and project metadata do not authorize it.

No root can inherit framework truth from another through project lookup. The
larger risk is the opposite: discovery may fail to compile one owner, or a
durable textual producer may claim framework meaning without structured import
authority.

## 7. Evidence dependency graph

### 7.1 Durable occurrence path

```text
canonical ContextA.cs
  + source bytes / parser-owned class span
  + .NET meta-layer enabled configuration
  + requested compilation fidelity
  + .NET semantic producer generation
  + detected ': DbContext'
  + at least one detected DbSet<T>
       -> HasEntity edge
       -> dotnet/DbContext occurrence
```

The durable producer declares no external `SemanticDependency`. `.csproj`,
package metadata, other C# declarations, and another root are not validity
inputs to this path.

### 7.2 Request-local exact edge path

```text
admitted C# source snapshots
  + structured class declarations
  + structured base references and imports
  + exact EF framework qualification/import condition
  + OnModelCreating evidence
  + referenced entity/configuration declarations
  + nearest-project equality for assembly scanning only
       -> request-local HasEntity edges
```

This catalog is rebuilt inside `PreparationContext`; it is not published as the
durable owner projection and cannot explain an unfiltered entity unless a
separate durable edge already registered that identity.

### 7.3 Mutable external dependencies

For the durable path, byte-identical `ContextA.cs` does not change truth when a
project/package file changes because those files are not consulted. This is
internally cache-consistent, though the textual authority may be architecturally
weaker than the desired structured contract.

For the request-local path, other admitted sources and project layout can change
edge truth while `ContextA.cs` remains identical. That projection is ephemeral,
so persistent semantic-coverage invalidation does not apply to it; it must be
rebuilt from the current admitted catalog for each preparation context.

## 8. Cache and generation analysis

### 8.1 Source and canonical identity

The source cache and semantic owner use physical/canonical paths, not bare names
or relative basenames. Root A and Root B do not share a source-cache identity
merely because both contain `ContextA.cs` with the same bytes.

### 8.2 Compilation artifact identity

`ArtifactIdentity` includes canonical owner, source hash, fidelity, and producer
generation fingerprints. This is sufficient to distinguish the two files and
to describe a compiled artifact precisely.

### 8.3 Workspace semantic coverage

`SemanticCoverage` stores file owner, source hash, fidelity, and declared
`SemanticDependency` records. `has_current_semantic_projection` checks exactly
those fields. It does not itself compare the artifact's producer-generation
fingerprint.

The durable .NET layer emits no semantic dependencies, so currentness for its
published projection is effectively owner + source hash + sufficient fidelity.
This leaves a static audit concern: a projection already resident in the index
can be treated as current without an explicit producer-generation comparison at
this boundary. Other restore/adoption compatibility gates may prevent stale
cross-process state, but the field report does not contain the lifecycle needed
to prove or exclude this as its cause.

This concern is not yet a proven Issue #146 defect. The bounded experiment must
record whether Root A was skipped as `Current`, and the coverage hash/fidelity
and running producer generation when that happened.

### 8.4 Weak-to-strong reuse

Coverage requires stored fidelity to be at least the requested fidelity. A Low
projection cannot satisfy a High request. `find_entities` requests
`LegacyEdit`, normalized to High coverage, so a lower-fidelity projection should
be recompiled unless another path incorrectly recorded its coverage.

### 8.5 Global invalidation

Global cache invalidation is neither justified nor necessary. Owners and
dependencies are already modeled granularly. If runtime evidence identifies a
generation omission, repair the coverage compatibility key; if it identifies
discovery omission, repair the root-scoped acquisition proof.

## 9. Identity and collision analysis

The relevant keys are not name-only:

- source cache: file path plus content state;
- semantic publication: canonical owner path and operation generation;
- artifact: canonical owner, hash, fidelity, generations;
- index occurrence: semantic identity plus file provenance;
- file projection/lifecycle: canonical file owner;
- project lookup cache: complete file string within one catalog build.

The `name_index` maps a name to multiple semantic identity keys, and each key
retains file-distinct occurrences. Registration is performed before adjacency
deduplication. Consequently Root A and Root B `ContextA` occurrences cannot
collapse solely because their names and relative paths match.

The structured C# catalog also retains declaration candidates with their files,
namespace paths, nesting, and arity. Some resolution operations deliberately
fail closed on ambiguity rather than choosing a same-name owner. The DbContext
framework-base recognition for each declaration reads references associated
with that declaration's own file and owner.

**Static result:** cross-root name collision and wrong-owner selection are
falsified for the durable occurrence registration boundary. They remain worth
observing in the exact request-local catalog, but that catalog is not the source
of the reported unfiltered occurrence.

## 10. Query-history analysis

For fixed acquired source and successful current publication, query order cannot
change `analyze_efcore` output: it is a pure file-local function of isolated
class text and fidelity. Index registration is deterministic and owner-scoped.

History can affect whether those prerequisites are reached:

- discovery completion caches can skip rediscovery;
- semantic coverage can skip compilation as current;
- external edits require refresh/invalidation unless encountered through a
  source reread path;
- CBM readiness can change the candidate-discovery route;
- a failed/superseded publication leaves the previous projection; and
- batch preparation can share hydration work.

Therefore the sequences Root A first, Root B first, together, cold/warm, A→B,
and B→A are semantically required to converge but are not statically sufficient
to explain the reported divergence. The bounded experiment must compare their
acquisition and publication receipts, not only final entity lists.

No cross-root mutable `EfCoreCatalog` survives beyond one
`PreparationContext`; within a batch its catalog is built over the shared
effective scope. Project lookup cache entries are keyed by file, so one root's
nearest-project result cannot overwrite the other's.

## 11. Provider-independence analysis

CBM supplies candidate paths and readiness/completeness signals. It supplies no
DbContext semantic authority. Every discovered file must still be admitted,
compiled by Clean-CTX, and published through the normal owner boundary.

Equivalent acquired authoritative evidence therefore produces the same durable
meta output whether CBM is disabled, unavailable, initializing, or ready.
Provider state can still change whether the evidence is acquired at all. Such a
failure is an acquisition defect, not a classification rule.

The runtime experiment must separate:

```text
candidate absent from discovery
candidate discovered but not compiled
candidate compiled with no EF edge
edge emitted but publication failed
edge published but excluded by scope
```

Only the third item is directly a semantic-producer classification question.

## 12. Relationship to Issue #144

**Classification: CANNOT DETERMINE WITHOUT RUNTIME EVIDENCE.**

There is a plausible causal path:

```text
incomplete multi-root acquisition
  -> Root A candidate not compiled/currently republished
  -> no Root A EF edge endpoint in WorkspaceIndex
```

But the visible builtin Class occurrence means Root A had some published source
projection, and the report does not establish its generation, fidelity, or
body. A current builtin declaration can coexist with absent EF relationship
evidence for legitimate or stale reasons. The #144 `lower_bound` result alone
does not prove which one occurred.

Do not wait for #144 before running the small #146 experiment. Capture #146's
per-root acquisition state so the two investigations can independently converge
or separate.

## 13. Deterministic falsification regression

A tracked symmetric-control test is now prepared in
`src/tests/mcp/workspace_query_warm_state_authority.rs` as
`issue_146_equivalent_cross_root_dbcontexts_keep_independent_occurrences`. It has
been run by the user and passed. No narrower RED regression is justified until
the original field roots expose their first unequal source, acquisition,
artifact, coverage, or publication fact.

The control uses:

```text
RootA/
  ProjectA.csproj
  ContextA.cs
RootB/
  ProjectB.csproj
  ContextA.cs
```

Use byte-equivalent project metadata and class sources. Because the durable
occurrence requires a relationship endpoint, each class must contain a minimal
DbSet:

```csharp
using Microsoft.EntityFrameworkCore;
public interface IContextA { }
public class EntityA { }
public class ContextA : DbContext, IContextA
{
    public DbSet<EntityA> Entities { get; set; }
}
```

The public-query regression must assert, with canonical file provenance:

1. both roots independently contain `builtin/Class/ContextA`;
2. both roots independently contain `dotnet/DbContext/ContextA`;
3. the two same-name occurrences remain file-distinct;
4. querying roots in reverse order produces the same per-root results; and
5. a shared batch produces the same result.

Negative controls:

- `class ContextA` produces no DbContext occurrence;
- `class ContextA : FakeDbContext` produces no DbContext occurrence;
- a `DbContext` spelling without structured EF authority must not satisfy the
  exact request-local catalog;
- one legitimately EF-enabled root and one non-EF root must remain asymmetric;
- a class header with no relationship-producing payload should not be asserted
  to have a durable occurrence under the current contract.

After runtime localization, add only the extra condition needed to reproduce
the defect: additional root, nested root, provider state, currentness skip,
generation mismatch, or publication order. The tracked regression belongs
under `src/tests/**`; any live harness is supplementary evidence only.

## 14. Bounded runtime falsification plan

Do not start a broad Claude pilot. Use one local, deterministic two-root fixture
and capture one concise evidence table for each query order.

### 14.1 Fixture

Create byte-equivalent Root A and Root B fixtures with distinct canonical paths,
one `.csproj` each, exact EF import, and one DbSet as above. Configure one as the
requested root and the other as an additional root, then reverse those roles.

### 14.2 Scenarios

Run only:

1. cold A→B;
2. cold B→A in a fresh state;
3. both names in one batch;
4. warm repeat of the same batch; and
5. provider-unavailable/filesystem fallback if the first four diverge.

### 14.3 Required observations per canonical file

- candidate discovered and by which provider;
- admission result and effective root;
- canonical owner;
- source hash and isolated class capture hash;
- requested/stored fidelity;
- semantic coverage current/compile decision;
- producer-generation fingerprint;
- .NET applicability decision;
- `analyze_efcore` invocation;
- detected DbSet count;
- emitted semantic-edge identities;
- publication ticket outcome;
- final `file_map`, coverage, and matching occurrence;
- query scope admission; and
- final serialized entity.

Prefer existing test-only work metrics, hydration diagnostics, artifact access,
and index oracles. If one observation is unavailable, add the smallest
test-only observation hook. Do not add production logging, and do not let
instrumentation affect discovery or authority.

### 14.4 Decision table

| First mismatch | Classification |
|---|---|
| Root A candidate absent/incomplete | ROOT-SCOPED EVIDENCE ACQUISITION DEFECT |
| Same candidate, different class capture | parser/capture or source-state defect |
| Same capture, producer skipped | SEMANTIC PRODUCER EXECUTION ASYMMETRY |
| Same producer output, one publication absent | publication/lifecycle defect |
| Root A incorrectly skipped as current | DERIVED-SEMANTIC CACHE INVALIDATION DEFECT |
| Symmetric fixture passes and field sources differ in authority | EXPECTED BEHAVIOR — AUTHORITATIVE EVIDENCE DIFFERS |

## 15. Rejected shortcut fixes

- **Special-case ContextA or DbContext:** bypasses the authority model.
- **Classify from base spelling alone:** the durable producer already does a
  textual approximation; extending it would not repair acquisition or
  structured authority.
- **Classify from the using alone:** an import is not type resolution.
- **Copy classification from a same-name entity in another root:** violates
  file ownership and makes ambiguity unsound.
- **Merge same-name declarations across roots:** contradicts occurrence
  identity and would destroy provenance.
- **Globally mark the workspace EF-enabled:** project/framework authority is
  not global.
- **Force both roots to one project:** erases legitimate project boundaries.
- **Warm one root before the other:** encodes query-history dependence.
- **Broaden every query to every root:** cannot distinguish missing acquisition
  from wrong authority and changes query cost/scope globally.
- **Make CBM classify DbContext:** makes an optional provider semantic authority.
- **Invalidate every semantic cache globally:** conceals the missing dependency
  or compatibility key and imposes unrelated cost.
- **Rerun every semantic producer unconditionally:** abandons valid incremental
  currentness instead of repairing its smallest broken input.

## 16. Files and symbols inspected

- `src/mcp/tool_handlers/query/entities.rs` — unfiltered entity visibility and
  shared hydration.
- `src/mcp/tool_handlers/query/prepare.rs` — preparation-local catalog and
  hydration caching.
- `src/mcp/tool_handlers/query/csharp_resolution.rs` — structured declaration,
  base-reference, import, and owner catalog.
- `src/mcp/tool_handlers/query/csharp_resolution/dependencies/efcore.rs` — exact
  EF recognition, project lookup, and request-local edges.
- `src/dotnet_meta/mod.rs` — `DotNetMetaLayer`, applicability, evaluation, and
  empty dependency publication.
- `src/dotnet_meta/detect.rs` — .NET file applicability signals.
- `src/dotnet_meta/semantic.rs` — durable .NET semantic-edge dispatch.
- `src/dotnet_meta/efcore.rs` — durable textual DbContext/DbSet producer.
- `src/ir/pipeline/meta_layer.rs` — parser-owned class spans, context assembly,
  provenance attachment, and semantic collection.
- `src/layers/meta/context.rs` — meta evaluation inputs and outputs.
- `src/layers/meta/builtin.rs` — independent source declaration carrier.
- `src/layers/meta/semantic.rs` — entity, edge, and semantic dependency types.
- `src/mcp/tool_helpers.rs` — trusted-path resolution and compilation entry.
- `src/mcp/semantic_compilation/artifact.rs` — artifact identity and producer
  fingerprints.
- `src/mcp/tool_handlers/hydration.rs` and children `filesystem.rs`,
  `execution.rs`, `publication.rs`, `acquisition.rs`, and `invalidation.rs` —
  roots, discovery, compilation, currentness, and commit lifecycle.
- `src/workspace/index.rs`, `index/edges.rs`, `index/coverage.rs`, and
  `index/declarations.rs` — identity, endpoint registration, coverage, and
  declaration separation.
- `src/tests/mcp/workspace_query_warm_state_authority.rs` — current two-root
  structured C# authority expectations.
- `src/tests/mcp/workspace_query_dotnet_efcore_configurations.rs` — exact
  request-local EF relationship behavior.
- `src/tests/mcp/workspace_query_performance_work_counts.rs` — EF preparation
  and work-count lifecycle.
- `src/tests/workspace/index_declaration_authority.rs` — declaration versus
  relationship occurrence authority.
- `docs/plans/ISSUE_145_FILTERED_DECLARATION_CONSISTENCY_INVESTIGATION_2026-10-18.md`
  — declaration/framework authority distinction.
- `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`
  and its follow-up performance/startup artifacts — acquisition context kept
  separate from this classification investigation.
- `docs/ARCHITECTURAL_INVARIANTS.md` — source authority, publication lifecycle,
  provider independence, and exact C# capability boundaries.

## 17. Smallest architectural boundary requiring repair

No production repair boundary is proven yet.

The smallest diagnostic boundary is the handoff:

```text
per-file SemanticCompilationArtifact.semantic_edges
  -> publication currentness/commit decision
  -> WorkspaceIndex::replace_semantic_projection_with_dependencies
```

Observe both roots there. If Root A's artifact lacks the edge, move one step
upstream to meta-layer applicability and `analyze_efcore` inputs. If both
artifacts contain it, the defect is publication/index lifecycle. If Root A never
produces an artifact, the defect is acquisition. This binary search avoids any
global architectural change.

One separate architectural debt is visible but not established as the field
cause: the durable .NET producer uses textual `: DbContext`/`DbSet` extraction,
while the exact request-local producer uses structured C# and import authority.
Unifying or redefining those authorities would have cross-cutting externally
observable effects and requires explicit architectural approval. Do not fold
that redesign into Issue #146 unless runtime evidence places the failure there.

## 18. Recommended next phase

**PROCEED TO BOUNDED RUNTIME FALSIFICATION.**

The prepared two-root controlled fixture passed when run by the user. The
focused command was:

```powershell
cargo test --all-features issue_146_equivalent_cross_root_dbcontexts_keep_independent_occurrences -- --nocapture
```

This command was handed off and was not run by Agent MaxHeadRoom; the user
reported it green. Do not begin a production fix, do not merge with Issue #144,
and do not launch a Claude field pilot. The next bounded step is to compare the
original field roots' actual isolated class bodies, coverage, and publication
receipts. Specifically establish whether both contained relationship-producing
`DbSet<T>` evidence in the observed generation and whether Root A was compiled,
skipped as current, failed, or omitted. Until that evidence exists, retain
`RUNTIME FALSIFICATION REQUIRED` for the original observation while recording
that the general symmetric cross-root hypothesis has been falsified.
