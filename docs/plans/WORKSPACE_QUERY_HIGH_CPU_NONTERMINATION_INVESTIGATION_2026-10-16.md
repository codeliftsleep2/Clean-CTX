# Workspace Query High-CPU Nontermination Investigation — 2026-10-16

**Investigator:** Agent MaxHeadRoom  
**Status:** Narrow amplification repair implemented; focused RED→GREEN, rebuilt-binary MCP stdio verification, and the complete authoritative repository gate passed. Independent timeout/cancellation disposition remains a release blocker.

## Amplification Mechanism

The production path was:

`workspace_query` batch → `find_entities` preparation → `hydrate_workspace_index_for()` → candidate loop → `compile_candidate_for()` → semantic publication → `reconcile_csharp_after_authority_change()` → `reconcile_csharp_exact_calls()`.

Every published C# candidate synchronously initiated a global C# reconciliation. One reconciliation compiles every admitted C# file twice: first to reconstruct complete declaration authority, then to recompile each projection against that complete authority and resolve exact calls. For `P` published candidates and `C` admitted C# files, this contributed approximately `P × 2C` compilations, in addition to candidate compilation and repeated controller catalog parsing. The observed sustained Clean-CTX CPU with idle CBM is consistent with this finite but pathological amplification.

## Reconciliation Ownership Before Repair

`compile_candidate_for()` owned both one candidate's publication and the global reconciliation side effect. That local helper lacked the acquisition-cycle boundary needed to know that sibling candidates were about to publish. It therefore repeated repository-wide work after each evidence atom.

## Reconciliation Ownership After Repair

Candidate publication now returns an explicit `PublishedCSharp` outcome when an actual committed publication changes C# authority. `hydrate_workspace_index_for()` aggregates that outcome across its candidate loop and invokes `reconcile_csharp_exact_calls()` once after all candidate publications finish.

The test-only `compile_candidate()` wrapper is the only other caller of `compile_candidate_for()`. It retains its prior standalone behavior by reconciling a `PublishedCSharp` result itself. No production publication path outside the hydration cycle lost reconciliation.

The governing boundary is: global reconciliation belongs to the completed acquisition generation, not to each publication contributing to it. Within one hydration cycle, zero relevant publications cause zero reconciliation; one or more relevant publications cause one reconciliation.

## Structural Regression

`red_controller_batch_reconciles_csharp_once_after_candidate_publication` exercises three controller `find_entities` items plus one controller `entities_in_file` item. Test-only thread-local instrumentation counts entry into `reconcile_csharp_exact_calls()` without introducing production state or synchronization.

The unchanged implementation produced the required RED evidence: expected `1`, observed `3`. After the narrow repair, the exact same regression passed GREEN unchanged. The test also repeats the request against warm/current state and requires a count of `0`, proving that discovery or selection alone cannot trigger reconciliation.

## Generation/Completion Ordering

Each candidate still commits through its existing semantic publication ticket. The hydration owner records whether any committed publication requires C# reconciliation, completes the candidate loop, and then performs reconciliation. Only after reconciliation succeeds does it mark discovery scopes complete. A reconciliation error returns from hydration before discovery completion is recorded or query evaluation begins.

Failed candidate publication still makes the hydration report partial. Successful C# publications are reconciled even if a sibling publication failed, while the cycle remains retryable because discovery completion is not committed.

## Second-Pass Stability

The batch's first preparation pass performs missing candidate publication and the single required reconciliation. The second indexed-query preparation pass uses the same request-local hydration results and the reconciled WorkspaceIndex generation. With unchanged source hashes, requested fidelity, and current semantic dependencies, it publishes no candidate and triggers no reconciliation. The second pass remains intact.

## Two-Pass C# Compilation Finding

**Classification: REQUIRED DISTINCT AUTHORITY PASSES.**

The first reconciliation pass reconstructs declaration authority from every admitted C# source. The second recompiles caller-bearing projections against that complete authority so exact calls can be established or revoked independent of file order. Removing either pass would weaken the existing exact-call completeness contract and is outside this repair.

## Catalog Reparse Finding

Controller refinement still calls `Catalog::build()`, which reparses indexed C# source snapshots. Batch preparation and controller ancestry traversal can therefore perform repeated full-catalog construction. This remains bounded but potentially expensive after the primary amplification repair. Persistent catalog caching, incremental catalogs, parser reuse, and controller shortcuts were deliberately excluded to preserve causal attribution and avoid an unauthorized architectural expansion.

## Windows Edit-Safety Status

The initial workspace context used the non-canonical spelling Windows `Get-Item` and `fsutil reparsepoint query` found no reparse point on the Desktop or repository path, but the approved patch helper rejected the resolved path. After the workspace was reopened, the same approved patch mechanism succeeded. No reparse safety was disabled or bypassed.

## Timeout/Cancellation Status

Request execution timeout and cooperative cancellation remain independent and unimplemented. The dispatcher has queue-send timeout and slow-request observation, but no caller-visible execution deadline that cancels compilation/reconciliation and returns a structured MCP failure. Its policy, cancellation propagation, partial-publication semantics, error shape, and post-cancellation server usability require separate architectural approval.

## Runtime Verification Required

The unchanged focused command passed GREEN after first producing the intended RED failure:

```powershell
cargo test --all-features mcp::tool_handlers::query::tests_batch_preparation::red_controller_batch_reconciles_csharp_once_after_candidate_publication -- --exact
```

That regression produced the intended RED count of three reconciliations and
then passed GREEN unchanged after the repair. After rebuilding the production
binary, run the tracked MCP stdio harness:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-BatchQueriesLive.ps1
```

The harness exercises the controller-heavy four-item batch cold and warm,
records captures under `target/workspace-query-batch-verification/`, and checks
ordered semantic results plus warm acquisition reuse. It is operator evidence,
not a substitute for the tracked reconciliation-count regression or final CI
gate.

Operator result: **PASS** after rebuilding the all-feature `clean-ctx` binary.
The controller-heavy cold batch completed with all expected results, and the
warm repeat preserved semantics without reported discovery/compilation work.
This is recorded as MCP stdio production-path evidence, separately from the
tracked Rust RED→GREEN result.

The user also reported the complete Final Verification Gate from
`docs/agent/verification.md` as **PASS**, including formatting, zero-warning
Clippy, the all-target/all-feature workspace test suite, file-size guard tests,
the active-file size guard, UTF-8 validation, and the Rust encoding gate.

After focused GREEN, run the authoritative Final Verification Gate from `docs/agent/verification.md`. No live Claude pilot should occur until local deterministic verification is complete and remaining field-only questions are consolidated.
