# Issue #157: Angular module metadata dependency investigation

Agent: Agent MaxHeadRoom

Date: 2026-10-20

Status: **operator-observed runtime RED established: 1 passed, 4 failed**.
Static production repair and supplemental coverage are now authored; see
[the implementation handoff](ISSUE_157_IMPLEMENTATION_HANDOFF.md).
Runtime GREEN remains NOT ESTABLISHED. The producer description below records
the investigated pre-repair implementation.
The agent inspected `target/issue-157/red.log` (65181 bytes); the focused run
completed in 12.73 seconds with 4117 tests filtered out. No production behavior
was changed. The agent started no runtime process. The initial investigation
and command-handoff statements below are historical and superseded only by
the observed receipt in the next section.

Initial investigation status: no Cargo,
Clean-CTX, MCP, CBM/provider, or live acceptance process was invoked. Source tools
were bypassed because this task explicitly prohibits runtime/provider operations;
bounded filesystem reads and searches supplied static evidence instead.

## Observed focused RED receipt

Operator execution: `target/issue-157/red.log`. The tracked test SHA-256 remains
`A5F8C6C90BD0396B8F12796B56966C24D49345CA865FA8671CE88420CAB906A1`,
matching the authored pre-run snapshot. Compilation succeeded; the four failures
are behavioral assertions, not compilation failures.

| Candidate | Observed classification |
|---|---|
| direct_cold_warm_and_admission_orders | PASS / NO REPRODUCTION for initial order: nonempty Directive relationship converges across all authored orders and repetition |
| direct_contributor_content_mutation_retracts_old_kind | VALID RED: initial Directive and fresh changed-state Pipe controls succeed; warm unchanged target still emits Directive instead of Pipe |
| direct_contributor_deletion_and_restoration | VALID RED at deletion: fresh control has no membership; warm unchanged target retains Shared Directive. Restoration assertions were not reached |
| imported_array_tracks_leaf_and_array_content | VALID RED at leaf mutation: fresh control emits Pipe; warm target retains Directive through target -> array -> declaration. Later array-empty assertions were not reached |
| reversed_batch_after_contributor_mutation | VALID RED for module-first batch: contributor item publishes Shared Pipe, while module item still returns Shared Directive. The reversed-order iteration was not reached |

This confirms the suspected contributor-validity defect through public production
dispatch. It does not establish restoration, array mutation, reversed-order batch,
durable reopen, arbitrary concurrent supersession, or full preservation outcomes.
Lower-bound coverage does not justify returning the stale positive relationship.

No production repair is included in this investigation phase. Before an approved
repair, preserve the exact test and its registration using the required narrow
stash procedure. The evidence supports a producer-owned consumed-source footprint;
the repair design must also account for snapshot consistency, missing-file
witnesses and final-publication validation before choosing its full boundary.

## Finding and earliest divergence

`src/angular_meta/module_metadata.rs::project` returns only edges and unresolved
spread names. `compose` and `resolve_leaf` read contributing TypeScript files
directly with `std::fs::read_to_string`. Neither returns source/hash obligations.
`src/layers/meta/angular_semantic.rs::evaluate` collects dependencies from routing
and selected NgRx object-action evidence, but never from module composition.

The earliest actionable divergence is the **producer result boundary**: source
evidence affects a module edge but its source identity/content does not survive
as a dependency. The shared downstream storage can retain dependencies, but
cannot validate contributors the producer never supplied. This is a static
causal finding, not observed behavioral RED.

Importantly, direct disk reads mean a contributor need not be indexed for its
declaration to affect a cold query. A cold/warm acquisition-order failure is
therefore not presumed. The primary candidate is stale derived truth after a
contributor changes while the target source stays unchanged.

## Supported envelope

These classifications describe the inspected implementation and existing tests,
not an assertion of complete Angular compiler semantics.

| Fact | Classification | Existing authority and limits |
|---|---|---|
| Decorated module identity and written metadata fields | SOURCE-LOCAL ANGULAR FACT | decorators::extract_graph_entries / extract_module_declarations; syntax-owned decorator extraction, not a new canonical Angular symbol proof |
| Local declaration membership in declarations/imports/exports | SOURCE-LOCAL ANGULAR FACT | metadata leaf plus declaration_kind in target source; Component, Directive, Pipe, Module kinds; Service rejected |
| Local const-array spread | SOURCE-LOCAL ANGULAR FACT | const_array finds a const lexical array; leaves resolve in target source |
| Direct relative imported declaration/module | DEPENDENCY-BACKED ANGULAR FACT | unique named import binding, resolved source path, decorated class with imported spelling |
| One imported const-array spread | DEPENDENCY-BACKED ANGULAR FACT | target binding, array source, matching const array, then each leaf's decorated declaration |
| Imported array leaf imported from another file | DEPENDENCY-BACKED ANGULAR FACT | array file's own named import and declaration source; both files contribute |
| RouterModule and RouterModule call expression | SOURCE-LOCAL ANGULAR FACT | exact @angular/router import; call branch checks base binding, not arbitrary framework module resemblance |
| Standalone component in imports | DEPENDENCY-BACKED ANGULAR FACT when imported | declaration kind is supported (existing #139 test); standalone compatibility rules are not validated here |
| Bare array identifier, nested imported spread, arbitrary expressions | UNSUPPORTED / LOWER-BOUND CAPABILITY | bare array identifier is treated as a leaf; imported nested spreads are recorded unresolved; no general evaluator |
| Re-export/barrel chain, default/namespace import, package resolution, tsconfig mapping | UNSUPPORTED / LOWER-BOUND CAPABILITY | no traversal in this resolver |
| Recursive module export visibility / exclusive declaration ownership | UNSUPPORTED / LOWER-BOUND CAPABILITY | emitted membership edges are bounded assertions, not Angular compilation ownership/visibility proof |

Existing oracle authority: `src/tests/angular_meta/module_metadata_issue_139.rs`
tests local/imported arrays, Directive/Pipe kinds, imported Component/Module and
RouterModule calls. The new tests require those same declaration-backed
relationships through public dispatch. Changing the decorator of the same class
from Directive to Pipe changes an existing supported kind, without adding a new
feature or changing path/name identity. Fresh changed-state controls must pass
before a stale-state comparison can establish RED.

## Complete production trace

1. `mcp/tools.rs::dispatch_tools_call` routes `workspace_query` to the query
   handler. Request operations select preparation in `query/request.rs`.
2. `query/edges.rs::try_prepare_forward_edges` obtains scope and exact identity,
   then calls `PreparationContext::hydrate`. Discovery chooses candidate files;
   candidate names are hints, not authority. Tests disable CBM.
3. `hydration/publication.rs::prepare_candidate` resolves/authorizes the candidate,
   observes target bytes/hash and tests `has_current_semantic_projection`.
   A current candidate returns without composition. Noncurrent candidates enter
   the shared compilation executor and semantic publication ticket lifecycle.
4. Canonical compilation executes `ir/pipeline/meta_layer.rs`, dispatching the
   registry's combined metadata evaluation to `angular_semantic::evaluate`.
5. `extract_non_testing_edges` selects NgModule captures and invokes
   `module_metadata::project`. Decorator metadata extraction provides three
   lists. `compose` expands bounded spreads; `resolve_leaf` proves class kinds.
6. Imported sources are read from disk independently of the source/index
   admission pipeline. They are not necessarily admitted before composition.
   `resolve_module` uses target parent + relative specifier, accepts an existing
   exact file or appends `.ts`; no configured-root/exclusion check exists inside
   this helper. The fixtures keep all files inside one authorized root.
7. ModuleProjection supplies edges but no dependencies. Angular evaluation
   supplies routing dependencies and conditional NgRx object dependencies only.
   The registry combines, sorts and deduplicates supplied dependency paths.
8. MetaLayerPass propagates edges/dependencies into compilation state; missing
   entity file provenance is filled with the target file. Compiler/artifact
   retain these outputs (`ir/compiler.rs`, `semantic_compilation/artifact.rs`).
9. Hydration commit checks target source hash via its publication ticket and
   replaces the target projection with the artifact's edges/dependencies.
   `query/entities.rs` has the analogous explicit-file admission/reuse route.
10. `workspace/index/coverage.rs::has_current_semantic_projection` requires
    target hash, sufficient fidelity and every recorded dependency current.
    `SemanticDependency::is_current` rereads and hashes the recorded path;
    absent hashes can represent absence. An omitted dependency is never checked.
11. `query/edges.rs` creates an indexed evaluator; classification reads stored
    edge occurrences and scopes by asserting file. Module edges are not
    recomposed in this evaluator. Coverage comes from exact_identity_coverage;
    do not equate ordinary indexed Angular edges with source-complete closure.
12. `query/dependencies.rs` skips Angular forward/reverse queries. Its C# owner,
    ancestry and EF adapters therefore cannot close this gap. PreparedQuery's
    extra publication validator is C#-specific. Batch coordination reprepares
    indexed items and evaluates against a shared final index guard; that alone
    does not validate an unrecorded external source read.
13. Retained/rendered/durable paths carry artifact dependencies. Durable reuse's
    `compatibility/validator.rs::validate_semantic` checks only the supplied list.
    `semantic_compilation/dependencies.rs::current_parts` returns non-C# artifact
    parts unchanged. No inspected downstream boundary reconstructs missing module
    dependencies. Durable reopen behavior has not been executed or separately
    covered by this initial candidate set.

## Concrete composition and contributor graph

```text
AppModule metadata
  direct leaf Shared -> named import -> leaf.ts -> decorated Shared
  ...LOCAL -> target const array -> target-context leaves
  ...SHARED -> named import -> shared.ts -> const SHARED array
      -> bound leaf Shared (retains shared.ts context)
          -> decorated Shared in shared.ts
          OR -> shared.ts named import -> leaf.ts -> decorated Shared
```

`BoundLeaf` encodes the array file and leaf spelling with a separator. It preserves
resolution context; it does not itself record semantic validity. The array file
is read once for expansion and again for each bound leaf's resolution. Every
consulted contributor in this graph has a semantic role, not merely a discovery
role. The imported spelling and binding select the declaration; terminal-name
search over the workspace is not used in this path.

Tracking is **absent for module composition**, including its supported two-hop
array-to-declaration case. Routing/NgRx dependencies may incidentally cover the
same file, but the new fixtures contain neither and must not rely on that.
Nested spread recursion is not supported: imported array items beginning `...`
are recorded unresolved. Local arrays are also expanded only one level. A new
recursive A -> B array -> C spread contract would be feature expansion and is
not an initial #157 oracle.

## Existing dependency machinery and authority limits

Routing's `routing/structured.rs` explicitly records consulted paths using
SemanticDependency::current. NgRx object evidence contributes its own dependency
list, conditionally assembled by angular_semantic. Both flow through the same
registry, compilation artifact, semantic coverage and durable validator.
Module metadata can use that shared representation without pretending to be
routing or NgRx. No second generic dependency framework is needed to express
path/content validity. Whether acquisition/publication requires additional
producer-owned obligations must follow observed RED and the scope analysis.

Aliases are handled by named import specifiers; zero/multiple matching bindings
fail closed. Type-only imports/specifiers are excluded. Relative module paths
distinguish same-name declarations during resolution. However emitted EntityRef
equality ignores file, and MetaLayerPass assigns missing object file provenance
to the target. Therefore this investigation does not claim a complete canonical
cross-file Angular identity model. Duplicate class/const declarations within a
file can be found by first traversal match; scoped lexical binding, export
legality and decorator shadow identity are not fully proven by these helpers.
Do not repair these broader authority limitations under an unobserved lifecycle
RED or weaken #152. Unsupported dynamic leaves are omitted; unresolved spread
names can render as metadata evidence through module_metadata::unresolved_spreads.

## Tracked candidates and expected classification

`src/tests/mcp/workspace_query_issue_157.rs` is registered from query.rs under
the angular test feature. Five tests use real public tools dispatch, with no
manually seeded edges/artifacts and no cold-case prepublication of contributors.

| Candidate | Expectation and classification rule |
|---|---|
| direct_cold_warm_and_admission_orders | A target first, B contributor forward query first, C entities_in_file first, D unrelated first, E repeat; every state must contain Shared Directive. Static expectation: likely NO REPRODUCTION for order alone |
| direct_contributor_content_mutation_retracts_old_kind | F cached target, stable leaf path/name, Directive -> Pipe; fresh control proves Pipe, warm result must drop Directive and emit Pipe. Candidate VALID RED only if fresh control succeeds |
| direct_contributor_deletion_and_restoration | baseline nonempty; delete leaf, fresh and warm target must omit membership; restore same class as Pipe, both must emit Pipe |
| imported_array_tracks_leaf_and_array_content | target -> shared.ts array -> leaf.ts declaration; change leaf kind, then empty the array; both contributor layers must invalidate unchanged target |
| reversed_batch_after_contributor_mutation | warm target then mutate contributor; query module + explicit contributor admission in both orders; module result must reflect Pipe after completed preparation |

All are NOT RUN. An initial empty result is a FIXTURE/ORACLE DEFECT or unsupported
public-path envelope until investigated, never VALID RED. A compile failure is
not RED. Passing equivalent nonempty states means NO REPRODUCTION for that
probe. Later assertions in a test are unreachable after an earlier failure:
deletion/restoration, array changes and second batch order are authored probes,
not independently observed receipts. Do not report them passed or failed without
their execution output. No injected mid-preparation mutation test is included;
the batch probe tests later contributor publication after an already-performed
mutation, not arbitrary concurrent mutation or a full #155-style late hook.

## Lifecycle, cache and performance implications

Static prediction: compiling a fresh target reads current contributors without
needing prior contributor admission. Warming the unchanged target can bypass
composition after contributor mutation/deletion because its module footprint is
empty. Publishing the changed contributor separately need not replace the
asserting target's edges. Restoration to identical old bytes can conceal stale
state, so restoration uses a different declaration kind.

Unchanged target bytes are used throughout each lifecycle probe. Fresh controls
use equivalent files and new McpState instances. Content mutation uses a stable
path/class name, avoiding timestamp-only or rename assumptions. No measured
performance claim is made. Existing raw reads and repeated tree-sitter parsing
per value are real costs. A repair should capture the minimal actually consumed
source snapshots/negative resolution witnesses and validate reuse, rather than
parse every TypeScript file. Hashing after an independent reread can certify
different bytes from those used to derive the edge; any repair must bind evidence
to consumed snapshots and surviving publication, not just append paths afterward.

## Preservation matrix and decision boundary

| Boundary | Static preservation / runtime status |
|---|---|
| #148 provider neutrality | providers disabled in tests; production unchanged; preservation NOT RUN |
| #151 completeness/acquisition | no capability/schema changes; preservation NOT RUN |
| #152 NgRx authority/lifecycle | no NgRx production edits; preservation NOT RUN |
| #155 surviving-publication proof | existing C# validator unchanged; Angular applicability documented, NOT RUN |
| #156 producer obligations/shared mechanics | EF adapter unchanged; pattern informs possible future design only |
| Routing, source-local Angular, module behavior | no production edits; #139 establishes static oracle provenance; existing suites NOT RUN |
| Cache/generation/durable reuse | representation traced; no lifecycle policy changed; runtime NOT RUN |
| Batch/single-flight/work counts | existing coordination unchanged; batch candidate authored; concurrency/performance preservation NOT RUN |
| #158 NgRx neighborhood / #159 selector-template completeness | excluded; no claims of resolution |

No architectural decision is implemented or needed to author this investigation.
After valid RED, the decision to review is whether producer-owned consumed-source
footprints suffice or module-specific acquisition/publication obligations must
also join shared preparation. Effects include Angular projection reuse, source
scope and publication proof. Alternatives are a narrow footprint repair with
snapshot validation, or a producer-owned frontier using shared acquisition
mechanics. Recommend selecting the smallest mechanism that closes the observed
production lifecycle, preserving supported semantics and fail-closed identity.
Any global cache policy, public completeness change, source-scope reinterpretation
or expanded Angular identity contract requires explicit approval before editing.

## First runtime command: operator handoff

Run from the repository root in PowerShell 7:

```powershell
New-Item -ItemType Directory -Force target/issue-157 | Out-Null
cargo test --quiet --all-features issue_157_ -- --test-threads=1 *> target/issue-157/red.log
$LASTEXITCODE
Get-Content target/issue-157/red.log -Tail 100
```

Expected authored selection: five tests. Runtime RED is NOT ESTABLISHED until
operator execution confirms an intended behavioral assertion with a valid
nonempty control. If confirmed, preserve the exact regression and registration
using the narrow stash procedure in architecture.md before production repair.
The final repository gate remains the authoritative procedure in verification.md;
it has NOT RUN for this change. Batch local evidence before any last-resort
Claude field session. No rebuild or environment switch is requested now.
