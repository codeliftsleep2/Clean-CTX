# Agent MaxHeadRoom — Issue #144 Startup Connection Timeout / Duplicate Process Investigation

**Date:** 2026-10-18  
**Scope:** causal investigation only  
**Production changes:** none  
**Runtime activity:** none; no Cargo, build, test, MCP session, CBM process, or
live verification was started

## 1. Executive verdict

The initial connection regression has a specific static cause.

Current Clean-CTX synchronously launches and admits the optional CBM frontend
inside `McpState::new`. Admission sends CBM `initialize`, sends
`notifications/initialized`, requests `tools/list`, and validates the provider
contract. That entire operation occurs before Clean-CTX constructs its MCP
dispatcher, acquires stdin, reads the host's `initialize`, or can emit its own
initialize response. It uses `cbm.query_timeout_ms`, whose default is exactly
30,000 ms.

The earliest main-versus-current divergence is commit `3fe756d6` (`fix(cbm):
admit frontend protocol and capabilities before usability`). `main` already
spawned CBM and started indexing during `McpState::new`, but it treated child
spawn as availability and did not wait for provider admission. The current
branch added `c.admit()` directly to the same pre-readiness constructor path.
The outer MCP entry point, stdin loop, dispatcher, router, and initialize handler
are byte-identical between `main` and the current branch.

This proves that optional CBM readiness incorrectly gates core Clean-CTX MCP
readiness and can consume the host's whole connection window. It does not prove
which internal provider/daemon step consumes the time on the affected machine;
CBM frontend admission transitively includes account-daemon coordination.

The duplicate Clean process is related but not statically reduced to the same
defect. While Clean A is blocked in pre-readiness CBM admission, it has not begun
reading stdin and therefore cannot observe host EOF. After admission returns,
Clean exits on stdin EOF, but it ignores stdout write/flush failures and has no
parent-death, abandoned-initialize, or single-instance mechanism. A first
process that remains alive well after the bounded admission path most strongly
implies that the host kept its stdin pipe open or did not terminate the failed
child. A small runtime trace is required to distinguish that host lifecycle
from a Clean shutdown defect.

### Required categorical verdicts

- **A. Initial connection timeout:** **CBM WORK INCORRECTLY BLOCKS MCP
  READINESS**.
- **B. Duplicate Clean process:** **RUNTIME TRACE REQUIRED**.
- **C. Relationship:** **TIMEOUT CAUSES EXPOSURE OF SEPARATE SHUTDOWN DEFECT**,
  with the owner of that second defect not yet proven.
- **Final recommendation:** **PROCEED TO BOUNDED RUNTIME FALSIFICATION**.

## 2. Evidence and comparison boundary

Known-good control:

- `main` at `b5a152e8343808ab84c105d77b46458593608d03`.

Investigated current branch:

- `(fix)Performance-Regression-Issues` at
  `b496dd05e8f585300aa790b9d978adb6fe5e75a6`.
- Merge base with `main` is the `main` commit above.

Field evidence supplied for this investigation:

- Clean A start approximately 5:01:56 PM.
- Clean B start approximately 5:03:21 PM.
- Initial host connection commonly times out around 30 seconds.
- Manual reconnect starts a second Clean process and succeeds.
- Clean A remains alive.
- A standalone observation found roughly 20 seconds to first useful
  interaction in some conditions; this is supporting context, not causal proof.

Evidence labels used below:

- **PROVEN:** direct current/main source or commit-history fact.
- **FIELD:** supplied observation.
- **INFERENCE:** best explanation consistent with proven facts.
- **UNKNOWN:** requires runtime evidence.

## 3. Exact current startup call path

```text
clean-ctx.exe
  -> main
  -> mcp::run
  -> mcp::server::run
     -> observability::init_tracing
     -> analytics::bpe_or_init
     -> find_project_root
     -> CleanCtxConfig::load
     -> McpState::new                         PRE-READINESS
        -> SqliteStore::open                 PRE-READINESS
        -> list durable contexts             PRE-READINESS
        -> init_cbm_bridge                    PRE-READINESS
           -> GraphBridge::try_create_with_roots
              -> resolve_cbm_binary
              -> CbmClient::try_launch
                 -> spawn CBM frontend
                 -> create pipe workers
              -> CbmClient::admit             BLOCKING, PRE-READINESS
                 -> CBM initialize request
                 -> wait for response
                 -> notifications/initialized
                 -> CBM tools/list request
                 -> wait for response
                 -> validate capabilities
              -> start_indexing_roots         BACKGROUND after admission
           -> GraphCacheStore::open           PRE-READINESS
        -> SemanticCompilerExecutor::new      PRE-READINESS
     -> optional proxy startup/cache query    PRE-READINESS
     -> Dispatcher::new
     -> optional metrics thread
     -> stdin.lock
     -> read_request_line                     TRANSPORT INPUT BEGINS HERE
     -> enqueue host initialize
     -> router::dispatch
     -> handlers::handle_initialize
     -> protocol::send_response + flush
```

Clean-CTX does not expose a listening socket or independent ready signal. For
this stdio server, effective external readiness begins only when it starts
reading the host pipe and can answer `initialize`. Constructing the child
process and pipes is not Clean MCP readiness.

The initialize handler itself does not touch CBM, persistence, workspace
hydration, the semantic compiler, or the graph index. It serializes the fixed
protocol version, capabilities, server info, and workflow instructions. The
Issue #144 reverse-edge acquisition/reconciliation path is unreachable before
an initialized client later invokes `workspace_query`.

## 4. Main-versus-current startup diff

The following core files are unchanged between `main...HEAD`:

- `src/main.rs`;
- `src/mcp/server.rs`;
- `src/mcp/dispatcher.rs`;
- `src/mcp/router.rs`; and
- `src/mcp/handlers.rs`.

Therefore the regression is not a changed host initialize handler, changed
stdio loop, or changed dispatcher construction order.

| Difference | Main | Current | Classification | Causal status |
| --- | --- | --- | --- | --- |
| CBM child launch in `McpState::new` | Synchronous spawn | Synchronous spawn | PRE-READINESS BLOCKING | Existing risk, not the branch divergence |
| CBM protocol admission | None | `c.admit()` waits for initialize and tools/list | PRE-READINESS BLOCKING | Earliest proven regression boundary |
| CBM admission budget | Not used at startup | `query_timeout_ms`, default 30,000 ms | PRE-READINESS BLOCKING | Matches host timeout scale |
| CBM indexing | Starts after spawn | Starts after successful admission | BACKGROUND | Not required for Clean initialize; not direct initial wait after thread spawn |
| CBM frontend lifecycle ownership | Loose child/pipes | Explicit `FrontendSession`, retirement, worker joins | SHUTDOWN/LIFECYCLE | Relevant to CBM child cleanup, not evidence Clean A exits |
| Persistence stats rebuild | Synchronous | Removed | NOT CAUSAL | Current branch does less work |
| Durable context enumeration | Synchronous | Synchronous | PRE-READINESS BLOCKING | Same basic path; no matching branch divergence |
| Semantic compiler pool | Absent | Constructed in state | PRE-READINESS | Small bounded thread creation; no 30-second wait in source |
| Clean MCP initialize | Fixed response | Same fixed response | POST-READINESS | Not causal |
| Issue #144 query acquisition | Request-triggered | Request-triggered | LAZY | Not reachable on initial connection |

Commit `3fe756d6` changed `GraphBridge::try_create_with_roots` from accepting
`try_launch` success immediately to calling `c.admit()` before the bridge can be
available. The admission deadline is one shared `Instant` budget across the CBM
initialize response, initialized notification, tools-list response, validation,
and final liveness check. A timeout retires the frontend and may spend up to an
additional five seconds in direct-child cleanup/worker joining.

## 5. Startup critical-path accounting

| Stage | Main | Current | Blocking? | External dependency? | Potential latency | Must precede truthful Clean initialize? |
| --- | --- | --- | --- | --- | --- | --- |
| Tracing setup | Yes | Yes | Local | No | Small/unknown | No |
| BPE eager initialization | Yes | Yes | File/CPU | Local files | Unknown | No; source comments already call failure non-fatal |
| Project-root/config resolution | Yes | Yes | File metadata/read | Filesystem | Usually small | Configuration must be known, but not all derived work |
| Persistence DB open | Yes | Yes | SQLite open | Filesystem/lock | Unknown | No for initialize contract |
| Durable owner enumeration | Yes | Yes | SQLite query | Filesystem/DB | Data-dependent | No for initialize contract |
| CBM binary resolution | Yes | Yes | Filesystem/PATH probes | Installation | Small/unknown | No |
| CBM frontend spawn | Yes | Yes | Process spawn | Provider executable/OS | Unknown | No; CBM is optional |
| CBM initialize/admission | No | Yes | **Waits up to 30 s** | Frontend + shared daemon | 0–30 s plus retirement | **No** |
| CBM tools/list validation | No | Yes | Same admission deadline | Frontend + shared daemon | Shares 30 s | **No** |
| Root indexing submission | Yes | Yes | Thread spawn only at caller | CBM/daemon after spawn | Work is background | No |
| CBM graph-cache DB open | Yes | Yes | SQLite open | Filesystem/lock | Unknown | No |
| Rayon executor creation | No | Yes | Thread-pool creation | OS | Small/unknown | No |
| Optional proxy startup | Yes | Yes | Spawn/port/cache probe | Proxy process | Unknown | No |
| Dispatcher creation | Yes | Yes | Thread creation | OS | Small/unknown | Required by current architecture before requests |
| Read host stdin | After all above | After all above | Blocks awaiting host | Host pipe | Intended | This is effective readiness boundary |
| Handle Clean initialize | Same | Same | Fixed serialization | No | Small | Yes |

The configuration field `cbm.auto_launch` is declared and defaults to true, but
production startup does not read it. `GraphBridge::try_create_with_roots` gates
launch only on `cbm.enabled`. Consequently, setting `auto_launch = false` does
not make the documented “launch on first graph query” behavior true and cannot
remove this pre-readiness path. This is a configuration/implementation
discrepancy relevant to startup, though the default-enabled field means it does
not by itself explain the main/current difference.

## 6. CBM startup topology and blocking authority

```text
Clean-CTX process
  -> owned CBM MCP frontend child
     -> account-level cohort/bootstrap coordination
     -> authenticated IPC to shared account daemon
     -> daemon application session
     -> provider MCP initialize/tools registry
  <- frontend JSON-RPC admission replies
```

**PROVEN:** Clean owns the direct frontend child and its stdin/stdout/stderr
pipes. It does not own the shared account daemon or global provider runtime.
`FrontendSession` retains the direct `Child`, buffered stdin, stdout receiver,
and two worker join handles. Its retirement closes stdin, drops the receiver,
kills only the direct child when necessary, polls it for up to five seconds,
and waits boundedly for its pipe workers. It intentionally does not kill the
shared daemon or a process tree.

**PROVEN:** admission is synchronous in the Clean startup thread. Provider
indexing begins only after admission succeeds and is placed on detached
`cbm-indexer` threads. `ensure_indexed` is report-only for ordinary cold
startup state, although later dirty-project queries can synchronously reindex.

**PROVEN BY EXISTING DOCUMENTED PROVIDER INSPECTION:** the frontend itself
coordinates with the shared account daemon before serving its stdio frontend.
Therefore Clean's admission wait can include or await daemon bootstrap,
compatibility coordination, account admission, and frontend MCP readiness.

**UNKNOWN:** which provider substage consumes the affected 20–30 seconds. The
current Clean trace has no timestamps around frontend spawn/admission phases.

**INFERENCE:** the successful reconnect may benefit from daemon/bootstrap or
provider state completed during Clean A's first attempt. The static code permits
that sequence but does not prove it; the second success could instead be timing
or host behavior unrelated to warm state.

## 7. Exact shutdown and lifecycle ownership path

Normal Clean shutdown is stdin-owned:

```text
host closes Clean stdin
  -> read_request_line returns None
  -> server logs stdin exhausted
  -> auto-started proxy child is stopped
  -> Dispatcher is dropped
     -> shutdown flag set
     -> request senders removed
     -> worker JoinHandles joined
     -> final McpState Arc released
        -> GraphBridge/CbmClient eventually dropped
           -> direct CBM frontend retired
  -> mcp::server::run returns
  -> main returns
  -> clean-ctx.exe exits
```

Important qualifications:

1. Before `stdin.lock`/`read_request_line`, Clean cannot observe EOF because it
   is executing state construction. EOF may already be pending, but it is not
   consumed until CBM admission finishes or times out.
2. `send_response` discards both `writeln!` and `flush` errors. Broken stdout
   therefore does not become a terminal session event.
3. There is no explicit MCP session state, initialize deadline, parent PID
   watchdog, abandoned-client flag, or single-instance guard.
4. Reconnect has no in-process rendezvous with Clean A. The host can launch
   Clean B independently, and both can open the same configured persistence and
   graph-cache databases and contact the same account daemon.
5. `Dispatcher::Drop` claims a timeout but performs ordinary blocking
   `JoinHandle::join`; its calculated remaining duration does not bound a join
   already entered. This can retain Clean after EOF if a dispatched tool call is
   still blocked. No such dispatched call is needed to explain the initial
   startup timeout because the dispatcher does not yet exist during CBM
   admission.
6. Detached metrics/proxy-monitor/indexing threads do not, by themselves, keep
   a Rust process alive after `main` returns. They can retain resources while
   `main` is alive, and the indexing thread's `Arc` can delay `CbmClient::drop`,
   but they are not static proof that the OS process survives a completed main.

## 8. Duplicate-process causal chain

Expected failure lifecycle:

```text
Clean A launched
  -> host abandons connection
  -> host closes A stdin or terminates A
  -> A observes EOF (after any pre-readiness block returns)
  -> A exits
  -> reconnect launches B
  -> only B remains
```

Observed lifecycle:

```text
Clean A launched
  -> A blocks in CBM admission before reading host initialize
  -> host's connection deadline expires
  -> A survives
  -> reconnect launches Clean B
  -> B succeeds
  -> A and B remain
```

The earliest point where A could learn that the stdio session is gone is its
first `read_request_line`, which occurs only after state construction. There is
no independent cancellation/liveness channel during construction.

Static classification of the duplicate:

| Candidate | Static result |
| --- | --- |
| HOST NEVER TERMINATES FAILED CHILD | Plausible and strongest explanation if A remains blocked on an open stdin pipe |
| CLEAN FAILS TO EXIT AFTER TRANSPORT LOSS | Partially supported for broken stdout because errors are ignored; not proven for stdin EOF |
| BACKGROUND OWNER PREVENTS EXIT | Not proven; detached threads do not prevent process exit after main returns |
| CBM CHILD/WAIT PREVENTS EXIT | Explains delayed EOF observation for up to admission/cleanup bounds, not indefinite Clean survival |
| DISPATCHER LIFECYCLE DEFECT | Possible only after dispatcher creation and a stuck request; not initial causal path |
| INITIALIZE WORK OUTLIVES SESSION | CBM pre-initialize construction does; host-side session loss delivery remains unknown |
| OTHER | No static evidence |

The 85-second separation between example process starts exceeds the normal
single 30-second admission budget and five-second retirement window. That makes
“Clean A is still solely inside the first bounded admission” insufficient as a
complete explanation for its continued survival. Whether A is then blocked on
stdin, blocked in a dispatcher join, or elsewhere requires timestamps and pipe
events from the actual host launch.

## 9. Shared-resource consequences

Two surviving Clean instances can concurrently:

- open the same configured Clean persistence SQLite database;
- open the same global or per-workspace CBM graph-cache SQLite database;
- spawn separate CBM frontend children;
- attach separate provider sessions to the same account daemon; and
- start indexing the same configured roots.

SQLite supplies its own locking, and CBM owns daemon/project coordination, but
those mechanisms do not turn two Clean instances into one logical MCP session.
There is no Clean single-instance ownership token or database lease. This report
does not conclude that one is required; it establishes only that duplicate
instances are currently permitted and uncoordinated at the Clean process level.

## 10. Proven facts versus remaining hypotheses

### Proven facts

- Current Clean blocks on optional CBM admission before reading its MCP stdin.
- The default admission timeout is 30 seconds.
- This blocking call was added after `main` in commit `3fe756d6`.
- The Clean initialize handler is fixed, local, and does not require CBM.
- Core MCP startup/transport files are unchanged from `main`.
- Reverse-edge hydration and exact-call reconciliation are not on initialize's
  call path.
- CBM indexing is background work after successful admission.
- Clean terminates normally on observed stdin EOF.
- Broken stdout is ignored and does not cause shutdown.
- No parent monitor, session-abandonment channel, or single-instance mechanism
  exists.
- Direct CBM frontend cleanup is owned and bounded; shared daemon shutdown is
  intentionally outside Clean ownership.

### Hypotheses requiring runtime falsification

- Provider daemon/bootstrap is the component consuming most of admission's
  30-second budget.
- Clean A completes one-time provider work that makes Clean B fast.
- The host retains A's stdin pipe after declaring the connection failed.
- The host closes stdin but A is retained by a later blocking join or request.
- A's delayed initialize response encounters broken stdout.
- B succeeds because of warm provider state rather than nondeterministic timing.

## 11. Smallest runtime falsification plan

Do not run this plan as part of the investigation. The user must separately
authorize a specific runtime command/session.

Use one opt-in lifecycle trace sink on stderr or a dedicated file. Emit one
line per boundary, never per source file, with monotonic elapsed time, wall
time, Clean PID, parent PID, and thread name. Required events:

1. `process_entry`;
2. `project_config_complete`;
3. `state_construction_begin`;
4. `cbm_spawn_begin` / `cbm_spawn_end` with frontend PID;
5. `cbm_admit_initialize_begin` / response or error;
6. `cbm_admit_tools_list_begin` / response or error;
7. `cbm_admission_complete`;
8. `cbm_index_submit` only (not per-file/index progress);
9. `state_construction_end`;
10. `dispatcher_ready`;
11. `stdio_read_begin`;
12. `clean_initialize_received`;
13. `clean_initialize_response_begin`;
14. `clean_initialize_response_flushed` or `stdout_error`;
15. `stdin_eof` or `stdin_error`;
16. `dispatcher_shutdown_begin` / `dispatcher_shutdown_end`;
17. `cbm_frontend_retire_begin` / end;
18. `process_exit`.

Collect the same trace for one `main` launch, current Clean A's failed launch,
and current Clean B's reconnect. Capture the host's child-launch/timeout/pipe
closure events if the host exposes them. A process snapshot at the timeout and
after B connects should record PID, PPID, start time, command line, and direct
children. No source indexing or query workload is necessary.

The trace discriminates the required hypotheses:

| Observation | Conclusion |
| --- | --- |
| No `stdio_read_begin` by 30 s; CBM admission open | Clean never became MCP-ready because pre-readiness CBM blocked |
| Initialize received; response begins late | Synchronous work after receive delays response |
| Response flush errors | Transport delivery failed and Clean currently ignores the terminal signal |
| Host timeout precedes admission completion | Host abandoned while initialization continued |
| Host closes stdin; no `stdin_eof` after admission | Clean input lifecycle defect |
| `stdin_eof` occurs; no process exit | Dispatcher/resource shutdown defect |
| A admission completes provider bootstrap; B admission becomes fast | One-time warm-state relationship supported |
| B is fast without A completing relevant work | Warm-state theory falsified or unrelated |

Existing tracing records dispatched-request queue/execution timing, but it
starts only after dispatcher construction. It cannot observe the proven
pre-dispatch admission gap, so bounded startup events are necessary.

## 12. Architectural boundary established, without fix design

The smallest boundary that must change is ownership of optional-provider
admission relative to core MCP readiness:

> Truthful Clean-CTX initialize readiness must not depend on launching,
> admitting, indexing, or otherwise waiting for optional CBM.

This is stronger than merely reducing the CBM timeout. The provider may still
need strict admission before any provider-backed tool reports availability;
that provider truth boundary is valid. What is invalid is placing it in the
constructor that gates Clean's transport/session establishment.

No implementation design is approved here. Moving admission, choosing lazy
versus background behavior, defining provider status transitions, and defining
abandoned-session shutdown semantics require a separate design phase after the
bounded trace resolves the duplicate-process owner.

## 13. Issue ownership recommendation

Keep the startup timeout under Issue #144 until the causal startup regression is
closed because it is the remaining observed Issue #144 application-entry
failure and its regression boundary is now proven.

Create a dedicated follow-up lifecycle issue if the bounded trace shows either:

- the host closes or breaks transport and Clean fails to exit; or
- the host intentionally retains failed children and needs an explicit launch
  ownership/termination contract.

Do not combine that lifecycle repair with reverse-edge performance or exact-call
authority. The optimized semantic query path is not involved in MCP initialize.

## 14. Recommended next phase

**PROCEED TO BOUNDED RUNTIME FALSIFICATION.**

The startup defect itself is statically proven, but the requested duplicate-
process verdict cannot responsibly be assigned to Clean or the host without
observing stdin/stdout/parent lifecycle events for Clean A. After that one
trace, proceed to startup/lifecycle design with two independently stated
contracts if the evidence confirms separate owners.

## 15. Bounded falsification instrumentation status

**Executed 2026-10-18.** An opt-in lifecycle trace records
the boundaries listed in section 11 when `CLEAN_CTX_STARTUP_TRACE_FILE` is set.
The value may contain `{pid}` so simultaneous Clean A/B processes write separate
JSONL files. With the variable absent, the trace performs no file opens, writes,
or event formatting.

The operator workflow and a read-only Windows PID/PPID snapshot script are in
`verification/startup-lifecycle/`. Generated evidence remains under
`target/issue-144-startup/**`. No timeout, provider, startup, transport,
shutdown, or public MCP behavior when the variable is absent.

## 16. Authorized implementation status

After static inspection established the pre-readiness admission defect and the
field observations confirmed its latency impact, the user authorized the
provider-lifecycle correction. The implementation now keeps CBM
out of `McpState` construction, triggers configured auto-launch only after a
successfully flushed Clean initialize response, coalesces launch through one
session owner, reports initialization explicitly, fails provider consumers
closed until admission succeeds, and joins admission plus background indexing
before direct-frontend retirement. `auto_launch = false` now defers the same
lifecycle until the first provider-dependent consumer.

Focused tracked regressions are GREEN for core readiness independence,
lifecycle trigger/coalescing/shutdown, and indexing-worker/frontend ownership.
The authoritative all-feature final verification gate is GREEN.

## 17. Release-binary startup acceptance

The post-fix release trace for PID 6324 establishes the corrected ordering:

```text
state construction end       344 ms
dispatcher ready             345 ms
stdio read begins            345 ms
Clean initialize received    346 ms
Clean initialize flushed     383 ms
CBM spawn begins             388 ms (cbm-admission thread)
CBM admission complete     8,955 ms
```

Core readiness therefore preceded optional-provider launch, and the roughly
8.6-second provider admission did not consume the host connection window. A
read-only process check during the connected session found one release
`clean-ctx.exe` (PID 6324). Two CBM processes were present: the directly owned
frontend and the provider's shared process; this is not the duplicate-Clean
defect.

The saved `processes-after-connect.json` predates PID 6324 and contains zero
processes, so it is not accepted as connected-state evidence. The Codex MCP
settings toggle was disabled and the reconnect scenario could not be executed.
Reconnect duplication is therefore **NOT RUN**, not inferred green. Initial
startup readiness is **VERIFIED**; the prior host-exit snapshot contained zero
Clean processes, while shutdown for the current traced session remains covered
by deterministic ownership regressions rather than a matching post-exit trace.
