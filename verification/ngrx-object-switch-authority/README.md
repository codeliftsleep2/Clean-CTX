# NgRx object-action and switch-reducer live acceptance

This operator harness drives a freshly built production `clean-ctx` binary
through its registered MCP stdio boundary with CBM disabled. It verifies the
bounded supported envelope for exported string-valued object members, named
aliases, exact NgRx Store dispatch, `StoreModule.forFeature` registration,
switch/case binding, cross-file reconciliation, and authority retraction.

Build the release binary and run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File ./verification/issue-reconciliation/Invoke-NgrxObjectSwitchAuthority.ps1 `
  -BinaryPath ./target/release/clean-ctx.exe
```

The harness also proves that fake dispatch receivers, unregistered reducers,
and incorrectly bound switch discriminants fail closed. It writes requests,
responses, provenance, stderr, and verdicts beneath
`target/ngrx-object-switch/<timestamp-guid>`.

This is operator/live verification evidence, not a Rust test or CI result. The
tracked regression contract remains under `src/tests/**`.
