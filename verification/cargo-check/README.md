# CargoCheck operator verification with Codex

This package follows the existing verification patterns:

- `verification/context-compression/scripts/McpSession.ps1` supplies MCP session
  creation, initialization, RPC/tool calls, and shutdown. The harness dot-sources
  it without changing it.
- `verification/context-compression/codec/scripts/Run-CodexReasoning.ps1` supplies
  the Codex CLI discovery and `codex exec` pattern: fresh working directory,
  stdin prompt, read-only sandbox, ignored user configuration/rules, disabled
  plugins, and captured final answer. This harness enables only the pilot MCP
  server with per-invocation configuration instead of disabling all servers.

No manual MCP registration or VS Code restart is needed. Codex launches the
same candidate stdio server for each fresh model trial. Your normal Codex MCP
configuration and existing servers are not modified. Existing Codex
authentication is reused; no credential is requested or copied.

## Run

Requirements: PowerShell 7, working Rust/Cargo, and an authenticated Codex CLI
(the installed VS Code extension's executable is also discovered using the
existing runner's pattern). On Windows, use the MSVC-loaded PowerShell that
passed the real-Cargo lifecycle checks. If the CLI's default model is unsuitable,
pass `-Model` with your configured model name.

After the candidate's CI gate is green, run from the repository root:

```powershell
pwsh -NoProfile -File .\verification\cargo-check\scripts\Verify-CargoCheckCodex.ps1 -Build -CIGateConfirmed
```

`-CIGateConfirmed` records your confirmation; the harness does not query CI or
prove that it passed. Do not supply it while CI is unknown or failing. No full
test suite is run. `-Build` builds the candidate once with all features; the
same binary is used throughout the run.

Optional parameters:

| Parameter | Use |
| --- | --- |
| `-BinaryPath` | An existing candidate executable; omit `-Build` if already built. |
| `-CargoPath` | Approved absolute toolchain Cargo executable; otherwise resolves `rustup which cargo`. |
| `-CodexPath` | Explicit Codex CLI path when standard discovery does not find it. |
| `-Model` | Explicit model; otherwise the CLI default under isolated configuration. |
| `-RepresentativeWorkspace` | Approved real Rust workspace; defaults to this checkout. Invoking the harness approves this workspace for its CargoCheck trial. |
| `-ProtocolOnly` | Runs direct MCP observations without Codex or the live-pilot CI prerequisite. |

For protocol observations alone:

```powershell
pwsh -NoProfile -File .\verification\cargo-check\scripts\Verify-CargoCheckCodex.ps1 -ProtocolOnly
```

Do not paste PowerShell `>>` continuation prompts into these commands.

## What runs

The harness creates a dependency-free Rust fixture and records candidate commit,
binary SHA-256, Cargo path, Codex version/model selection, and operator CI
confirmation. Generated files live in a unique
`target/cargo-check-verification/<run-id>/` directory; repeated runs preserve
earlier evidence. No source file in your real workspace is edited. Cargo itself
may create normal build artifacts or fetch dependencies in that admitted
workspace.

Direct MCP observations cover catalog presence/closed arguments, success,
E0308 with synthetic credential redaction and workspace path display, forced
host cancellation, owned build-script termination, recovery in the same session,
and EOF cleanup. The waiting fixture has a 30-second fail-safe, not a production
timeout. Cleanup/cancellation observations come from real MCP requests and the
real build-script process. These do not claim that Codex sent the cancellation.
The readiness wait concurrently drains the MCP response. If Cargo returns before
the build-script marker, the sanitized response is saved in
`protocol-cancel-startup.json` or `protocol-eof-startup.json` and a bounded text
excerpt is printed. If the readiness deadline expires while Cargo is running,
the harness cancels the request and saves its outcome. Diagnose this evidence
before attributing readiness failures to a compiler, MSVC, or antivirus issue.
Each cleanup case also writes a stages capture: request sent, marker/PID
observed, cancellation or EOF sent, and child cleanup verified. Preparatory
failure is reported separately as readiness FAIL and cleanup NOT RUN. Cargo
process creation and live build-script compilation are not exposed as progress
events by this MCP operation and are explicitly marked unobserved; the driver
does not infer them from elapsed time. The successful protocol case now includes
a trivial build script to verify compilation/linking before the waiting cases.

Fresh Codex invocations cover neutral success/failure, explicit guidance,
constrained synthetic secrecy, unavailable authority, absent provider fallback,
and the approved representative workspace. Each uses the same binary with a
closed CargoCheck-only MCP provider. CLI JSON events identify actual selected
tools and any other actions. A terminal fallback cannot satisfy native selection.
The constrained secrecy case is invalidated if other actions occur; its token
is never placed in a model prompt. The model working directory is separate from
the fixture. This is observational discipline, not an OS restriction preventing
every source read; all exposed action events must be checked.

The constrained case uses a persistent Codex invocation so the harness can
inspect only rollout files matching that invocation's returned thread ID under
the active `CODEX_HOME/sessions` (or the standard user Codex home). It scans those
files for the exact synthetic marker without copying credentials or unrelated
sessions. Missing/unexposed logs are `NOT OBSERVABLE`; this is not a claim about
every possible host store. The owner oracle and fixture/compiler artifacts are
separate from native response/model/host persistence evidence.

## Read the results

The console prints the run directory and verdict table. `summary.json` is the
machine-readable report. Each case preserves RPC responses or Codex prompt,
JSONL events, final answer, stderr, and selection/result surface observations.
Per-case registration captures show the requested CLI configuration; event
inventories preserve the observed event/item identities, server/tool tuple,
statuses, and errors. They distinguish a tool never recognized from an observed
terminal failure. The parser matches `clean_ctx_cargo_pilot` / `cargo_check`, not
an invented Codex-native tool name. Unknown event shapes require raw evidence;
they are not silently normalized into successful tool calls.
`owner-oracle.json` contains the synthetic token for exact owner inspection;
keep it out of model prompts. No real credential is used as a test marker.

Exit codes:

- `0`: all recorded observations passed (normally only a fully observable lane).
- `1`: at least one observation failed; inspect its capture/error.
- `2`: no failures, but incomplete/inconclusive/unobservable cells remain.

The full Codex run intentionally retains explicit limitations: CLI evidence
does not establish VS Code UI rendering or which result channel the model used;
direct protocol cancellation does not establish Codex UI cancellation. The real
workspace's usefulness and absent/unavailable model fallback should also be
reviewed in their saved answers. Do not convert these into an overall pilot PASS
because tool selection or protocol checks passed.

These are operator observations, not tracked regression test results and not a
substitute for CI. Any discovered product defect must receive a tracked test
under `src/tests/**`. See `docs/architecture/CARGOCHECK_CODEX_PILOT.md` for the
independent pilot verdicts and host-scope limitations.

The harness has not been executed on Windows by the cloud agent. PowerShell is
not installed in that cloud session; user execution remains necessary to verify
the driver and installed Codex CLI compatibility.
