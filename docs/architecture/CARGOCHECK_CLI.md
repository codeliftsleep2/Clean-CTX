# CargoCheck CLI — Phase 9

The local operator adapter invokes the approved CargoCheck core and emits its
bounded sanitized projection:

```text
clean-ctx cargo-check --workspace-root <absolute-path> --cargo-path <absolute-path> [--json]
```

The workspace argument is mandatory. An explicit Cargo path takes precedence
over `CLEAN_CTX_CARGO_PATH`; missing Cargo authority fails admission. An invalid
explicit path does not fall back. There is no CWD, PATH, repository configuration,
or workspace-environment fallback. The environment is captured once at startup;
invocation facts disclose winning authority sources and environment presence and
shadowing through booleans, without exposing environment values.

The operation remains exactly `cargo check --message-format=json`. Caller-supplied
Cargo arguments are rejected. Existing commands and no-argument MCP startup retain
their existing dispatch.

## Projection and exit contract

Text is the default, bounded to 24 KiB and 240 lines. `--json` emits the checked
structured envelope, bounded to 512 KiB of serialized UTF-8. Both include the named
CLI outcome and exit code inside the measured projection. The writer adds no bytes
after projection, including no extra newline.

| Code | Outcome |
| --- | --- |
| 0 | Cargo exited successfully |
| 1 | Cargo exited unsuccessfully |
| 2 | Authority admission or platform support failed |
| 3 | Internal processing, startup, projection, or output failure |
| 4 | Cancellation |
| 5 | Cleanup uncertainty |

Precedence is cleanup uncertainty, cancellation, internal failure, admission
failure, Cargo failure, success. The actual Cargo OS exit code remains a separate
process fact and is never propagated as the CLI code. Malformed or mixed producer
evidence remains disclosed without replacing the authoritative OS outcome.

Ctrl-C requests immediate owned cancellation through the existing signal-handler
dependency. Handler registration failure returns an internal failure before Cargo
starts. Live console signal behavior remains part of production lifecycle
verification; it was not exercised by this phase's focused tests.

Failures use small deterministic text or JSON artifacts containing static labels.
They omit caller paths, environment values, and producer text. Post-execution
projection failures preserve root outcome, cancellation, cleanup, capture, and
coverage facts when available. Output I/O failure returns code 3 unless code 4 or
5 already takes precedence.

Owned execution now supplies the admitted workspace to diagnostic compilation.
Typed primary, related, and child span paths become workspace-relative or redacted
external paths, with explicit classification and transformation counts. Standalone
compiler instances without admitted authority retain unknown classification.
Opaque diagnostic messages continue through the established sanitizer.

## Focused verification

- CLI RED: 3 intended assertion failures before command registration; the exact
  preserved regression source passed all 3 checks after implementation.
- Path RED: 96 passed and 1 intended assertion failed before workspace mapping;
  the exact preserved regression and producer fixture passed after implementation.
- Final CargoCheck library checks: 97 passed, 0 failed.
- Final CLI checks: 7 passed, 0 failed.
- Package Clippy: zero warnings after correcting one test-only owned-comparison
  lint without suppression.

These results were observed on Linux. Windows verification remains owner-run:

```text
cargo test --locked -p clean-ctx --bin clean-ctx --all-features -j 4 cargo_check_cli_
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

The full suite remains CI-only. No Claude pilot was run.

## Next phase

Integrate the typed MCP tool with explicit startup authority and the same core
result. Then complete production lifecycle verification and the final bundled
Claude pilot. Existing operational policies remain frozen.
