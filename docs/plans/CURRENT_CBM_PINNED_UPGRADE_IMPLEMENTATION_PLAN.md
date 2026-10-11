# Current pinned CBM upgrade implementation plan

Date: 2026-10-11. Owner: Agent MaxHeadRoom.
Status: original planning record. At authoring, no implementation or verification
had run. Current local checkpoints are [A0/A1](CBM_A0_A1_IMPLEMENTATION_EVIDENCE.md),
[A2](CBM_A2_IMPLEMENTATION_EVIDENCE.md), [A3](CBM_A3_IMPLEMENTATION_EVIDENCE.md)
and [A4](CBM_A4_IMPLEMENTATION_EVIDENCE.md). [A5](CBM_A5_ENVIRONMENT_RECOVERY_AND_ACCEPTANCE.md)
is accepted with declared live limitations; A0–A5 are frozen. The owner authorized
Phase B continuation; B0 is the first implementation slice. B3's collection-policy
approval point and Issue 142 remain separately gated. The historical design below
does not itself prove acceptance.

## 1. Executive implementation verdict

Implement one preservation-first migration in two ordered phases: **A**, trustworthy
frontend/session transport and lifecycle; **B**, faithful current tool/wire
consumption. Keep CbmClient, GraphBridge, subprocess MCP, private pipes,
background stdout reader, stderr drain, raw interception and compression.

Phase A can start from this design on the required child branch after explicit
implementation authorization. Phase B is planned but gated on Phase A evidence,
current-contract fixtures and approval of the bounded collection policy below.
No readiness, completeness or performance claim is earned by a planned test.

Do not merge/rebase the historical CBM branch. Do not implement #142 or Hybrid
LSP enrichment. After A+B acceptance, STOP production work and conduct only the
separately authorized fixed-source live comparison described in §35.

## 2. Pinned source baselines

| Input | Exact baseline / meaning |
| --- | --- |
| Implementation parent | `(feat)Semantic_Intelligence_Architecture_Boundaries` at `4c1627fcefe698977447dcee4e88830c20c7f422` |
| Supported CBM source | `82b1ec908f975354ceb4bac8a8bfc594942a9bc4` in codeliftsleep2/codebase-memory-mcp-clean-ctx |
| Historical reference only | `feature/update-CBM-0.10.8` at `c899507678f7188e5fd161625bf92acdc2697d4c` |
| Capability investigation | `CURRENT_CBM_AND_CLEAN_CTX_AUTHORITY_RECONCILIATION.md` |
| Transport investigation | `CBM_SUBPROCESS_SESSION_DAEMON_RECONCILIATION_INVESTIGATION.md` |
| Deferred acquisition investigation | `QUERY_HISTORY_INDEPENDENCE_AND_DISCOVERY_COMPLETENESS_INVESTIGATION.md` |

Current CBM packaging advertises 0.11.0. That string does not prove this source
revision or exact executable build. The plan pins source/contract expectations;
live acceptance must separately record the artifact and its build provenance.
Current Clean-CTX package is 0.9.3; version changes are a separate release decision.

Evidence in this plan: **INSPECTED CODE**, **INSPECTED TEST**, **INVESTIGATION
CONCLUSION**, **DESIGN**, **UNVERIFIED**. A referenced existing test was read,
not run for this task. Planned captures are not called live evidence.

## 3. Branch ancestry and integration strategy

Required parent is the completed semantic-boundary feature commit above, **not
main**. Proposed child: `feature/current-cbm-pinned-upgrade`. No branch was
created or switched in this planning task.

```text
main
  -> (feat)Semantic_Intelligence_Architecture_Boundaries @ 4c1627f...
     -> feature/current-cbm-pinned-upgrade
        -> reviewed Phase A + Phase B implementation and acceptance
     <- merge completed child into semantic-boundary feature
  <- later normal reviewed integration of combined feature into main
```

The child inherits every completed semantic/invocation change; do not reconstruct
those changes from main. Verify parent SHA before child creation. If parent
advances, reconcile ancestry explicitly rather than silently selecting main.
The two investigation documents and this plan are planning artifacts presently
outside committed baseline; carry/review them without changing the required code
ancestry. Do not confuse committing documents with changing the implementation
base. No wholesale historical cherry-pick, merge or rebase is recommended.
Rebasing the child onto main before parent integration risks bypassing or
recreating the frozen semantic boundary and violates the required topology.

## 4. Governing invariants

1. CBM is optional; native core correctness remains available without it.
2. Provider candidates/advice never become canonical semantic authority.
3. Acceleration changes acquisition cost, not access to native core evidence.
4. Warming does not strengthen authoritative completed core truth; intentional
   indexed-only/lower-bound contracts remain explicitly partial.
5. JSON parse success is insufficient: envelope, role and request ID correlate.
6. Clean-CTX owns frontend/process/pipes, not shared daemon or other sessions.
7. Uncertain/dead transport is retired, never reused as healthy.
8. Provider index freshness does not establish native source/projection freshness.

Do not promise this migration resolves known #142 violations. Preserve existing
native contracts and expose provider limits honestly; do not broaden public
completeness semantics or repair native discovery under an adapter change.

## 5. Existing production call path

**INSPECTED CODE:** McpState bridge initialization →
GraphBridge::try_create_with_roots → binary resolution → CbmClient::try_launch
(no CWD/environment binding or handshake) → background fast root indexing.
Client serializes tools/call with allocated numeric ID, writes/flushed stdin,
reads stdout channel until any complete JSON, then parsed/raw branches consume it.
Mutex serialization means one Clean-CTX call per frontend at a time.

Parsed path checks RPC and tool soft errors but drops paging in typed projections.
Raw path returns the envelope → proxy project routing/gate/trace identity →
caller verification → compression → model-facing Clean-CTX result.
Timeout kills child; retry resends on that same child. Oversize fails without
retiring stream; breaker cooldown resets counters/status without respawn.

Current typed search assumes results arrays; query keeps columns/rows only;
trace assumes older callers/callees arrays; machine calls omit explicit format.
The modern provider defaults to tree and uses grouped JSON on several tools.
Graph result caches contain legacy payloads scoped to project/root with TTL.

## 6. Target production call path

```text
existing native root/binary/config authority
  -> launch specification (resolved binary + admitted primary CWD)
  -> private frontend pipes, mandatory drains, bounded frames
  -> correlated initialize / initialized / tools-list validation
  -> admitted usable provider session (not necessarily indexed)
  -> project-explicit operation, replay class and shared deadline
  -> matching JSON-RPC result/error
  -> tool-specific explicit JSON decoder + provider limits/provenance
  -> bounded page consumption OR truthful partial/unavailable outcome
  -> existing bridge/proxy consumer and native verification
  -> faithful compressed optional result
```

Failure: retire frontend only → breaker-controlled replacement admission for
safe reads, or explicit unreplayed mutation failure → native fallback where core
operation supports it. No daemon ownership/IPC/store access is added.

## 7. Phase A transport/lifecycle architecture

Keep CbmClient as the owner of launch identity, breaker and frontend session.
Separate its reusable logical client from live process/pipe state internally:
`Option<FrontendSession>` plus immutable launch specification is sufficient.
FrontendSession owns Child, stdin writer, stdout receiver and thread handles.
None means no viable transport; counters cannot manufacture a session.
Use direct private types/modules, not a transport trait/provider registry.

src/cbm/client.rs is currently 807 lines. Editing activates the 615-line ceiling:
first extract semantic units mechanically, preserving existing tests and call
surface. Suggested modules `client/transport.rs`, `client/protocol.rs`,
`client/lifecycle.rs` and `client/wire.rs`; keep error/compatibility API in
client.rs. Final decomposition follows real responsibilities, not line arithmetic.
Other active files must also remain under ceiling; do not broaden decomposition.

A logical state sequence spawned → initializing → usable is private; only usable
maps to public Available. Existing Unavailable/Degraded cover failures unless a
new externally visible state is demonstrably necessary and approved. Construction,
propagation, owner lifetime, consumer and MCP exposure must all be traced.

## 8. Request/response correlation design

Smallest representation: a private per-frame classification, with no general RPC
framework and no backlog for unrelated requests. Allocation remains monotonic
numeric u64 per logical client; replacement sessions continue allocation to avoid
confusing old diagnostics with a new request. Validate version exactly “2.0”,
object envelope, method/ID role and exclusive result/error.

| Frame | Validation / action |
| --- | --- |
| Matching response | No method; id is exactly pending numeric integer, not string/float/null; exactly one of result or non-null well-formed error. Complete only this request |
| Other response | Valid nonmatching ID; no legitimate second outstanding call exists. Protocol failure, retire; never stash or accept |
| Notification | No id member, string method and supported params shape; do not complete request. Syntactically valid notification ignored with diagnostics, with no unsupported notification-side effects; all waits retain original deadline |
| Server request | method + non-null id; unsupported client capability. Fail/retire; do not fabricate a response framework |
| Startup/lifecycle error | id:null, no method, valid error; report startup/lifecycle failure, retire. Not ordinary matching success and not automatically a DaemonConflict |
| Malformed/unsupported | Non-object, invalid version/ID, result+error, missing both, malformed error, unsupported framing, invalid UTF-8: explicit protocol failure and retire |
| EOF before reply | Process/transport failure, retire; no empty result |

Validate error code/message; optional data retained as provider diagnostics.
Matched RPC error preserves provider cause. Tool result.isError is a second
classification after successful RPC correlation, shared by parsed/raw bookkeeping.
Raw proxy can preserve a correlated error envelope for truthful presentation,
but breaker bookkeeping must not record provider failure as successful work.

Newline framing remains the chosen contract: compact JSON request + newline,
provider mirrored newline reply. Read bounded bytes before allocating a line;
strict UTF-8 then JSON. Existing 4 MiB receive cap remains; tree content newlines
are escaped inside envelope. No Content-Length support or pretty-JSON accumulation
is necessary for this exact frontend mode. Mismatched framing is failure.

## 9. Admission/readiness design

Perform once per new session, not once per Clean-CTX request:

1. Launch with explicit primary root; successfully install mandatory reader/drainer.
2. Send initialize with existing interoperable protocolVersion `2024-11-05`,
   tools-capable client identity and no advertised unsupported client features.
3. Require matching success, supported negotiated version, expected server identity
   and capabilities. Current pinned code supports this protocol version.
4. Send notifications/initialized (no response expected).
5. tools/list with correlated IDs; follow nextCursor if present within the same
   startup attempt/deadline, validating required consumed tools and schemas.
6. Only then publish usable Available and start existing explicit-root indexing.

No extra daemon-status command or daemon log scan is needed: a successful
frontend initialize proves admitted session/transport. It does not prove full
index coverage. Required tools: six existing proxy tools plus detect_changes
if its current consumer remains, and Phase B internal status/coverage tools when
that phase enables them. Compare necessary request capabilities, not an exact
all-tool list; unused new tools need no implementation.

Use existing query_timeout_ms as a bounded startup-attempt ceiling for this phase;
do not invent a separate global timeout. Report startup timeout distinctly in
internal diagnostic context, without a new public schema by default. Launch/
admission can run on the existing initialization worker pattern so ordinary
native requests do not wait for global provider indexing. Until usable, bridge
reports unavailable/degraded and native operations retain existing fallback.
Audit locks: do not hold global MCP/WorkspaceIndex locks during admission.

Initialize triggers CBM's own background behavior in pinned application code.
Record it; do not add a second eager global synchronization barrier. If existing
root indexing duplicates provider initialization, coalesce only after evidence;
do not silently change indexing mode or policy during Phase A.

## 10. Transport retirement/re-admission design

One operation retires on timeout, EOF/process death, invalid/mismatched envelope,
invalid framing/UTF-8, oversize, pipe write failure, or mandatory thread failure.
Take live session out of Option first; close/drop own pipes, kill if necessary,
reap under existing shutdown bound, and clean thread ownership. Never kill shared
daemon/worker descendants as though private. Capture original error before cleanup.

The reader must stop on receiver disappearance/EOF and never retain unlimited
lines. Preserve queue capacity 256; cap queued bytes as well using the existing
4 MiB receive bound as the proposed local ownership budget. This is a concrete
bound interpretation requiring implementation review, not an authorized cap
increase. Prefer one pending frame handoff over a general asynchronous queue.
No transport-synchronization uncertainty can be repaired by clearing counters.

Writes/flush must also participate in the attempt deadline: synchronous write
can block before current recv timeout begins. Use a contained writer worker with
request receipt/error handoff if existing request path cannot be bounded; retire
and close only frontend handles on expiry. This is transport ownership, not a
second RPC architecture. Test blocked write and cleanup on Windows/POSIX before
claiming bounded attempts.

Replacement reuses the same resolved launch spec/account environment and must
repeat admission. No source/index cache receives an invented fresh generation
from reconnect. Provider caches from prior session are not evidence of newly
admitted/indexed readiness.

## 11. Retry/circuit-breaker reconciliation

Preserve one retry, existing failure threshold and cooldown. Count failed attempts
explicitly; a successful admitted correlated operation resets failure count.
Cooldown permits a half-open attempt, not a premature Available assignment.

| Failure / operation | Same-session retry? | Replacement / replay rule |
| --- | --- | --- |
| Matching -32603 on known read-only tool | At most current one retry if transport healthy | No mandatory replacement; one retry total |
| Timeout/EOF/broken pipe/retired transport on read | Never reuse session | At most one replacement+admission+read retry; no nested retry multiplication |
| Wrong ID/malformed frame/oversize | No | Return explicit failure; retire. Later half-open/new request may re-admit; do not auto-repeat the same oversize/protocol defect |
| Admission identity/config/root mismatch | No | No immediate destructive correction or automatic cohort turnover; report cause/native fallback |
| Tool semantic failure / method missing / bad params | No | Preserve failure; do not recover by empty set |
| Mutation with uncertain outcome | No | Never replay automatically; next explicit operator request may inspect status |

Read allowlist: search_graph, read-only query_graph, trace_path,
get_architecture, list_projects, index_status, check_index_coverage and read
scope of detect_changes. Derived helpers inherit their underlying read class.
Index_repository with indexing/cross-repo work is mutating; async startup/join is
also mutating. Its proven status:true form is a status read only when arguments
are validated. initialize is fresh-session lifecycle setup, not replay of a tool.
Unknown raw tool is non-replayable; keep proxy whitelist unchanged.

No automatic replay of index jobs after frontend replacement. Preserve/report
unknown committed outcome, reconcile via supported status before a new explicit
index operation. Current circuit failure isolation remains local to CBM.

## 12. Cache-root migration

Inherit CBM_CACHE_DIR unchanged, or omit override when it is unset so CBM resolves
its account default. CBM canonicalizes/validates the directory and performs cohort
admission. Clean-CTX does not create a second cache-root authority or private cohort.

Current CbmConfig has no cache_root field. cache_scope/cache_db_path configure
Clean-CTX result cache, not provider storage. Keep them unchanged. Historical
branch cache_root override is rejected. Audit actual config deserialization for
unknown legacy fields: document any previously ignored setting rather than
silently adopting it. If users deliberately inject CBM_CACHE_DIR into the parent
environment, honor it and explain compatibility with other account participants.
Do not silently move/delete graph files or rewrite explicit settings. No installer
or automatic migration of provider persistent data is in scope.

## 13. Version/build compatibility

Source pin is `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`. Record it in adapter
contract fixtures and operator evidence metadata. Require known compatible server
identity/version/schema at initialize/tools-list. Do not claim those fields prove
source SHA or executable fingerprint: initialize exposes version, not the Git
revision or all cohort identity values.

For pinned live acceptance, operator supplies a build/release artifact with
traceable provenance to this SHA; record executable path, bytes hash and build
metadata. Different platform artifacts have different hashes. Do not invent a
universal checksum or silently add an installer. Without provenance, mark the
run compatibility-only, not pinned acceptance.

CBM owns exact byte/ABI/root comparison. An incompatible active A can refuse
frontend B: retain native Clean-CTX, useful RPC/exit/diagnostic cause and no hidden
cache fork. Do not run daemon stop/update or change account installation. Pinned
main has conditional newer permanent-daemon upgrade behavior downstream of
cohort admission; prove intended deployment does not retire unrelated sessions.
If launching the supported binary can cause such takeover in intended conditions,
stop Phase A acceptance and seek deployment/provider revision rather than patching
around it through direct IPC. Live mismatch/control tests require an isolated
operator-approved account or seam-enabled fixture.

## 14. stderr behavior

Mandatory background drain; fail launch if thread installation fails. Continue
existing stderr forwarding, ensure host/harness also drains it. Treat messages
as diagnostics, not authoritative daemon-state classification. Preserve primary
protocol/exit cause; optionally retain bounded context only under a reviewed
existing-bound mechanism, never unlimited strings/log reads.

Null-ID JSON-RPC startup error and matching error give structured evidence where
emitted. Pinned cohort/context/watchdog paths can still exit stderr-only; no broad
“daemon/ABI/conflict” matcher is ported. General startup/connection failure with
context is correct when exact cause is unproven. Do not parse daemon-conflicts.ndjson
or activation-events.ndjson in production. Operators may inspect them during
controlled verification, distinguishing missing record from successful logging.

## 15. Explicit launch-root design

Use the canonical primary root already selected by McpState/GraphBridge and
passed to try_create_with_roots. Pass it in immutable launch spec; set child
current_dir explicitly. Resolve configured binary to an absolute executable
before changing child CWD; otherwise a relative path can select a different binary.
Preserve existing native root/path authorization and configured override behavior;
no new independent root discovery.

withinPath constrains native query scope, not frontend launch root. Additional
roots remain explicit admitted project mappings; querying B/C does not turn A
session into B/C-rooted session or watched subscription. Do not spawn per-query
frontends. Nonexistent primary root is a provider launch failure/native fallback,
not a reason to choose another root heuristically. Audit fallback binary selection
and Windows shims; unsupported shim launch should fail explicitly, not silently
relax identity. Fixing discovery/installer policy beyond this may require approval.

## 16. Frontend/daemon ownership

Normal exit closes session; clean stdin EOF can allow queued work to drain.
Drop/retirement may kill/reap own frontend under current bound. No daemon global
stop, kill-tree, cache deletion or activation is added. Another frontend or a
permanent daemon can keep shared runtime alive. Last ephemeral session may stop
it according to CBM; that is provider policy, not ownership transferred to us.

Async jobs can survive originating-session cancellation while suitable daemon
host remains. Timeout does not prove a mutation never ran. Locks/subscriptions
are cleaned by provider runtime asynchronously. Re-admission does not claim
cancellation acknowledgement. Maintenance/update frontend exits are diagnosable
unavailability, not automatically crashes or version conflicts. All cleanup
paths must trace creation → propagation → ownership → cancellation → reaping.

## 17. Phase B tool/wire migration

Only consumed capabilities migrate. Existing proxy whitelist stays six tools;
status/coverage may be internal adapter calls, not new public tools. Inspect
actual code consumers before deleting Cypher helpers; unlike the historical
claim of no consumers, current bridge/query.rs contains active derived paths.

| Tool / consumers | Legacy request/response | Pinned request/response and migration |
| --- | --- | --- |
| search_graph: typed graph_search, hydration/project search, caller candidates | name_pattern/project/label; expects results array, no format | Explicit json, retain filters/limit/offset; grouped cols/rows with QN/file/lines and metadata. Normalize by column names, validate grouping; do not infer missing file/QN |
| query_graph: graph_query, inbound paths, importance/dead-code/blast-radius/call helpers | query/project; columns/rows only | Explicit json; columns/rows plus returned,total,total_relation,has_more,truncated,reason,next_cursor/next_offset. Preserve projection semantics/aliases and string cells |
| trace_path: graph_trace/proxy/trace source resolution | function/direction/depth/project; legacy caller/callee arrays | Explicit json for typed use; grouped callers/callees tables, directional totals/relations, truncation and next cursor. Retain canonical source/QNs; no Hybrid binding consumption |
| get_architecture: typed overview/proxy | project; assumes older overview | Explicit json with only consumed aspects; current aspect/grouped output. Preserve unknown aspect limits; no new architecture feature inventory |
| list_projects: proxy/project selection | args often empty; older identity list | Explicit json for machine parsing; identity vs stats, limit/offset,total/has_more. Validate root/identity; route by actual project, not basename |
| index_repository: startup/lazy/manual/proxy | repo_path/mode=fast, treats successful call as local completion | Preserve mode initially; current synchronous/async/status and error outcomes, terminal jobs separate from serving index. No blind async conversion/global index barrier |
| detect_changes: bridge detect_changes | project; looks for graph_version | Current diff/impact snapshot and generation-bound cursors; graph_version not found in inspected handler. Remove unsupported freshness inference, preserve optional None/diagnostic instead of inventing version |
| index_status: new internal truth preservation | Not consumed reliably | project/format=json; serving readiness/count/root/watch/coverage best-effort; missing/unknown states retained |
| check_index_coverage: bounded internal diagnostics | Not current required path | project + exact paths/scopes + independent offsets, format=json; recording/generation/freshness/partial flags; clean not completeness |

All tools retain RPC-error vs result.isError distinction. Some soft failures are
plain text, others JSON text; decoder must not assume every failure body is JSON.
Unused get_code_snippet/get_file_outline/compare_graphs, document retrieval and
Hybrid resolution do not migrate just because they exist.

## 18. Explicit format handling

Machine-parsed search/query/trace/architecture/list/status/coverage/diff requests
set format=json where supported. index_repository/status job surface uses its
actual fixed format; do not invent a format parameter. Decode outer MCP content
then tool body; reject missing/wrong required tables instead of empty success.
Preserve false, zero and explicit empty arrays as meaningful values.

Raw proxy preserves intentional caller format/default tree. Its envelope still
passes A correlation/error checks. Do not force JSON globally on human output or
interpret raw tree as a typed search result. Grouped JSON normalization must use
pinned schema/column fields; no ad hoc split on whitespace or QN prefix guessing.

## 19. Paging/continuation

Use tool-specific continuation; never one generic “cursor means complete” rule.

| Surface | Continuation / limitation |
| --- | --- |
| query_graph | Opaque next_cursor binds query/project/graph, store generation and result digest. Keep semantic args fixed; preserve columns across pages. Never silently fall back to live offset on stale cursor |
| trace_path | Opaque next/next_cursor with traversal identity; directional totals and engine ceiling retained. Follow only supported continuation; no invented nearest-row completion |
| structural/BM25 search | offset/next_offset and saturation/budget metadata; offsets do not earn snapshot binding. Full offset consumption still not complete source-universe proof |
| semantic search | Separate semantic_offset/limit/totals; currently unused optional expansion remains out of scope |
| list_projects | Stable ordered offset page; no global atomic multi-project generation implied |
| check_index_coverage | Independent path/scope offsets over explicit supplied sets; no unauthorized whole-workspace scan |
| detect_changes | Existing current generation-bound snapshot/cursor only if needed; never treat it as native source hash |

Proposed collection rule for internal consumers requiring a full provider result:
no unbounded loop; use existing query_timeout_ms as total logical collection
ceiling and existing 4 MiB as maximum aggregate retained encoded payload, plus
per-frame cap. Use provider default page sizes; deduplicate identities only where
existing consumer semantics allow, retain deterministic provider order. Stop on
budget/deadline, engine/window saturation, nonprogress/repeated token, missing
continuation, stale snapshot or mid-page error. Disclose partial/failure; do not
raise output budget automatically or restart from page one without remaining
attempt budget. Exact graph completion requires terminal indication, eq total
and no unresolved truncation. All-source completeness is still unearned.

**Approval point:** an aggregate collection ceiling/shared logical deadline is
new cross-call adapter policy, though derived from existing numbers. Approve it
before B3 implementation; this planning task does not authorize enforcement.
Alternative is one-page typed outcome with truthful partial metadata and explicit
caller continuation. If existing public contracts cannot carry that truth, stop
and review contract change rather than drop metadata. Do not add arbitrary page
count limits or silently fetch the provider engine maximum.

Raw proxy returns one requested page, retaining continuation; it must not
implicitly crawl all results. Cursor bytes are opaque and never parsed to access
provider storage. Missing response fields must be incompatible/unknown, not eq.

## 20. Readiness/index lifecycle mapping

Keep transport usable separate from provider project evidence. Add small private
project observation carrying proven job state, serving-index status and watch/
coverage diagnostics. Do not replace native semantic publication/cache authority.

| Provider observation | Adapter meaning |
| --- | --- |
| queued / running | Job not terminal; prior serving graph may exist; not fresh completed acquisition |
| succeeded | That index job/mode succeeded; coverage still needed for stronger indexed-scope claims |
| failed / cancelled | Preserve failure/cancel cause, prior serving-index preservation and generation where supplied |
| No job / unknown project / no index | Not-indexed or unknown as proven by wire; do not invent job terminal state |
| Unsupported status provider | Unknown/unavailable capability; never assume Ready |
| Metadata changed / missing / not tracked | Freshness diagnostic; not native source/projection decision |
| Metadata match | mtime+size match only, not SHA equality |
| Watch enabled/registered/disabled/unavailable | Independent observation, not job state or freshness certificate |

A successful search does not overwrite weaker project state. A synchronous index
reply is decoded against actual current success/failure contract, not any JSON.
Async jobs are polled only when explicitly requested under existing operation
ownership; no perpetual per-request polling. Status reads may be cached as advisory
observations but not promoted to live completion after TTL/session replacement.

Preserve this information in adapter return values up to existing provider
acquisition boundary. Do not edit HydrationReport::is_complete or repair #142
aggregate completion here. If that boundary requires a completeness-changing
public fix to use current provider truth safely, invoke hard stop §34.

## 21. Coverage/completeness preservation

A private provider-result structure should carry value plus query limits and
project observation. Minimum: project/root binding, optional provider generation,
returned/total and eq/gte/unknown, truncation/reason, continuation/pages consumed,
partial failure and freshness/coverage recording limitations. Keep fields optional
only when actual wire lacks them; unknown is never complete.

Differentiate query page complete, indexed-result complete, accepted-source index
job succeeded and native authorized-source evidence sufficient. No generic boolean
complete may collapse all four. Best-effort coverage with clean status is not
proof of no excluded files, full language semantics or native adequacy.

Extend private QueryRows/normalized search/trace outcomes rather than mirror all
CBM types. Audit every Vec-only legacy consumer: if unable to represent partial,
return explicit provider failure/unavailable and existing native fallback; never
truncate into a successful empty/full-looking result. Do not silently change
public graph_query or core completeness schema. Required public exposure of limits
uses an already-supported diagnostic representation, or becomes an approval stop.

## 22. Project/index/watch identity

Continue native canonical primary/additional-root maps and explicit project args.
Session launch root, queried project/root, serving generation, job observation
and watch observation remain distinct private fields. Verify provider project/root
against admitted map when used for native candidate paths. Unknown project may
be queried through existing proxy behavior but cannot infer native authorization
or a disk-cache root from slug.

A session rooted in A can query indexed B/C. It does not ensure those projects
are watched. Status diagnostics can expose that limitation; stale/missing hints
fall back to native acquisition, not false absence. No per-repo frontend expansion,
watcher substitute, global coordinated snapshot or arbitrary additional-root
registration is added. withinPath continues to constrain native acquisition.

## 23. Advisory-intelligence migration

Baseline current branch does not consume historical CbmIntelligence/D1/D2/D4 in
production context selection; cbm_informed stays false. Therefore “restore” does
not authorize enabling those historical features in this migration. Preserve
current importance-derived helpers and optional graph tools where actually used;
adapt their wire projection conservatively.

D0 health and D2-Wire decoding concepts/tests may be mined. D1/D2 related-file/
symbol/data-flow/service advice and D4 metadata are reference-only for now. If an
existing approved current path needs such fields, preserve it narrowly; otherwise
adding _meta.cbm or changing fidelity selection is a separate approved capability
phase. Do not consume Hybrid LSP targets, resolver confidence or CALLS as native
invocation evidence. Unknown provider evidence remains optional/unavailable.

## 24. Proxy/compression compatibility

A shared validated RPC outcome supports parsed and raw paths. Compression must
preserve apparent provider truth even if notation differs. Protected information:
RPC/tool error, isError, project/QN/file identity, ambiguity/candidates, readiness,
generation limitations, totals/relations, has_more/truncated/reason, cursor and
coverage/freshness flags. No empty/null stripping may erase meaningful unknown,
false, zero or “no continuation possible” state.

Add deterministic fixtures comparing these facts before/after compression across
JSON body, tree body, soft error, RPC error, multiple content blocks and any
structuredContent supplied. Current compressor prioritizes content and may ignore
other structured fields; production change must preserve protected fields rather
than presume duplicated channels. Keep existing response mode/envelope unless
necessary contract approval is obtained. Do not add a new public semantic graph
schema or claim byte-equivalence to direct CBM MCP.

If key shortening merges distinct protected fields or formatting makes values
ambiguous, minimally preserve those fields in readable notation. A bounded faithful
optional error/partial result is preferable to impressive savings with lost truth.
Freeze existing aliases/identity verification/proxy whitelist and telemetry.

## 25. Historical branch mining map

| Candidate | Classification | Method / reason |
| --- | --- | --- |
| `029ec9b` transport migration | PORT CONCEPT / PORT TEST | Manual current implementation; read session/admission intent, do not import private cache or broad conflict classes |
| `74f75d3` D0 health recovery | PORT CONCEPT / PORT TEST | Use as reference authority; implement current dead-session/re-admission under existing breaker |
| `b3c8363` D2-Wire, old client normalization | PORT CODE WITH REVISION | Copy only small useful decoder algorithms after pinned fixture tests; reconcile totals/cursors/grouped JSON |
| `5ed9522` coverage lifecycle | PORT CONCEPT / PORT TEST | Current status/coverage contract differs; manual adapter truth preservation |
| `3f09c79` state removal | SUPERSEDED | Do not restore CbmFilterState; preserve current native ownership |
| `211246c` D1 / `9eab40d` D2 | SUPERSEDED for current migration scope | Reference only; not shipped current baseline optional behavior to restore |
| `c899507` D4 and d4_advisory_meta tests | PORT TEST concept only | Ordering/omission/bounds/authority tests reusable if separate metadata capability approved; no automatic activation |
| Historical optional fallback and explicit fidelity/intent tests | PORT TEST | Adapt assertions to current native/meta/batch paths, not old ownership |
| Historical cache_root override | REJECT | Account cohort cannot be split by private roots |
| Historical substring DaemonConflict matcher | REJECT | Diagnostic text is not exact lifecycle authority |
| Child-as-runtime cleanup / old flattened schemas | REJECT | Shared daemon ownership / metadata loss incompatible |

No commit is recommended for immediate cherry-pick: they combine obsolete policy,
old file layout and context-selection changes. Mine manually, retain useful small
code/tests with attribution, and reject unrelated historical fixes. Do not merge
or rebase the branch. Current native publication/identity/invocation tests prevail.

## 26. RED test matrix

All new regressions live under src/tests/** and compile through existing #[path]
module registration. Operator assets are not tests. Existing inspected tests:
CBM query_wire/deterministic, trace_wire, project_search, proxy_errors,
cache_ownership, freshness/project-identity/retry/circuit regressions, integration
compression and unavailable handler tests; invocation/C#/Angular tests protect
parent semantic boundaries. No tests were run during planning.

| New regression family | Required RED behavior before fix | GREEN contract |
| --- | --- | --- |
| cbm_transport_identity_ | Wrong ID/null-ID success accepted or unsupported envelope reaches consumer | Only pending matching response succeeds; lifecycle/notification distinguished |
| cbm_transport_retire_ | Timeout/oversize/death leaves reusable session | Unusable state and owned frontend cleanup, no dead-pipe resend |
| cbm_admission_ | Spawn publishes Available despite refusal/malformed initialize | Available only after valid initialize/initialized/list; no-CBM core unaffected |
| cbm_recovery_ | Cooldown/reset retries dead child; mutation repeated | Read re-admitted once; mutation unreplayed; breaker preserved |
| cbm_launch_context_ | Child CWD differs from canonical primary root; binary changes after CWD | Stable resolved binary/root/env inheritance |
| cbm_current_wire_ | Missing modern schema becomes empty result | Exact normalization and truthful incompatibility |
| cbm_continuation_ | First page called complete; cursor/limits discarded | Partial preserved, snapshot errors/no-progress fail safely |
| cbm_provider_state_ | Search success upgrades running/failed observation | Job/serving/watch/coverage remain independent |
| cbm_proxy_truth_ | Error/false/gte/cursor/ambiguity lost in compression | Protected facts survive without fabricated authority |
| cbm_upgrade_no_provider_ | Incompatibility/death gates native context or returns false empty | Native current-source operation succeeds under its existing declared contract |

Observe RED by intended assertion, not compilation error, fixture failure or zero
selected tests. For active oversized legacy files, extract mechanically first
with existing contract checks, then apply repository RED-first procedure:
new regression observes failure → stash only regression/registration → implement
production fix → restore regression → GREEN. Do not manufacture a passing test
by changing its expected behavior after discovering a different failure.

## 27. Process-fixture test design

Prefer controlled subprocess scripts/fixtures alongside tracked Rust tests for
mechanics; no live CBM dependency for framing/recovery. A deterministic fake
speaks initialize, initialized, tools-list and selected tools with supplied IDs.
Cases: delayed/blocked write, wrong ID, notification before correct reply,
null-ID startup error, stderr flood, exactly-at/over-limit line, invalid UTF-8,
EOF mid-frame, immediate death, second launch success and mutation request count.

Use existing repository process/harness conventions; inspect available pattern
before choosing platform mechanism. Existing CBM live-e2e setup is not a fake
transport test and graceful skip is not a GREEN result. If no portable fake
executable pattern exists, use a narrowly scoped test-only subprocess fixture
under src/tests, not a production CLI flag or generic mock framework. Windows
must not depend on bash; PowerShell/Python dependency must be explicit and CI
supported, otherwise use Rust test fixture. Test assertions remain Rust tests;
fixture/support split follows ceiling/#[path] rules.

Fixture artifacts/workspaces under target or temporary directories, tracked
scripts/definitions under test tree. Deterministic synchronization markers replace
fragile fixed sleeps. Record launch count, received IDs/tool args and cancellation
ownership. Kill/reap only fixture-owned processes. Required drainer-thread failure
can be injected by small test-only constructor seam, not a broad abstraction.

## 28. Live operator verification matrix

Reuse `verification/cbm/scripts/Investigate-DuplicateTraceNames.ps1` and
`verification/context-compression/scripts/McpSession.ps1` patterns; use the
notification-safe deadline/result capture convention already in invocation live
harness. Add a CBM-upgrade driver with Phase A/B selection, immutable run manifest,
real registered Clean-CTX MCP calls, per-case evidence and explicit exit failure.
Do not invoke Codex/Claude for mechanical frontend validation. Host rendering is
separate if later needed. Scripts planned, not created/run in this task.

| Gate | Cases / acceptance |
| --- | --- |
| A basic | Explicit root, initialize/tools-list, sequential IDs, matching read, EOF shutdown; actual admitted Available |
| A framing | JSON/tree body, stderr drain, legal large reply/local oversize failure; wrong-ID fixture contrast |
| A lifecycle | Compatible A/B sessions, A exit B survives, ephemeral vs permanent, frontend timeout/read re-admission |
| A admission | Safely isolated incompatible build/root, useful diagnostic, no daemon takeover or private cache fork |
| A root/scope | Parent CWD vs explicit native root; A session querying B/C; watch state recorded separately |
| B schemas | Typed graph_search/query/trace, raw equivalents, architecture/list/index/status/coverage as used; valid identity/error fields |
| B paging | More than defaults, budget clipping, engine/window saturation, stale cursor, failed page; no false completion |
| B compression | RPC/soft errors, ambiguity, totals/continuation, coverage and fresh/stale observations survive |
| B native preservation | C# same-named interface/class across files; Angular DI/selectors/template and Rx; Karma/Jasmine and Vitest/TestBed fixtures; cross-meta-layer and batch routes |

Evidence records operator verification, not CI/test GREEN. Build artifacts need
current-source freshness check and pinned CBM provenance. Do not touch unrelated
account daemon; conflict/activation scenarios require separate safe ownership.
Full test suite remains CI-only. No runtime command is authorized by this plan.

## 29. No-CBM preservation matrix

| Touched capability | With current CBM | Without/unusable CBM | Required truth equivalence |
| --- | --- | --- | --- |
| Launch/admission | Compatible initialized provider optional | Missing/disabled/incompatible/rejected means unavailable with diagnostics | Native core unaffected |
| Canonical context/fidelity | Native compile, optional existing hints only | Same native strategy/compile | Yes for explicit core contract |
| Core discovery/relationships | Verified provider path hints then native evidence | Existing authorized native discovery | Yes for authoritative completed claims; known #142 gap recorded, not redefined |
| C#/Angular/TS/meta producers | Native owned evidence | Same native evidence | Yes |
| Invocation/calls_in_file | Private compilation-local native occurrence | Same occurrence boundary | Yes |
| Batch/indexed-only queries | Existing declared native completeness/lower bound | Same declared contract | Yes for contract; lower-bound indexed contents may differ |
| Optional graph proxy tools | Truthful current provider output/limits | Explicit unavailable/error, not successful empty | No optional intelligence equivalence promise |
| Timeout/death/recovery | Retired session, safe read re-admission | Native fallback until admitted | Core unaffected; no false recovery |
| Readiness/coverage | Private provider observations | Unknown/unavailable, never Ready by default | Does not decide native projection adequacy |
| Compression/cache | Preserve actual optional truth | No stale cached payload passed as fresh current authority | Yes for core; optional stale advice disclosed or omitted |

Exercise missing, disabled, incompatible, rejected admission, running index,
frontend death and unavailable daemon as distinct cases. Also test stale provider,
provider error and not-watched B. Do not hide current native completeness defects
behind a successful fixture; those remain the later #142 experiment's subject.

## 30. Compatibility/generation analysis

No canonical IR/semantic producer generation, WorkspaceIndex schema or private
invocation generation changes are justified by transport repair. Keep Binary0x04,
source hash publication, producer identity and public native graph contracts.

Adapter payloads DO change. Old cached QueryRows/search/trace shapes lack limits
and can poison new decoders. Add a narrow adapter-wire namespace/epoch to relevant
GraphBridge memory/disk keys, scoped by existing project/root owner; ignore old
entries in new namespace, do not wipe CBM stores or unrelated Clean-CTX cache.
Proposed namespace includes source contract pin and explicit wire revision;
this is payload compatibility, not a claim that cache data came from that SHA.
Cache metadata includes actual admitted provider identity when available; never
upgrade missing generation to fresh. Existing TTL/invalidation remains until
separate policy approval; generation-bound cursors are not persisted as portable
cross-session continuation certificates.

GraphCacheStore SQL schema need not change if JSON/keys suffice. If it must,
justify a narrow cache-only migration with old-entry rejection tests. Do not bump
semantic producers prophylactically. D4 _meta.cbm is not added; if later approved,
its host/model-envelope compatibility gets separate review.

## 31. Performance analysis

Qualitative expectations only; no benchmark ran. Admission adds initialize/list
round trips once per frontend replacement; daemon reuse amortizes indexing, not
native semantic truth. Page traversal adds requests and temporary buffers; bounded
retention/deadline avoids unlimited acquisition. Raw proxy stays single-page.
Mandatory drains/bounded frame ownership improve reliability with small overhead.

Native requests must not wait for global indexing. Existing per-project readiness
and fallback remain. Reconnect attempts obey current breaker budget; no per-query
fresh process, per-root frontend explosion, constant status polling or new global
synchronization. Measure startup/read latency, pages, byte peak, request/retry count
and multiroot cost after implementation. No speed claims from source history.

## 32. Rollback/failure behavior

Small reviewed commits on child branch; rollback is revert of a slice plus its
adapter cache namespace expectations. Do not “rollback” by weakening correlation,
reviving an uncertain client or returning false empty success. If modern adapter
cannot run safely, disable optional provider path using existing configuration
and preserve native behavior until fixed. Do not downgrade/install CBM automatically,
change account root or terminate other users' sessions.

Wire epoch prevents new/old payload mixing; old native source/persistence formats
remain untouched. Keep legacy captures as compatibility/reference tests, not an
unsupported production fallback to arbitrary old versions. Rollback does not
claim every pre-existing #142 behavior becomes correct.

## 33. Exact implementation sequence

Sequence is deliberately dependencies-first: decompose oversized client before
semantic repair; bind launch spec before re-admission; complete retirement before
usable handshake/retry. Every implementation slice gets focused authorized RED /
GREEN commands with explicit test filter from §26; never run broad CBM filters
that silently start live-e2e cases. Final package lint/build and live runs require
specific authorization. Full repository suite is CI-only.

| Slice | Production files affected | Tests / protected invariant | RED → GREEN acceptance | Compatibility / live / rollback |
| --- | --- | --- | --- | --- |
| A0 freeze/extract | client.rs → focused client modules; no policy change | Existing deterministic query/trace/proxy/no-provider; add baseline assertions only where needed | Extraction compiles and preserves baseline; no artificial defect RED required for relocation | Internal only; live no; revert extraction |
| A1 correlation/bounded frame | client/protocol.rs, transport.rs, errors | cbm_transport_identity_, retire oversize tests; invariant 5/7 | Wrong/null ID/malformed accepted or unbounded line → strict pending response, local cap before allocation and retire | Protocol failure honest; fake only; revert to disabled provider if unsafe |
| A2 launch/retirement owner | client/lifecycle.rs, bridge/lifecycle.rs, binary.rs | cbm_launch_context_, cbm_transport_retire_; invariants 6/7 | Wrong CWD/dead session reusable → resolved binary/root, mandatory drains, retired ownership | Native unchanged; fake only; revert launch spec/owner together |
| A3 admission | client lifecycle/protocol; bridge/indexing initialization call | cbm_admission_; provider usable distinction | Spawn marked Available on refusal → correlated handshake/list before Available | One-session setup; fake then pinned A gate; revert admission slice with provider disabled |
| A4 retry/half-open | client lifetime/retry; bridge/cache.rs status sync | cbm_recovery_, mutation count; invariant 7 | Dead writer reused/mutation repeated → safe read replacement once, mutations unreplayed | Existing budgets preserved; fake only; revert retry slice without reviving dead transport |
| A5 account ownership acceptance | config/binary docs and diagnostics only if needed; no private root | No-provider/config/env/admission + operator A cases | Separate cache assumed / incompatible available → inherited agreed root, honest compatibility | Pinned live required; no installer; fail A acceptance if takeover possible |
| B0 fixture/epoch freeze | bridge/cache.rs/cache key helper; minimal QueryRows private metadata | Old/current deterministic fixtures + cache_ownership | Legacy cached shape accepted → old namespace ignored, native persistence unchanged | Cache-only epoch; no live needed initially; revert epoch/decoders together |
| B1 explicit requests/decoders | client/wire.rs; bridge/query.rs; trace_wire/map helpers | cbm_current_wire_ including aliases and soft errors | Tree/wrong group becomes empty → explicit machine JSON and validated table normalization | Existing tool surface; pinned B fixture capture required; revert decoder by capability |
| B2 private limits/project outcomes | client wire; project_search.rs; bridge/indexing.rs/status | cbm_provider_state_, cbm_continuation_ metadata | Search success erases job/coverage → independent preserved observations | No #142 aggregate fix; fake/current fixtures; stop if public contract redesign required |
| B3 bounded continuation | tool-specific wire paging; bridge query cache paths | cbm_continuation_ exhaustion/stale/budget/no-progress | First page treated complete → approved bounded collection or explicit partial/failure | Policy approval required; pinned paging live; revert collection preserving single-page truth |
| B4 proxy/compression | proxy.rs/json_compress.rs/caller verification adapters | cbm_proxy_truth_ protected fields, whitelist/identity tests | Error/gte/cursor dropped → faithful readable compressed truth | No new semantic schema; live raw/typed compare; revert compressor only with truth preserved |
| B5 preservation/operator acceptance | existing harness definitions/docs; no D1/D2 enabling | Focused relevant parent tests and live B matrix | Any current wire/core regression → all required scoped checks green and live cases evidenced | Pinned live + CI owns full suite; no overall completion on skips |
| STOP | No new production files | Review A+B evidence, all invariant traces | Freeze completed upgrade; authorize only experiment next | No #142/Hybrid implementation |

For each defect slice follow exact repository RED-first procedure in §26, then
review lifecycle integration, not merely unit success. A fake's protocol failure
is not expected RED unless the assertion proves the production defect.
Commands are handed off/authorized per run at implementation time, with copy-
pasteable filter matching actually compiled tests. This planning task does not
supply blanket permission or claim those tests exist today.

## 34. Hard stop conditions

Stop and report rather than silently broaden if:

1. Pinned provider cannot support subprocess MCP/private pipes.
2. Intended pinned account deployment cannot be deterministic without takeover,
   hidden cache fork or unsupported coordination.
3. Provider requires authority over native source/semantic truth.
4. A native core capability loses no-CBM support.
5. Paging/limits cannot remain truthful without unapproved public contract change.
6. Provider source identity cannot remain advisory while native freshness is checked.
7. Recovery requires unsafe mutation replay.
8. Correct adapter implementation requires solving #142.
9. Implementation would rebase/rebuild from main, bypassing the exact feature parent.
10. New caps/timeouts/global retries/caching/public errors are needed without approval.

Report exact decision, affected consumers, alternatives and recommendation; do
not create a framework or redefine core completeness to evade the blocker.
B3 policy approval and pinned artifact provenance are concrete acceptance gates,
not permission to run verification or implement new semantics during planning.

## 35. Post-upgrade #142 experiment

Design only. After accepted A+B, STOP implementation and obtain specific live
experiment authorization. Extend existing PowerShell operator assets; same
primary/additional roots, exclusions, producer settings/fidelity, source contents
and native public contract across paired runs.

Two provider arms: **N**, disabled/absent CBM; **P**, pinned compatible CBM. For
named setup queries X/Y and target Q run fresh→Q, fresh→X→Q, fresh→Y→Q,
fresh→X→Y→Q, fresh→unrelated→Q. Avoid label confusion between experiment arms
and setup queries. Fresh native server and explicitly recorded provider graph/
watch/job state; native “fresh” does not imply cold shared daemon. Reset only
owned fixture state, never unrelated account graph/runtime.

Cover C# base/interface classification, reverse targets, same-named cross-file
identities, Angular selector/template and service High adequacy, cross-meta-layer
calls, transitive lower bounds and batch catalog order. Include Karma/Jasmine
and Angular Vitest/TestBed fixtures where touched framework evidence is relevant;
do not treat test-framework source support as a Rust environment pilot.

Record separately semantic result, resolution state, ambiguity, candidate set,
provider readiness/coverage/generation, pages/bytes consumed, files acquired/
compiled, required/publication fidelity, native WorkspaceIndex evidence and source
SHA, latency and provider request count. No timing-derived correctness inference.

Mutation matrix: edits including same-length edits, deletes/restores, native
restart, single repo/additional roots, item-order/singular-vs-batch, indexed but
not watched B, provider still indexing/stale/failed. Fixed-source equivalence
compares semantic facts/status/ambiguity and declared completeness; intentional
indexed-only lower bounds may vary truthfully and must not become absence claims.

Experiment completion produces evidence-backed remaining #142 gaps and smallest
fix proposal. Do not design or implement that production correction before results.
No Hybrid binding/target persistence experiment is authorized by this plan.

## 36. Explicit non-goals and final decisions

No #142 fix, generalized resolution/provider framework, Hybrid LSP enrichment,
invocation-target resolution, durable derived edges/private occurrence persistence,
replacement WorkspaceIndex, direct daemon IPC/database access, embedded CBM,
Clean-CTX-owned daemon/graph store, mandatory CBM, automatic installer/update,
new public semantic graph schema, global budgets or unrelated capability migration.
No branch was created/switched and no production code/test/harness was changed.
Only this planning report was added; existing investigation documents preserved.

Implementation verdict: **READY FOR PHASE A IMPLEMENTATION**.

Transport architecture: **PRESERVE WITH TARGETED REPAIR**.

Historical branch: **MINE TESTS AND CONCEPTS**.

Branch strategy: **CHILD BRANCH FROM SEMANTIC-BOUNDARY FEATURE**.

#142: **DEFERRED UNTIL POST-UPGRADE LIVE FALSIFICATION**.
