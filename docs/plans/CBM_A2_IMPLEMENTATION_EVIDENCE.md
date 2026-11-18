# Current CBM upgrade — A2 implementation evidence

Phase scope: explicit launch context and directly-owned frontend retirement.
A3 admission, A4 replacement/recovery, Phase B and Issue 142 remain deferred.

## Branch and authority

- Branch: `feature/current-cbm-pinned-upgrade`.
- Verified parent: `4c1627fcefe698977447dcee4e88830c20c7f422`.
- Completed predecessors: A0 `87cbd04`; A1 `cb005bc`.
- CBM source authority reverified: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
- No main rebase, historical-branch switch or CBM revision substitution.

## Production behavior

**PROVEN BY CURRENT CODE:** `client/lifecycle.rs` contains the private
`FrontendLaunchSpec` and `FrontendSession` ownership boundary. Resolution makes
both executable and root absolute/canonical before spawn; invalid roots fail
honestly. `GraphBridge::try_create_with_roots` supplies its existing canonical
primary root to `CbmClient::try_launch`. The internal Rust launch signature now
requires this root. No new workspace authority or MCP argument is introduced.

**PROVEN BY CURRENT CODE:** the native startup chain remains
`find_project_root` → `McpState::init_cbm_bridge` →
`GraphBridge::try_create_with_roots` → launch spec → `Command::current_dir`.
Existing native bootstrap root discovery is unchanged. Once that authority has
selected a root, an unrelated inherited parent CWD cannot replace the frontend
root. Relative executable resolution occurs before switching the child CWD.

**PROVEN BY CURRENT CODE:** `withinPath` remains query-scope narrowing; it is not
an input to frontend construction. Additional roots populate the existing
root/project maps and indexing requests, outside the single launch operation.
They do not launch extra frontends. Existing project-explicit query routing is
unchanged. A frontend CWD is not an indexing/watcher/freshness/completeness claim.

**PROVEN BY CURRENT CODE:** stdout-reader and stderr-drainer startup is mandatory.
A partially constructed session already owns the child and each installed
resource; pipe or worker setup failure returns `LaunchError` and drops that
session through the same retirement operation. Test-only worker failure
injection does not alter non-test behavior.

**PROVEN BY CURRENT CODE:** retirement removes the live session from the client
before cleanup. It closes buffered stdin without flushing uncertain requests,
drops the stdout receiver to release a channel-blocked reader, terminates only
the direct child if needed, reaps it and joins finished workers. Timeout,
write/flush failure, stdout error/EOF/disconnection, A1 protocol poison, explicit
internal retirement and Drop converge on this path. Retirement is idempotent.
Liveness is checked before a new request ID is allocated or stdin is written.

**PROVEN BY CURRENT CODE:** terminal sessions cannot be revived by circuit
cooldown or reused by the old retry loops. The loops retain their existing
behavior for a still-live session; terminal failures bypass retries and preserve
the original error. This does not add replacement, admission or replay policy.

## RED evidence

Authorized filter: `cargo test --locked -p clean-ctx --lib --all-features -j 4 cbm_a2_`.
The valid final pre-fix run selected 13 tests: nine defect regressions FAILED,
three A2 baseline cases passed, and one pre-existing caller-verification test
passed. The following names use prefix `cbm_a2_`:

| Defect regression suffix | Pre-fix assertion failure / defect established |
| --- | --- |
| `launch_uses_explicit_root_when_parent_differs` | Observed parent CWD instead of the intended root |
| `bridge_primary_root_is_frontend_cwd_despite_additional_roots` | Production bridge launch inherited parent CWD |
| `stdout_worker_failure_rejects_partial_launch` | Missing stdout worker accepted as successful construction |
| `stderr_worker_failure_rejects_partial_launch` | Missing stderr worker accepted as successful construction |
| `eof_retires_and_blocks_next_write` | Dead frontend retained five owned resources |
| `explicit_retirement_is_idempotent_and_closes_resources` | Retirement retained five owned resources |
| `protocol_poison_uses_coordinated_retirement` | A1 poison retained pipes/workers |
| `timeout_retires_without_reusing_dead_transport` | Original timeout replaced by dead-pipe retry error |
| `retirement_releases_full_stdout_queue` | Queue-blocked reader not joined |

The queue fixture observes frame 257 before send, with no request-side receive:
256 frames are already queued. Its readiness gate therefore establishes the
full-queue condition rather than assuming it from process startup. This gate
was replayed against the pre-A2 implementation and failed at the worker-join
assertion, then passed with the A2 fix.

Regression assertions in `src/tests/cbm/frontend_lifecycle.rs` were authored
before production implementation, observed RED, stashed with the registration
and test-only hooks, restored and verified byte-identical for GREEN. Private
fixture adapters were adapted to the new owner layout, without changing the
regression assertions. Fixture compilation/setup mistakes were corrected and
are not counted as defect RED evidence.

## GREEN evidence

**PROVEN BY EXISTING TEST:** final authorized A2 command passed all 15 selected
tests: the nine defect regressions, three preservation baselines, two structural
launch-spec baselines, and the pre-existing caller-verification test.

The missing-worker cases additionally verify that the directly spawned fixture
process is gone after launch failure. The explicit-retirement case checks joined
workers, no remaining owned resources, unchanged request counter after a rejected
subsequent call, repeated retirement and Drop. An independent fake frontend
continues answering after retirement/Drop of another session.

The additional-root baseline records production `query_graph_scoped` requests
for both canonical project IDs and observes exactly one frontend launch. The
new launch-spec baselines establish canonical deterministic values (including
relative executable input) and invalid-root rejection before spawn. These are
structural coverage, not manufactured defect RED claims.

| Authorized preservation / lint command | Result |
| --- | --- |
| `cargo test --locked -p clean-ctx --lib --all-features -j 4 cbm_a0_` | 2 passed |
| Same test command, filter `cbm_a1_` | 8 passed |
| Same test command, filter `cbm::tests::query_wire::deterministic` | 16 passed |
| Same test command, filter `cbm::tests::integration` | 8 passed |
| `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings` | Passed, no warnings |

Only local fake process fixtures ran, on Linux. No live CBM server/daemon,
operator harness, standalone binary build or full repository test suite ran.
Windows fixture paths are present but were not executed in this environment.

## Ownership and cache/environment result

Clean-CTX owns one direct frontend Child, its stdin writer, stdout receiver,
stdout reader and stderr drainer. It does not own the account daemon, account
locks/configuration, persistent provider store, other sessions/subscribers or
installation. There are no process-tree kills, account shutdown commands or
direct daemon IPC operations in this retirement path.

**PROVEN BY CURRENT CODE:** the pinned provider bootstrap separates daemon
stdio/lifetime from the frontend (`src/daemon/bootstrap.c`: Windows detached
spawn without handle inheritance; POSIX null stdio). This supports the owner
boundary; it is not live deployment acceptance evidence.

**PROVEN BY CURRENT CODE:** obsolete private `CBM_CACHE_DIR` injection was absent
from the production launch path; none was added or removed. Environment remains
inherited, including an explicit account cache override. Provider canonical-cache
admission remains the provider/A3 acceptance responsibility. Clean-CTX's own
`cache_scope`/`cache_db_path` graph-result cache settings are unchanged; they do
not configure the provider daemon cache root. No account mutation, installation,
update, new cache cohort or cache takeover occurred.

## Limitations and next gate

The existing five-second cleanup bound is retained and shared by child reaping
and worker completion. Verified fixture paths fully reap/join. If a kill/reap
fails or a pipe/output consumer keeps a worker blocked past that bound, unfinished
thread handles are released, cleanup is explicitly reported uncertain in status
and diagnostics, and the client remains terminal. No successful cleanup or daemon
cancellation is inferred in that case; global termination is never attempted.
Synchronous stdin writes are unchanged: this phase does not claim an end-to-end
bounded write attempt. Per-frame limit and channel capacity are unchanged; no
aggregate queue-byte budget is newly imposed.

Spawn plus required worker construction still is not admission/usability proof.
The legacy spawned `Available` behavior awaits A3. Replacement, half-open recovery
and semantic replay remain A4. No live daemon compatibility, Phase B or Issue 142
completion is claimed. Shared background indexing ownership is unchanged.

A2 COMPLETE — READY FOR A3 AUTHORIZATION
