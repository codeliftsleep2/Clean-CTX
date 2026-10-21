# Issue #156: EF dependency convergence investigation and first-run handoff

Agent: Agent MaxHeadRoom

Date: 2026-10-20

Status: **VALID RED established for all three candidates: direct model, explicit
configuration, and local-project assembly application**. The completed
operator-run log was inspected after the operator supplied the assembly failure.
The subsequently authorized static production repair and supplemental regressions
are recorded in [the implementation handoff](ISSUE_156_IMPLEMENTATION_HANDOFF.md).
Focused runtime GREEN is now **18 passed, 0 failed**, as recorded in that handoff;
the complete repository gate remains pending. The agent executed no Cargo command,
Clean-CTX binary, MCP tool, provider, or live harness. The investigation below
preserves the original pre-repair production trace and RED receipts.

## Operator evidence and frozen regression

### Completed capture: all three candidates classified VALID RED

The subsequent inspection found the completed `target/issue-156/red.log`
(401941 bytes). It supersedes the partial-log observations below.

| Candidate | Final assertion location in log | Classification |
|---|---|---|
| Direct model entity | Line 866, convergence assertion at test file line 157 | VALID RED |
| Explicit configuration | Line 1834, same convergence assertion | VALID RED |
| Local-project assembly | Line 2762, same convergence assertion | VALID RED |

All three pass the supported positive warm control before reaching this
assertion. Each reports exactly the cold, unrelated, repeat/first and repeat
mismatches with count=0. Both dependency-first orderings match the required
resolved relationship. Thus the explicit-configuration failure is now classified
from its own captured behavioral assertion, not inferred from the failure list.

Captured summary at log line 2879: **0 passed; 3 failed; 0 ignored; 0 measured;
4098 filtered out; finished in 32.41s**. Cargo reports test failure and suggests
`--lib` for rerunning. No unrelated-suite pass is implied by filtered tests.
The regression hash remains
`46847103F63B45A412BACD6CD90D4DFD8B606B9B4C95FB79BA0931A51ADFCDAB`.
The agent started no runtime, changed no oracle, and made no production repair.
Further operator reruns are unnecessary to establish this initial RED envelope.

### Local log inspection after operator completion report

The agent inspected `target/issue-156/red.log` (209814 bytes at inspection).
It contains the direct-model candidate's final convergence assertion failure,
not a warm-control or compilation failure. Its mismatch list is cold,
unrelated, repeat/first, and repeat; every captured answer has count=0 and
edges=[]. Dependency-name-first and dependency-file-first comparisons completed
without mismatches. Thus the direct-model candidate is also **VALID RED**:
the supported resolved HasEntity exists in the legitimate warm control and is
absent in those four equivalent-source answers.

The available log ends during explicit-configuration execution. It contains
neither that candidate's panic nor an assembly execution result nor a final
test-result summary. Treat this file as a partial capture despite the operator's
completion report; no aggregate count, exit status, or explicit failure cause
can be inferred from it. The earlier pasted assembly failure remains valid
independent evidence. The frozen test SHA-256 was rechecked and remains identical.
No runtime was started by the agent, and no production or regression code changed.

### Earlier operator-pasted assembly receipt

The operator reported the focused test run flooded the terminal and supplied
the assembly candidate's final admission-order assertion plus a failure list
containing all three candidates. The captured panic is at the final assertion
in convergence, after assert_supported_fact has returned successfully.

This establishes the assembly positive control: one Target -> Record HasEntity
with the expected resolved entity file. The comparison loop completed, so the
absence of dependency_names and dependency_files from its accumulated mismatch
list also establishes that those two orderings matched the positive control.

| Assembly ordering | Operator-captured observation |
|---|---|
| Cold target first | count=0, edges=[], identity_indexed=true |
| Dependency names first | Matched positive control, as established by the completed mismatch accumulator |
| Dependency files first | Matched positive control, as established by the completed mismatch accumulator |
| Unrelated query first | count=0, edges=[], identity_indexed=true |
| Repeated target, first answer | count=0, edges=[] |
| Repeated target, second answer | count=0, edges=[] |

The four captured empty answers report capability_not_established,
source_complete=false, omitted_possible=true, result_semantics=lower_bound,
and dependency_proof.status=complete. This is observed semantic order divergence,
not a claim that ordinary edge coverage certified absence. The dependency proof
is insufficient to establish EF configuration-universe closure.

Classification of the earlier pasted output: **VALID RED** for the assembly candidate. The same fixture and
source root produced the supported positive relationship after legitimate public
dependency queries, and omitted it in cold/unrelated/repeated paths. Direct and
explicit candidates remain **FAILED, CAUSE NOT CAPTURED**; a failure-list entry
alone cannot distinguish their assertion failure from a fixture/control defect.
No overall run count, timing, or unrelated-suite result was captured.

The test file was rehashed after this report, before any production repair:
`46847103F63B45A412BACD6CD90D4DFD8B606B9B4C95FB79BA0931A51ADFCDAB`.
It exactly matches the pre-run authoring receipt below. Preserve this file and
its oracle unchanged. No test edits, stash operation, or production repair were
performed after the report. Follow architecture.md's regression-preservation
procedure before beginning any repair.

To capture missing diagnostics without terminal flooding, run in PowerShell 7
from the repository root:

```powershell
New-Item -ItemType Directory -Force target/issue-156 | Out-Null
cargo test --all-features issue_156_ -- --nocapture --test-threads=1 *> target/issue-156/red.log
$LASTEXITCODE
Get-Content target/issue-156/red.log -Tail 12
```

The log contains all output; only the exit code and last twelve lines appear in
the terminal. The agent can inspect the local log afterward. This rerun is
operator-owned and has NOT RUN by the agent. It is for missing candidate
diagnostics, not a prerequisite to recognizing the captured assembly RED.

The remainder records the original static investigation and first-run plan;
its pre-execution statements are historical and superseded by this section.

## Supported semantic envelope

The authority is executable production behavior, bounded by
`CSHARP-RESOLUTION-001` and `DOTNET-001` in
`docs/ARCHITECTURAL_INVARIANTS.md`. Extraction comments alone do not establish a
semantic API contract.

| Family | Current source authority and output | Boundary |
|---|---|---|
| DbContext | Canonical C# resolution proves the framework base identity, including indirect inheritance; `dotnet_meta/semantic.rs` emits the durable role | A framework-looking name alone cannot authorize it |
| DbSet | `dotnet_meta/efcore.rs::analyze_authorized_efcore` consumes the authorized class source and emits `HasEntity` for written `DbSet<T>` payloads | Source-local written relationship; it does not require or prove the entity declaration, assembly membership, or resolved entity file |
| Direct model entity | Request-local `EfCoreCatalog::build` parses `OnModelCreating` with a ModelBuilder-shaped parameter and `builder.Entity<T>()`; the catalog resolves T from the context's base-reference scope | Dependency-backed `DbContext -> Entity` HasEntity with entity file; requires an admitted class declaration |
| Explicit configuration | Parser recognizes `builder.ApplyConfiguration(new T())`; catalog resolves T and its `IEntityTypeConfiguration<E>` base contract, then resolves E | Dependency-backed context-to-entity HasEntity, not a separate configuration ownership edge |
| Configuration ownership | The configuration's resolved EF interface and generic argument associate configuration declaration key with entity declaration | Internal mapping only; no public configuration-to-entity or context-to-configuration relation found on this path |
| Assembly application | `builder.ApplyConfigurationsFromAssembly(typeof(T).Assembly)` resolves T and requires the context and marker to share one nearest `.csproj`; admitted configurations in that project contribute entities | Existing local-project convention, not compiler-grade assembly closure; arbitrary assembly expressions produce no added entities |
| Marker/attribute detail | DbSet, fluent configuration, key/column/navigation-like syntax may contribute rendered markers | Do not promote rendered markers to new dependency-backed semantic relations |

Existing contract tests are
`src/tests/mcp/workspace_query_dotnet_efcore_configurations.rs`:
`direct_explicit_and_dbset_entities_are_exact_and_deduplicated`,
`proven_project_assembly_scan_returns_ten_entities_and_excludes_other_projects`,
`unproven_assembly_expression_and_removed_registration_fail_closed`, and the two
R6 role/interface-alias tests. These were read, not executed this session.

Namespace qualification and captured imports/aliases participate in query catalog
resolution. Configuration interface aliases use the neutral resolver, with an
existing positive test. `resolve_with_reference` rebuilds a TypeShape and calls
the catalog declaration resolver; path plus generic arity select candidates,
multiple declarations at a selected path return no target. Terminal names are
not enough to establish entity identity. Do not assume this request-local
resolver implements every canonical resolver disposition or import ambiguity rule.

Partial contexts/configurations require a distinction: the canonical role path
composes valid partial symbols; the request-local EF catalog retains individual
declaration captures and its synthetic neutral index sets `is_partial: false`.
`resolve_written_class` requires an owner base-reference record. A model body in
a partial declaration without that reference, or a configuration family spanning
captures, is not thereby proven supported end-to-end. Duplicate non-partial
targets and ambiguous families must not be guessed. These are follow-up probes
after the initial supported single-declaration control, not new first-run oracles.

Scope uses authorized roots and optional path admission for catalog files. The
assembly convention additionally requires a unique nearest project; multiple
projects in the same directory yield no project proof. Canonical C# acquisition
uses `csharp_resolution_partition`; root admission is not project-reference
legality. The EF catalog's direct resolution is scoped to its admitted catalog,
without a new project-reference legality proof. Cross-project/root semantic
discovery is not certified by the existing single-project tests. Do not infer
arbitrary external assembly, NuGet, predicate, reflection, or overload support.

## Production causal trace

Paths below are relative to `src/`.

| Stage | Production authority and evidence | What #155 supplies / EF gap |
|---|---|---|
| Entry and semantic request | `mcp/tools` dispatches workspace_query; `query/request.rs::prepare` calls prepare_base then dependencies::prepare | Forward dotnet/DbContext asks for outgoing relationships, not just a source declaration |
| Initial preparation | `query/edges.rs::try_prepare_forward_edges` calls context.hydrate with edge requirement and the target name | Acquires target candidates; does not derive a model/configuration frontier |
| Acquisition requirement | `query/dependencies.rs::acquire` selects C# owner files from the query/index; `prepare_csharp_semantic_owner` delegates to hydration dependencies | File ownership and ancestry proof are explicit; no EF-specific obligation is emitted |
| Discovery | `hydration/dependencies.rs::attempt_once` inventories configured roots, filters C# entries to the production partition, selects owner and obligation discovery terms | Canonical owner family, base/alias and attribute evidence can be acquired. Model entity references, configuration generic entity arguments, and configuration universe are not explicit requirements |
| Canonical compilation | Frontier publication uses `hydration/publication.rs::compile_candidates_for`; retained artifacts contain canonical C# units/source identity | Existing compiler and structured role resolution remain authoritative; source inventory text matches are only candidate hints |
| Reconciliation/publication | `mcp/csharp_calls/reconciliation.rs` resolves partitioned units, projects artifact semantics, attaches dependency footprints, replaces projections under one index guard, and commits publication generation | Reprojects durable DbContext role and source-local DbSet facts. Does not populate request-local EF model/configuration edges as durable artifact facts |
| EF catalog construction | `query/prepare.rs::efcore_catalog` builds C# Catalog from index.file_map and cached source snapshots; `csharp_resolution/dependencies/efcore.rs::build` iterates admitted contexts/configurations | Not a filesystem acquisition operation. Available source absent from index/snapshot universe is absent from this catalog |
| Structured resolution | configuration_entities resolves EF interface then generic entity; resolve_written_class resolves direct entity, explicit configuration, and assembly marker | Missing declaration returns None and the projection continues; no residual EF acquisition obligation is recorded |
| EF semantic projection | has_entity_edge binds context and entity files; explicit/assembly config evidence contributes the same edge | Model-derived edges live in request-local EfCoreCatalog, not the canonical compilation artifact |
| Consumption/response | edges.rs combines indexed stored edges with catalog.forward/reverse using append_distinct_json_edges | Response augmentation occurs after durable publication. The catalog survives in the prepared query closure, not persisted index ownership |
| Coverage/proof | coverage.rs::exact_identity_coverage keeps ordinary EF edge results lower_bound/source_complete=false; DependencyEvidence::apply attaches the separate dependency proof | A current ancestry proof does not prove the EF universe. Empty EF output is not verified absence |
| Final validation | PreparedQuery::validated binds PublicationStamp; #155 query and batch machinery reprepare/validate against surviving publication | Protects represented requirements from supersession, but cannot establish an omitted EF requirement |

For every EF dependency-backed family, the extra evidence differs: direct entity
needs its class declaration; explicit registration needs configuration declaration,
proven interface identity and entity declaration; assembly registration additionally
needs marker/project evidence and a closed applicable configuration universe.
None is implied by closing DbContext inheritance. DbSet written facts are different:
their entity spelling is already in the context source and is not declaration proof.

The **earliest actionable suspected divergence** is requirement derivation before
frontier acquisition: the query asks for EF semantic truth but the shared adapter
derives ancestry obligations only. The eventual `None`/continue in EF projection
is downstream. This is static susceptibility, not an observed production failure.
Discovery can incidentally admit dependencies because conservative text hints
match their sources; only execution can determine whether this fixture isolates
the gap. Later public queries can enlarge the catalog universe and potentially
change the answer with unchanged target source.

## Relationship to #155 and possible repair boundary

Reuse candidates, conditional on valid runtime RED:

- Shared inventory, authoritative candidate publication, batching, single-flight,
  acquisition satisfaction, and generation reconciliation are mechanisms.
- SourceDependencyClosure currently has an ancestry target identity and matching
  contract in hydration/dependencies.rs; it is not proof of all semantic families.
- EF must own structured obligations retaining owner identity, written reference,
  scope, relationship kind, resolver disposition, and configuration-universe scope.
- PublicationStamp can validate a represented EF proof against final publication;
  a stamp alone proves neither source discovery nor configuration-universe closure.
- `semantic_compilation/dependencies.rs::footprint(s)` derives ancestry-related
  consulted evidence. It must not be described as retaining request-local EF
  model/configuration dependencies today. Persisted EF facts, if later introduced,
  would need their own real footprint and lifecycle trace.

The minimum candidate boundary is producer-owned EF requirements feeding shared
acquisition/proof, followed by EF projection at the stable surviving publication.
This is a recommendation to investigate, not an implemented or approved public
contract change. Do not overload ancestry obligations, use a name-hydrate-retry
loop, or globally widen compilation. If a repair needs new public completeness,
global cache/lifecycle policy, or project/assembly authority, report the exact
decision and alternatives under the Architectural Approval Gate first.

For entities, canonical written scope may provide discovery hints; actual
declaration identity must decide success/ambiguity. For explicit configurations,
registration syntax narrows the obligation, while interface/generic argument
evidence establishes membership. For assembly application, enumerating admitted
configurations cannot prove closure; a project-scoped authoritative inventory and
membership predicate would be required. If unprovable, preserve lower-bound
coverage and unresolved evidence rather than certifying absence.

## Tracked first-run candidates and oracle provenance

`src/tests/mcp/workspace_query_issue_156.rs` is registered through query.rs with
the existing dotnet/csharp test feature gate. All calls use public tools dispatch;
no direct index setup, compile helper, manual semantic publication, or provider is
used. The control warms through legitimate entities_in_file queries. The target
query is the first admission operation in the cold state.

| Candidate | Existing behavior required by oracle |
|---|---|
| issue_156_direct_model_entity_admission_converges | Direct builder.Entity<T> emits context HasEntity to the resolved separate entity file |
| issue_156_explicit_configuration_admission_converges | Explicit ApplyConfiguration(new T) plus proven IEntityTypeConfiguration<E> emits context HasEntity to E |
| issue_156_local_project_assembly_admission_converges | Existing typeof(local marker).Assembly and nearest-project convention includes the registered configuration entity |

Every fixture uses separate target, inherited Bridge, entity and configuration
files, an authorized temporary root with one project, and same-terminal-name
lookalikes in another namespace. Direct/configuration registrations are namespace
qualified. CBM is explicitly disabled. The positive control must produce exactly
one HasEntity with Target/Record names and the canonical Record.cs object file.
This prevents equality between empty outputs and rejects a lookalike substitution.
Warm-control failure stops that candidate before any cold mismatch is called RED.

Each candidate compares equivalent files under fresh state for target-first,
dependency-name-query-first, dependency-file-query-first, unrelated-query-first,
and repeated-target-query. The first repeated answer is checked too. Mismatches
are collected across all five orders and include full answers/coverage. Expected
supported behavior is the same single resolved HasEntity in every ordering.
Tests compare complete serialized HasEntity facts, not unrelated coverage counters.

Original first-run classification criteria:

1. VALID RED: warm control passes and equivalent-source cold/order results differ
   on the supported relationship. Record exact outputs and the test hash before
   any repair; preserve the regression using architecture.md's RED/GREEN process.
2. FIXTURE/ORACLE DEFECT: compilation failure, malformed response, failed warm
   control due to setup/assumption, or wrong path/name assertion. Investigate it
   before claiming RED; correct and rerun the candidate if necessary.
3. UNSUPPORTED CAPABILITY: evidence shows the oracle exceeds the actual supported
   envelope. Document why; do not implement new semantics to satisfy it silently.
4. NO REPRODUCTION: all orders converge with the required positive facts. Trace
   incidental acquisition or existing closure; do not manufacture a repair.

These are candidates, not frozen observed RED. Compute SHA-256 after the first
valid failing execution and before changing production. Preserve the exact file
and registration; supplemental adversarial tests belong separately.

Static authoring receipt (not runtime evidence): test file SHA-256
`46847103F63B45A412BACD6CD90D4DFD8B606B9B4C95FB79BA0931A51ADFCDAB`.
Fast byte-level reads verified strict UTF-8 without BOM for all three changed
files. Code files contain 466 and 180 newline-terminated lines respectively,
within the 615-line ceiling. `git diff --check` reported no whitespace errors.
Compilation, formatting, repository encoding guard, and file-size guard are
NOT RUN; their authoritative commands remain in verification.md. No commit or
push was made.

## Preservation, lifecycle and performance plan

| Preservation family | Required authority / future verification |
|---|---|
| #153/#154 | Preserve declaration identity/span correlation; existing compilation identity and preservation tests |
| #155 | Preserve ancestry/attribute closure, retained footprints, final-publication proof and mixed/batch convergence tests |
| Existing EF | Existing configuration, DbContext role, durable/source-local DbSet and project-lookup tests |
| Other .NET | SignalR R8, AutoMapper R7, ASP.NET R9/R11, MSTest R10 and convergence R12 suites |
| #148/#151 | Provider-neutral acquisition and capability-specific satisfaction/completeness suites |
| Angular/NgRx | Preserve their independent authorities, existing query/refinement regressions |
| Concurrency/cache | HYDRATION-SINGLE-FLIGHT-001 followers, source/hash/generation compatibility and publication retry/supersession |
| Work/order | Existing workspace-query performance work counters and batch preparation/order regressions |

All runtime preservation gates are NOT RUN. No production repair means no
preservation pass is claimed. The full final gate remains solely authoritative
in `docs/agent/verification.md`.

After valid RED, extend separately to missing/ambiguous targets, aliases, partial
and duplicate declarations, root/batch order, provider equivalence, cached/follower
queries, and final-publication supersession. Validate unchanged target source
while dependency absence becomes appearance, dependency retargeting retracts old
truth, deletion retracts truth, and restoration restores it. Query-local EF
catalog rebuilding and durable source-local projection have different ownership;
do not claim durable model-edge integration without tracing an actual consumer.

Performance is unmeasured. Existing request-local catalogs amortize work and are
invalidated after publication; #155 acquisition caches bind owner source hash,
partition, root/producer and C# publication generations. New EF evidence must
participate in compatibility rather than reusing an ancestry-only proof as EF
closure. Measure parse/compile/reconciliation/inventory counters and repeated warm
queries before/after any fix. Prefer a narrow authoritative candidate frontier,
batched publication, and one stable reconciliation phase. Never compile the
whole workspace on each EF query. Inventory enumeration and compilation are
distinct costs; a correct inventory proof is not permission for broad compilation.

## Exact first runtime command: operator-owned

Run from the repository root, then return the complete output:

```powershell
cargo test --all-features issue_156_ -- --nocapture --test-threads=1
```

This was the original operator handoff. The operator subsequently reported the
failures recorded above, establishing assembly RED. No agent-run runtime or
Claude pilot was started. Use the redirected capture command above for missing
diagnostics. Claude field testing is not needed at this gate.
