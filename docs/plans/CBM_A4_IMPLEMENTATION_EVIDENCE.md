# A4 replacement, re-admission and replay evidence

Branch: `feature/current-cbm-pinned-upgrade`.
Verified ancestry: `4c1627fcefe698977447dcee4e88830c20c7f422` → A0 `87cbd04`
→ A1 `cb005bc` → A2 `14d05cc` → A3 `3fe756d` → this A4 change.
Pinned source HEAD verified: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
No merge, rebase, historical branch switch or push was performed.

## Operation inventory and pinned authority

[The pre-implementation replay inventory](CBM_A4_REPLAY_INVENTORY.md) records
all seven production provider operations, each caller, duplicate-execution risk,
classification and pinned handler/engine locations. It preceded retry changes.

**PROVEN BY CURRENT CODE:** replayable operations are `search_graph`,
`query_graph`, `trace_path`, `get_architecture`, `list_projects`, and the
existing project-only `detect_changes` request. They read graph/metadata/Git
evidence, not enqueue indexing or
write graph state. Query-only store resolution does not create/quarantine/rebuild
indexes; the Cypher engine explicitly rejects CREATE/DELETE/SET/MERGE.
Session store selection can change during a read, but repeating the same explicit
project selection does not create a second graph/job/registration mutation.
Results can differ between observations; replay safety does not imply snapshot
stability or complete/fresh knowledge. No Phase B response interpretation changed.

`index_repository` is mutation-capable: a daemon-backed request can dispatch
shared registry work after workspace admission. Acceptance may precede a lost
response. Even provider idempotent annotations do not establish safe duplicate
job execution. It is never automatically replayed, including conservative
handling of status-mode requests. Unknown names are also never replayed. Argument-dependent detect_changes
workflows are not inferred safe from readOnlyHint: scope="tests" creates work
state and enters a separate engine. The source audit corrected that initial
name-only classification; only the actual production request with exactly one
string project field is replayable. Extended/unknown shapes are unreplayed.

## Replacement eligibility and ownership

**PROVEN BY CURRENT CLEAN-CTX CODE / LOCAL FIXTURES:** automatic replacement is
limited to terminal ConnectionLost or Timeout from a previously admitted client,
with direct child reaped and owned workers joined. This covers frontend exit,
EOF/disconnect and write/flush loss. Admission refusal, CapabilityError,
ParseError/protocol poison, launch/root failure and uncertain cleanup are not
replacement triggers. A failed replacement disables demand recovery so
incompatibility cannot become a respawn loop.

The client retains its canonical A2 launch specification. `client/recovery.rs`
uses the normal `try_launch` and `admit` boundaries; there is no shortcut
constructor or duplicate handshake. Each replacement gets new owned pipes,
workers and request-ID epoch. Old receivers are released before reuse of IDs;
no old frame or writer survives into the new epoch.

Transition:

terminal transport → A2 retire/reap/join → one replacement using original
executable/root/configuration → full A3 initialization/capability admission →
usable frontend → independent replay decision.

Only observational operations can be resent, once. Mutation/unknown calls may
recover the frontend but return the original failure without resending arguments.
No account cache override, daemon attachment/control, process-tree kill, extra
root frontend or generalized supervisor was introduced.

The existing GraphBridge `Arc<Mutex<Option<CbmClient>>>` serializes calls;
replacement requires the same exclusive mutable client ownership. Four
concurrent callers produce one replacement in the fake-process regression.
Background indexing owns that same mutex, not a separate recovery service.

## Retry, circuit and indexing semantics

Existing budgets are unchanged: one retry, 100-ms backoff, configured consecutive
failure threshold and cooldown, existing receive timeout, 4-MiB frame cap,
256-frame queue and five-second cleanup observation. Replacement/admission
adds protocol requests but never more than one replacement per operation.
Entry-time half-open consumes the retry budget too, preventing a second launch
if that operation fails. Repeated terminal loss permits a later demand-driven
half-open only after existing cooldown; status inspection is passive.

Half-open now creates and admits a new frontend instead of resetting dead pipes
or reusing a surviving open-circuit frontend. Failure counters reset for that
fresh half-open attempt and on successful operation completion. Admitting a
replacement after an uncertain mutation does not clear an already-open operation
circuit; the original failure count and cooldown remain authoritative.

Surviving typed -32603 errors still permit one retry for proven observational
operations, never mutation/unknown operations. Raw pipe calls preserve existing
correlated RPC-envelope behavior; they do not invent typed -32603 retry semantics.
This is not a Phase B proxy/error-envelope migration.

MCP health/indexing guards now admit eligible demand-driven attempts without
claiming Available before replacement. A surviving usable client below the
configured failure threshold can still dispatch even when status retains a
previous retry failure; a diagnostic string cannot create permanent lockout.
`can_attempt_provider` is eligibility,
not status or a launch side effect. The actual tool call owns the transition.
`get_cbm_status` and native enrichment do not poll or launch recovery.

Clean-CTX background `index_repository` scheduling is not repeated by recovery.
Its uncertain failure remains Failed; lazy freshness is not advanced on failure.
Existing per-project scheduling/freshness state and failure accounting remain;
this phase does not infer idempotence or solve indexing completeness. Provider
initialization can independently auto-index under the already-approved A3
contract; not resending Clean-CTX's indexing operation is not a claim that the
provider does no indexing during initialization.

## Failure preservation and native independence

The original operation error is returned when recovery fails or replay is unsafe.
Degraded status retains both original and replacement/admission cause, including
candidate transport cleanup uncertainty. If an allowed replay fails, the original
timeout or structured RPC cause survives; status also records the later replay
failure and transport state. Permanent refusal remains typed and unreplayed.

Native heuristics remain independent of unavailable provider state. This is
established by the retained local baseline and existing integration filters,
not a claim that every native MCP request was exercised live. No provider failure
is synthesized into successful-empty knowledge. The registered graph_query
regression crosses dispatch, health guard, indexing guard, serialized client,
A2/A3 replacement and model-facing JSON-RPC response using fake provider data.
Its legitimate empty response proves recovery delivery, not graph completeness.

## RED → GREEN

Commands used the authorized package, locked dependencies, library, all features
and four jobs:

```sh
cargo test --locked -p clean-ctx --lib --all-features -j 4 cbm_a4_
```

Initial fixture compilation problems (wrong enum path and an unavailable mock
constructor) were corrected before behavioral RED. They are not RED evidence.
The final pre-A4 behavioral RED selected 18 tests: **13 intended failures,
5 passing baselines**. The preceding 17-test run had 12 failures before the
registered MCP entry-path regression was added. First implementation GREEN:
18/18 passed. RED regressions were stashed without production changes, restored
with unchanged assertions and exercised after the fix. Subsequent formatting
changed layout only.

Pre-A4 failing cases:

| Regression suffix (`cbm_a4_`) | Observed defect |
| --- | --- |
| observational_eof_replaces_once_and_replays_after_full_admission | Recoverable read failed instead of admitted replay |
| timeout_read_replaces_but_mutation_returns_original_without_replay | Read timeout did not recover |
| all_proven_observational_tools_have_bounded_replay | First safe tool did not recover; GREEN loop verifies all six |
| mutation_internal_error_is_not_retried | index_repository executed twice |
| unknown_internal_error_defaults_to_no_replay | Unknown tool executed twice |
| raw_boundary_replaces_without_changing_rpc_envelopes | Raw read did not recover |
| replacement_refusal_preserves_original_and_recovery_cause | No replacement attempt; subsequent cause unobservable |
| missing_replacement_capability_never_replays_or_loops | No replacement attempt |
| repeated_transport_failure_stops_after_single_replacement | No replacement attempt |
| half_open_uses_new_admitted_frontend | Old child reused |
| concurrent_callers_share_one_recovery_owner | Concurrent calls failed rather than sharing recovery |
| background_indexing_internal_error_does_not_duplicate_uncertain_job | Two indexing requests |
| registered_mcp_query_can_recover_after_circuit_cooldown | Registered handler rejected recovery at health guard |

Passing baselines: explicit retirement remains terminal; surviving observational
RPC retry remains one; refusal/protocol poison do not respawn; native heuristics
remain usable. The fifth selected baseline is the existing deterministic C# IR
caller test `red_cbm_a4_same_name_same_arity_is_ambiguous`, not a process/live test.

Implementation audit regressions were observed separately:

| Audit case | RED | Subsequent GREEN |
| --- | --- | --- |
| Failed replay hides original timeout behind EOF | 18 passed / 1 failed | 19 passed |
| Surviving RPC retry hides structured -32603 cause behind EOF | 20 passed / 1 failed | 21 passed |
| Mutation replacement falsely clears open-circuit status | 21 passed / 1 failed | 22 passed |
| Diagnostic degradation blocks MCP below circuit threshold | 22 passed / 1 failed | 23 passed |
| Name-only detect_changes allowlist replays unproven scope workflow | 23 passed / 1 failed | 24 passed |

The final MCP audit initially used an incorrect JSON-RPC error oracle for a
handler that returns result.isError/structuredContent. That setup issue was
corrected and RED repeated against the pre-fix guard: request count remained two
instead of four. The corrected oracle and primary assertion were restored
unchanged for GREEN. This is not a weakened assertion or a passing-harness claim.

The added terminal-cooldown epoch baseline passed immediately and is not RED
proof. Final A4 filter: **24 passed, 0 failed** (23 phase tests plus existing
caller baseline). Coverage includes all six safe operations, mutation and
unknown classes, both timeout and EOF, full replacement admission/root ownership,
refusal/capability loop prevention, serialized concurrency, background indexing,
MCP entry reachability, argument-dependent replay safety and preserved
error/circuit semantics.

Preservation commands used the same test prefix with these filters:

| Filter | Passed |
| --- | ---: |
| cbm_a0_ | 2 |
| cbm_a1_ | 8 |
| cbm_a2_ | 15 |
| cbm_a3_ | 14 |
| cbm::tests::query_wire::deterministic | 16 |
| cbm::tests::integration | 8 |

A2 transport fixtures still isolate lifetime contracts via stub admission;
`admitted_once` is set only by actual A3 admission, so those baselines do not
claim automatic recovery. A4 fixtures use real admission and the same portable
fake-process constructor pattern already used by A2/A3. No installed CBM ran.

Package Clippy command:

```sh
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

The first lint run found an unnecessary return in a fake fixture. Removing it
changed no behavior or assertion; the subsequent lint run passed. Final checks
were repeated after the last MCP eligibility fix. No warnings were suppressed.

## Audit and limits

The new recovery module is the sole typed/raw retry owner. Old duplicate loops
are removed. Creation, retained launch context, serialized ownership, A2
retirement, A3 admission, tool consumption and registered MCP exposure were
traced. Existing tests/decoders/native acquisition were not replaced. Active
code files remain below 615 lines and strict UTF-8 without BOM.

**NOT ESTABLISHED:** installed pinned artifact provenance, live daemon reuse,
canonical account-cache compatibility, Windows live process behavior, watcher
freshness, first/second account frontend acceptance or incompatible live cohort
refusal. [A3 prepared live cases](../../verification/cbm/A3_ACCEPTANCE.md) remain
NOT RUN. Fake process evidence cannot establish these A5 claims.

Existing synchronous stdin writes and serialized provider-lock contention remain
A2 limitations; the receive/cleanup budgets do not establish an end-to-end wall
clock guarantee. A4 introduces no unbounded replacement loop or native recovery
waiter; it does not claim to repair every pre-existing blocking-I/O limitation.
No new timeout, indexing completeness or global lifecycle policy was introduced.

No full suite, live daemon, operator harness, standalone build or A5 acceptance
was run. Full repository verification remains CI-only. Phase B decoder/paging/
coverage/proxy truth migration and Issue 142 acquisition/hydration remain deferred.

**A4 COMPLETE — READY FOR A5 AUTHORIZATION**

Local A4 contract readiness does not waive the separately authorized pinned
artifact/account/live deployment acceptance gate. Stop here; do not begin A5,
Phase B or Issue 142.
