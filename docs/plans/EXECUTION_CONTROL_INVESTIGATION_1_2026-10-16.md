# Agent MaxHeadRoom — Execution-Control Investigation 1

**Repository:** `codeliftsleep2/Clean-CTX`  
**Date:** 2026-10-16  
**Phase:** Investigation / architecture only  
**Production implementation:** Not authorized; none performed

## Executive verdict

Clean-CTX does not currently have an end-to-end request execution-control contract.

The MCP server moves each accepted JSON-RPC request into an independently owned closure on a fixed worker thread. `workspace_query` then runs synchronously. No request-scoped cancellation signal or deadline accompanies that closure. The router has no cancellation-notification arm, there is no active-request registry keyed by protocol request ID, and cancellation or request timeout cannot become an observable request outcome.

Consequently:

- MCP cancellation does not reach Clean-CTX production work.
- Dropping or abandoning the client request does not stop semantic work.
- stdin EOF ends admission but does not interrupt an in-flight handler.
- The dispatcher provides queue backpressure, metrics, tracing, panic isolation, and slow-request reporting—but no execution timeout.
- CBM has provider-scoped deadlines, but these do not bound filesystem fallback, compilation, reconciliation, catalogs, graph processing, projection, or response serialization.
- Per-file semantic publication is source-validated and generation-ordered.
- Discovery completion is withheld until selected publications and required C# reconciliation succeed.

The decision-standard gap exists:

```text
caller no longer wants result
        ↓
no cancellation reaches the worker closure
        ↓
filesystem / compilation / reconciliation / derivation continues
        ↓
shared evidence may publish and a response may still be attempted
```

The smallest coherent architecture is a request-scoped execution context created at MCP admission. It should carry one cooperative stop signal, an optional absolute deadline, and a distinct stop cause. It should initially propagate only through `workspace_query` preparation and hydration, with checkpoints at natural coarse boundaries.

**Final recommendation: PROCEED TO EXECUTION-CONTROL DESIGN**

## Current production lifecycle

```text
src/main.rs::main
  → clean_ctx::mcp::run
  → src/mcp/server.rs::run
      stdin.lock + read_request_line
      serde_json::from_str::<JsonRpcRequest>
  → Dispatcher::spawn
      crossbeam bounded per-worker queue
  → worker std::thread closure
  → router::dispatch
  → tool_dispatch::dispatch_tools_call
  → HandlerRegistry workspace_query closure
  → query::handle_workspace_query
      batch::handle OR WorkspaceQueryOperation::prepare
  → PreparationContext
      scope
      hydration
      request-local C#/EF/Angular catalogs
  → hydrate_workspace_index_for
      discover_candidate_paths
        CBM project search and/or filesystem::scan
      select_candidates
      compile_candidate_for, per selected file
        resolve_file_path_checked
        preflight_semantic_publication
        compile_file_ir_focused
          read_source
          IRCompiler::compile
          PassPipeline::run
        SemanticPublicationTicket::commit
          validate current source hash
          WorkspaceIndex replacement
      reconcile_csharp_exact_calls when C# authority changed
        compile admitted C# files for declaration authority
        compile admitted C# files again for exact calls
        validate every source hash
        replace projections under one index write guard
      mark_discovery_complete only after publication/reconciliation
  → request-local semantic refinement/resolution
  → PreparedQuery evaluator against WorkspaceIndex
  → response projection and JSON construction
  → protocol::send_response through stdout mutex
```

Batch requests first prepare all items to union acquisition, prepare index-backed items again against the common post-acquisition evidence generation, and then evaluate them under one `WorkspaceIndex` read guard. This is request-local orchestration, not a publication transaction.

## Boundary classification

| Boundary | Execution shape | Caller abandonment stops it? |
|---|---|---:|
| stdin reception | blocking reader thread | EOF only stops new admission |
| dispatcher queue | bounded crossbeam channel | only before acceptance |
| request handler | synchronous worker closure | no |
| CBM discovery | blocking shared subprocess request | only provider-local deadline |
| filesystem discovery | blocking traversal and Rayon reads | no |
| parser/compiler | synchronous CPU work | no |
| publication loop | checkpointable loop with shared mutations | no |
| C# reconciliation | two synchronous compilation passes | no |
| derived catalogs | request-local CPU work | no |
| graph/refinement | synchronous index/request-local work | no |
| serialization/stdout | synchronous CPU and blocking write | no |

The production MCP request path has no Tokio task, request future, `spawn_blocking`, request `JoinHandle`, or abort handle.

## Existing execution-control primitives

No production request cancellation primitive exists.

| Mechanism | Scope | Actual behavior |
|---|---|---|
| `Dispatcher::shutdown_flag` | server/session | observed by workers between requests; cannot interrupt a running closure |
| dispatcher channel closure | server/session | allows idle workers to exit |
| stdin EOF | transport/session | ends admission, shuts down proxy, drops dispatcher, and waits for workers |
| panic recovery | one handler | catches/logs panic; not cooperative termination |
| CBM frontend retirement | provider session epoch | retires provider transport/process after terminal timeout or failure |
| proxy shutdown signal | proxy session | terminates the shared proxy lifecycle |

The C# `CancellationToken` strings under `src/test_files/dotnet` are sample inputs, not Rust control primitives.

## Existing timeout and deadline primitives

There is no request-level execution timeout.

`DispatcherConfig::send_timeout` exists, but request admission uses `try_send`; the value does not bound handler execution. Slow-request thresholds only observe latency. `Dispatcher::shutdown(timeout)` sleeps after setting shutdown and closing senders; it does not interrupt work. `Dispatcher::drop` calculates a nominal ten-second join budget, but `JoinHandle::join` is unbounded once invoked.

`CbmClient` has provider-scoped deadlines for admission, transport requests, and logical paginated collections. It uses `recv_timeout`, retains one deadline across pages/retries, and retires the frontend after terminal timeout or transport failure. These deadlines do not bound filesystem fallback or downstream Clean-owned work.

Other unrelated timeouts include proxy upstream HTTP, connection/startup probes, port probes, and process-retirement grace periods. None is an MCP request timeout.

## MCP cancellation capability versus propagation

| Claim | Verdict |
|---|---|
| Requests have IDs | Yes |
| Cancellation notifications are routed | No |
| A server library exposes cancellation | No; Clean-CTX owns a custom stdio JSON-RPC loop |
| Active requests are registered by ID | No |
| Cancellation reaches handlers | No |
| Cancellation reaches semantic work | No |
| Caller abandonment stops work | No |

An unknown cancellation notification without an ID reaches no router arm, produces no response, and changes no state. Protocol expressibility does not establish Clean-CTX support.

## Long-running work inventory

| Work | Classification | Natural checkpoint |
|---|---|---|
| configured-root discovery | checkpointable loop | between roots or directory batches |
| supported-file matching | blocking Rayon work | before/after parallel phase initially |
| source read | atomic short blocking unit | before/after file |
| hydration candidate loop | checkpointable loop | before candidate and after committed candidate |
| parse/compile | CPU-bound | before compile and after compile |
| semantic commit | shared publication | outside the atomic commit only |
| C# declaration pass | checkpointable CPU loop | between files |
| C# exact-call pass | checkpointable CPU loop | between files |
| C# replacement | shared publication | before the complete replacement section |
| C#/EF/Angular catalogs | request-local derivation | between file/source observations |
| graph traversal | request-local/index read | between frontier or node batches |
| provider call | blocking external operation | before send and after response/timeout |
| projection/serialization | normally bounded request-local work | before serialization |

Arbitrary checks inside parser, tree-sitter, or tight graph-edge loops are not justified for the first slice.

## Mutation and publication safety

| Mutation | Existing safety |
|---|---|
| per-file semantic projection | source-hash validated, generation-ordered, replacement-style |
| C# reconciliation projections | all source hashes validated before index replacement |
| discovery completion | generation-bound memo storing no candidates or answers |
| request-local catalogs | discarded with request and invalidated after evidence change |
| provider observations | advisory provider-session state |
| CBM/proxy state | shared session state |

Cancellation may leave a completed per-file publication that is individually valid even though the request did not finish. That evidence should not be rolled back.

```text
valid completed publication may survive
        ≠
acquisition or request completion was earned
```

The existing hydration order supports this distinction:

1. Discover candidates.
2. Compile and publish selected candidates.
3. Perform required C# reconciliation.
4. Only then mark discovery complete.

Handled publication or reconciliation failures return before completion marking.

## Completeness and warm-state analysis

Current ordinary failure handling is conservative:

- root, traversal, and file-read failures make filesystem discovery incomplete;
- candidate publication failure makes hydration partial;
- failed publication prevents discovery completion;
- failed C# reconciliation returns before completion;
- stale source or superseded publication generations are rejected;
- incomplete hydration produces `source_complete: false` and lower-bound semantics.

The dangerous future transition is:

```text
some candidates published
→ cancellation
→ discovery completion still recorded
→ later request skips discovery
→ incomplete acquisition is treated as complete
```

Cancellation observed before completion commit must leave discovery incomplete and retryable. Request-local C#, EF, and Angular catalogs cannot contaminate later requests because they are owned by `PreparationContext`. Valid shared per-file publications may warm later requests, but cannot independently certify complete acquisition.

## Provider execution-control analysis

CBM is a shared optional acquisition provider. Its frontend subprocess is session-owned and reused across requests. Its request IDs and deadlines belong to the Clean-CTX-to-CBM protocol, not the outer MCP request.

- A provider timeout may still permit filesystem fallback and a complete Clean-owned result.
- A request timeout means the outer request deadline elapsed; later fallback or semantic processing cannot earn success for that request.
- An outer cancellation should not automatically kill the shared CBM frontend.
- A late provider response may update provider-local health state, but the request must checkpoint before using returned paths for Clean-owned compilation or publication.

## Subprocess and process-tree ownership

| Process | Owner | Reuse | Request-owned? |
|---|---|---|---:|
| CBM frontend | `CbmClient` session epoch | shared across requests | no |
| auto-started proxy | MCP server session | shared | no |
| adopted existing proxy | external owner | shared | no |
| standalone proxy command | explicit CLI process | CLI-session scoped | no |

No relevant production subprocess is owned by one `workspace_query`. CBM cleanup is explicit through frontend retirement and `Drop`; there is no request-level `kill_on_drop` behavior.

The proxy uses Windows `taskkill /T /F`, explicitly handling the process tree. The Unix path signals the root PID and therefore does not prove arbitrary descendant-tree termination. Request cancellation must not terminate these shared processes in the first slice.

## Cancellation versus timeout

They should share one propagation mechanism but retain distinct causes:

```text
RequestExecutionContext
  state: active | stop_requested
  cause:
    client_cancelled
    transport_closed
    server_shutdown
    deadline_elapsed
  deadline: Option<Instant>
```

The deadline should be absolute rather than reset at each layer. A typed `checkpoint()` can translate an expired deadline into `deadline_elapsed`, while explicit cancellation retains its original cause.

## Observable outcome boundary

Current query outcomes are `Result<QueryAnswer, QueryFailure>`. `QueryFailure` covers invalid parameters and internal/CleanCtx/projection failures. Successful incomplete semantic results use coverage metadata.

Cancellation and timeout are execution outcomes, not semantic failures or semantic absence. The narrowest boundary is above `QueryResult`:

```text
RequestExecutionOutcome
  Completed(QueryResult)
  Cancelled { cause }
  TimedOut { deadline }
```

This avoids adding cancellation variants throughout semantic result types. For batches, request cancellation should be request-wide and must not masquerade as independent per-item semantic errors. The exact public MCP error code and data shape require design approval because they affect the external contract.

## Propagation options considered

| Option | Assessment |
|---|---|
| explicit request execution context | recommended; scoped, testable, synchronous-compatible |
| cancellation token alone | insufficient without cause, deadline, and explicit propagation |
| Tokio `select!` | poor fit; no async request task and synchronous work would continue |
| task/thread abort | unsafe or unavailable for invariant-preserving Rust work |
| timeout wrapper | only stops waiting; production work continues |
| extend acquisition budget | wrong authority boundary; would conflate execution with completeness |
| unrelated local timeouts | fragmented behavior and no coherent request outcome |
| global request registry alone | useful only for routing ID to request-scoped control |

## Recommended propagation architecture

1. Create `RequestExecutionContext` when an MCP request is accepted.
2. Register the request ID only for cancellation routing.
3. Store no semantic evidence in the context.
4. Carry an optional absolute deadline, stop state, and stop cause.
5. Pass it explicitly through router/tool dispatch into `workspace_query`, `PreparationContext`, and hydration.
6. Add one typed, inexpensive `checkpoint()`.
7. Initially checkpoint before query preparation, before each hydration candidate, after a candidate’s atomic publication, before discovery completion, and before fixing and sending the response outcome.
8. Do not interrupt inside `SemanticPublicationTicket::commit`.
9. Do not interrupt inside the C# replacement critical section.
10. Let the handler boundary own the success/cancel/timeout race.
11. Remove request-registry ownership only after the outcome is fixed; later cancellation becomes a no-op.

## Candidate checkpoint table

| Boundary | Shared mutation? | Safe checkpoint? | Initial granularity |
|---|---:|---:|---|
| before preparation | no | yes | once |
| root discovery | not until completion | yes | per root |
| Rayon matching | no | limited | before/after phase |
| provider call | provider-session state | yes conceptually | before and after call |
| candidate loop | per-file publication | yes | per candidate |
| parse/compile | publication follows | outside only | before/after compile |
| publication commit | yes | no inner check | atomic commit |
| C# passes | not yet | yes | per file |
| C# replacement | yes | before only | whole replacement |
| discovery completion | yes | critical | immediately before commit |
| catalogs | no shared state | yes | per file if needed |
| graph traversal | no mutation | yes | per frontier/node batch |
| serialization | no | yes | before serialization |
| response | transport only | must own race | once |

The safest first meaningful checkpoint is before each selected hydration candidate.

## Failure and race analysis

### Cancellation versus canonical publication

An already-entered per-owner commit should finish. It produces one valid, source-current publication. Check cancellation immediately afterward and stop before the next candidate. Do not roll back the completed publication and do not mark acquisition complete.

### Timeout versus reconciliation completion

Reconciliation should validate and commit as its existing atomic semantic unit. The request checks execution state before discovery completion and success. Evidence may remain valid even when the request times out, but timeout must prevent incomplete acquisition from earning completion.

### Cancelled request versus later warm request

Retain valid completed projections. Leave discovery completion absent if required work was skipped. The later equivalent request can reuse valid projections but must rediscover and finish remaining acquisition.

### Provider response versus cancellation

Provider paths remain advisory. Check execution state after response and before compiling them. Late provider-local observations are acceptable; new Clean-owned publication after cancellation is not.

### Transport response versus cancellation

The handler boundary must perform one active-to-completed/cancelled/timed-out transition, serialize exactly that result, and then remove registry ownership. This prevents both success and cancellation responses for one request ID.

## Deterministic regression strategy

Use test-only barriers or hooks, not wall-clock sleeps. Required regressions belong under `src/tests/**`.

| Scenario | Deterministic trigger | Expected state |
|---|---|---|
| cancel before work | pre-preparation barrier | cancelled; no publication or completion |
| cancel during acquisition | barrier before second candidate/root | earlier valid units may remain; completion absent |
| cancel between publications | publication counter/barrier | first projection valid; completion absent |
| cancel before reconciliation | barrier after C# publication | projection valid; completion absent |
| cancel during local derivation | catalog iteration hook | cancelled; no derived shared state |
| timeout during work | injected clock plus barrier | timed out; cancellation completeness rules |
| success before deadline | deterministic unexpired clock | normal success/completion |
| cancel then warm equivalent | chained requests | second request completes with cold/warm truth parity |
| provider response after cancel | fake frontend barrier | no new Clean publication; provider remains usable |
| completion-boundary race | barrier before outcome transition | exactly one outcome; completion iff success won |

## Smallest first vertical slice

```text
MCP cancellation notification for one active workspace_query request ID
→ request registry routes to RequestExecutionContext
→ explicit propagation through workspace_query / PreparationContext / hydration
→ checkpoint before each selected candidate and before completion
→ typed outer Cancelled outcome
→ deterministic regression proving:
   one completed candidate publication remains valid,
   discovery completion is absent,
   a later equivalent request completes correctly
```

This proves protocol reception, request-ID routing, propagation, meaningful production termination, observable cancellation, shared-evidence validity, completeness safety, and warm-state correctness.

## First-slice non-goals

- No default timeout duration.
- No parser or tree-sitter inner-loop cancellation.
- No Rayon per-file cancellation retrofit.
- No C# reconciliation inner-loop cancellation yet.
- No provider frontend killing.
- No proxy lifecycle or process-tree change.
- No global semantic execution state.
- No cross-request semantic state.
- No query budget.
- No completeness, authority, or provider-policy change.
- No cancellation checks throughout every refinement loop.
- No batch per-item cancellation; cancellation is request-wide.
- No broader transport redesign beyond receiving and routing cancellation.

## Required architectural answers

1. **Does MCP cancellation currently reach Clean-CTX?** No.
2. **Does dropping a request future stop semantic work?** No. Production uses a synchronous worker closure, not a cancellable future.
3. **Is there a request-level timeout?** No.
4. **Are provider timeouts distinct?** Yes. Provider deadlines exist; request deadlines do not.
5. **Which loops continue after abandonment?** Filesystem traversal/matching, hydration compilation/publication, both C# reconciliation passes, catalogs, refinement, graph traversal, projection, and response construction.
6. **Which shared mutations can occur afterward?** All normal mutations, because abandonment is invisible: file publication, reconciliation replacement, discovery completion, cache/alias warming, and provider observations.
7. **Can partial acquisition be mistaken for complete?** Ordinary handled failures are conservative. Cancellation has no representation, so no cancellation guarantee exists.
8. **Can abandoned work affect a later warm request?** Yes, because work continues. Under the recommended design, only completed valid units may warm state; interrupted acquisition must not earn completion.
9. **Which subprocesses are request-owned?** None identified. CBM and proxy processes are session/shared; adopted proxy ownership is external.
10. **Where should execution outcome become observable?** Above `QueryResult`, at the request handler/delivery boundary.
11. **What is the narrowest propagation mechanism?** Explicit request-scoped context with stop cause and optional absolute deadline, routed by request ID.
12. **What is the safest natural checkpoint?** Before each selected hydration candidate, plus before completion/outcome.
13. **What is the smallest first vertical slice?** MCP cancellation → request context → hydration candidate loop → cancelled outcome → warm-state/completeness regression.
14. **What remains out of scope?** Duration policy, deep checkpoints, process killing, ownership changes, semantic changes, and a repository-wide retrofit.

## Files inspected

Primary evidence included:

- `docs/agent/tooling.md`
- `docs/agent/architecture.md`
- `docs/ARCHITECTURAL_INVARIANTS.md`
- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/protocol.rs`, `src/proxy_spawner.rs`
- `src/mcp/{server,dispatcher,router,tool_dispatch,discovery_cache,state,tool_helpers,csharp_call_reconciliation}.rs`
- `src/mcp/state/publication.rs`
- `src/mcp/tool_handlers/registry.rs`
- `src/mcp/tool_handlers/query.rs` and focused `query/**` modules
- `src/mcp/tool_handlers/hydration.rs` and focused hydration modules
- `src/workspace/index.rs` and focused index modules
- `src/ir/{compiler,pipeline}.rs`
- `src/cbm/client.rs`, focused client modules, `src/cbm/project_search.rs`, and focused bridge modules
- `proxy/src/{main,server}.rs`

Focused repository searches covered cancellation, deadlines, timeouts, processes, spawning, shutdown, request IDs, hydration, publication, reconciliation, and completion. No CBM or MCP runtime probe was issued.

## Files changed

- Added `docs/plans/EXECUTION_CONTROL_INVESTIGATION_1_2026-10-16.md`.

No production or test source changed.

## Static checks performed

- Bounded `rg` searches of production source, proxy source, manifests, and tests.
- Exact source reads of request, hydration, publication, provider, and subprocess lifecycles.
- `git status --short` before editing: clean.
- `git diff --check` after document creation.

No Cargo command, test, build, Clippy run, binary, MCP execution, CBM execution, benchmark, or live verification was run.

## Runtime verification and separately authorized commands

No runtime verification is required to complete this static architecture investigation. Commands requiring separate authorization now: **none**.

Future implementation verification must use the authoritative procedure in `docs/agent/verification.md`.

## Final recommendation

**PROCEED TO EXECUTION-CONTROL DESIGN**

Clean-CTX already has the essential semantic foundations: source-current ordered publication, completion bookkeeping separate from evidence, request-local derived catalogs, and provider-independent authority.

It lacks the outer request lifecycle mechanism that tells those components when work is no longer authorized to continue and exposes that termination distinctly to the caller.

Design the narrow request context and completion race first. Do not begin with timeout constants, task abortion, or process killing.
