# CBM subprocess, session, daemon and pipe-transport reconciliation investigation

Date: 2026-10-11. Investigator: Agent MaxHeadRoom.

## 1. Executive Verdict

The supported integration remains a spawned CBM MCP frontend over private
stdin/stdout pipes. There is no evidence requiring direct daemon IPC, embedding,
SQLite access, a second coordinator or a Clean-CTX-owned daemon.

However, current Clean-CTX does not fully implement that boundary correctly:
response correlation is absent; oversized responses leave a reusable transport
in an uncertain state; retries after killing the frontend reuse the dead child;
launch does not bind the frontend CWD to the resolved Clean-CTX root; availability
is assigned before admission/initialization; daemon freshness differs from native
projection truth. These are localized adapter/lifecycle reconciliation needs.

The statement “Clean-CTX owns its CBM frontend/session process and transport,
but does not own the CBM daemon or global CBM runtime” is **PROVEN** as the
resource boundary in current source. Clean-CTX's implementation does not always
respect the operational consequences of that distinction.

The old private-cache recommendation is not safe as a general default. Current
CBM requires one account-wide active compatible cohort, including canonical
cache root. This was already enforced at the inspected 0.10.8 comparison anchor;
the previous documentation's “one version per cache root” was misleading then,
not an entirely new 0.11 restriction.

Final categorical decisions appear at the end. No production changes or live
verification occurred.

## 2. Scope and Evidence Rules

Pinned CBM source: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`, fork
`codeliftsleep2/codebase-memory-mcp-clean-ctx`. Current Clean-CTX inspected at
`4c1627fcefe698977447dcee4e88830c20c7f422`, branch
`(feat)Semantic_Intelligence_Architecture_Boundaries`.
Historical Clean-CTX migration: `c899507678f7188e5fd161625bf92acdc2697d4c`
on `feature/update-CBM-0.10.8`. Historical CBM comparison anchor `5821078e` is
packaging 0.10.8, not a recovered exact historical deployed binary SHA.

Evidence labels: **PROVEN BY CURRENT CODE**, **PROVEN BY EXISTING TEST**,
**PROVEN BY DOCUMENTED CONTRACT**, **PROVEN BY LIVE/REPRODUCED BEHAVIOR**,
**HISTORICAL — NO LONGER CURRENT**, **INFERENCE**, **PROPOSAL**,
**NOT ESTABLISHED**. Existing-test evidence means inspected assertions, not a
run or newly verified pass. There is no LIVE/REPRODUCED evidence in this report.

Only this report was created for this task. The earlier authority reconciliation
report was already present and remains unchanged. No Cargo, CBM executable,
server, build, test runner, activation, daemon control command or installer was
started. Historical source was read from separate inspection repositories;
no source branch was merged, rebased or changed.

## 3. Previous Clean-CTX Assumptions

The two migration documents are absent from current Clean-CTX and were inspected
at the historical branch: `docs/CBM_MIGRATION_DECISION.md` and
`docs/CBM_INTEGRATION_BLUEPRINT.md`. Current
`docs/plans/CBM_INTEGRATION_PLAN.md` contains older pipe/proxy plans.
Later evidence: `CURRENT_CBM_AND_CLEAN_CTX_AUTHORITY_RECONCILIATION.md` and
`QUERY_HISTORY_INDEPENDENCE_AND_DISCOVERY_COMPLETENESS_INVESTIGATION.md`.

**HISTORICAL — NO LONGER CURRENT:** old migration §6.2 said:

> Pin `CBM_CACHE_DIR` in the spawn environment to a stable, Clean-CTX-managed
> canonical root ... so project slugs and persistence are deterministic and do
> not collide with another tool's CBM cache.

It also said “one CBM version per cache root,” treated daemon conflicts as
retryable, proposed separate startup coordination timeout, and summarized
first-session startup / last-session daemon shutdown. Its same §6.1 correctly
noted that active processes must agree on canonical cache root.

These statements are internally in tension: an independent root is not an
independent account-level runtime namespace. The historical branch implements
an optional cache-root spawn override and textual classify_startup_failure;
current Clean-CTX has neither. Do not ascribe those old implementations to
current main-derived production code.

The current integration still owns a Child and raw pipes, discovers a binary,
marks it available after spawn, starts root indexing, retries transport failures
and kills the child at timeout/drop. Whether these mechanisms are sufficient
must be established from code, not the old document's status.

## 4. Current CBM Process Topology

**PROVEN BY CURRENT CODE:** no-argument invocation selects the MCP client role
in `src/main.c`. It captures executable identity, canonicalizes cache root,
enters the account-wide version cohort, bootstraps/joins a daemon, registers a
client and installs session context before running
`cbm_daemon_frontend_mcp_run(..., stdin, stdout)`.

```text
Clean-CTX MCP server
  -> spawned CBM MCP frontend (private child pipes)
     -> authenticated account-local daemon IPC
        -> shared daemon application session
           -> project store / project mutation locks
           -> supervised index jobs / shared watcher / background services
     <- JSON-RPC application reply
  <- frontend stdout JSON-RPC frame
  -> parsed bridge use or raw interception/compression
  -> Clean-CTX model-facing MCP response
```

`src/daemon/service.*` supplies the stable rendezvous and compatibility policy;
`version_cohort.*` guards process lifetimes; `bootstrap.*` launches/reuses the
detached daemon; `runtime.*` authenticates/registers clients and transports
application requests; `application.*` owns per-session MCP server context and
shared jobs/watches; `host.*` owns daemon services/logs.

The child is a session frontend, not the index worker or persistent graph owner.
A permanent daemon started through daemon controls differs from a session-managed
generation. Current CLI execution is also nuanced: ordinary one-shot tools may
use a local server, but indexing is daemon-supervised in current `main.c`; the
old blanket “only CLI is daemon-free” statement must not be extended to all
current indexing/control paths.

## 5. End-to-End Production Call Path

**PROVEN BY CURRENT CODE (Clean-CTX):**
`McpState::try_init_graph_bridge` →
`GraphBridge::try_create_with_roots` in `bridge/lifecycle.rs` →
`bridge/binary.rs::resolve_cbm_binary` → `CbmClient::try_launch`.
It passes no CBM arguments, no explicit child CWD and no environment override.
The child inherits the parent's environment/CWD; stdin/stdout/stderr are piped.

Launch takes the pipe handles; a stdout thread reads lines into a bounded
256-item channel; a stderr thread prints `[cbm-stderr]` lines to Clean-CTX stderr.
Both thread-spawn failures are converted to None with `.ok()`, rather than
failing launch. Child spawn success marks availability. No initialize/tools-list
handshake or admission validation occurs in this client launch route.
`start_indexing_roots` then submits fast indexing work using the shared client.

**PROVEN BY CURRENT CODE (CBM):** `main_build_identity` and cohort acquisition →
`main_client_bootstrap_with_upgrade` → runtime HELLO/session →
`main_set_client_context(..., NULL, ...)` → frontend stdin reader → queue/FIFO
worker → `cbm_daemon_application_client_mcp_tagged` → runtime application callback
→ `application_mcp_request` → `cbm_mcp_server_handle` → tool dispatcher.
Indexing routes through the daemon index executor/supervisor, with project
mutation guards; graph reads use the selected project store.

Reply: application enforces its IPC payload maximum → private IPC response →
frontend_write_response → stdout line → Clean-CTX channel →
`send_and_receive_raw`.
Parsed path: call_tool_inner parses RPC/tool error then returns result to bridge.
Raw path: `GraphBridge::proxy_call` in `bridge/cache.rs` → call_tool_raw →
`handle_cbm_proxy` → error enhancement / caller verification →
`compress_cbm_response` → Clean-CTX response with caller's Clean-CTX request ID.
The outer Clean-CTX ID and internal CBM request ID are different authorities.

CBM accepts direct tools/call without an enforced initialize gate in the
inspected dispatcher, so omission is not proven to make every call fail.
Nevertheless successful initialize triggers background initialization in
application_mcp_request; current Clean-CTX does not trigger that lifecycle.
MCP-conformant initialization and current capability discovery remain a small
adapter reconciliation candidate, not a reason to replace transport.

## 6. Ownership Matrix

| Resource | Actual owner | What Clean-CTX owns |
| --- | --- | --- |
| Frontend stdin | Child reads; Clean-CTX writes private pipe | Writer and requests |
| Frontend stdout | Child writes JSON-RPC | Reader, framing/correlation/interception |
| Frontend stderr | Frontend diagnostics | Drain and diagnostic forwarding, not daemon-state authority |
| Frontend process | Clean-CTX child; CBM parent watchdog/session machinery | Spawn/terminate/reap its child |
| Daemon IPC/session | CBM frontend/runtime | Indirect session lifetime by owning frontend |
| Index worker/process tree | CBM supervisor/application | Request/cancel intent, not global worker ownership |
| Watcher registrations | Daemon, reference-counted session subscriptions | Its session's indirect subscription only |
| Persistent graph/cache files | CBM project store | No direct storage ownership |
| Background jobs | CBM application/coordinator | Its request subscriptions; other subscribers can remain |
| Project mutation/native locks | CBM daemon/project_lock | No direct lock ownership |
| Daemon operation/conflict logs | CBM private log paths | Optional operator diagnostics, not required log parser |
| Session cancellation | Runtime disconnect/cancel + application cleanup | Close/kill frontend or protocol cancellation, not daemon shutdown |
| Clean-CTX graph-result cache | GraphBridge/GraphCacheStore | Own cache keys/lifetime/freshness interpretation |

Clean-CTX graph cache_scope/cache_db_path are its result-cache configuration,
not proof of a CBM_CACHE_DIR override. They must not be conflated.

## 7. stdin/stdout Transport Findings

**PROVEN BY CURRENT CODE:** CBM `cbm_mcp_read_message` accepts newline JSON or
Content-Length framing. `frontend_write_response` mirrors the input framing:
newline input produces JSON bytes plus newline. Clean-CTX serializes compact JSON
and writes one newline. This specific framing pairing remains valid. Tree text
is inside JSON content strings; escaped newlines are not raw protocol boundaries.
Clean-CTX need not implement Content-Length merely because CBM supports it.

CBM normal application output is serialized JSON-RPC, not daemon IPC frames.
Detached daemon spawn does not inherit frontend stdio: POSIX redirects to
/dev/null and closes descriptors; Windows CreateProcess disables handle
inheritance; Darwin uses spawn file actions. Daemon logs have their own sink.
Thus daemon diagnostics do not ordinarily contaminate frontend stdout.

Startup can emit a protocol-clean JSON-RPC error with id:null via
main_report_client_failure, before reading requests. Not every startup exit uses
that helper: the inspected cohort-conflict path, rejected context, UI setup and
parent-watchdog failure have stderr-only exits. The comment claiming universal
startup explanations does not establish a universal stdout guarantee.

| Question | Finding |
| --- | --- |
| stdin/stdout still supported? | Yes, explicit production frontend route |
| stdout clean? | Protocol output in inspected MCP routes; user-selected executable/shim behavior and every failure path remain live concerns |
| Startup/index/watch text on stdout? | Startup error may be JSON-RPC; ordinary diagnostics use stderr/log sink. Help/CLI commands are different roles |
| Responses line-delimited? | Yes for Clean-CTX's newline request mode |
| Notifications? | Client notifications, especially exact cancellation, are supported and have no reply. No general unsolicited server-notification stream found in inspected frontend/application path; update notices are injected into tool content |
| Multiple outstanding calls? | CBM queues pipelined requests (8 items / 12 MiB queued bytes with backpressure); one FIFO application worker. Clean-CTX mutex and &mut client serialize its own calls |
| ID matching? | CBM preserves request IDs. Clean-CTX allocates IDs but does not compare jsonrpc/id to the pending request on either parsed or raw receive |
| Accumulate until parses? | Works for normal serialized newline replies; JSON parse success alone does not establish RPC identity, message role or frame ownership |

**PROVEN BY CURRENT CODE:** Clean-CTX caps accumulated response at 4 MiB, but the
stdout reader first allocates an unbounded line before the main thread checks
that cap. The channel item bound is not a byte bound. CBM daemon frame limit is
10 MiB; application payload maximum is that minus 16 bytes. Query output budget
can reach 1,000,000 estimated tokens at four bytes each; outer JSON escaping and
envelope add size. Other controls/projections must also be checked. Therefore
4 MiB is a local supported-result restriction, not a proven upper bound on all
legal provider responses. Realistic exceedance frequency is NOT ESTABLISHED.

On oversize, current Clean-CTX returns ConnectionLost but does not close/retire
the child or reset framing. Retry resends on the same stream. On malformed or
unexpected JSON, recovery/identity is likewise not established. Preserve the
existing local cap unless its change is authorized; reconcile bounded reading,
explicit failure and retiring uncertain streams instead of silently raising it.
A valid unmatched/null-ID error must be treated as lifecycle/protocol evidence,
not an ordinary successful response to the current query.

## 8. Pipe-Level Interception Findings

**PROVEN BY CURRENT CODE:** parsed and raw calls share send_and_receive_raw.
The normal intercepted object is the complete frontend JSON-RPC response, without
IPC/session tokens. It is the same supported frontend result surface a direct
MCP client obtains for that request/context. Different session state, update
notices, project selection or parameters can naturally produce different replies;
this is not a claim of byte-identical replies across sessions.

Raw interception still makes sense for tree or JSON tool bodies. Tool wire-schema
compatibility is independent: current Clean-CTX typed search expects older
results arrays and omits explicit JSON; current provider defaults/tree and grouped
JSON require separate decoding work. That does not invalidate the stdout pipe.

Compression changes the model-facing representation; it is not a transparent MCP
forwarder. Current compress_cbm_response extracts content text when present,
otherwise serializes result, and captures top-level RPC error messages. Parsed
call_tool checks result.isError; raw call_tool_raw does not run that check and can
record transport success for a failed tool result. Proxy soft-error handling
covers a path, but it is not proof of identical classification on every raw use.
New structured fields, limits and notices require preservation verification
through compression. No fixed 5000→1100 token ratio is established by comments.

Verdict: keep pipe interception, repair transport ownership/correlation/retirement
and lifecycle semantics. No alternate integration architecture is justified.

## 9. stderr and Failure-Authority Findings

**PROVEN BY CURRENT CODE:** the frontend emits coordination/admission/refusal
and lifecycle diagnostics to stderr. Clean-CTX drains and forwards these; its
comment saying “into a log file” does not describe its actual eprintln route.
No stderr message is stored in current CbmClient as authoritative startup state.

Daemon operation logs use `<canonical cache>/logs/cbm-daemon.log` through host
log sink; detached bootstrap failures have cbm-daemon.start-failures.log and
bootstrap reporting. Conflict records can be appended to daemon-conflicts.ndjson;
activation controls open activation-events.ndjson in `src/cli/cli.c`. Recording
is an attempted diagnostic action, not a universal guarantee of durable success.
Not all daemon failures are reflected in frontend stderr: frontend_worker can
_Exit on IPC failure without logging, deliberately avoiding blocked stdio.

Historical classify_startup_failure matches “conflict”, “daemon”, “admission”,
“cache root”, “version mismatch”, “abi” and similar text. It is absent from current
production. “Daemon unavailable” and “daemon startup failed” are not proof of a
version/build conflict; substring classification overclaims. Current source also
has stderr-only cohort refusal, so structured stdout is not universally available.

**PROPOSAL:** retain stderr as diagnostic evidence. Prefer explicit startup/RPC
error evidence when available, child exit/EOF and known admission observations;
classify unproven causes as startup/connection failure with diagnostic context.
Do not infer exact daemon state or introduce an automatic durable-log parser.
Specific conflict classification needs evidence narrow enough to distinguish
admission from crash, maintenance, unsafe root, unavailable daemon and timeout.
Frontend stderr alone is not an authoritative general DaemonConflict oracle.

## 10. Cache-Root Reconciliation

**PROVEN BY CURRENT CODE:** main_build_identity resolves/canonicalizes the cache
directory, validates its private directory, sets CBM_CACHE_DIR to that canonical
path and hashes the path into cache_fingerprint. The account runtime endpoint
excludes cache root, executable path, version and build. version_cohort_acquire
compares cache fingerprint in addition to version/build/ABIs.

If another active client uses canonical root R and Clean-CTX starts with distinct
root S, the second admission waits/retries within the caller's deadline and can
return cache-root conflict; it cannot create a valid independent account cohort.
Aliases resolving to the same canonical directory match. Distinct physical roots
are not silently rewritten to the active root. Behavior is account-coordinated
for participating processes; CLI role differences do not justify bypassing the
cohort. Test-only alternate runtime namespaces are not production isolation.

**PROVEN BY EXISTING TEST:**
`version_cohort_rejects_exact_build_with_different_cache_root`,
`version_cohort_shares_exact_build_rejects_conflict_and_turns_over`, and deadline/
holder-leaves tests in `tests/test_version_cohort.c`.

Historical CBM anchor 5821078e already compares cache fingerprints in
version_cohort.c and has the same rendezvous exclusion in service.h. Current
history adds conflict retry/turnover behavior (`9104feb6`), not an independent
root namespace. The private-root advice was overbroad under the inspected old
contract too.

Old decision classification: **UNSAFE UNDER CURRENT CBM** when “private” means
an independent root alongside other active account participants. A stable
operator-selected root is valid only if all active participants agree.

Final cache policy recommendation: **USE CBM ACCOUNT CANONICAL CACHE ROOT**.
This means honor the account's agreed environment/configuration, not forcibly
choose the default or overwrite an operator's custom common root. Current
Clean-CTX already inherits CBM_CACHE_DIR rather than pinning it; this report
recommends against restoring the old independent override.

## 11. Binary / Version / ABI Reconciliation

**PROVEN BY CURRENT CODE:** CBM captures executable-byte fingerprint, semantic
version and protocol/store/feature ABI; cohort and HELLO admit compatible peers
at one account endpoint. A differently named path with identical compatible
bytes may work; equal semantic version with different build bytes is insufficient.
Different roots do not solve version/build conflict.

Current Clean-CTX selects config binary_path, then PATH, then common locations;
missing explicit path can fall through. Windows discovery includes exe/cmd/bat
candidates. It does not establish version/build/ABI admission at spawn;
cbm_min_version is a configuration field, not a production gate in this route.
Do not call this an independently enforced version pin or installer.

Can Clean-CTX use a pinned executable? Yes, **conditionally**: its identity/cache
must match active participants, or the account must have no conflicting lifetime
holders and allow that build to become the agreed cohort. Independently choosing
an incompatible parallel build is not safe. Smallest model: operator-selected
compatible account installation, with honest failed admission and native fallback.
No installer redesign is needed for this investigation.

Activation/update uses maintenance/admission/lifetime barriers and frontend
maintenance monitors. Running sessions can be drained/exited; a replacement
frontend must admit the new compatible build. main_client_bootstrap_with_upgrade
contains conditional newer-semver permanent-daemon drain logic, but it is downstream
of earlier cohort acquisition and cannot be assumed to bypass an active
incompatible cohort. Live reachability of each update combination is NOT
ESTABLISHED. Same-version different-build/dev conflicts are not automatic upgrades.

## 12. Shutdown / Timeout / Restart Semantics

**PROVEN BY CURRENT CODE:** CbmClient Drop kills its child and polls try_wait up
to five seconds. Timeout kills the child and performs try_wait, without waiting
for daemon cleanup acknowledgement. It does not stop or kill the daemon directly.
Both retry entry points retry once after 100 ms on ConnectionLost/Timeout/internal
RPC error; no relaunch occurs. Circuit cooldown resets counters/status, not the
Child or pipes. Bridge status synchronization is not process recovery.

**PROVEN BY CURRENT CODE (CBM):** kernel IPC disconnect reaches
runtime_worker_disconnect → session_cancel → application cancellation; runtime
joins/reaps request callbacks then session_close. Application cancellation
releases watch ownership and job subscriptions, marks active work cancelled and
prevents late watch recreation. Shared work can remain for other subscribers.
Specifically, application_index_start_async retains one application-owned
subscriber reference per async job until terminal publication. That job survives
request cancellation and originating-session close while a suitable daemon host
remains; final ephemeral-session exit or daemon shutdown can cancel it.
Closing one frontend is not a blanket guarantee that its initiated index
operation stopped.

| Event / resource | Actual consequence |
| --- | --- |
| Normal stdin EOF | Frontend drains accepted FIFO work with progress-based window, then closes session; distinct from hard kill |
| Drop/hard frontend kill | Kernel disconnect triggers daemon session cleanup; no frontend graceful acknowledgement |
| Query timeout | Same disconnect cleanup; operation may have committed before timeout, and other subscribers/shared ownership can continue |
| Other frontend remains | Daemon and remaining sessions survive; only departing session ownership released |
| Last session-managed frontend leaves | Ephemeral generation stops/cancels jobs; bootstrap linger can affect transitions |
| Permanent daemon loses last frontend | Remains until control/maintenance stop; watcher subscription for departed session still released |
| Persistent index/project registration | Store files survive normal session/daemon turnover; not removed merely by disconnect |
| Project locks | Provider releases through cancellation/reaping/native lifetime; release is not synchronous with Clean-CTX kill return |
| Daemon failure | Frontend IPC error may fail-stop without stderr; caches/source freshness must not be assumed current |
| Activation/maintenance | Intentional monitor cancellation/exit and generation turnover, not necessarily crash or incompatibility |
| Retry after dead child | Current Clean-CTX retries dead writer/receiver; no new admission occurs |

Killing only the frontend is a valid isolation operation: do not kill the shared
daemon or process tree as though private. It is insufficient as proof of completed
shared-work cancellation or as a recovery strategy. A new frontend must re-admit,
initialize/validate and query state. Mutating index calls cannot be replayed as
though timeout proves “never executed”; distinguish read retries from potentially
committed work. No new global retry budget or error taxonomy is authorized here.

**PROVEN BY EXISTING TEST:** runtime disconnect releases only connection
subscriptions, blocked non-index child cancellation preserves another session,
final ephemeral disconnect exits, permanent service survives; application tests
cover final-subscriber cancellation and exact-token detach; frontend tests cover
EOF/backpressure/maintenance. None was run during this audit.

## 13. Working Directory / Session Root Findings

**PROVEN BY CURRENT CODE:** CBM main_set_client_context with preferred root NULL
calls main_session_context, canonicalizing “.” before MCP input. CBM_ALLOWED_ROOT,
when set, is separately canonicalized as policy. Session context is installed
once via application_set_context. Initialize does not replace that context using
MCP roots; no roots/list exchange was found in this production route.

Clean-CTX inherits its process CWD for the child. Its own project discovery can
instead use CLEAN_CTX_PROJECT_ROOT, walk upward from CWD, or executable directory;
GraphBridge receives that resolved root plus additional roots. No CBM child
current_dir binding or parent set_current_dir reconciliation was found in the
inspected server/launch path. Therefore correct CBM session root for every
supported invocation is **NOT ESTABLISHED**, and explicit-root/CWD divergence is
possible from code. Relative configured binary paths also depend on parent CWD.

**PROPOSAL:** bind child CWD to the intended admitted primary project while
preserving explicit project tool routing and native additional-root authorization.
Do not confuse the separate HTTP/proxy-spawner working-directory policy in
mcp/server.rs with CBM frontend launch; they are different subprocesses.

## 14. Multi-Repository Findings

A single admitted frontend can query multiple indexed projects using explicit
project arguments. Provider application sessions switch/load selected graph
stores; Clean-CTX maintains canonical-root→slug and slug→root maps, derives
path-based slugs, starts indexing each configured root and applies project-specific
proxy gates. Client mutex serializes calls: no evidence requires one frontend per
query/project merely for transport.

But four sets can diverge:

| Set | Determined by | Consequence |
| --- | --- | --- |
| Frontend session root | Child launch CWD / installed context | Primary session and watch association |
| Indexed projects | Explicit repo_path/name indexing and existing stores | A/B/C can be queried if admitted and indexed |
| Watched projects | Active session-root subscriptions, settings, existing DB, registration success | Indexing or querying B does not automatically give this A session a B watch |
| Clean-CTX authorized scope | Native primary/additional roots and withinPath | Provider selection cannot widen native authority |

application_refresh_watch_locked uses session_project/session_root, auto-watch
policy and existing project DB, reference-counting shared roots. It does not
subscribe a session to every graph named by tool arguments. Watching can be
disabled/refused, collision-limited or unavailable; Git/non-Git strategy matters.
Other B/C-rooted sessions can maintain those watches. Explicit project arguments
are sufficient for routing, not freshness guarantees. Separate rooted frontends
might be an optional watch strategy only if that need is demonstrated; avoid
creating them merely because multiple repositories exist.

Cross-repo intelligence uses indexed target stores and can report partial/skipped
projects; it does not create a universal atomic live graph. Clean-CTX local dirty
tracking/lazy reindex does not observe every external client/editor mutation.
Native current source acquisition must remain authoritative.

## 15. CBM Freshness vs Clean-CTX Truth

| Concern | Owner / investigation boundary |
| --- | --- |
| Native candidate completeness, fidelity/hash adequacy, catalog timing | Clean-CTX WorkspaceIndex / #142 |
| Accepted graph sources, serving generation, watcher updates, job terminal state | CBM graph lifecycle |
| Pipe framing/IDs, child availability, disconnect, frontend admission/recovery | Subprocess/session adapter |
| Provider hints mistaken for complete current native truth | Cross-system composition |

Fresh vs warm daemon, unwatched vs watched project, or another frontend's indexing
can change provider advisory output. That is legitimate only with truthful
coverage/freshness, not as proof of complete core answers. Cached GraphBridge
results are TTL/project-owned and have local invalidation; external daemon
mutation does not automatically become a verified native projection generation.
CBM generation and metadata freshness cannot substitute for Clean-CTX source SHA.

The separate #142 audit remains open. Transport correction does not fix native
already-indexed skipping or batch catalog snapshots. Conversely native hydration
fixes cannot cure a dead child retry or wrong session root.

## 16. Previous Decisions That Remain Valid

| Identified old decision | Current evidence | Verdict |
| --- | --- | --- |
| Own frontend/session, not global daemon | main/frontend/runtime/application route | Retain |
| JSON-RPC stdin/stdout | Mirrored newline frame path | Retain |
| Background stdout reader | Current channel allows receive deadlines without blocking main read_line | Retain mechanism; strengthen byte-bound/correlation |
| stderr draining | Frontend has real diagnostics; pipe backpressure is possible | Retain; do not confuse forwarding with durable logging |
| Raw interception before compression | Frontend reply contains no daemon IPC framing | Retain boundary |
| Provider graph is advisory | Native compilation/source ownership independent | Retain |
| Degrade gracefully without provider | Spawn/bridge optional paths exist | Retain; no false successful empty result |
| Reuse existing daemon through executable | bootstrap/session admission is supported | Retain; no direct IPC attach needed |

## 17. Previous Decisions That Must Change

| Old/current assumption | Invalidating or qualifying current evidence | Smallest recommended correction |
| --- | --- | --- |
| Old §6.2 “pin ... Clean-CTX-managed canonical root ... do not collide” | Account rendezvous excludes cache; cohort rejects distinct roots, including old anchor | Use agreed account canonical root; private root only if shared account configuration deliberately agrees |
| Old “one CBM version per cache root” | Cohort is account-wide, checks exact build/ABI/root | Say one compatible active account cohort |
| Old “first starts, last stops” without qualification | permanent daemon and bootstrap linger exist | Distinguish session-managed/permanent/transition states |
| Old conflicts “retryable” via ordinary call loop | Active identity conflict needs holder departure/reconfiguration; current retry never re-admits | Honest admission diagnostics; only justified restart/retry with state change |
| Old substring classify_startup_failure | Generic daemon/ABI words do not prove conflict; current helper absent | Do not port broad matcher; prefer supported structured evidence and unknown failure state |
| Current spawn success means Available | Admission is asynchronous to parent spawn; launch has no handshake | Establish ready admitted client before authoritative availability |
| Current next parsed JSON is current response | CBM can emit startup id:null; correlation absent | Verify envelope/message role/ID; handle startup lifecycle error separately |
| Current oversize can retry on same stream | Receive cap < provider legal maximum; oversized path does not retire stream | Preserve local cap, explicitly fail and retire uncertain transport |
| Current timeout retry can recover child | Child killed, same client reused; cooldown only resets counters | Re-admit a replacement when justified, not reuse dead pipes |
| Inherited CWD is correct project root | Native root resolution can differ from process CWD | Explicit admitted launch root; preserve multiroot scope |
| Indexed additional roots stay watched | Watch subscription follows session root/settings, not every explicit project | Disclose freshness/perform justified refresh; do not imply watched |

These are recommendations, not implemented architectural policy changes.

## 18. Risks / Unknowns

- No live pinned executable/session was run. Actual platform startup timing,
  executable/shim output, parent job containment and failure messages remain
  unverified.
- No new response-correlation regression was executed. Lack of unsolicited
  notification emission in inspected routes narrows the ordinary mismatch risk;
  it does not justify JSON-only validation as a general contract.
- Legal large-response support exceeds the local receive restriction; actual
  workload incidence is unmeasured. Reader line allocation is not bounded by the
  4 MiB accumulator limit.
- Drainer/reader thread startup is silently optional; stderr forwarding can itself
  block if the host never drains Clean-CTX stderr. Absolute no-deadlock guarantee
  is not established.
- Exact durable conflict/activation records are not guaranteed under unsafe paths,
  log IO failure or crash. Automatic log parsing would add unjustified coupling.
- Some startup refusals lack JSON-RPC stdout despite broader comments. Do not claim
  all failures become structured errors.
- Crash/session reaping is provider-owned and may be asynchronous; kill return is
  not a lock-cleanup certificate.
- Conditional upgrade bootstrap versus prior lifetime admission needs a controlled
  test; no safe parallel incompatible-binary behavior is assumed.
- The current proxy compressor must be tested for retention of error/completeness
  fields. Supported transport does not prove supported result decoding.

## 19. Live Falsification Matrix

**PROPOSAL, unrun:** extend existing CBM PowerShell/MCP harness patterns under
verification, with evidence under target. Keep regression tests under src/tests.
Record pinned executable version/build, account cohort/cache, platform, launch
CWD, request IDs, full raw stdout/stderr and case-specific verdicts. Use isolated
fixtures; do not stop another user's daemon, mutate account installation, switch
shared cache roots or provoke activation without explicit operator scope.
Test-only runtime isolation requires a seam-enabled binary, not a production
CBM_CACHE_DIR trick.

| Case | Smallest stimulus | Required observation / falsifier |
| --- | --- | --- |
| Basic supported transport | Spawn in fixture root; initialize; initialized notification; tools/list; simple tools/call; close stdin | Matching IDs, usable result, clean EOF; launch alone not readiness |
| Current direct-call contrast | Separate frontend tools/call without initialize | Capture actual dispatcher acceptance and missing background initialization; no protocol claim inferred from permissiveness |
| Repeated calls | Three sequential different IDs | No crossing/stale reply; same session reusable |
| Tree/JSON content | Same narrow query with explicit tree then JSON | Outer one-line JSON envelope; escaped content newlines; body decoder separate |
| stderr pressure | Existing controlled producer fixture or provider diagnostic workload | Both pipes drain; host stderr also drained; no deadlock |
| Large response | Legal bounded query near/beyond 4 MiB with escaping | Provider bytes, local explicit cap failure and subsequent-stream state; no silent empty success |
| RPC identity | Inject wrong-ID/notification/startup-null-ID frames into tracked client fixture | No frame accepted as ordinary pending-query success |
| Daemon reuse | A starts, B joins same identity/root; A exits; B queries | Separate private pipes; B survives; disconnect owns only A subscriptions |
| Last-client/permanent | Controlled ephemeral generation and separately controlled permanent generation | Correct lifetime distinction; persistent store survives; cleanup eventually observed |
| Existing daemon | Start compatible frontend against operator-owned existing generation | Same admission, no unintended shutdown/config mutation |
| Admission mismatch | Safe isolated test account/seam: version/build/root/ABI alternatives | Exit/stdout/stderr/conflict record distinct; no independent cache cohort; unchanged active client |
| Timeout/restart | Controlled long request; terminate frontend only; admit replacement | Session cleanup / shared subscriber survival; next query works; no reuse of dead pipes or assumed no-commit |
| Daemon loss | Controlled isolated daemon failure | Frontend failure may have empty stderr; honest unavailable state; replacement recovery |
| CWD/root | Parent CWD A, native explicit root B; launch contrasts | Session root/watch status and explicit graph route independently recorded |
| Multi-project | A-rooted frontend indexes/queries B/C | Correct project rows, watch status separately checked, no scope widening |
| Concurrent sessions | Two rooted clients, different IDs/projects | No stdout/request crossing; shared-job cancellation affects only correct subscribers |
| Activation | Only in safe explicitly owned test lifecycle | Maintenance vs crash; old frontend exits; new admitted compatible build resumes |
| Composition | Fresh/warm/edited/stale provider with native current-source query | Advisory limits survive compression; core completed truth independent of provider warming |

Relevant inspected provider test authorities: tests/test_daemon_frontend.c
(cancellation, queue backpressure, EOF and maintenance); test_daemon_runtime.c
(session disconnect/other-session preservation/permanent lifetime);
test_daemon_application.c (context, shared watches, subscriber cancellation,
initialize coalescing); test_version_cohort.c (cache/build conflicts and turnover);
test_daemon_bootstrap.c, test_project_lock.c and activation tests. Current
Clean-CTX circuit/proxy/error/identity tests remain useful, but do not prove modern
startup admission or dead-child relaunch. No full suite is authorized locally.

## 20. Smallest Recommended Next Phase

**PROPOSAL:** a focused frontend-adapter reconciliation phase, preserving
CbmClient, GraphBridge, private pipes, background readers and compression.
First author tracked regressions using existing client fixtures for request-ID/
startup classification, oversized stream retirement, dead-child retry and
explicit launch root. Prepare the bounded existing-pattern operator harness
cases above. Obtain narrow run authorization under repository policy; no tests
or runtime commands are implicitly authorized by this design report.

Then propose implementation confined to admission/readiness, response validation,
transport retirement/re-admission and launch context. Preserve account cache/build
ownership and native fallback. Do not change global caps/timeouts/retry budgets,
new public error states or provider authority without the required architectural
approval. Tool-body schema/paging correction remains a distinct adapter concern;
#142 native discovery/refinement work remains separate.

### Required decision table

| Decision | Verdict | Evidence-backed reason |
| --- | --- | --- |
| Launch CBM as subprocess | KEEP | No-argument MCP frontend is the supported production entry point |
| JSON-RPC over stdin/stdout | KEEP WITH REVISIONS | Framing pairing works; validate envelope/ID and lifecycle errors |
| Background stdout reader | KEEP WITH REVISIONS | Receive deadlines useful; line allocation not byte-bounded and thread failure silently ignored |
| Pipe-level raw interception | KEEP WITH REVISIONS | Complete frontend envelope remains interceptable; ownership/correlation and failure-state handling need repair |
| stderr background drainer | KEEP WITH REVISIONS | Drain useful; startup failure should not be silently optional; stderr is diagnostic rather than authoritative daemon status |
| Child process represents entire CBM runtime | REMOVE | Child is a frontend; shared daemon/worker/store survive according to provider policy |
| Clean-CTX owns daemon lifecycle | REMOVE | Only session/process owned; no global stop/kill authority implied |
| Kill frontend on timeout | KEEP WITH REVISIONS | Valid session isolation; not complete shared-work cancellation; retire dead transport and do not blindly replay mutations |
| Private Clean-CTX CBM_CACHE_DIR | REPLACE | Agreed account canonical root, not independent parallel cohort |
| Independently pinned CBM binary | KEEP WITH REVISIONS | Only compatible account identity or coordinated turnover; semver alone insufficient |
| stderr substring daemon-conflict classification | REMOVE | Historical matcher absent current code; broad hints cannot distinguish failure causes |
| Inherited child working directory | REPLACE | Explicit native-root/CWD mismatch possible; bind intended admitted root |
| One frontend for multi-repo queries | KEEP WITH REVISIONS | Explicit project routing works; indexed set is not watched/fresh set |
| Existing retry/circuit-breaker semantics | KEEP WITH REVISIONS | Failure isolation useful; counter reset and same dead child retry cannot recover process |

Primary verdict: **PIPE TRANSPORT REQUIRES REVISION**.

Old cache-root decision: **USE CBM ACCOUNT CANONICAL CACHE ROOT**.

Current pipe-level interception mechanism: **PRESERVE WITH TRANSPORT CHANGES**.
