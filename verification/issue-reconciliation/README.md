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
