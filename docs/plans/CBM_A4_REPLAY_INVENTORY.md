# A4 pre-implementation operation and recovery design

Source authority: CBM `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
Branch and ancestry verified against `4c1627fcefe698977447dcee4e88830c20c7f422`;
pre-A4 baseline is `3fe756d`. No live provider or Cargo verification run here.

## Actual production operations

Every operation passes through serialized `CbmClient::call_tool` or
`call_tool_raw`; GraphBridge owns one `Arc<Mutex<Option<CbmClient>>>`.
The proxy whitelist adds no operation beyond the following seven. Internal
intelligence helpers issue query_graph rather than additional provider tools.
Every dispatched operation can have been accepted before EOF/timeout/write
failure; transport uncertainty cannot prove that execution did not happen.

| Tool | Existing callers | Semantics / duplicate effects | Replay class and pinned evidence |
| --- | --- | --- | --- |
| search_graph | client/wire search; bridge/query; proxy | Graph reads, query-only store opening; no graph/job mutation | Observational: mcp.c:5206 handler, resolve_store:2600 and query-only opening:2453 |
| query_graph | client/wire query; bridge/query; intelligence helpers; proxy | Query-only store; read engine rejects writes. Results may change between attempts. | Observational: mcp.c:5949,6060; cypher/cypher.c:848–859 rejects CREATE/DELETE/SET/MERGE |
| trace_path | client/wire trace; bridge/query; proxy | Traverses existing relationships using query-only store; no duplicate mutation | Observational: mcp.c:9572 handle_trace_call_path / resolve_store |
| get_architecture | client/wire architecture; bridge/query; proxy | Reads existing graph projection | Observational: mcp.c:7921 / resolve_store |
| list_projects | raw proxy / discovery | Scans existing cache database metadata/statistics | Observational: mcp.c:3005; no indexing/registration dispatch |
| detect_changes | bridge/cache invalidation (project-only arguments) | Default production request reads root/Git/graph impact. Other arguments can select a separate work-producing workflow. | Observational only for the existing project-only request: mcp.c:17641, get_project_root:10317. Extended shapes default unknown/no replay; see audit addendum. |
| index_repository | client/wire; explicit, background, lazy root indexing; proxy | Can dispatch shared daemon jobs, alter graph and session project/store; acceptance may precede failed transport | Mutation-capable, never automatic replay: mcp.c:11994,12097–12107,12203–12207. Provider annotation idempotent=true is insufficient to authorize duplicate job execution. Even status-mode is conservatively non-replayed. |
| Any unrecognized tool | Internal raw/typed API accepts a name; future callers | Safety not established | Unknown: never replay |

PROVEN BY CURRENT CODE: pinned annotations at mcp.c:853–889 corroborate the
normal read paths, but are insufficient for argument-dependent workflows.
Classification above also traces handlers. Query-only
resolve may update the frontend's open-store selection; repeating that selection
is not a second graph mutation/job/registration. Initialization itself may
start provider-owned auto-indexing: replacement reuses the approved A3 boundary,
not a claim that replacement is entirely side-effect-free.

## Existing retry discrepancy

Pre-A4 typed calls retry -32603 once after 100 ms on a surviving transport,
regardless of mutation semantics. Raw calls return correlated error envelopes
without interpreting -32603. Terminal transport errors are never retried after
A2, and cooldown cannot restore a retired transport. Preserve raw envelope
behavior; fix typed unsafe retries and introduce admitted terminal replacement.

## Intended smallest policy

- Safe allowlist consists only of the six observational operations above;
  detect_changes is restricted to the existing project-only request.
- Replacement targets terminal ConnectionLost and Timeout only, with confirmed
  direct-child/worker cleanup. Parse/protocol poison, admission refusal, missing
  capabilities and launch/root failures never automatically replace.
- Preserve canonical A2 executable/root and existing timeout/failure/cooldown
  parameters. Replacement calls normal try_launch and A3 admit. No alternate
  handshake, daemon control, account override or background-index scheduling.
- One failing operation permits at most one replacement and at most one replay,
  using the existing one-retry/100-ms budget. Mutation/unknown may recover the
  frontend but return the original error without resending their operation.
- Failed replacement closes automatic recovery to prevent refusal loops. Preserve
  the original operation error; degraded status records replacement cause too.
- If the replay itself loses the replacement, further demand-driven recovery
  waits existing cooldown; no status polling or per-query normal-case spawning.
- Circuit half-open uses a fresh admitted frontend, never dead pipes. Existing
  surviving -32603 retry remains only for proven observational operations.
- The existing client mutex and mutable ownership serialize the transition;
  no supervisor, extra process per root or new synchronization abstraction.
- Native code paths remain independent; no successful-empty provider claim.

This table is design evidence, not verification or live acceptance. A5, Phase B
and Issue 142 remain deferred.

## Post-implementation source audit addendum

PROVEN BY CURRENT CODE: `mcp.c:17710` routes scope="tests" to
`detect_test_impact` (`17583`). That path creates a private test-impact work
directory and calls `cbm_test_impact_run`; `test_impact_engine.c:583,633` prepares
working state and runs a separate analysis workflow. Its replay safety is NOT
ESTABLISHED, despite the tool-wide readOnlyHint annotation. This contradicts the
initial overly broad name-only classification, not the existing project-only
production cache caller. No live workflow was run.

A separate audit regression observed duplicate fake requests for scope="tests".
The final classifier checks both tool and arguments: only an object containing
exactly one string project field is replayable for detect_changes. scope="tests",
unknown scopes and all other extended shapes are conservatively unreplayed.
This is a local A4 replay decision, not a change to provider tool schemas or
execution policy. The other five observational handlers remain replayable.
