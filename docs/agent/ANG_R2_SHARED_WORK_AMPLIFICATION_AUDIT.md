# Shared semantic work amplification: ANG-R2 field follow-up

Agent: **Agent MaxHeadRoom**. Investigation checkpoint, 2026-11-19.
Status: source investigation; **no repair or runtime diagnosis certified**.

## Trigger, scope and evidence standing

The operator reports a non-returning, memory-growing workspace_query with
type=resolver_registrations, direction=forward and a nonexistent route_path,
including a workspace with no resolvers. Hydration is separately reported healthy.
The operator explicitly requested investigation of similar problems throughout
the system, not a resolver-only assumption. Exact field logs, workspace size,
allocation traces and a behavioral RED remain uncaptured here.

This audit follows the actual producer/consumer chain and direct shared consumers:
inventory, nomination, compilation preparation/execution, publication, currentness,
query evaluation, concurrency and stability retries. Source discovery used the
available repository tools; connected Clean-CTX/PCG tools are unavailable. Retained
actual DOC-270/VERIFY-001 context remains applicable; no new runtime acquisition,
admission, test, benchmark or production process was started by the agent.
This is not a claim to have inspected every repository allocation or language.

## Call-path census and findings

| ID | Confirmed source mechanism and consumers | Consequence and qualification |
| --- | --- | --- |
| A01 | `hydration/resolver_registrations.rs::hydrate_resolver_owners` inventories source, selects every TypeScript owner and requests High semantics. `query/resolver_registrations.rs::prepare_inner` applies the selector only after preparation. | Absent-route requests do not avoid unrelated owner work. Confirmed broad-work path; operator-reported field failure, not an allocation-leak proof. |
| A02 | `hydration/bootstrap.rs::hydrate_bootstrap_owners` performs the same all-TypeScript preparation. `query/edges.rs::try_prepare_forward_edges` invokes it for exact angular/ApplicationBootstrap/bootstrapApplication after ordinary hydration. | A second public family shares the broad-work pattern; ordinary preparation may precede it. No bootstrap hang has been reproduced. |
| A03 | `hydration/publication.rs::compile_candidates_for` prepares all jobs, calls the shared executor, then validates the entire returned artifact vector before committing owners. Executor `compile` collects all results. | Peak lifetime includes prepared jobs/sources and completed artifacts for the batch. Bounded worker count does not bound total retained batch data. Applies to resolver, bootstrap, ordinary hydration and dependency frontier publication. |
| A04 | `semantic_compilation/compile.rs::compile_source_artifact` captures TS authority separately per source. `typescript_resolution/capture.rs` creates a fresh Resolver/Host. `program.rs::program_units` can enumerate/read/parse project units per resolver. | Independent owner compilations can repeat project-wide acquisition. Shared query compilation corrected by CI-VERIFY-R1 does not share TypeScript project evidence. Actual repetition depends on syntax/resolution/project configuration; no universal exact complexity is claimed. |
| A05 | `typescript_resolution/program.rs::program_units` returns `units.clone()` on a hit and stores `result.clone()` on initial completion. Program expansion also collects cloned units. `syntax::Unit` owns nested maps/vectors/bindings. | Resolver-local cache presence does not make all program use allocation-free. Tree handles and owned binding structures have different clone costs. Calls through global/ambient resolution need measured counts/bytes before optimization. |
| A06 | `workspace/index/coverage.rs::has_current_semantic_projection` checks every retained TS witness and semantic dependency. `typescript_resolution/host.rs::Witness::current` performs filesystem enumeration/read/hash operations. Used during candidate preparation and again by final resolver footprint validation. | Warm queries can repeat real filesystem work across owners sharing witnesses; index lookup is not the full cost. These checks enforce currentness and cannot be removed or globally memoized on convenience grounds. |
| A07 | Ordinary hydration and C# dependency closure use `hydration_flights.run`. Resolver/bootstrap owner preparation directly calls publication and has no equivalent flight wrapper. Dispatcher supports concurrent handlers with bounded queues. | Equivalent requests can overlap expensive preparation on these paths. Queue capacity is not unbounded; overlapping active work remains a different amplification risk. No concurrent memory-growth measurement exists yet. |
| A08 | `hydration/execution.rs` adds all retained TS owners inside roots to discovered candidates before semantic preparation, independent of the queried name. | Even a sparse/absent ordinary name query may revalidate many retained owners. This protects source-current truth; scope and avoided/repeated work need independent accounting. |
| A09 | `hydration/dependencies.rs::attempt` retries generation-invalidated proofs; its inventory acquisition retries until the root observation is stable. EFCore `obligations.rs::attempt` repeats inventory/closure when final inventory, publication stamp or nearest project changes. | Loops have no fixed attempt bound. Under sustained invalidation they may keep working without response. This does not establish an infinite loop on stable input; changing failure/retry policy requires owner approval. |
| A10 | `csharp_calls/reconciliation.rs::current_csharp_artifacts` collects missing jobs, consumes a complete executor result vector, and retains artifacts in C# authority. `hydration/dependencies.rs::publish` also invokes the shared publication batch. | C# shares batch-lifetime risks, but it does not follow the TS capture path. Persistent authority retention is intentional ownership, not automatically a leak. Inspect retirement and repeated-call steady state separately. |
| A11 | `hydration/filesystem.rs::build_inventory` reads supported source entries into a complete manifest before some callers filter source kind. | Temporary full-source storage can coexist with candidate sources and batch artifacts, including nonselected languages. Exact duplicate byte ownership depends on Arc/cache behavior; no invented byte multiplier. |
| A12 | `query/batch.rs::try_execute` prepares all items, reprepares index-backed items, and retries if a publication qualification changes. It retains all PendingItems until one shared final index guard is obtained. | Batch item count is explicitly capped at 32, but work per item and stability retries are separate dimensions. Resolver nomination is reused in PreparationContext; do not misreport every reprepare as full renominations. Investigate repeated final currentness and other catalog preparation under invalidation. |
| A13 | `tool_helpers.rs::compile_source_ir_focused_with_identity` also calls the same compile_source_artifact entry point. Its callers include compress, delta, provide-code-context evaluation, edit and calls-in-file. | Per-source project acquisition is not exclusively a workspace_query issue. These explicit-file operations do not themselves establish all-workspace nomination; repeated/multi-file use can nevertheless repeat the underlying TS project work. Measure enclosing callers separately. |

Paths above are relative to `src/mcp/tool_handlers/` unless explicitly qualified.
Executor/compile paths are under `src/mcp/semantic_compilation/`; TS paths are under
`src/ir/`; workspace index and C# reconciliation paths are under `src/`.
The exact source snapshot is the feature-branch implementation delivered at
0dd8d7503af3116241ad7c1e23c293b1d5849cc1; only investigation documents are modified.

## Important negative findings and preservation requirements

- Dispatcher request channels and trace retention are bounded in source. Do not
  diagnose an unbounded request queue solely from observed process growth.
- Standard hydration flights deliberately coordinate attempts, not semantic result
  authority. Followers rerun authoritative checks after a leader finishes; they
  must not receive unqualified cached proof by changing the flight abstraction.
- The existing publication batch checks all compilation failures and source hashes
  before its commit loop. Simply publishing each artifact as soon as it finishes
  can change failure/partial-publication behavior. The earlier streaming suggestion
  is a candidate requiring this preservation analysis, not an approved correction.
- Fresh resolver hosts and per-owner witnesses enforce source/project identity,
  provenance and invalidation. Reuse must preserve consumed snapshots, project
  options, authorized roots, source changes, failure distinctions and ownership.
- Route-name/resolver-name text filters can miss alias/barrel/namespace/local
  registrations. Fast empty answers are not correct discovery. Keep lower-bound
  semantics and explicit acquisition failures; never certify absence from failure.
- No accepted P0 batching regression has been established. The record-growth test
  varied 2/16/64 records at fixed owners and did not test unrelated-project growth.
  Prior CI and 185-check stdio results retain their exact-source scope, while field
  acceptance is blocked by the new report.

## Bounded reproduction and verification questions

Use tracked regressions under src/tests and the existing test metrics/transport.
The following is a proposed evidence matrix, not an executed result or new global
performance threshold. Exact operator commands follow construction of the narrow
regression; no broad test/benchmark campaign is proposed.

| Question | Controlled dimensions and evidence | Required preserved result |
| --- | --- | --- |
| Does an absent-route request terminate on stable input? | Fresh state, no resolver mappings; small increasing unrelated TS owner sets; one request per fresh state, exact source/project identity and externally bounded operator process. | Correct empty lower-bound result or applicable explicit failure; no false completeness. |
| Is growth caused by owner count or project size? | Vary unrelated owners separately from imported/program units, keeping registration count fixed. Observe full semantic parses, project acquisition, source reads, witness checks, response and process memory. | Distinguish per-owner preparation from per-project repeated acquisition and legitimate program evidence. |
| Does memory remain retained after completion? | Capture peak plus post-request retained state for the same pinned build; repeat a bounded number of equivalent requests in one session. | Separate temporary batch peak, intended caches/authority, allocator retention and accumulating unreachable allocations. RSS alone is not leak proof. |
| Can concurrent requests duplicate preparation? | One request versus two equivalent requests, then different selectors sharing roots; exact attempt/compiler counts and retained memory. | Preserve currentness, publication ordering and failure propagation; no global serialization policy inferred. |
| Does bootstrap share the same defect? | Existing exact bootstrap forward-edge operation on stable project with/without relevant owner; same owner/project growth axes. | Preserve native bootstrap/application configuration authority and omission behavior. |
| Do ordinary TS lookups pay for unrelated retained owners? | Existing focused find_entities/edge query before and after unrelated owners are retained. | Preserve invalidation and stale-owner retirement while identifying repeated checks. |
| Can stability retries make progress? | Existing C# dependency/EF witnesses with controlled one-shot invalidation, then retained field evidence if invalidation persists. | Preserve reacquisition versus deletion, historical/current distinctions and existing failure contracts. |
| Do shared batch changes preserve failures? | Existing publication failure, supersession, sibling, multiple-support, persistence and replay tests. | Do not turn all-before-commit failure behavior into unapproved partial publication. |

No real workspace should be repeatedly driven into exhaustion to gather evidence.
Retain operator request/log identity and stop the reported test process if growth
continues. Any external reproduction deadline/memory ceiling is a test harness
control, not a newly introduced product timeout, budget or semantic fallback.

## Repair decision points, not blanket optimization authorization

1. Project acquisition reuse: choose an exact operation-local snapshot/ownership
   boundary that prevents repeated acquisition while preserving per-owner evidence.
   Compare with structural nomination, which changes the discovery predicate and
   requires proof it preserves all admitted source forms. Recommend evaluating
   operation-local reuse first; no session/global cache is justified yet.
2. Batch lifetime: reduce redundant retained intermediate data without altering
   existing precommit validation/failure guarantees. Streaming publication is not
   an interchangeable implementation until those contracts are reconciled.
3. Currentness/concurrency/retry findings: measure independently. Do not remove
   witness checks, add global timeouts/caps or change coordination policy as an
   incidental resolver fix. Their affected contracts and approval differ.

The owner authorized this broader investigation. No explicit bounded shared-repair
approval has yet been received. Under AGENTS.md Architectural Approval Gate,
cross-cutting authority/reuse/publication/concurrency/retry decisions require an
exact owner disposition before implementation. Investigation can continue without
that approval; it does not authorize speculative remediation of every finding.

The immediate priority is A01/A03/A04 with a no-resolver/absent-route tracked
witness. A02 and A06/A08 are preservation/investigation companions; A05/A07/A09/A10/
A11/A12/A13 remain source-confirmed risks pending attributable measurements. These are
investigation standings, not approved AF-PERF FIX decisions or waived invariants.

## Knowledge and publication continuity

Covered by DOC-270 as a field-triggered shared-consumer investigation. This report
records operator observations, static findings and pending decisions separately.
Actual admission and repository guards remain required before publication. It
does not amend controlling contracts, create protected owner authorization, close
ANG-R2 field acceptance or revive a rejected optimization candidate.
