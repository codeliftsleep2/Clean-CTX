# CargoCheck typed MCP integration — Phase 10

The static `cargo_check` tool invokes the same prepared invocation, owned
execution, semantic compiler, and bounded projection used by the CLI. Its input
schema is an object with no properties and `additionalProperties: false`.
The handler accepts omitted arguments or an empty object; it rejects null,
non-object arguments, and every caller-supplied property before execution with
a static JSON-RPC `-32602` error. Tool arguments cannot select paths, commands,
environment variables, flags, or producers.

## Startup authority

```text
clean-ctx --workspace-root <absolute-path> --cargo-path <absolute-path>
```

With no subcommand, the binary starts stdio MCP. For each authority independently,
an explicit root startup option wins over the corresponding operator environment
variable: `CLEAN_CTX_PROJECT_ROOT` or `CLEAN_CTX_CARGO_PATH`. Invalid explicit
authority fails rather than falling back. The server captures the environment
once before initialization and admits an immutable prepared invocation.
Source and shadowing facts flow through the existing invocation projection.

There is no repository configuration, CWD, executable-directory, PATH, additional
root, or model-argument fallback for execution authority. Existing configuration
discovery continues serving other operations without granting CargoCheck
authority. Root startup options cannot be combined with subcommands; the CLI
operation continues using its own required workspace argument. Help and version
exit through clap, and existing command dispatch is preserved.

Missing, invalid, or unsupported authority leaves the server and other tools
available. CargoCheck remains cataloged with its closed schema and a static
unavailability explanation appended to its description. Calling it returns a
bounded tool failure with `isError: true`. `McpState::new` grants no execution
authority; production admission occurs in server startup.

## Result and protocol boundary

`structuredContent` is the complete checked core projection, bounded to 512 KiB
of serialized UTF-8. `content` contains one text block derived from that sanitized
projection, bounded to 24 KiB and 240 lines. Neither surface contains CLI-specific
outcome/code fields. MCP framing and result-wrapper bytes are separate from these
approved projection budgets.

`isError` uses the same authoritative execution classification as the CLI.
Cargo nonzero, cancellation, incomplete cleanup, and internal failure return true.
Malformed or mixed producer evidence remains disclosed without replacing the OS
outcome. Admission, launch, and projection failures return static labels;
post-execution projection failures retain numeric/typed root, cleanup, capture,
coverage, and Cargo evidence facts without producer text. JSON-RPC errors never
include caller arguments or producer evidence.

CargoCheck declares `readOnlyHint: false`, `idempotentHint: false`, and
`destructiveHint: false`; its normal operation produces build artifacts rather
than deleting source. `openWorldHint: true` reflects approved dependency
networking. Annotations are hints, not execution authority. Other tool hints
retain their previous values.

## Cancellation and shutdown

The stdin reader reserves each CargoCheck request before enqueueing it. MCP
`notifications/cancelled` is handled directly on the reader thread, so queued
requests and operations occupying all workers remain cancellable. The request ID
selects an existing cancellation token; the notification reason is ignored.
Unknown IDs grant no authority and allocate no retained state. Duplicate active
CargoCheck IDs are rejected without replacing the existing token.

Queued cancellation prevents process creation. Running cancellation reaches the
existing immediate forced process-tree cancellation and quiescence verification.
A guard releases request registration after normal completion or unwinding;
argument rejection and queue admission failure release pending reservations.
Stdin exhaustion cancels queued and running CargoCheck operations before dispatcher
teardown. Calls beginning after shutdown receive a pre-cancelled token.

No intrinsic timeout, execution-policy budget, or general dispatcher lifecycle
policy was added or changed. Linux process-group/controller limitations remain
those documented in the owned-execution phase.

## Verification and remaining boundary

The four tracked regressions failed at their intended assertions against the
unfixed implementation. They were stashed during implementation and restored
byte-for-byte. The same focused command then passed all 20 checks, including
startup precedence and environment snapshot behavior, catalog/dispatch coverage,
argument rejection, process outcomes, both protocol surfaces' recognized-secret
handling, queued cancellation, shutdown, and live owned process-tree cleanup.

Final Linux verification also passed all 97 CargoCheck library checks and all 12
startup/CLI checks. Package Clippy completed with zero warnings.

The full suite remains CI-only. Windows checks remain owner-run. No Claude pilot
was conducted. Production lifecycle tracing and representative real-workspace
verification are the next phase, followed by the final bundled Claude pilot.

Focused owner commands:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 cargo_check_mcp_
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo test --locked -p clean-ctx --bin clean-ctx --all-features -j 4 cargo_check_
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```
