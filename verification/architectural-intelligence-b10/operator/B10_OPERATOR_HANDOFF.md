# B10 next operator run

Agent: **Agent MaxHeadRoom**. Run from PowerShell 7 at the repository root:

```powershell
pwsh -NoProfile -File verification/architectural-intelligence-b10/infrastructure/invoke-host-smoke.ps1 `
  -WorkspaceRoot 'C:\Users\MNasty\Desktop\RustContextLayerAI' `
  -OutputRoot 'C:\Users\MNasty\Desktop\RustContextLayerAI\target\pcg-b10\host-smoke-01'
```

This starts one fresh Codex CLI session, using existing authentication. It asks
the agent to exercise all five knowledge modes through actual host-mediated MCP
calls. It does not build anything, change normal configuration or copy credentials.
The read-only agent shell remains subject to normal host security controls. The
authorized existing MCP tool may reconcile its derived catalog.

Paste the terminal result back. The investigator will inspect host-events.jsonl,
agent-response.md, stderr.txt, manifest.json and exit-code.txt. Preserve failures;
do not silently retry, change executable, relax controls or overwrite receipts.
If host approval requires interaction or an option is unsupported, report it.
An agent's written success claim requires corroborating actual tool-call events.

The wrapper was parsed successfully with the PowerShell syntax parser; it has
not been executed. No CLI/runtime success is implied by that static check.
No new Cargo verification is needed just to exercise the accepted executable.
The final repository gate remains separate and UNRUN; its authoritative command
list is docs/agent/verification.md, not duplicated here.
