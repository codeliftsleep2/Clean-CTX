# Current #133–#141 reconciliation after the pinned CBM upgrade

The subsequent [end-to-end investigation](POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md)
supersedes closure recommendations below: it expands #133 acceptance and finds
#141 linked-HTML entities_in_file still returns an unsupported-extension error.
The 39 passing template assertions remain valid for their tested operations.

## Authority and evidence

Current issue bodies/state were retrieved from public GitHub issue pages during
this audit. All nine are **OPEN**. The API routes were forbidden; HTML embedded
issue data supplied the actual titles/bodies. No issue or comment was changed.

The older architecture investigation explicitly could not retrieve these bodies
and mapped finding families. Its #134–#137 labels are not the actual issue titles
or acceptance criteria. This report supersedes that mapping for issue disposition.

Binary: B4 all-feature production build, SHA256
cd62c9e229f5698875832b792c5f15a22473cbeb9ffd7695ce12de76d2fe8219.
Native acceptance probes use controlled C#/TypeScript fixtures and provider-disabled
real MCP sessions. CBM adapter verification is not a substitute for native feature
acceptance. This is investigation/reconciliation only: no native production fixes.

## Issue dispositions

| Issue | Current finding | Evidence / disposition |
| --- | --- | --- |
| [#133](https://github.com/codeliftsleep2/Clean-CTX/issues/133) C# cold constructor recall | **Improved: controlled large cold probe passes.** After refresh, 55/55 constructor consumers in five directories are returned; all 55 edges are HasConstructorParameterType. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Strong candidate for resolved cold-recall subfinding. Do not close the entire issue on one interface: original ten-interface/real-repo set, withinPath, provider-enabled behavior and partial-disclosure criteria remain unestablished. |
| [#134](https://github.com/codeliftsleep2/Clean-CTX/issues/134) .NET-domain controllers/EF/tests/traversal | **Still real:** a controller two project bases below ControllerBase is not delivered as a dotnet Controller, even after publishing the whole base chain. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Keep open. EF configuration, target-verified Tests and complete traversal/cycle criteria are separate subfindings, not closed by this probe. |
| [#135](https://github.com/codeliftsleep2/Clean-CTX/issues/135) C# fidelity/DI/calls/context parameters | **Still real:** extract_di_registrations explicitly truncates the fact list to 10, without emitting the original total/omission marker there. | PROVEN BY CURRENT CODE: src/dotnet_meta/general.rs:269–313 and its extract_general caller. Keep open. No complete signature/attribute/record/partial-class/call-owner/batch-parameter acceptance is claimed. |
| [#136](https://github.com/codeliftsleep2/Clean-CTX/issues/136) apply_edit | **Still real:** valid member insertion before EditProbe.First is rejected by the syntax gate; nothing is written. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Matches the insert-before report, not a successful placement. Source still plans insert_before at record.start_byte, while declaration_span is separately available. Keep open; remaining selector/construct/message criteria require their own fixes/probes. |
| [#137](https://github.com/codeliftsleep2/Clean-CTX/issues/137) diff_commits | **Still real:** changing only int Get() to long Get() in committed C# history produces `= class DiffProbe (unchanged)` and zero member changes. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Keep open. File-level changes were observed, so the failure is member comparison, not Git setup/ref resolution. Attribute/body-neighbor/noise/loss disclosure are not inferred resolved. |
| [#138](https://github.com/codeliftsleep2/Clean-CTX/issues/138) Angular DI/member forms | **Still real:** pipe constructor injection and class inject(AuditApiService) do not reach the requested semantic Injects edges. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Current semantic.rs explicitly admits Component/Service/Directive for these edges and excludes Pipe; signal extraction existing internally is not proof of public delivery. Keep open. Functional/token/provider/signal/decorator criteria remain distinct. |
| [#139](https://github.com/codeliftsleep2/Clean-CTX/issues/139) Angular/TS parsing/modules/routes/entities | **Still real:** decorated non-exported OwnerProbe fails qualified Edit focus with `focus owner not found`; the service fixture is returned as builtin Class, not a declared angular Service. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. Keep open. The current find_entities API is cross-domain/type; verifier filters returned identities explicitly rather than assuming request filters are applied. Spread/kind/lazy-route/constraint criteria are not claimed fixed. |
| [#140](https://github.com/codeliftsleep2/Clean-CTX/issues/140) NgRx identities/linkage | **Still real:** SET_AUDIT appears as an Action occurrence in dispatch.component.ts but has no declaration occurrence in legacy.actions.ts. | PROVEN BY LIVE/REPRODUCED BEHAVIOR. A nonzero count alone would falsely pass the declaration criterion. Keep open; declaration-site identity/handler chaining and repo-scale recall remain unresolved/unestablished. |
| [#141](https://github.com/codeliftsleep2/Clean-CTX/issues/141) Angular templates | **Strongest closure candidate:** all 39 current live checks pass, including arbitrary ownership routing, conditions/occurrences, modern control headers, truthful degradation, canonical linkage and refresh/delete/restore. | PROVEN BY LIVE/REPRODUCED BEHAVIOR through the existing harness plus expanded modern-control case. Resolved in tested Linux MCP scenarios. Recommend owner closure review after Windows/original real-template confirmation; no issue was closed automatically. |

## Live artifacts and harnesses

Native issue probes: `verification/issue-reconciliation/Invoke-NativeIssueReconciliation.ps1`.
Final run: `target/issue-reconciliation/20261012-001611-0de51d27`.
Nine acceptance probes: one PASS (#133) and eight FAIL findings across the remaining
probed issues. Exit 1 intentionally records unresolved acceptance, not a failure of
the completed CBM adapter checks. Raw requests/responses, source fixtures, oracle,
Git history and exact error messages are retained.

The first audit iteration was corrected before interpreting results: framework
find_entities counts were initially counting cross-domain builtin entities; high
focus was an invalid request under the current Edit-only focus contract; action
counts initially ignored declaration provenance. Final fixtures use normal multiline
Angular imports/decorators/class bodies. Those earlier apparent passes/errors are
not treated as proof of framework fixes or failures.

Template harness: `verification/live-acceptance/angular_template_linkage_live_acceptance.mjs`.
Evidence: `target/live-acceptance/angular-template-linkage/responses.json` and
`console.log`. All 39 checks pass. An intermediate High formatting assertion was
corrected to accept proven byte-exact source fallback; normalized Medium spelling
is not required of exact source output.

## Recommended next steps

1. Review #141 for closure with the original platform/workspace checks.
2. Expand #133 across its complete acceptance corpus before closure; preserve its
   current 55-file cold success as a guard.
3. Keep #134–#140 open and split multi-part acceptance criteria into narrow fix
   slices. Highest-confidence reproduced correctness defects are #136 insertion
   placement and #137 false unchanged; missing native semantic delivery in
   #138–#140 remains producer/integration work, independent of the CBM adapter.
4. Keep #142 cold/warm aggregate truth and Hybrid experiments separately gated.

Root cause for every remaining subfinding is **NOT ESTABLISHED** by this report.
Evidence proves the observable failure; absence of a typed declaration or edge
must not be silently attributed solely to extraction, projection or identity.
Neither the upgraded provider nor passing internal feature tests earns closure
of a failing model-facing native contract.
