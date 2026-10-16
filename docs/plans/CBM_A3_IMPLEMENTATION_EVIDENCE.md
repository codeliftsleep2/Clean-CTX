# A3 implementation evidence — pinned frontend admission

Scope: A3 only on `feature/current-cbm-pinned-upgrade`, based on A0–A2
(`87cbd04`, `cb005bc`, `14d05cc`) and parent
`4c1627fcefe698977447dcee4e88830c20c7f422`. Current pinned source was verified at
`82b1ec908f975354ceb4bac8a8bfc594942a9bc4` before implementation.

## Source contract and production integration

**PROVEN BY CURRENT CODE:** [pinned admission contract](CBM_A3_PINNED_ADMISSION_CONTRACT.md)
records exact source locations, provider startup/session topology, supported
negotiation and the seven required tools with their existing consumers.
The provider performs account daemon admission before serving its stdio
frontend. Clean-CTX owns that frontend and pipes, not the shared daemon.

The production adapter now follows:

transport constructed (degraded) → correlated `initialize` → validate protocol,
identity and tools capability → no-ID `notifications/initialized` → correlated
`tools/list {}` → validate complete minimum registry → Available → indexing/tools.

`client/admission.rs` owns the gate; `transport.rs` shares the A1 bounded frame
and ID classifier between admission and tool RPC. Required tools must appear
exactly once with an object input schema. Missing/invalid capabilities and
unexpected paginated listings reject admission. Optional newer tools are not
required. `client.rs` prevents unadmitted calls or circuit state from publishing
Available. `bridge/lifecycle.rs` admits before retaining the usable client or
starting Clean-CTX indexing; existing native state propagates degraded status
and preserves optional-provider fallback. No production MCP schema changed.

Admission failure uses A2 retirement. Null-ID `-32001` refusal remains distinct
from correlated RPC error, malformed protocol and transport exit. Even when the
frontend exits before the initialize write, queued evidence passes through the
same A1 classifier before receiver retirement. Provider text is retained without
inventing a conflict subtype or automatic account repair.

## RED → GREEN evidence

**PROVEN BY LOCAL FAKE-FRONTEND TESTS**, not live provider acceptance:

- Initial `cbm_a3_` RED: 10 intended admission regressions failed; two baselines
  and one existing deterministic caller test passed (13 selected).
- Initial GREEN: all 13 passed with the original regression assertions retained.
- Subsequent implementation audit RED: the added exited-refusal regression
  failed with generic connection loss instead of its queued structured cause;
  the other 13 passed. This is an audit defect, not a pre-A3 failure claim.
- Final GREEN: **14 passed**, zero failed. The added audit assertion is unchanged.

The A3 cases cover spawn versus availability; exact initialize/notification/list/
tool sequence and IDs; wrong ID; notification-only timeout; refusal and owned
retirement with another frontend surviving; exit during initialization; each of
all seven absent required tools; invalid version, identity or capability;
bridge indexing order; failure rather than successful-empty projection;
notification-before-response compatibility; independent native heuristics;
and structured refusal queued before frontend exit.

The filter also selects `red_cbm_a3_live_order_by_shape_filters_one_argument_calls`:
despite its historical name it is a deterministic C# IR fixture, not a live CBM
run. The native baseline checks heuristics, not a live MCP rendering claim.
A2 transport fixtures explicitly stub already-admitted state to isolate their
original lifecycle contracts; A3 fixtures invoke the real production gate.

Preservation checks (same locked library/all-features command, four jobs):

| Filter | Passed |
| --- | ---: |
| `cbm_a0_` | 2 |
| `cbm_a1_` | 8 |
| `cbm_a2_` | 15 |
| `cbm::tests::query_wire::deterministic` | 16 |
| `cbm::tests::integration` | 8 |

`cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`
passed. No full suite, standalone build or live CBM server ran.

## Limits and next gate

Admission shares one existing receive deadline across initialization and tool
listing. Synchronous stdin writes remain the documented A2 limitation; this is
not an end-to-end bounded-call guarantee. Existing 4 MiB frame cap, 256-frame
queue and five-second cleanup observation are unchanged; no new aggregate
capture budget, timeout, retry or circuit policy was introduced. Cleanup
uncertainty remains explicit. Provider-owned auto-indexing may start inside
initialize; the gate controls Clean-CTX's indexing, not all daemon work.

**NOT ESTABLISHED:** live daemon reuse, exact installed artifact, account cache
admission, Windows live retirement, watcher freshness and cold/warm completeness.
Initialize does not expose exact build/ABI/cache identity. Those cannot be
promoted from source inspection or fake processes to live proof.

[Prepared bounded live acceptance cases](../../verification/cbm/A3_ACCEPTANCE.md)
are NOT RUN and remain the later live acceptance gate. No replacement session,
re-admission, Phase B decoder migration or Issue 142 hydration work was added.

**A3 COMPLETE — READY FOR A4 AUTHORIZATION**

A4 remains unauthorized. Readiness means local A3 contracts are established;
it does not waive the later pinned-artifact/live acceptance requirement.
