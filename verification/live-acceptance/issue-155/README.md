# Issue #155 MCP dependency convergence acceptance

Agent: Agent MaxHeadRoom

Latest result (2026-10-20): operator-run PASS, 392/392 checks. Receipt
`target/live-acceptance/issue-155/2026-10-20T22-41-41-624Z-1364/report.json`
records a complete run with 19 finished sessions and zero errors; inspected by
the agent. No runtime was executed by the agent. This supersedes the failed-run
and pending-rerun statuses below. It is local binary/stdio acceptance, not a
Rust/CI result, final verification gate result or Claude field acceptance.

Status: first operator run reported FAIL (23/24 checks) in
`2026-10-20T22-31-43-671Z-16928`. Controller file cold/warm passed; the next cold
Controller edge response failed registered coverage validation because production
inserted `evidence_requirements: null`. Production now uses non-inserting access
for that optional field, with a new tracked Rust regression. The schema oracle
is unchanged. Focused regression and rebuilt harness rerun are pending.

Second operator run `2026-10-20T22-37-11-974Z-21604` reported FAIL (347/348
checks). Same-process Foundation restoration returned an unproven dependency
certificate with `dependency_source_generation_changed`. Frontier reads had
retired the request's inventory observation. Shared acquisition now rebuilds
inventory/frontier proof after this retirement, with a new transitive context
mutation regression. Focused GREEN and rebuilt MCP rerun are pending; harness
assertions are unchanged.

This operator-run harness
checks the real binary over MCP stdio. It is separate from the focused Rust
GREEN reported by the operator and cannot replace regressions or the final gate.

## Run

From the repository root, using an explicitly rebuilt all-feature binary:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/issue_155_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

On other platforms supply `./target/debug/clean-ctx`. The driver rejects a binary
older than Rust source/build inputs and records their hashes. It does not build
anything. Node built-ins suffice; no installation or network credentials are
required. Do not run concurrently with source edits or a rebuild.

Optional operator-owned syntax checks, without launching the MCP binary:

```powershell
node --check verification/live-acceptance/issue_155_live_acceptance.mjs
node --check verification/live-acceptance/issue-155/scenarios.mjs
```

## Scenario matrix

The run uses 19 sequential fresh MCP processes, with isolated synthetic
workspaces except for four deliberate durable restart sessions:

| Sessions | Observation |
|---|---|
| Eight cold queries | Each Controller, Hub, MapperProfile and DbContext is first requested independently through file entities or forward edges. Two source inheritance hops must be acquired automatically. No dependency is explicitly published. Warm repetition must retain identical facts and proof generation. |
| Three attribute cases | Source method suffix shadow and class attribute shadow cannot grant HTTP/Controller authority. Competing framework/source identity remains partial with an unproven proof and nonempty reasons. Cold/warm observations agree. |
| Two mixed query batches | Forward/reverse item order mixes edges, owner file, another dependency chain and an unrelated TypeScript lookup. Successful C# proofs must match subsequent unchanged observations of their respective owners. |
| Two context batches | Forward/reverse context batches with distinct source chains preserve item order, succeed and return complete proofs matching each owner's subsequent unchanged observation. |
| Four durable sessions | Context-first cold/warm acquisition; unchanged process restart; source mutation while stopped followed by another restart; same-process restoration/retraction/restoration without refresh; final restart verifies restored facts. SQLite evidence must exist and be nonempty. |

All tool calls pass an explicit `workspaceRoot`. Fresh workspace configuration
disables CBM, proxy auto-start and (except durable sessions) persistence. No
explicit base-file context call, refresh, provider search or query prewarming
precedes an initial owner request. Later verification queries run only after
the context request has already returned its complete certificate.

Coverage is validated against the schema obtained from that process's
`tools/list`, including dependency status, bounded reasons and complete/unproven
proof alternatives. The small validator supports the current coverage schema's
assertion keywords and fails on an unknown keyword; it is not a general JSON
Schema implementation. Semantic assertions independently require facts and
truthful completeness; schema validation alone cannot pass a scenario.

## Evidence and failure behavior

Each run retains evidence under:

```text
target/live-acceptance/issue-155/<UTC timestamp>-<pid>/
  report.json
  <scenario>/rpc.jsonl
  <scenario>/stderr.log
  <scenario>/workspace/        generated sources/config/runtime data
  durable/workspace/          retained across durable sessions
  durable/<phase>/rpc.jsonl
  durable/<phase>/stderr.log
```

The report includes binary/input hashes, exact fixture sources/configuration,
mutations, per-session completion, assertions and errors. RPC logs retain full
requests/responses. Failure stops the run, preserves evidence and exits nonzero;
empty arrays cannot substitute for missing/malformed response arrays. PASS
requires the full matrix to finish, every assertion to pass, and input/binary
hashes to remain unchanged. An interrupted/partial run is FAIL.

The unchanged #153/#154 transport is reused. It bounds requests to 120 seconds,
sessions here to 20 minutes, and shutdown to six seconds with child termination.
These are harness bounds, not production policy. Ctrl+C records failure and
terminates the active child. The harness never modifies repository source or
deletes evidence. Its initialize client name remains the reused transport's
`csharp-compilation-identity-153-154`; the report identifies this #155 campaign.

## Proof limits

This establishes observable local binary/stdio acceptance only after a complete
operator-reported PASS. The mtime freshness check is not build attestation.
Unrelated TypeScript lookup is included for batch mixing, not a TypeScript
completeness certificate. EF coverage is limited to the original #155 ancestry
case; EF configuration acquisition and the other sibling campaigns are excluded.

The external context response exposes a dependency certificate, not its internal
semantic edge snapshot. The harness requires that certificate before querying
facts; the later query can itself reconcile semantics. Exact retained/durable
dependency hashes and invalidation are therefore proved by the authored Rust
regressions, not inferred from a later MCP query. SQLite existence plus restart
does not prove a durable cache hit or inspect its internal certificate contents.

Proof generation hashes are owner-specific requirement certificates, not shared
publication counters. The harness compares each item with its own subsequent
unchanged observation, never equates distinct owners' certificates.

Mixed batches exercise real preparation in both orders. They cannot force the
precise mid-request supersession controlled by the Rust test-only hook. Comparing
publication generations checks exposed surviving state; it does not fabricate
an internal race or prove synchronization grants semantic authority.

After local acceptance and the authoritative final gate, consolidate remaining
field questions into the last Claude pilot. Keep proprietary source local and
return only sanitized results. Report this harness separately from Rust/CI
results; do not close #155 on harness authorship alone.
