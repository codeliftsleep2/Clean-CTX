# Issue #157 Angular module provenance live MCP acceptance

Agent: Agent MaxHeadRoom

Status: authored; **operator/live verification NOT RUN**. The operator reported
full repository gates green before harness authoring. This is a reusable operator
verification asset, not Rust regression coverage, a CI result or Claude field
acceptance. Deferred Angular coverage gaps remain outstanding upgrade work.

## Operator commands

Run from the repository root, using an explicitly rebuilt all-feature binary:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/issue_157_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

On other platforms pass `./target/debug/clean-ctx`. Neither command was run by
the agent. The runner never builds or installs anything. It uses Node built-ins
and the established MCP stdio transport. Source edits and builds must finish
before starting it; stale binary timestamps or changed input hashes reject PASS.

Return the printed summary/evidence directory instead of the entire RPC trace.

## Scenario matrix: 27 fresh sessions

| Sessions | Oracles |
|---|---|
| 10 | Direct and imported-array controls, cold target-first, contributor-query-first, unrelated-first and repeated query. Every positive query requires Shared Directive; fresh orders must match their nonempty control |
| 4 | Direct/array leaf content mutation or deletion, Pipe restoration and Directive restoration with unchanged target bytes |
| 4 | Direct/array module-first and contributor-first batches after leaf mutation; batch IDs/order/status, current membership and repeated query |
| 2 | Concurrent cold requests correlated by transport response IDs, plus warm repeat |
| 2 | Existing imports and exports relationship kinds survive declaration-kind mutation |
| 2 | Imported array empties or disappears, then restores membership |
| 3 | Named import alias, missing declaration appearance, type-only import exclusion |

Each scenario owns a fresh synthetic workspace and server process. CBM, proxy
auto-start and persistence are explicitly disabled. tools/list discovers the
registered schema without semantic prewarming. Cold scenarios do not manually
publish contributors. No refresh tool or target edit is used after mutations.
The report retains fixture sources/config, mutation history and the target hash.

Membership assertions require the expected relation, exact object names/kinds,
Angular subject/object domains and the asserting module provenance. They preserve
the existing producer's file attribution rather than claiming canonical imported
declaration identity. Coverage must remain lower-bound, source_complete=false,
omitted_possible=true. The advertised schema is captured for diagnosis; this
driver does not claim general JSON Schema validation. Empty expected results
are restricted to negative/retraction scenarios; order comparisons require
successful nonempty controls.

## Evidence and limits

Generated artifacts are kept under:

```text
target/live-acceptance/issue-157/<timestamp-pid>/
  report.json
  <scenario>/workspace/...
  <scenario>/rpc.jsonl
  <scenario>/stderr.log
  <scenario>/workspace-query-schema.json
```

Independent sessions continue after a scenario fails to collect diagnostics in
one run. PASS requires all 27 sessions finished, every assertion passing, no
transport/tool/shutdown errors, unchanged input hashes and no interruption.
The reused transport allows 120 seconds per request; the runner uses the existing
#155/#156 20-minute session deadline convention. These are harness bounds, not
production timeout changes. Latency and discovery diagnostics are observations,
not compiler work counts or proof of single-flight multiplicity.

The harness cannot deterministically place a mutation inside the final server
publication guard. The tracked Issue #157 Rust supersession regressions remain
the authority for that controlled case. Durable storage/currentness is covered
by the tracked Rust test; persistence/reopen is not exercised by this harness.
CBM equivalence and proprietary real repositories are not exercised. Nested
imported spreads, re-export chains, expressions, package/tsconfig resolution and
broader Angular authority remain deferred coverage work; they are not declared
complete or permanently excluded by this verification scope.

Optional syntax-only checks, which do not start the binary:

```powershell
node --check verification/live-acceptance/issue_157_live_acceptance.mjs
node --check verification/live-acceptance/issue-157/scenarios.mjs
```
