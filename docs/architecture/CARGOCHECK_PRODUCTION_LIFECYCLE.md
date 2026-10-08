# CargoCheck production lifecycle — Phase 11

Status: production source paths traced and all five tracked real-Cargo black-box
checks passed on Linux. The owner subsequently reported all four Windows
lifecycle checks, the previously failing synthetic cancellation check, and the
remaining focused tests and Clippy GREEN. The Claude pilot and CI status remain
outstanding. A workspace-path disclosure in retained producer text was found
while reviewing the Windows evidence. Its correction is documented in
`CARGOCHECK_OPAQUE_WORKSPACE_PATHS.md` and needs Windows confirmation before
the pilot.

## Production call paths

| Boundary | Production path | Established behavior |
| --- | --- | --- |
| CLI startup | `src/main.rs`: `StartupCli` → `cmd_cargo_check` | Explicit CLI workspace, captured environment, Ctrl-C cancellation handler; root MCP options cannot grant CLI authority. |
| MCP startup | `src/main.rs` → `mcp::run_with_options` → `src/mcp/server.rs` | Snapshot before server initialization; `prepare_cargo_check_startup` independently selects and admits root/Cargo authority. Configuration discovery does not grant execution authority. |
| MCP admission | `src/mcp/cargo_check.rs`: `CargoCheckSession` | Prepared invocation is immutable for the session. Missing/invalid authority is disclosed through a static catalog explanation and tool failure. |
| Request scheduling | `src/mcp/server.rs` → `src/mcp/dispatcher.rs` | Reserve CargoCheck ID before enqueue; queue failure releases pending reservation; duplicate IDs cannot replace cancellation tokens. |
| Closed dispatch | `src/mcp/tool_dispatch.rs` → `cargo_check::handle` | Arguments are absent or an empty object; every other shape/property is rejected before execution. |
| Common execution | `execute_cargo_check` in `src/diagnostics/cargo_check/execution/mod.rs` | Revalidation before process creation; exact fixed Cargo arguments, explicit child environment, no stdin, no shell. |
| Linux ownership | `execution/linux.rs`: `OwnedProcess` | Dedicated process group before exec, nonblocking pipes, pinned leader status, live-member quiescence accounting, group SIGKILL, final reap. |
| Windows ownership | `execution/windows.rs`: `OwnedProcess` | Suspended creation and Job assignment before resume; Job-owned forced termination and quiescence. This phase needs Windows owner evidence. |
| Capture/compilation | `execution/capture.rs` → `CargoCheckCompiler` | Concurrent stdout/stderr draining, approved capture/frame budgets, online sanitized semantic retention, omission and coverage facts, no raw spool. |
| Outcome | `execute_cargo_check` → shared execution classification | OS root status, Cargo evidence, cancellation, drain failures, and cleanup remain distinct. Cargo evidence cannot override OS status. |
| Presentation | `presentation.rs` → CLI/MCP adapters | Full serialized structured envelope and text checked against approved limits; no raw producer strings in protocol errors. |
| Host cancellation | MCP reader → session token → executor | Notification does not wait behind workers; queued calls are pre-cancelled, running calls force owned-tree termination. Notification reason is ignored. |
| CLI cancellation | Ctrl-C handler → token → executor | User cancellation uses the same owned cleanup and returns exit 4 unless cleanup uncertainty takes precedence. |
| MCP disconnect | Stdin exhaustion → session shutdown → dispatcher teardown | Existing queued/running tokens are cancelled; future starts receive cancelled tokens. Existing general dispatcher queue/drop semantics are unchanged. |
| Final release | Executor finishes/reaps owner; request guard drops | Root outcome, quiescence and drain facts precede projection. Request registration is removed after normal completion or unwinding. |

The existing dispatcher can stop workers before draining every queued request on
disconnect. Such requests have cancelled tokens and cannot start Cargo later;
no guarantee of a response for every queued request after disconnect is added.
The lifecycle tests close stdin only after observing a running build script when
checking final cancellation results.

There is no intrinsic producer timeout. Test deadlines are test-harness failure
bounds and do not change production behavior. Linux's documented deliberate
detachment and controller-death limitations remain explicit; this phase does not
claim Windows-equivalent containment on Linux or any macOS support.

## Tracked real-workspace checks

`src/tests/mcp/cargo_check_lifecycle.rs` and its separate harness are registered
through the normal library test convention. They run the built production binary
and an explicitly configured absolute Cargo executable against dependency-free
temporary Rust packages. They are ignored by default and require an explicit
owner-run command. The current checkout's full suite is never invoked.

The executed Linux checks cover:

1. CLI JSON success, real rustc E0308 failure, bounded default text, approved
   environment stripping observed inside a real build script, and secret
   sanitization in compiler evidence and build warnings.
2. MCP initialize/catalog/tool calls through actual stdio, success → failure →
   recovery in one session, startup-option authority facts, and both result
   surfaces plus captured server stderr.
3. MCP cancellation of a running real build script, verified owned quiescence,
   environment authority facts, and successful reuse after cancellation.
4. Stdin closure during a running real build script, host cancellation and
   clean server exit.
5. Linux SIGINT during a real build script, CLI exit 4, user cancellation,
   forced termination and clean owned quiescence. Windows console signal testing
   remains owner-specific; the other four checks are portable.

Fixtures inject only synthetic credential/proxy/wrapper/flag values. Routine
Clean-CTX output is checked for those values and the recognized producer-secret
marker. Tests make no claim about Claude transcripts or other host persistence,
and do not treat Cargo's own target artifacts as Clean-CTX raw capture spools.

## Focused commands and observed evidence

The owner authorized these Linux cloud commands for this phase:

```bash
export CARGO_HOME=/workspace/.tooling/cargo
export RUSTUP_HOME=/workspace/.tooling/rustup
export PATH="$CARGO_HOME/bin:$PATH"
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
CLEAN_CTX_TEST_BINARY=/workspace/Clean-CTX/target/debug/clean-ctx CLEAN_CTX_TEST_CARGO=/workspace/.tooling/rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo cargo test --locked -p clean-ctx --lib --all-features -j 4 cargo_check_lifecycle_ -- --ignored --test-threads=1
cargo test --locked -p clean-ctx --lib --all-features -j 4 cargo_check_mcp_
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo test --locked -p clean-ctx --bin clean-ctx --all-features -j 4 cargo_check_
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

For Windows, set `CLEAN_CTX_TEST_BINARY` to the freshly built absolute
`clean-ctx.exe` path and `CLEAN_CTX_TEST_CARGO` to an approved absolute Cargo
executable, then use the same Cargo commands. The Linux SIGINT test is excluded.

The production binary build passed. The lifecycle command ran all five selected
ignored checks explicitly: 5 passed, 0 failed, 0 ignored, 3,595 filtered out.
The lifecycle test run completed in 23.15 seconds. These are new end-to-end
verification checks; no RED/GREEN defect claim is made because no defect was
observed. No production behavior or frozen policy changed. Focused MCP regression checks
passed 20/20, core CargoCheck checks passed 97/97, and startup/CLI checks passed
12/12. Package Clippy passed with zero warnings. Changed files were checked for
UTF-8 without BOM, modified Rust files satisfied the 615-line ceiling, and
`git diff --check` passed. The complete CI gate was not run locally.

## Remaining gate

Confirm the workspace-path correction on Windows, confirm the CI gate, and retain
the full suite exclusively in CI. Then prepare one candidate and one preserved context package
for the bundled Claude pilot described in the architecture freeze. No Claude
pilot has been started.

## Windows owner follow-up

The real-Cargo fixtures initially lacked the MSVC linker in the selected child
environment. Loading the installed Visual Studio developer shell supplied the
already-approved PATH/INCLUDE/LIB toolchain environment. The subsequent build
script launch failure also reproduced with ordinary Cargo and direct executable
launch; the owner identified and resolved an antivirus block. No execution or
environment policy was relaxed. The owner then reported the four portable
lifecycle checks and the focused synthetic cancellation test GREEN, followed by
GREEN for the remaining focused verification. Windows console Ctrl-C remains
a host-specific pilot observation; the Linux SIGINT check does not run there.
