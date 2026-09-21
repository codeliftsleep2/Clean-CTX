# Workspace-query model-visibility verification

This package audits the boundary between the authoritative `WorkspaceIndex`
answer and what an MCP host may place in model context. It makes no production
contract change.

The fixture is published through registered `provide_code_context`, then all six
`workspace_query` operations are invoked through the registered MCP dispatcher.
Each capture retains the complete JSON-RPC response, `content`, and
`structuredContent`.

Run after rebuilding `target/debug/clean-ctx.exe`:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Capture-WorkspaceQueries.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-WorkspaceQueries.ps1
```

Generated evidence is written beneath
`target/workspace-query-verification/`. It is operator evidence, not a tracked
test and not a substitute for the repository gate.

The deterministic verifier proves only that registered production queries
return the expected structured graph facts. Its visibility report separately
records whether `content` itself contains those facts. A real Claude/Codex host
trace is still required to establish whether that host exposes
`structuredContent` to the model.
