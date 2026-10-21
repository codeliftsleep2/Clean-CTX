# Issue #156 EF model/configuration live MCP acceptance

Agent: Agent MaxHeadRoom

Status: authored; **operator/live verification NOT RUN**. JavaScript syntax-only
checks passed. The operator reported the repository gates green before this
harness was added. This package is an operator verification asset, not a Rust
regression, CI result, or substitute for the Final Verification Gate.

## Run

From the repository root, build an explicit all-feature binary and run the driver:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/issue_156_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

On other platforms pass `./target/debug/clean-ctx`. The driver never builds the
binary. It rejects a binary older than Rust/build inputs and hashes it and the
source/harness inputs before execution. A successful run also requires unchanged
input hashes afterward. Rebuilds and source edits must finish before starting it.
The driver uses Node built-ins and the existing MCP stdio transport; no package
installation, credentials, or external service is required.

Both commands are operator-owned and have NOT RUN by the agent. The runner
prints one PASS/FAIL summary and an evidence directory. Do not paste the entire
RPC capture into the terminal; return the summary or the directory path.

## Scenario envelope: 28 isolated sessions

Every session launches the actual supplied binary with a fresh synthetic workspace
and its own process/index. Each workspace explicitly disables CBM, proxy auto-start,
and persistence. Startup tools/list is schema discovery, not semantic prewarming.

| Sessions | Semantic oracle |
|---|---|
| 15 query-order sessions | Direct Entity<T>, explicit configuration, and supported local-project assembly application, each with dependency-files positive control, cold target-first, dependency-names-first, unrelated-first, and repeat-first orderings |
| 3 lifecycle sessions | Direct entity disappearance/identity change/restoration; configuration retargeting/deletion/restoration; assembly expansion/contraction and project membership changes |
| 2 batch sessions | Forward and reversed mixed batches preserve request order and match a subsequent target observation |
| 2 additional-root sessions | Reversed additional-root order with previously queried same-terminal-name declarations in different namespaces cannot replace the intended entity |
| 1 concurrent session | Two outstanding cold RPC requests and a warm repeat return equivalent authoritative facts, correlated by response ID |
| 1 cold reverse session | Entity reverse lookup acquires the context/configuration authority and returns its supported HasEntity |
| 1 alias session | Entity and configuration type aliases resolve to the intended namespace/file |
| 1 ambiguity session | Duplicate non-partial entities fail closed and keep dependency proof unproven |
| 1 missing session | An unavailable model entity does not create a relationship or complete dependency proof |
| 1 marker/project session | Marker identity changes and ambiguous project evidence retract assembly relationships and restoration restores them |

The first control for each relationship family must produce the expected nonempty
HasEntity, including the resolved entity and context files. Fresh-state results
must match that control after normalizing workspace-relative provenance. This
cannot pass by comparing two empty results. Lookalike classes live in separate
namespaces and files. Neither fixture setup nor the driver manually publishes
the dependencies for a cold query; controls use legitimate public workspace queries.

Mutation scenarios record every replacement/deletion, preserve the context's
byte hash, and query again in the same process without refreshing or editing the
context. An empty, closed assembly configuration universe is distinct from a
missing explicit configuration dependency. Ordinary EF relationship coverage
must always remain lower-bound, even when internal dependency proof completes.

Coverage is validated against the schema returned by tools/list. The local
validator covers the registered coverage vocabulary and rejects unknown schema
keywords; it is not advertised as a general JSON Schema implementation. The
returned schema is saved for diagnosis rather than silently accepting drift.

## Evidence and failure handling

Generated state stays under a unique timestamp/PID directory:

```text
target/live-acceptance/issue-156/<run>/
  report.json
  <scenario>/workspace/...
  <scenario>/rpc.jsonl
  <scenario>/stderr.log
  <scenario>/workspace-query-schema.json
```

The report contains binary/source/harness hashes, synthetic fixture/config data,
session status/timing, named assertions, mutations, query latency, coverage and
discovery diagnostics, and failures. Independent sessions continue after a case
fails so one operator run collects the remaining diagnostics. Interruption,
transport/tool/schema errors, failed shutdown, stale/changed inputs, or fewer
than 28 finished sessions prevent PASS. Existing transport limits are 120 seconds
per request; the driver adds the same 20-minute session deadline used by #155.
These are operator harness limits, not changes to production timeouts or policy.

Query latency is observed wall time. Discovery.compiled describes the reported
hydration diagnostic, not total EF parse/compile/reconciliation work. Do not infer
zero compiler work or single-flight multiplicity from these fields. The tracked
Rust work-count and follower regressions remain the authority for those claims.

This harness cannot deterministically inject #155-style late publication inside
the server's final response guard. It verifies external batch/concurrency truth;
the unchanged supplemental Rust regression proves the controlled supersession
case. It does not exercise arbitrary assembly expressions, durable EF model-edge
ownership, provider/CBM equivalence, or proprietary repositories. EF model edges
remain query-local, so persistence is deliberately not used as an integration
oracle. This is actual binary/stdio acceptance, not Claude field acceptance.

Cheap optional syntax checks (do not launch the binary):

```powershell
node --check verification/live-acceptance/issue_156_live_acceptance.mjs
node --check verification/live-acceptance/issue-156/scenarios.mjs
```
