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

The primary capture also verifies generic structural inheritance through the
production path: class `Extends`, class `Implements`, and interface `Extends`
must be visible through both forward and reverse workspace queries.

## Live contract harnesses

The tracked scripts below drive a freshly built server over MCP stdio and
exercise focused production boundaries:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-EntitiesInFileLive.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-HasCycleLive.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-IdentityResolutionLive.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-BatchQueriesLive.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/workspace-query/scripts/Verify-CSharpConstructorDependenciesLive.ps1
```

These are repeatable operator-verification assets, not Rust regression tests or
CI gates. The corresponding authoritative contracts remain under
`src/tests/**`.

`Verify-BatchQueriesLive.ps1` creates an isolated TypeScript workspace and
drives the freshly built production binary over MCP stdio. It verifies the
published batch schema, mixed ordered results, item-local failure isolation,
discovery-completion reuse on a repeated batch, and semantic parity with the
equivalent legacy single calls. It also verifies that `name` remains a singular
non-empty string and that an array is rejected with guidance to use top-level
`queries`. Generated captures are written beneath
`target/workspace-query-batch-verification/`; they are operator evidence only.

`Verify-CSharpConstructorDependenciesLive.ps1` creates an isolated multi-file
C# workspace and proves the production path from constructor parameter type to
canonical IR, generic semantic projection, `WorkspaceIndex`, and direct MCP
`reverse_edges` / `forward_edges` answers. It also checks ordinary-method and
interface-implementation negative controls, repeated-query stability, and stale
edge removal after source replacement. The relation is the source-signature
fact `HasConstructorParameterType` targeting `builtin/TypeRef`; the harness does
not claim runtime .NET DI registration or implementation resolution.
