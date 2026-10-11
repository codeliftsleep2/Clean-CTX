# Current CBM upgrade — A0/A1 implementation evidence

Branch: `feature/current-cbm-pinned-upgrade`.
Verified parent: `4c1627fcefe698977447dcee4e88830c20c7f422`.
Authority source: CBM `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
Scope: A0 extraction/freeze and A1 correlation/bounded framing only.

## Production changes

A0 (`87cbd04`) carries the three reconciliation/planning documents and extracts
frontend transport, envelope framing, and typed wire helpers from `client.rs`.
Parsed and raw calls retain one shared receive path; public APIs are unchanged.

A1 validates JSON-RPC version, object envelope, exclusive result/error, error
shape and exact pending numeric request ID. Valid notifications are ignored
with a diagnostic without extending the request deadline. Wrong/missing IDs,
null-ID startup errors, unsupported server requests and malformed envelopes are
protocol failures. Raw replies are returned only after this validation.

The stdout worker now checks the existing 4 MiB frame limit while reading bounded
chunks, before retaining a whole line. Newline termination and UTF-8 are required.
Escaped tree-content newlines are preserved. Invalid framing/correlation marks
this frontend terminal and kills only the directly owned frontend process;
subsequent sends cannot reuse its pipes. Shared-daemon termination is not added.

## Observed verification

All commands used `--locked -p clean-ctx --lib --all-features -j 4` except
Clippy, which used `--all-targets --all-features -j 4 -- -D warnings`.

| Authorized command/filter | Observed result |
| --- | --- |
| `cargo test … cbm_a0_` | 2 baseline tests passed after extraction |
| `cargo test … cbm_a1_` before fix | 7 new regressions failed; 1 pre-existing caller-verification test passed |
| `cargo test … cbm_a1_` after fix | All 8 passed, including all 7 new regressions |
| `cargo test … cbm::tests::query_wire::deterministic` | 16 passed |
| `cargo test … cbm::tests::integration` | 8 passed |
| `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings` | Passed |

New regressions cover wrong/missing/string/float/null IDs, notification separation,
startup-error rejection, malformed envelopes/errors, pre-consumption cap
checking, unterminated EOF, exact-limit/over-limit frames and invalid UTF-8.
The observed RED test additions were stashed during implementation and restored
byte-identically for GREEN; assertions were not weakened.

## Evidence limits and next gate

**PROVEN BY EXISTING TEST:** deterministic envelope/framing behavior and the
listed baseline/projection/integration contracts.

**PROVEN BY CURRENT CODE:** both parsed and raw calls route through the validated
receive path; invalid frames retire the directly owned frontend and prevent
subsequent pipe writes. No live process-lifecycle claim is made from these
helper tests.

No full suite, live CBM server, daemon admission trial or MCP user-verification
harness ran. This is not final pinned-CBM compatibility acceptance.

Stop after A1. A2 requires separate authorization: explicit launch root/context,
mandatory reader/drainer ownership and coordinated retirement. Initialization,
usable admission and replacement/retry changes remain later slices (A3/A4).
Existing timeout/dead-pipe retry behavior, bounded stderr handling and aggregate
queue ownership are not repaired by this A1 change. Phase B wire/schema/paging
work and Issue 142 hydration/discovery work remain deferred.
