# COMPACT-A1 production edge-case operator verification

This package drives a freshly built Clean-CTX binary over the tracked
TypeScript/Angular and C# fixtures. It is operator evidence, not a replacement
for the tracked tests under `src/tests/**`.

```powershell
cargo build --all-features --bin clean-ctx
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\edge-cases\scripts\Capture-EdgeCases.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\edge-cases\scripts\Verify-EdgeCases.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\scripts\Measure-CompactA.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\verification\context-compression\scripts\Verify-CompactA1.ps1
```

The last two commands encode the new live captures with A1, add them to the
token report, and prove normalized equality plus exact body frames. Java/Spring
remain deferred from this heavier operator pass.
