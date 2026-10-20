# Issue #155: cold semantic dependency convergence

Agent: Agent MaxHeadRoom

Latest acceptance (2026-10-20): operator reports focused GREEN after the MCP-found
corrections, and #155 local binary/stdio PASS (392/392 checks). The retained
receipt is `target/live-acceptance/issue-155/2026-10-20T22-41-41-624Z-1364/report.json`;
the agent inspected its complete PASS with 19 finished sessions and zero errors.
This supersedes earlier pending focused/MCP statuses. Final verification and
original-workspace field acceptance remain separate outstanding gates.

Status: operator-reported RED established; accepted production repair and four
in-scope audit blockers implemented with static regressions. The operator reported
focused GREEN on 2026-10-20. Preservation and field acceptance remain pending.
Agent execution has remained static/non-compiling.

The original regression was isolated in a path-scoped stash before production
editing and restored unchanged. Its clean Git blob matches the stash:
`5ef4312270a072621bfa315ebe9e0221ebff0b60`. The stash remains available.

The operator reported seven failures: context edges/file, Controller file, Hub
edges/file, and Profile edges/file. Controller edges was the only initial case
not listed as failing. Do not infer a complete test-run summary beyond that.

The initial causal trace below describes the pre-fix paths. The implementation
now introduces canonical ancestry obligations, shared SourceDependencyClosure
requirements/proofs/cache matching, frontier acquisition, generation reconciliation,
and final publication validation for affected workspace queries. Both Controller
base loops are removed. File queries retain canonical artifacts for reuse.

Twelve supplemental tests have been authored separately, preserving all eight
original cases. They cover missing/ambiguous/duplicate owners, lookalikes, cycles,
partials, refresh/retraction/restoration, root/batch order, aliasing, occurrence
queries, dependency proof semantics, warm parse/reconciliation reuse, and followers.
They are included in the operator-reported focused GREEN; detailed output and
runtime-selected count were not supplied.

The subsequent audit records the accepted pre-blocker sibling findings:
`docs/agent/ISSUE_155_SIBLING_PATTERN_AUDIT.md`. It found unrepresented sibling
dependencies and consumers outside the query closure boundary. This draft must
not be called a repository-wide elimination of the excluded sibling defects.
The four authorized in-scope blockers and current handoff are documented in
`docs/agent/ISSUE_155_FOCUSED_GREEN_HANDOFF.md`; explicit separate campaigns are in
`docs/agent/ISSUE_155_FOLLOW_UP_CAMPAIGNS.md`. Focused GREEN is now reported; the
next user-owned gate is the authoritative `docs/agent/verification.md` gate.

## Earliest divergence and retained evidence

The earliest actionable boundary is compilation output to acquisition preparation:
the source's base reference and resolution scope survive compilation, but no
required ancestry acquisition obligation is emitted or consumed before answering.
This precedes the final missing-role identity lookup.

`src/ir/csharp_resolution/compilation.rs` retains `ClassInput.part` and `Scope`
inside `CSharpCompilationUnit`. `CSharpDeclarationPart.base_references` retains
written bases; scope retains namespace, containing types, ordinary imports, and
aliases. `resolve_compilation_units` builds a source symbol index from the supplied
units and resolves base arcs against that index plus known external identities.
`src/ir/csharp_resolution/ancestry.rs::resolve_type_ancestry` traverses those arcs
and preserves unresolved, ambiguous, unsupported, and cycle outcomes.

The missing evidence is the authoritative declaration and ancestry of the local
project base, recursively through intermediate bases, in the compatible resolution
partition. A terminal name alone is a discovery hint, never identity proof.
The raw reference has not been lost, but it is private compilation input rather
than an acquisition requirement. Dotnet meta evaluation currently returns an empty
`semantic_dependencies` vector (`src/dotnet_meta/mod.rs`).

## Four cold production paths (pre-fix)

All paths enter `workspace_query` dispatch and query preparation. Edge preparation
is `src/mcp/tool_handlers/query/edges.rs::try_prepare_forward_edges`; source
acquisition goes through `PreparationContext::hydrate`, hydration execution,
Clean filesystem discovery, candidate compilation, and publication.

| Role | Requirement and discovery | Additional ancestry work before answer |
|---|---|---|
| Hub | LegacyEdit hydration using target name | No base acquisition loop; edge reconciliation helper does not select Hub |
| MapperProfile | LegacyEdit hydration using target name | No base acquisition loop; edge reconciliation helper does not select MapperProfile |
| DbContext | LegacyEdit hydration using target name | Generation reconciliation is selected, but it only sees admitted files |
| Controller | LegacyEdit hydration using target name | Explicit base loop hydrates each candidate through `find_entities`, then generation reconciliation |

`edge_hydration_requirement` selects High semantic hydration only for the existing
Angular Service case; the C# roles use LegacyEdit. The Controller-only loop obtains
`Catalog::aspnet_base_candidate`, hydrates the candidate, and repeats with a visited
name set. The catalog helper actually reads the unique named class's base reference;
it does not constitute a general dependency-closure proof. This directly explains
why Controller can succeed cold without implying separate correct architecture.
Whether that loop explains every original field observation still needs execution.

Compilation produces canonical artifacts and local semantic projections. Candidate
publication retains C# artifacts and marks C# authority dirty. Resolution over an
incomplete admitted-unit set cannot establish the absent base's framework terminal.
Reconciliation can correct publication only after the needed units are admitted.
Query evaluation then reads the index; a missing role becomes `identity_not_indexed`.

## Why an unrelated base query changed truth (pre-fix)

`query/entities.rs::try_prepare_find_entities` hydrates the requested base name.
For ordinary occurrence queries it also walks base candidates, then calls
`context.prepare_exact_csharp_calls(state)`. Despite its name, this invokes shared
C# generation reconciliation, including framework ancestry semantics.

`src/mcp/csharp_calls/reconciliation.rs::project_csharp_exact_calls_for_generation`
enumerates admitted indexed C# files, obtains current artifacts, groups compilation
units by resolution partition, calls `resolve_compilation_units`, and reprojects
generation semantics for each artifact. It replaces index projections, commits
generation coverage, updates IR, and remembers session semantic projections.
Thus the newly acquired base participates in resolution of an already admitted
derived class. The derived file need not change for its role and edges to change.
This is source acquisition followed by cross-file reprojection, not proof supplied
by a provider or a cache hit.

## Why file completeness was overclaimed (pre-fix)

`query/entities.rs::try_prepare_entities_in_file` reads and hashes the authorized
file, checks current projection, or compiles and publishes it. Both branches issue
`CompletenessCertificate::current_file`. That constructor takes only path and source
hash but declares `dependencies_current`, `source_complete: true`, exact results,
and `authoritative_complete`. It receives no ancestry-resolution evidence.

`workspace/index/coverage.rs::has_current_semantic_projection` checks source hash,
fidelity, and all recorded dependency hashes. It cannot detect an obligation that
was never recorded. File read completeness and current recorded dependencies do
not establish semantic ancestry closure. File preparation also lacks the ordinary
find-query generation reconciliation call. Both cold compilation and the cached
projection branch require repair; changing only the response label leaves the
available-source convergence defect unresolved.

## Relationship to #144/#148/#151 and proposed repair

Provider readiness, discovery completion, and semantic completion remain distinct.
`HydrationReport::is_complete` tests discovery status, not dependency closure.
The shared provider-neutral acquisition model in `mcp/acquisition/requirement.rs`
currently supports `BuiltinTypeRefIncoming` and `SourceDeclarations`, with
`CurrentHighSemanticProjection` evidence. Neither describes ancestry closure.
Existing proof-backed execution already separates inventory, candidate processing,
publication validation, satisfaction, and completion-cache commit. Reuse these
boundaries rather than adding a parallel C# completeness boolean.

Recommended implementation sequence after observed RED:

1. Expose structured unresolved base obligations from canonical compilation input,
   including owner identity, scope, partition, and dependency disposition. Avoid
   reparsing headers in a new query-specific resolver.
2. Represent their required evidence in shared acquisition requirements/proofs.
   Derive the final capability shape from those evidence needs; no exact new enum
   name is committed by this report.
3. Discover candidate declarations under authorized roots using authoritative
   inventory. Compile candidate evidence and let structured resolution establish
   identity, ambiguity, or proven absence. Never pick the first same-name file.
4. Expand reachable source dependencies to a fixed point, with cycle detection;
   batch each frontier and reconcile after publication. Include partial declaration
   evidence needed to establish the owner, not merely the first matching base.
5. Route affected semantic preparation, including file cache hits, through that
   obligation. Replace the Controller workaround with the same boundary once
   equivalent behavior is protected. Do not add per-framework retry loops.
6. Issue completeness only from current closure and publication evidence. Missing
   or ambiguous dependencies retain fail-closed facts and explicit unresolved
   reasons. Available supported source should be acquired, not permanently omitted.

The prompt explicitly authorizes dependency-driven acquisition and truthful
completeness. Any additional change to public schema, resolution ownership, global
limits, or unrelated language behavior still requires a separate decision.

## Affected components and lifecycle

Expected edit areas: canonical C# compilation inputs; shared acquisition requirement,
proof and satisfaction; hydration discovery/execution/publication; query preparation
and file certificates; generation reconciliation; completion-cache compatibility;
tracked MCP regressions. #153/#154 declaration identity is preserved.

Required lifecycle: compilation emits obligations; production preparation acquires
them; retained artifacts preserve inputs; source/producer generation owns validity;
reconciliation publishes the current result; index consumers answer; proof certifies
the exposed response. Refresh, deletion, recompilation, and reset must retract both
dependent roles and closure satisfaction. This lifecycle is a repair target, not
an integration claim for code that has not been implemented.

## RED regression matrix and evidence status

`src/tests/mcp/workspace_query_issue_155.rs` is registered through query.rs with the
repository `#[path]` convention. Eight tests cover Hub, MapperProfile, DbContext,
and Controller, each through forward_edges and entities_in_file. Each fixture uses
two project-base hops and CBM disabled. A warm control explicitly queries both
bases; each comparison uses fresh state in the same source root.

Orders authored: target first; find_entities on bases then target; entities_in_file
on bases then target; unrelated query then target; repeated target. The oracle
requires the expected role/relationship and compares serialized fact sets, excluding
query metadata. File responses additionally require exact source completeness;
edge responses must not claim the expected role is unindexed. The oracle is
intentionally stronger than equality between two empty results.

Operator-reported RED is recorded above. The loop stops at the first failing
order; one failing assertion is not evidence that later orders ran. No agent-run
test compilation or execution occurred.

The initially missing envelope is now authored in the separate supplemental test
file described above. The sibling audit identifies additional necessary cases
not yet authored, including attribute authority and non-query consumers. No
operator/live harness has been authored for this phase.

## Preservation matrix

| Boundary | Required preservation evidence |
|---|---|
| C# identity | #153/#154 compilation identity and preservation tests |
| Framework projection | existing C# convergence R12, SignalR R8, AutoMapper R7, ASP.NET ancestry, EF configuration tests |
| Angular/NgRx | existing dependency, object-action, registration and retraction regressions |
| #148/#151 | acquisition proof/cache/provider-equivalence and declaration contract tests |
| Concurrency | hydration_single_flight, followers revalidate their own obligation |
| Performance | workspace query work-count and batch-preparation tests; stable warm generation |
| Lifecycle | refresh, deletion and publication-retry regressions plus new ancestry mutation cases |

All preservation execution is NOT RUN. No claim of unaffected runtime behavior.

## Performance and cache implications

Do not compile the whole workspace for each query. Inventory can narrow candidates;
resolution establishes identity. Acquire the reachable dependency closure, reuse
compatible declaration evidence, batch frontier publication, and reconcile toward
one stable generation rather than rebuilding globally per dependency atom.
No timing measurements were taken. Use existing parse/compile/reconciliation work
counters before introducing broader work or new limits.

Source-cache validity, canonical artifact validity, ancestry-result validity, and
publication validity are separate. Existing acquisition cache keys include target,
roots, admission fingerprint, producer generations and contract versions; entries
also bind root generations. A new closure obligation must participate in matching
and must validate current dependency/publication evidence. A discovery-cache hit,
current target hash, or a completed hydration flight cannot certify it.

`HYDRATION-SINGLE-FLIGHT-001` remains mandatory: after waiting, followers validate
their own source/scope/requirement against authoritative current publication.
Do not hold a hydration flight while recursively acquiring the same flight key.

## Adjacent findings and proof limits

The existing reconciliation partition function uses configured additional roots
and nearest unambiguous project-directory evidence. Root/partition order needs
preservation testing; this report does not authorize ownership redesign.
The Controller candidate helper uses a name-based traversal; replacing that with
structured dependency obligations is within this campaign, while unrelated
declaration correlation defects remain separate. No #153/#154 repair is reopened.

Static inspection explains the reported mechanism but does not reproduce the
original workspace or prove every cache/order combination. Proprietary source
stays local. Claude testing remains the last field gate, after local deterministic
and binary/stdio evidence is batched.

## Exact runtime handoff

After resolving the audit's in-scope blockers, the first operator-run GREEN gate
remains the exact original command from the repository root:

```powershell
cargo test --all-features tests_issue_155 -- --nocapture
```

Report the full failures, including any warm-control failure. This command was
not started by the agent. The regression isolation/restoration step has been
performed; the unchanged oracle remains in place. Do not weaken it.

Formatting and guards below remain deferred until after focused GREEN; they are
not additional requested runtime gates at this handoff:

```powershell
cargo fmt --all -- --check
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
```

The complete final gate remains solely in `docs/agent/verification.md`; it is not
replaced by the focused command. Preservation commands are to be handed off after
focused GREEN. No build, runtime, MCP, provider, live harness, Clippy, or test
process was run. The agent performed static searches, reads, edits, diff inspection,
bounded byte/line checks, and direct Rust formatting of changed production and
supplemental files. The original RED file was excluded from formatting.
