# Batched `provide_code_context` live acceptance

This package drives a freshly built production `clean-ctx` binary over MCP
stdio. It verifies the externally visible batched `provide_code_context`
contract against an isolated copy of the tracked TypeScript fixtures.

The harness checks:

- `tools/list` publishes the single/batch union, eight-item cap, and ordered
  result schema;
- mixed overview and focused Edit items succeed around one isolated missing-file
  failure;
- omitted `responseMode` resolves to safe `mirrored`, with byte-identical code
  in both MCP visibility channels;
- `structured` keeps exact code in each structured success when the top-level
  routing notice is discarded;
- `indexed` keeps exact code in `content_index`-identified top-level blocks
  while omitting structured item content;
- the focused Edit block is byte-identical to the equivalent legacy single
  response;
- a repeated batch preserves content and structured outcomes while increasing
  the reported prompt-cache hit count.

Build the production binary, then run the harness:

```powershell
cargo build --all-features --bin clean-ctx
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-batch/scripts/Verify-ProvideContextBatchLive.ps1
```

Generated captures are written beneath
`target/provide-context-batch-verification/captures/`. They are operator
evidence only, not tracked tests and not a substitute for the regressions under
`src/tests/**` or the repository verification gate.
