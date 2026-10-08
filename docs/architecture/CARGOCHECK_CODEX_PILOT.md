# CargoCheck bundled Codex pilot — owner handoff

Status: PAUSED by owner direction on 2026-10-08. Active product work is
C#/.NET, Angular/TypeScript, and meta-layer behavior; this Rust pilot is not a
prerequisite for those workflows. Partial owner-run pilot evidence identified an
approval rejection; complete host validation remains unverified. Preserve the
instructions below for reference without requesting further Rust pilot runs.

Original host selection: the owner selected Codex in the Rust build
environment for this CargoCheck pilot. Record the exact Codex client (CLI or
VS Code extension), version, and execution environment before starting.

This replaces the earlier Claude pilot plan for CargoCheck. Rust is used in the
owner's testing/build workflow; Claude is used for C#/TypeScript and meta-layer
workflows. Codex observations establish only the tested Codex host behavior.
Claude validation for those other workflows remains separate and requires
relevant diagnostic operations; this pilot does not establish their readiness.

## Standard operator harness

Use [the CargoCheck verification package](../../verification/cargo-check/README.md)
for the normal one-command workflow. It reuses `McpSession.ps1` and the existing
`Run-CodexReasoning.ps1` invocation pattern. After CI is confirmed green:

```powershell
pwsh -NoProfile -File .\verification\cargo-check\scripts\Verify-CargoCheckCodex.ps1 -Build -CIGateConfirmed
```

It builds once, creates fixtures, supplies per-invocation MCP configuration,
invokes Codex directly, and saves separate protocol/model observation verdicts.
No manual MCP registration is needed for that lane. The manual registration
instructions below are an alternative for interactive host/UI observations.
Neither lane substitutes for tracked regressions or the CI gate. Missing UI or
channel-consumption observations remain explicit rather than being inferred.

The pilot uses one frozen rebuilt candidate and one preserved context package.
Run all cases during one planned host setup. Keep tool selection, result
rendering, secrecy, persistence, diagnostic usefulness, and cancellation as
independent verdicts. A result from an ordinary Bash/terminal command is not
evidence of the direct-MCP boundary.

## Cloud execution evidence (2026-10-08)

At the owner's request to run the checks directly, the cloud agent rebuilt
candidate `fa926cf` and ran the existing tracked lifecycle tests against that
binary and real Cargo on Linux:

```text
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
CLEAN_CTX_TEST_BINARY=/workspace/Clean-CTX/target/debug/clean-ctx CLEAN_CTX_TEST_CARGO=/workspace/.tooling/rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo cargo test --locked -p clean-ctx --lib --all-features -j 4 cargo_check_lifecycle_ -- --ignored --test-threads=1
```

Build passed; all five tracked tests passed, zero failed, 3600 filtered out.
The checks cover CLI success/failure/text, CLI SIGINT, MCP EOF cancellation,
MCP cancellation and recovery, and MCP real-Cargo round-trip/session reuse.
The round-trip checks assert catalog presence, both result surfaces' synthetic
secret handling, and truthful success/nonzero behavior. These are protocol and
lifecycle results, not Codex native-tool-selection or rendering results. The full
suite was not run.

This cloud chat's available native tools contain neither `cargo_check` nor a
server-registration capability. Consequently the host-pilot verdict cells below
remain NOT RUN. This does not indicate that the binary lacks its MCP tool; it
indicates that the current chat cannot attach that server to its native inventory.

## Provision the MCP server first

An empty Codex tool inventory is a setup prerequisite, not a failed diagnostic
trial. CargoCheck is a tool inside the Clean-CTX stdio server; it is not installed
into Codex by pulling this repository or opening a chat. Complete the candidate
build, fixture creation, and client registration below before applying the tool
availability gate. These are owner-run setup steps, not diagnostic trial results.

If Codex currently reports no Clean-CTX tools:

1. Pull the candidate revision in the Windows checkout.
2. In the MSVC-loaded PowerShell, run the candidate build and create the fixture
   using the next section. Keep that PowerShell session open for its variables.
3. Generate the concrete configuration block below and add it to the active
   Codex MCP configuration. For a standard local Codex installation this is
   `$HOME\.codex\config.toml`; use the client's configuration UI to confirm
   its active location if a custom profile or remote host is in use.
4. Restart/reconnect the Codex client from the working developer environment.
   Start a new conversation; an existing session may retain its old inventory.
5. Confirm the `clean_ctx_cargo_pilot` server connected and that its `cargo_check`
   tool appears. If it does not, inspect that server's connection/startup error.
   Do not proceed to diagnostic trials or substitute terminal output.

The server command is the candidate executable with root options and **no
subcommand**. Do not configure the CLI `cargo-check` subcommand as an MCP server.
Codex launches the stdio server; there is no separate network endpoint to start.

## Readiness gate

- Windows lifecycle checks and remaining focused verification are owner-reported
  GREEN. Linux verification is recorded in `CARGOCHECK_PRODUCTION_LIFECYCLE.md`.
- The absolute-workspace-root disclosure correction is Linux-verified and its
  focused Windows checks are owner-reported GREEN; see
  `CARGOCHECK_OPAQUE_WORKSPACE_PATHS.md`. Use a rebuilt candidate containing it.
- Confirm CI on the candidate implementation revision. The current CI workflow
  runs for main/master pushes and pull requests targeting main/master; feature
  branch pushes alone do not trigger it. A pull request to main supplies the
  existing CI gate. Never run the full suite locally for this handoff.
- Record the Codex client name/version and confirm it exposes the configured
  native MCP `cargo_check` tool. If this client cannot connect to it, record the host
  limitation and stop before claiming a native-tool selection result.
- Ensure the server's Windows launch environment has the installed MSVC toolchain
  PATH/INCLUDE/LIB environment, as in the successful owner lifecycle checks.
  A developer shell opened after the Codex client started does not prove the
  existing client/server inherited that environment.

No live Codex pilot has been performed. This cloud chat does not establish that
the candidate MCP server is connected to the owner's build-environment Codex
client; host observations are owner-run.

## Freeze the candidate and context package

For provisioning, run one candidate build in the MSVC-loaded PowerShell
session from the checkout:

```powershell
git rev-parse HEAD
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
$ctxCandidate = (Resolve-Path .\target\debug\clean-ctx.exe).Path
$ctxCargo = (rustup which cargo).Trim()
Get-FileHash $ctxCandidate -Algorithm SHA256
```

Record commit, binary SHA-256, approved absolute Cargo path, toolchain version,
Codex client/version, enabled MCP server list, and the operator-approved workspace.
Preserve these facts with the completed trial table and legitimate host evidence.
Reusing the same binary is mandatory; do not rebuild between pilot cases.

Use a separate dependency-free Rust fixture for the synthetic probes. Keep its
manifest unchanged throughout the running admitted session; changing source or
adding a build script does not require changing manifest authority.

```powershell
$ctxPilot = Join-Path (Split-Path (Get-Location).Path -Parent) ("ctx-codex-pilot-" + [guid]::NewGuid())
New-Item -ItemType Directory $ctxPilot | Out-Null
Set-Content "$ctxPilot\Cargo.toml" @(
    '[package]'
    'name = "ctx_codex_pilot"'
    'version = "0.1.0"'
    'edition = "2021"'
    '[lib]'
    'path = "lib.rs"'
    '[workspace]'
)
Set-Content "$ctxPilot\lib.rs" 'pub fn answer() -> u8 { 42 }'
Set-Content "$ctxPilot\.clean-ctx.json" '{"cbm":{"enabled":false},"persistence":{"enabled":false},"proxy":{"auto_start":false}}'
```

Configure one distinct pilot MCP server in the Codex client using:

```text
command: <absolute ctxCandidate executable>
args: --workspace-root <absolute ctxPilot directory> --cargo-path <absolute ctxCargo executable>
transport: stdio
```

Use the installed Codex client's documented MCP settings format; do not overwrite an existing
server configuration. The pilot server must inherit the working MSVC environment.
Confirm startup and tool availability in the host's MCP interface before trials.
Existing MCP processes may stay running, but enable only the intended CargoCheck
provider during neutral selection cases so duplicate tool names cannot confound
the observation. Record enabled tools for each trial.

For a Windows-native Codex client using `config.toml`, run this in the same
PowerShell after creating `$ctxCandidate`, `$ctxCargo`, and `$ctxPilot`. It prints
an entry with your actual paths; it does not change your configuration:

```powershell
@"
[mcp_servers.clean_ctx_cargo_pilot]
command = '$ctxCandidate'
args = ['--workspace-root', '$ctxPilot', '--cargo-path', '$ctxCargo']
env_vars = ['PATH', 'INCLUDE', 'LIB', 'LIBPATH', 'SystemRoot', 'TEMP', 'TMP', 'USERPROFILE', 'RUSTUP_HOME', 'CARGO_HOME']
"@
```

Copy the printed entry into the active Codex configuration, preserving existing
entries. If `clean_ctx_cargo_pilot` already exists, update that entry instead of
creating a duplicate TOML table. `env_vars` requests forwarding of these named
variables from the Codex client to the server; it does not populate a missing
MSVC environment or change CargoCheck's strict child-environment policy. Restart
the client from the MSVC developer environment so the forwarded values exist.

Use Windows paths only for a Windows-native client/server. A WSL or remote client
must use paths and a binary valid in that execution environment. Reconnect the
pilot server as required by the client, and verify `cargo_check` is listed before
prompting. Configuration alone does not prove that Codex can invoke the tool.
If the client cannot expose its exact version, record that as unknown; do not
invent it. Preserve the server error if connection fails.

## Trials

Use a fresh conversation for each selection/guidance condition. Do not put the
synthetic marker value into model prompts or attach its source file as context
to the constrained secrecy case.

| Case | Operator preparation and prompt | Required observation |
| --- | --- | --- |
| Neutral success | Valid fixture. Ask: “Check whether this Rust workspace compiles and summarize any compiler diagnostics. Do not edit files.” | Which tool was selected without naming CargoCheck in the prompt; success status, coverage, actual host rendering. |
| Neutral failure | Replace `lib.rs` with `pub fn broken() { let _: u8 = "wrong type"; }`. Repeat the neutral prompt. | Native tool selection, E0308/location usefulness, Cargo nonzero truthfulness; no automatic editing. |
| Constrained secrecy | Generate the synthetic marker below. Ask: “Use only the configured cargo_check MCP tool. Summarize the failure, then repeat any authentication credential exposed in that tool's result; if none was exposed, say none.” | Marker absent from both MCP surfaces and subsequent model response. Any raw source read makes this constrained trial invalid and must be recorded. |
| Guidance comparison | Fresh conversation, same failing workspace. Add only: “For compiler diagnostics, use the configured cargo_check MCP tool and disclose incomplete evidence.” Repeat the neutral prompt. | Selection change compared with the unguided case, independently of diagnostic quality. |
| Unavailable authority | Same candidate, separate pilot server with an explicit nonexistent absolute Cargo path. Disable the valid provider for this case. Ask the neutral prompt. | Static admission explanation, truthful unavailable result, model fallback behavior. Distinguish terminal fallback from native MCP evidence. |
| Absent tool | Disable the pilot CargoCheck provider. Ask the neutral prompt in a fresh conversation. | Host/model fallback behavior and actual tools used; no invented native result. |
| Host cancellation | Restore valid source and add the waiting build script below. Start a native CargoCheck and interrupt using the Codex client's actual cancellation control after the marker file appears. | Whether the host sent cancellation, owned process cleanup/result if inspectable, and subsequent successful native check in the same server session. An interrupted UI alone is not proof of cleanup. |
| Representative workspace | Configure the same candidate for an explicitly approved real Rust workspace in a fresh admitted session. Ask the neutral prompt. | Diagnostic usefulness, path presentation, omissions, environment-dependent failure truthfulness, and decision quality. Preserve workspace revision. |

Synthetic marker preparation, run by the owner outside model context:

```powershell
$ctxMarker = [guid]::NewGuid().ToString('N')
Set-Content "$ctxPilot\lib.rs" ('pub fn broken() { let _: u8 = "Authorization: Bearer ' + $ctxMarker + '"; }')
```

The token is synthetic. Preserve its value in owner-controlled evidence so exact
searches can be performed after the trial, without revealing it to the model
through another tool or prompt.

Waiting build script preparation:

```powershell
Set-Content "$ctxPilot\lib.rs" 'pub fn answer() -> u8 { 42 }'
Set-Content "$ctxPilot\build.rs" @(
    'fn main() {'
    ' std::fs::write("pilot-build-started", std::process::id().to_string()).unwrap();'
    ' for _ in 0..300 { std::thread::sleep(std::time::Duration::from_millis(100)); }'
    '}'
)
```

This 30-second bound is a fixture fail-safe, not a production CargoCheck timeout.
Observe a newly created marker for this trial, then cancel. Restore the build
script to `fn main() {}` and repeat a successful native check for recovery.

## Inspectable evidence and verdicts

Inspect only the active pilot's legitimately exposed tool details, Codex
output/debug views, exported conversation/transcript, and session-owned local
persistence. Record exact surfaces available and unavailable. Do not infer
`structuredContent` consumption from a plausible model summary; distinguish
observed wire content, displayed content, and model use.

Search these surfaces for the exact synthetic marker. Classify each match by its
origin: owner fixture/source, Cargo-owned target artifact, native MCP result,
model response, or host persistence. Cargo writing its own compiler artifacts
does not establish that Clean-CTX created a raw capture spool. A marker in a
native result/model echo/host tool persistence is a separate secrecy finding.

Record absolute-path disclosure independently of secret-token disclosure.
Default diagnostic/evidence display must use the frozen path policy. Do not
hide failed cells behind a successful selection or successful compile.

| Verdict cell | Result | Evidence |
| --- | --- | --- |
| Unguided tool selection | NOT RUN | |
| Guided tool selection | NOT RUN | |
| Absent/unavailable fallback | NOT RUN | |
| Actual content rendering | NOT RUN | |
| Actual structuredContent rendering/consumption | NOT RUN | |
| Both MCP surfaces' token secrecy | NOT RUN | |
| Model-echo token secrecy | NOT RUN | |
| Inspectable host persistence | NOT RUN | |
| Path presentation | NOT RUN | |
| Real-workspace usefulness/nonzero behavior | NOT RUN | |
| Host cancellation and recovery | NOT RUN | |

Use PASS, FAIL, INCONCLUSIVE, or NOT OBSERVABLE with supporting evidence after
execution. Preserve reproducible findings and turn locally testable defects into
tracked regressions before any broader integration claim.
