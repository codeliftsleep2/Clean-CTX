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
