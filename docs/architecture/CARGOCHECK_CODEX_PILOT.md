# CargoCheck bundled Codex pilot — owner handoff

Status: prepared, not executed. The owner selected Codex in the Rust build
environment for this CargoCheck pilot. Record the exact Codex client (CLI or
VS Code extension), version, and execution environment before starting.

This replaces the earlier Claude pilot plan for CargoCheck. Rust is used in the
owner's testing/build workflow; Claude is used for C#/TypeScript and meta-layer
workflows. Codex observations establish only the tested Codex host behavior.
Claude validation for those other workflows remains separate and requires
relevant diagnostic operations; this pilot does not establish their readiness.

The pilot uses one frozen rebuilt candidate and one preserved context package.
Run all cases during one planned host setup. Keep tool selection, result
rendering, secrecy, persistence, diagnostic usefulness, and cancellation as
independent verdicts. A result from an ordinary Bash/terminal command is not
evidence of the direct-MCP boundary.

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

After readiness is green, run one candidate build in the MSVC-loaded PowerShell
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

For Codex clients using `config.toml`, the descriptor above corresponds to a
separate entry like this (replace all example paths with the recorded paths):

```toml
[mcp_servers.clean_ctx_cargo_pilot]
command = 'C:\absolute\path\clean-ctx.exe'
args = ['--workspace-root', 'C:\absolute\path\ctx-codex-pilot', '--cargo-path', 'C:\absolute\path\cargo.exe']
```

Add the entry to the client's active configuration; preserve existing entries.
Use Windows paths only for a Windows-native client/server. A WSL or remote client
must use paths and a binary valid in that execution environment. Reconnect the
pilot server as required by the client, and verify `cargo_check` is listed before
prompting. Configuration alone does not prove that Codex can invoke the tool.

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
