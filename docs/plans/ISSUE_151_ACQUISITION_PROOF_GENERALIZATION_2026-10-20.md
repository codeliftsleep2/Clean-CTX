# Issue #151: acquisition proof generalization

B4a source disposition: The focused acquisition contract owns relocated 151-A/B and lifecycle clauses. The remaining original NOT RUN, GREEN and operator-definition statements retain their chronological subjects; no current field acceptance or full final gate is inferred.

Agent: MaxHeadRoom
Status: focused Rust suites user-reported GREEN; full gate and live verification pending.

## Verdict and root cause

PROVEN BY SOURCE: `find_entities` previously used ordinary declaration-mode
discovery with `CleanAuthoritative` hydration. The shared proof infrastructure
was instantiated only for incoming builtin TypeRef relationships. A discovery
cache hit could suppress inventory production without supplying a compatible
acquisition proof. This is a missing certified acquisition capability, rather
than a missing rendering of a compatible proof already in the response.

The maintainer approved generalizing the existing acquisition architecture for
declaration mode. Ordinary occurrence search retains its broader, uncertified
producer obligation and reports a lower bound.

## 151-A scope contract

Current normative owner: [AUTH.DECLARATION_ACQUISITION](../architecture/contracts/declaration-acquisition.md). Original section bytes, chronology and enforcement attribution remain available at commit `0d8437cc`.

## 151-B public meaning and authority

Current normative owner: [AUTH.DECLARATION_ACQUISITION](../architecture/contracts/declaration-acquisition.md). Original section is retained in the baseline revision.

## Shared production lifecycle

Current normative owner: [AUTH.DECLARATION_ACQUISITION](../architecture/contracts/declaration-acquisition.md). Original section is retained in the baseline revision.

## Regression evidence and preservation

PROVEN BY TEST (user-reported RED only):
`find_entities_declarations_require_proof_after_ordinary_discovery_and_on_warm_reuse`
failed on the intended `source_complete` assertion: false versus true. The exact
regression was stashed before implementation and restored afterward unchanged.
The user subsequently reported GREEN for the focused Issue #151 contract,
declaration acquisition, shared acquisition, and reverse-edge completeness
suites after correcting the new test module's registration boundary. This is
user-reported Rust test evidence; the agent did not execute those commands.

Additional authored tests cover full-unit acquisition of a source lacking the
query spelling, publication failure/retry, proof invalidation, missing roots,
and capability/requirement separation. The focused suites are user-reported GREEN.

The existing exact incoming TypeRef query path, provider-neutral proof
validation, and publication/cache ownership remain the preservation authorities.
C# resolution campaigns and NgRx producers were not edited. Their runtime
preservation is NOT VERIFIED until the relevant suites and final gate run.

## Verification and disposition

Fast file-scoped formatting and `git diff --check` ran. Compilation, tests,
Clippy, full formatting gate, encoding guard, file-size scripts, and live
verification were NOT RUN by the agent. Follow `docs/agent/verification.md` for
the final gate. A Claude pilot remains the last field gate, after local evidence
has been collected and any remaining field questions can be batched.

Issue #151 remains open pending GREEN, preservation testing, the final gate, and
any applicable field acceptance. No commit or issue closure is claimed.

## Consolidated operator acceptance

`verification/issue-reconciliation/Invoke-Issue151Acceptance.ps1` is now the
tracked stdio acceptance definition. Its 15 cases cover three provider modes,
fresh and ordinary-discovery-first histories, warm reuse, answer scope,
mixed-language declarations, batch/channel parity, edit/delete refresh, missing
root recovery, and unreadable-source recovery. It reuses `PhaseBSession.ps1`.
PowerShell parsing passed without executing the harness. Live results remain
NOT RUN. The invocation and evidence rules are documented in the verification
directory README; #144/#148 preservation harnesses use the same binary.

```architecture-graph
node: DOC-448
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Declaration acquisition proof generalization and attributed evidence
track: REPOSITORY
artifact: docs/plans/ISSUE_151_ACQUISITION_PROOF_GENERALIZATION_2026-10-20.md
implements:
  - AUTH.DECLARATION_ACQUISITION
note: Historical implementation and source-attributed evidence only; no new execution, independent language certification, reclosure or banking.
```
