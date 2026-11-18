# #133 final acceptance reconciliation

**Agent:** Agent MaxHeadRoom  
**HEAD reviewed:** `6e8c8f38`  
**Date:** 2026-10-15

#133 ACCEPTANCE RECONCILED — READY TO CLOSE

## 1. Executive verdict

The original near-zero-recall defect is repaired on the strongest surviving controlled evidence. A first query after `refresh_workspace` returned 55/55 constructor consumers across five directories. The expanded ten-interface corpus returned exact native/provider, cold/warm sets: 24/24 #133 groups in each final run, covering 202 consumers, a 60-consumer first query, `withinPath`, and a matching-name implementation negative control.

Criterion 3 is now protected by a tracked 52-consumer/four-directory C# regression that starts from refresh, makes the reverse query as its first semantic query, distinguishes the legitimate matching-stem base-type control, and passed the focused all-feature command reported by the maintainer. Criterion 2's remaining cross-cutting actionable-diagnostic work is explicitly transferred to #142; no #133-specific obligation remains.

No currently recorded live-gate regression blocks #133.

## 2. Original #133 acceptance contract

[Issue #133](https://github.com/codeliftsleep2/Clean-CTX/issues/133) reported roughly 4% mean cold recall across ten C# constructor-interface queries, versus 100% after injector files were compiled. `withinPath`, indexing, and query order did not make cold results complete; refresh reintroduced the failure. Some results were matching-name implementation/base facts, not injectors.

Its explicit criteria are:

1. Cold `reverse_edges(builtin, TypeRef, <interface>)` returns every constructor consumer with recall and precision 1.0 on the ten-interface set.
2. If full hydration is not done, the response states that it is partial, explains why, and offers a completion path.
3. A regression test covers more than 50 injector files in more than three directories, queried first after refresh.

The issue preserved placeholders rather than private source, so the controlled corpus is the reproducible surrogate; unavailable private bytes are not a new requirement.

## 3. Acceptance matrix

| Obligation | Current evidence | Status and reasoning |
| --- | --- | --- |
| Repair cold near-zero recall | Inbound discovery compiles candidates; 55/55 and expanded exact corpus pass live | **SATISFIED.** The exact relation/query path is exercised. |
| Criterion 1: exact ten-interface recall/precision | 202-consumer oracle; native/provider cold/warm; 24/24 groups each | **SATISFIED** on the reproducible contract. |
| Criterion 2: partial plus reason/action | Partial/lower-bound truth exists; actionable source-read cause/action remains cross-cutting | **OWNED BY #142.** Explicitly transferred to #142 for implementation and closure accounting. |
| Criterion 3: tracked large cold regression | `workspace_query_csharp_constructor_large_cold.rs`: 52 consumers, four directories, refresh then first semantic reverse query; maintainer-reported focused test GREEN | **SATISFIED.** |
| `withinPath` | Exact 12 in-scope subjects in every expanded mode | **SATISFIED.** |
| Matching-name implementation negative | `Contract9 : IContract9` excluded from constructor subjects | **SATISFIED.** |
| Provider-disabled truth | Filesystem fallback and native cold run pass | **SATISFIED.** |
| Lifecycle/staleness support | File-owned replacement/retraction; lifecycle live PASS | **SATISFIED** supporting evidence. |
## 4. Current production path

```text
C# constructor parameter
  -> canonical C# IR (CoreOp::Param owned by constructor)
  -> ir::semantic_projection
  -> builtin/Class/<consumer> --HasConstructorParameterType-->
     builtin/TypeRef/<written type>
  -> file-owned publication into WorkspaceIndex
  -> reverse_edges chooses InboundReference discovery
  -> CBM candidate paths OR native filesystem name scan
  -> Clean-CTX path admission, source read, compile, publication
  -> exact identity resolution and scoped WorkspaceIndex reverse lookup
  -> public MCP structured result with coverage/discovery metadata
```

`src/ir/semantic_projection.rs` produces the typed relation. `src/mcp/tool_handlers/hydration.rs` selects inbound discovery, compiles candidates, and refuses to cache completion after publication failure. `hydration/filesystem.rs` is the provider-independent fallback. `src/workspace/index.rs` owns exact reverse occurrences and scope filtering. `query/edges.rs` performs preparation and lookup; `query/coverage.rs` and `query/diagnostics.rs` expose truthfulness metadata.

This is the real public lifecycle: the dispatcher invokes hydration; normal compilation publishes file-owned facts; the query consumes `WorkspaceIndex`; live stdio harnesses observe the MCP result.

## 5. 55/55 cold-probe reconciliation

The 55/55 probe directly covers the defect. It creates 55 C# constructors accepting `IEvidenceContract` across five directories, starts a provider-disabled MCP session, calls `refresh_workspace`, issues the original `reverse_edges(builtin, TypeRef, IEvidenceContract)` query, counts only `HasConstructorParameterType` subjects, and asserts 55 distinct expected consumers.

The expanded harness is stronger: fresh root/session per mode, ten interfaces, 202 consumers across five directories, 60-consumer case queried first while cold, exact set comparison, `withinPath`, and a matching-stem `Implements` negative. Final native and provider evidence each record 24/24 #133 groups.

These results establish repaired runtime behavior; the tracked large-cold regression now separately satisfies criterion 3.

## 6. Native-vs-provider authority analysis

Clean-CTX owns truth: its C# compiler produces parameter evidence, semantic projection creates the relation, and file-owned publication commits it to `WorkspaceIndex`. CBM may return candidate paths only. Those paths still undergo Clean-CTX admission, reading, compilation, and publication; no provider entity, edge, direction, count, or traversal becomes query authority.

With CBM disabled/unavailable, filesystem discovery feeds the same native pipeline. Passing native cold results establish optional-provider correctness. Fresh cold sessions query before unrelated consumer compilation and match warm exact sets, so cache warmth is not hidden truth for this corpus.

## 7. Regression/live-evidence inventory

Tracked component/public coverage includes:

- `src/tests/ir/semantic_projection.rs` — constructor parameter projection.
- `src/tests/mcp/workspace_query_csharp_constructor_types.rs` — small public forward/reverse relation, attributed parameters, and negative controls.
- `src/tests/mcp/workspace_query_csharp_constructor_large_cold.rs` — #133 acceptance regression: 52 exact constructor consumers in four directories, CBM disabled, explicit refresh, first semantic reverse query, and matching-stem base-type control.
- `src/tests/mcp/workspace_query_6.rs::red_f3_reverse_edges_discovers_every_consumer` — small exhaustive inbound discovery control.
- `src/tests/mcp/hydration_filesystem.rs::red_r5_no_cbm_reverse_edges_remain_authoritative` — provider-disabled fallback.
- `src/tests/mcp/workspace_query_publication_retry.rs` — failed publication remains partial and is retried.
- Generic scope, diagnostics, coverage, refresh, and reverse-index tests.

The focused all-feature regression command was run by the maintainer and reported GREEN after correcting the oracle to distinguish the legitimate `HasBaseType` control from the 52 required `HasConstructorParameterType` edges.

Live/operator evidence (not test/CI evidence):

- `Invoke-NativeIssueReconciliation.ps1`: 55-file/five-directory cold PASS.
- `Invoke-PostCbmInvestigation.ps1`: exact 202-consumer native/provider cold/warm corpus, 24/24 #133 groups in each final run.
- `Verify-CSharpConstructorDependenciesLive.ps1`: producer/query/negative/repeat/stale lifecycle; ledger PASS.
- `LIVE_GATE_REGRESSIONS.md`: no open #133 LGR.

No live/operator harness was rerun during final closure; the new tracked Rust regression was the only reported rerun.
## 8. #142 boundary analysis

[Issue #142](https://github.com/codeliftsleep2/Clean-CTX/issues/142) owns cross-cutting completeness metadata, refresh/index behavior, and actionable error messages across tools/languages. Shared reason codes or response-contract changes belong there and require architectural approval.

#133's cold C# recall, identity, native-authority, and large-regression obligations are satisfied. The remaining actionable-cause/completion-path portion of criterion 2 is explicitly transferred to #142 because it is the same cross-cutting response-contract concern across languages and tools. This transfer does not declare the diagnostic behavior fixed; it records #142 as its continuing owner.

## 9. Residual defects, if any

No #133-specific residual remains. The tracked large-cold regression is present and green. The incomplete-answer actionable-diagnostic concern is not fixed here and remains open under #142 by explicit transfer.

## 10. Closure recommendation

Close #133.

The original cold constructor-injection reverse-query defect is repaired, native and provider-enabled controlled corpora are exact, the public production path is traced, the required large-cold regression is tracked and green, and no current live-gate regression belongs to #133. Criterion 2's remaining cross-cutting diagnostic behavior is explicitly transferred to #142 without claiming it is fixed.

No live Claude pilot is required for closure.

## 11. Evidence references

- GitHub issues [#133](https://github.com/codeliftsleep2/Clean-CTX/issues/133) and [#142](https://github.com/codeliftsleep2/Clean-CTX/issues/142).
- `docs/plans/ISSUES_133_141_POST_CBM_RECONCILIATION.md` — 55/55 evidence.
- `docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md` — expanded corpus, exact oracle, modes, partial-read control, and 24/24 records.
- `verification/issue-reconciliation/{Invoke-NativeIssueReconciliation.ps1,Invoke-PostCbmInvestigation.ps1,README.md}`.
- `verification/workspace-query/scripts/Verify-CSharpConstructorDependenciesLive.ps1` and `verification/LIVE_GATE_REGRESSIONS.md`.
- Production/test files named in sections 4 and 7.
- Relevant history: `1e637ba9`, `4647341b`, `2986ce76`, `08f8f7e5`, `715902c6`, `93058c54`, `d3b85b6f`, and `fe7a47b1`.

## 12. Commands actually run

Agent-run work:

- `Get-Content` for the supplied task and bounded known files.
- Clean-CTX `get_cbm_status`, `graph_search`, and attempted batched `provide_code_context`; invalid focused requests failed, then bounded native reads were used for known paths.
- `gh issue view 133/142 --json`.
- Bounded `rg`, `git status`, `git log`, and `git show` inspection.
- A public GitHub page lookup missed cache; the authenticated read-only CLI result is the issue authority used here.
- Added and registered `workspace_query_csharp_constructor_large_cold.rs`; no production code changed.

Maintainer-run focused verification:

- `cargo test --all-features issue_133_large_cold_reverse_query_finds_every_constructor_consumer -- --nocapture` — GREEN (maintainer report).

Not agent-run: Cargo tests/check/build/Clippy/fmt/audit, binaries, MCP servers, live scripts, benchmarks, provider calls, indexing, refresh, or other runtime verification.
