# Native issue reconciliation over live MCP

This read-only investigation harness exercises current #133/#134/#136–#140
acceptance probes against a current all-feature Clean-CTX binary. It creates
owned C#/TypeScript sources and a small controlled Git history under target/.
Provider is explicitly disabled; nothing is built, installed, pushed or closed.
It reuses the existing Phase B ProcessStartInfo/stdio session helper.

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-NativeIssueReconciliation.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux pass `./target/debug/clean-ctx`. Run the existing Angular template harness
for #141, as documented in verification/live-acceptance/ANGULAR_TEMPLATE_README.md.
#135 DI truncation is additionally traced in the current production code report.

PASS means that specific acceptance probe passed. FAIL means the original issue
criterion still failed in the controlled scenario; it is intentionally retained
as evidence and the process exits 1. There are no expected-failure suppressions.
Find-entities identity/provenance is checked in returned data; a cross-domain count
or an entity at a use site does not pass a declaration criterion. Every RPC and
source fixture is retained under target/issue-reconciliation/.

See [the reconciliation report](../../docs/plans/ISSUES_133_141_POST_CBM_RECONCILIATION.md)
for current results, limitations and closure recommendations. No full test suite
or native production repair is part of this audit.

## Post-CBM causal investigation

The expanded investigation keeps native and provider-enabled cold/warm controls
separate, verifies provider admission before dependent checks, and compares exact
constructor subject sets for ten interfaces in five directories. Each mode creates
its own Git root, config, session, source oracle and evidence directory. No build,
shared-daemon termination or cache override occurs.

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-PostCbmInvestigation.ps1 -BinaryPath ./target/debug/clean-ctx.exe -ProviderPath <approved-pinned-CBM-executable> -ExpectedProviderHash <approved-artifact-SHA256>
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-NativeCausalControls.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

Use the artifact/hash already verified for the platform; the Linux artifact's hash
is not the Windows artifact's expected hash. On Linux omit the binary's .exe suffix.
The first harness accepts `-Scope native`, `provider`, or `controls` for targeted
reruns. Provider admission failure stops that mode; fallback answers cannot count
as provider-enabled evidence. Native-only causal controls need no CBM executable.

Capture cases record observed surfaces; a capture PASS proves protocol completion,
not semantic acceptance. Cases asserting feature truth remain FAIL when a defect
is reproduced. In the current baseline both harnesses intentionally exit 1 because
unresolved native criteria fail; the #133 recall groups themselves pass. Do not
convert these failures to expected-pass suppressions or omit them in summaries.

New evidence covers failed-read partial disclosure, DI40, accepted misplaced edit
comments, nonexistent test targets, return/attribute/body/parameter diff controls,
isolated Service/Action declarations, Pipe/inject targets, export-owner controls,
same-content template filename/nesting, and linked HTML entities_in_file.

See [the causal report](../../docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md)
for first failing boundaries, issue disposition, implementation proposals and
remaining original-corpus/platform acceptance. These are investigation harnesses;
permanent repaired-contract regressions should assert the public truth described
there, including precise file provenance and disclosure.

## #136 whole-declaration insertion regression

Run against a binary built from the insertion repair:

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-InsertBeforeDeclaration.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. The harness builds nothing and needs no CBM or model host.
It checks the advertised apply_edit tool through live stdin/stdout MCP, then
compares exact file bytes for C#/TypeScript first/middle/metadata-bearing members
at LF/CRLF, including parseable comments. Unproven leading documentation must
fail without writing. Replace/delete/after and atomic/syntax/source-authority
controls are included: 23 case groups, all required to pass for the repaired slice.

All fixture files are allocated before atomic edits to isolate declaration
placement from independent file-identity reuse. Requests, receipts, exact byte
oracles, stderr and binary provenance remain under target/insert-before-declaration.
See [the repair report](../../docs/plans/ISSUE_136_INSERT_BEFORE_DECLARATION_REPAIR.md)
for RED/GREEN evidence, the separately retained intermediate lifecycle failure,
and remaining #136 criteria. This does not claim the entire issue is resolved.

## #137 semantic signature evidence regression

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-SemanticSignatureEvidence.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

The harness creates an owned committed C# history, verifies Git/source evidence,
and checks registered diff_commits results at low/medium/high/edit/verbatim:
return-type and authorization-only changes, unchanged members, parameter changes,
the supported last-method body control, and unchanged neighbors. All 25 semantic
groups must pass. No build, CBM daemon or model host is invoked.

For RED/GREEN investigation, reuse the exact saved history with the candidate:

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-SemanticSignatureEvidence.ps1 -BinaryPath ./target/debug/clean-ctx.exe -HistoryPath <history-path-printed-by-the-RED-run>
```

Only this harness's target/semantic-signature-evidence histories are admitted for
reuse; expected committed source is checked before the test. The source history
is not recommitted during GREEN. On Linux omit .exe. Every run records artifact
hash, both commits, exact source oracle, raw MCP results and per-case verdicts.

The body control isolates the existing last-method contract; the separate
first-method/neighbor-marker defect is retained in
[the repair report](../../docs/plans/ISSUE_137_SEMANTIC_SIGNATURE_EVIDENCE_REPAIR.md).
Passing this harness does not close all #137 criteria or establish #135 fidelity.

## #135 DI retention regression

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-DiOmission.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. The provider-disabled live context harness exercises 0/9/10/
11/40 written registrations at Medium/High, distinct implementations/lifetimes,
nearby non-DI metadata and preservation of existing conditional written mentions:
16 required groups. It builds nothing and does not evaluate runtime activation.

Complete delivery may use the existing compact DI markers or existing byte-exact
source fallback. The latter is accepted only with document byte-exact metadata,
full source equality and the independent fixture's written-call count. Exact
marker identity/order assertions remain for compact output. No partial ten-entry
list is accepted as complete. All fixtures, requests/results, binary provenance
and verdicts are saved under target/di-omission.

See [the DI repair report](../../docs/plans/ISSUE_135_DI_OMISSION_REPAIR.md) for
same-source RED/GREEN evidence, producer scope, ordering and remaining #135 work.
The harness proves recognized written-fact delivery, not an evaluated DI container
or completeness of unsupported registration forms.

## #139 decorated declaration owner identity

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-DecoratedOwnerIdentity.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. This provider-disabled harness uses the existing stdio MCP
session helper and preallocates owned Git-root fixtures. Eight declaration
forms cover export/decorator/layout variations, arbitrary custom decorators,
basic generics and multiple classes with the same-named member.

Each form requires four live groups: exact qualified focus, builtin class/file
provenance, a controlled qualified replacement checked against the entire
expected source, and fresh focused output after editing (32 groups). Alpha's
same-named body must remain untouched and absent from OwnerProbe's focused
output. Edge-free Injectable cases record independent Service publication as a
separate observation; the subsequent publication harness owns that contract.
Fixtures, raw requests/responses,
binary hash and verdicts are saved under target/decorated-owner-identity.

See [the owner repair report](../../docs/plans/ISSUE_139_DECORATED_OWNER_IDENTITY_REPAIR.md)
for permanent regression RED/GREEN, separate earlier-body anomaly, preservation
results and remaining #139 criteria. This harness builds nothing and invokes
neither CBM nor a model host. It does not replace tracked tests or the CI gate.

## Independent framework declaration publication

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-FrameworkDeclarationPublication.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe. Rebuild the candidate separately before running. This driver
reuses the existing Phase B stdio MCP session helper and creates owned Git-root
fixtures with CBM disabled. It builds nothing and invokes no model host.

All 57 required groups must pass: edge-free Service/modern Action recognition and
publication; same-file multiple declarations; source-distinct same names; existing
constructor/dispatch relationships and reference-file occurrences; deduplication;
registered refresh/rename/removal/re-add/file deletion; and declaration removal
with an unchanged reference retained. #140 supersedes only the former legacy-class
absence control: its unchanged source now requires a recognized class Action.
Modern publication assertions retain their existing oracles. Exact declaring-file occurrences are checked
separately from existing relationship endpoint occurrences.

The public entity schema has no role/span field. Saved fixture roles are source
oracles, not invented response fields. The driver does not claim declaration-only
global lookup, cross-file binding or query-history independence. Source manifests,
every raw request/result, binary hash, mutation evidence and verdicts are saved
under target/framework-declaration-publication. FAIL is retained and exits 1.

See [the publication repair report](../../docs/plans/INDEPENDENT_FRAMEWORK_DECLARATION_PUBLICATION_REPAIR.md)
for permanent test RED/GREEN, live baseline/candidate evidence, durable generation
compatibility, preservation and remaining #139/#140 work.

## #138 Pipe constructor DI admission

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-AngularPipeDi.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit .exe and build separately before running. The existing Phase B
stdio helper drives provider-disabled owned Git-root fixtures. All 56 required
groups must pass: independent declarations/evidence, exact typed Injects identities,
repeat/deduplication, existing Service/Component/Directive controls, Pipe constructor
forms and multiple dependencies, empty/no-constructor and other-kind exclusions,
unresolved symbolic targets, source-distinct ambiguity, and refresh/change/removal/
file deletion. A mixed/functional-only Pipe guard prevents newly admitting the
separate wrong-field-name evidence.

Four inject(...) observations are captured without PASS/FAIL token acceptance.
That defect is not repaired by this driver or slice. Object.file is asserting
occurrence provenance, not proof of import-resolved declaration binding. Sources,
requests/results, hashes, mutations and verdicts stay under target/angular-pipe-di.
The driver builds nothing and invokes neither CBM nor a model host.

See [the partial #138 report](../../docs/plans/ISSUE_138_PIPE_DI_ADMISSION_REPAIR.md)
for permanent RED/GREEN, live evidence, supported constructor boundary, the
separate token-authority repair and remaining criteria.

### #138b structured inject-token verification

This provider-disabled harness runs 131 bounded MCP assertion groups: declaration/owner evidence, exact token markers and edges, repeated-query deduplication, fidelity controls, symbolic cross-file limits, and refresh/change/remove/delete. See `docs/plans/ISSUE_138B_STRUCTURED_INJECT_REPAIR.md`. The earlier blocked primary probe is retained in the historical evidence-gate report.

```powershell
pwsh -NoProfile -File verification/issue-reconciliation/Invoke-AngularInjectToken.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

Use `./target/debug/clean-ctx` on Linux. Generated evidence is retained under `target/angular-inject-token/`; no CBM provider is started.

## #140 independent legacy class Action declarations

```powershell
pwsh -NoProfile -File ./verification/issue-reconciliation/Invoke-NgrxLegacyDeclarations.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

On Linux omit `.exe`. This provider-disabled harness uses the established Phase B stdio session helpers. It requires **78 acceptance groups** for top-level class declarations implementing the directly imported NgRx `Action`: exact source occurrences, truthful class markers, no fabricated links, fidelity/repeat controls, source-distinct same names, and declaration rename/removal/deletion.

Seven separate handler/reference/alias/collision probes are recorded as **OBSERVED**, not PASS. They remain unresolved and do not affect the declaration-only acceptance verdict. The driver builds nothing, invokes no model host, and saves source oracles, binary provenance, requests/results and verdicts under `target/ngrx-legacy-declarations/`. See [the #140 class declaration report](../../docs/plans/ISSUE_140_LEGACY_CLASS_ACTION_DECLARATION_REPAIR.md) for the defect partition and remaining work.

## #140b-Prereq2 private Action/type binding preservation

```powershell
pwsh -NoProfile -File verification/issue-reconciliation/Invoke-NgrxActionTypeBindingPreservation.ps1 -BinaryPath ./target/debug/clean-ctx.exe
```

This provider-disabled harness uses the existing PhaseBSession stdio driver.
Nine initializer forms run at High, Medium and Low (27 groups). Each asserts the
exact admitted class Action identity/file, absence of a duplicate constant Action
and absence of a new public relation; source bytes must remain unchanged.

The new association is private compilation evidence. The harness does **not**
observe that association or establish reducer linkage. Registered
`ngrx_action_type_binding_` producer tests prove its exact source spans, file/hash
identity, direct-reference selection and refusal controls. See
[the binding report](../../docs/plans/ISSUE_140B_PREREQ2_ACTION_TYPE_BINDING.md).
