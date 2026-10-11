# Invocation evidence: live MCP verification

This operator harness starts the current Clean-CTX production binary in an
isolated temporary workspace, initializes MCP over stdio, inventories live tools,
and calls `compress_code_context`, `provide_code_context`, `workspace_query`, and
`refresh_workspace`. It reuses
`verification/context-compression/scripts/McpSession.ps1`, as the context-batch
and workspace-query harnesses do. Its local RPC adapter adds a request deadline,
notification-safe response matching, and startup ownership for cleanup.

No Codex/Claude configuration, Rust fixture builds, MSVC, Angular dependency
installation, Karma or Vitest runtime is needed. The fixture source is parsed by
Clean-CTX; it is not executed. This checks production MCP behavior rather than
model tool selection. It does not stop or reconfigure your existing MCP server.

From the repository root in PowerShell 7:

```powershell
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
pwsh -NoProfile -File ./verification/invocation-evidence/scripts/Verify-InvocationEvidenceLive.ps1
```

To use another current binary or retain the fixture workspace:

```powershell
pwsh -NoProfile -File ./verification/invocation-evidence/scripts/Verify-InvocationEvidenceLive.ps1 -BinaryPath ./target/release/clean-ctx.exe -KeepWorkspace
```

The script chooses `.exe` on Windows and the native filename elsewhere. It
rejects a missing binary or one older than repository Rust sources. It does not
build for you or run the full suite. `-RequestTimeoutSeconds 90` is the default
per-request harness deadline, not a production timeout policy.

Each run creates a unique directory under
`target/invocation-evidence-verification/<timestamp-id>/captures`. Requests,
responses, tool inventory, binary SHA-256, Git commit, PowerShell/platform facts,
and `summary.json` are retained there. MCP stderr flows to the terminal using the
existing helper pattern. Copy terminal startup errors along with captures when
reporting a failure. Exit 0 requires every case to pass; failed assertions,
missing tools, protocol failures, or a partial run return exit 1. All captures
are generated evidence and remain untracked. The temporary workspace is removed
unless `-KeepWorkspace` is supplied. Configuration restarts affect only the
server spawned by this script.

The expanded run contains **59 cases**, organized by phase. See
[COVERAGE.md](COVERAGE.md) for every T1–T24 contract, existing verification
ownership, new MCP cases, cross-file/layer coverage, and internal-only limits.

Run a phase independently with the same binary:

```powershell
pwsh -NoProfile -File ./verification/invocation-evidence/scripts/Verify-InvocationEvidenceLive.ps1 -Phase 7
```

The full run is required for complete acceptance; a selected phase is reported
as selected coverage only. Mixed-layer cases interleave call queries, reactive
context, and Karma/Vitest Tests linkage, and batch different files/fidelities
through all three context routes. A mixed query batch asks TypeScript, C#, and
Angular different questions while isolating a missing-file error and observing
a same-length edit.

## What the live tools can prove

`compress_code_context` exposes named canonical tuples (`pretty.ir`) and text;
the harness reads the existing `TYPE/@pipeRx` projection. `calls_in_file` remains
an owner-aware canonical summary. Its mirrored text and structured result must
agree, and its records must not fabricate resolution or private span fields.
`provide_code_context` single-file responses use text and `_meta`, as the current
production contract specifies. This harness does not assume a structured result
where the tool does not advertise one. Internal Binary0x04 persistence and
private invocation records are not newly exposed by this migration.

The private source-bound evidence intentionally has no standalone MCP tool or
public occurrence/span schema. Exact receiver/terminal/argument/type spans,
Named qualifiers, source-id/hash rejection, and parse counts are tested through
the tracked consumer/lifecycle regressions. Claiming those fields were directly
observed in MCP would be incorrect. Complementary focused checks are:

```powershell
cargo test --locked -p clean-ctx --lib --all-features -j 4 invocation_evidence
cargo test --locked -p clean-ctx --lib --all-features -j 4 invocation_generic_consumer
cargo test --locked -p clean-ctx --lib --all-features -j 4 invocation_fresh_state
cargo test --locked -p clean-ctx --lib --all-features -j 4 angular_meta::rx::
```

The full repository test suite remains CI-only. Karma/Jasmine and Vitest/TestBed
source analysis is verified through MCP here; this harness does not execute
either test runner or claim browser/Angular runtime validation.
