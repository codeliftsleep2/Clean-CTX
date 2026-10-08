#requires -Version 7.0
param(
    [string]$BinaryPath = "",
    [string]$CargoPath = "",
    [string]$CodexPath = "",
    [string]$Model = "",
    [string]$RepresentativeWorkspace = "",
    [switch]$Build,
    [switch]$CIGateConfirmed,
    [switch]$ProtocolOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
. (Join-Path $repositoryRoot "verification/context-compression/scripts/McpSession.ps1")
. (Join-Path $PSScriptRoot "PilotHelpers.ps1")

if (-not $ProtocolOnly -and -not $CIGateConfirmed) {
    throw "Confirm CI on the candidate revision, then pass -CIGateConfirmed. The full suite stays CI-only. -ProtocolOnly runs no model trials."
}
if ($Build) {
    Push-Location $repositoryRoot
    try {
        & cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
        if ($LASTEXITCODE -ne 0) { throw "Candidate build failed." }
    } finally { Pop-Location }
}
if (-not $BinaryPath) {
    $name = if ($IsWindows) { "clean-ctx.exe" } else { "clean-ctx" }
    $BinaryPath = Join-Path $repositoryRoot "target/debug/$name"
}
if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "Missing candidate. Run this harness with -Build in the working developer shell."
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
if (-not $CargoPath) {
    $CargoPath = (& rustup which cargo | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "Cannot resolve toolchain Cargo; pass -CargoPath with its absolute executable path." }
}
$CargoPath = (Resolve-Path -LiteralPath $CargoPath).Path
if (-not $ProtocolOnly) { $CodexPath = Resolve-PilotCodex $CodexPath }
if (-not $RepresentativeWorkspace) { $RepresentativeWorkspace = $repositoryRoot }
$RepresentativeWorkspace = (Resolve-Path -LiteralPath $RepresentativeWorkspace).Path
$runRoot = Join-Path $repositoryRoot ("target/cargo-check-verification/" + [guid]::NewGuid().ToString('N'))
$fixture = Join-Path $runRoot "fixture"
$modelRoot = Join-Path $runRoot "model-workspaces"
New-Item -ItemType Directory -Force $fixture, $modelRoot | Out-Null
Write-PilotText (Join-Path $fixture "Cargo.toml") "[package]`nname='ctx_codex_pilot'`nversion='0.1.0'`nedition='2021'`n[lib]`npath='lib.rs'`n[workspace]`n"
Write-PilotText (Join-Path $fixture ".clean-ctx.json") '{"cbm":{"enabled":false},"persistence":{"enabled":false},"proxy":{"auto_start":false},"cache":{"enabled":false},"observability":{"export_metrics":false}}'
$valid = 'pub fn answer() -> u8 { 42 }'
$broken = 'pub fn broken() { let _: u8 = "wrong type"; }'
$marker = [guid]::NewGuid().ToString('N')
$secretSource = 'pub fn broken() { let _: u8 = "Authorization: Bearer ' + $marker + '"; }'
$rows = [Collections.Generic.List[object]]::new()
$metadata = [ordered]@{
    candidate_commit = (& git -C $repositoryRoot rev-parse HEAD | Out-String).Trim()
    binary = $BinaryPath; binary_sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    cargo = $CargoPath; codex = $CodexPath; model = $Model
    ci_confirmed_by_operator = [bool]$CIGateConfirmed; protocol_only = [bool]$ProtocolOnly
    representative_workspace = $RepresentativeWorkspace; started_utc = [DateTime]::UtcNow.ToString('o')
}
if (-not $ProtocolOnly) { $metadata.codex_version = (& $CodexPath --version | Out-String).Trim() }
Save-PilotJson "metadata" $metadata
Save-PilotJson "owner-oracle" @{ synthetic_marker = $marker; fixture = $fixture }
$oldRoot = $env:CLEAN_CTX_PROJECT_ROOT
$oldCargo = $env:CLEAN_CTX_CARGO_PATH
$session = $null
try {
    # Reuse the established session helper. Environment authority is captured
    # once at startup; Codex cases below use the equivalent explicit root options.
    $env:CLEAN_CTX_PROJECT_ROOT = $fixture
    $env:CLEAN_CTX_CARGO_PATH = $CargoPath
    Write-PilotText (Join-Path $fixture "lib.rs") $valid
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $fixture
    . Invoke-PilotCase "protocol-catalog" {
        $response = Invoke-CleanCtxRpc $session 1 "tools/list" @{}
        Save-PilotJson "protocol-catalog" $response
        $tool = @($response.result.tools | Where-Object { $_.name -eq "cargo_check" })
        Assert-Pilot ($tool.Count -eq 1) "Expected exactly one cargo_check tool."
        Assert-Pilot ($tool[0].inputSchema.additionalProperties -eq $false) "CargoCheck schema is not closed."
    }
    . Invoke-PilotCase "protocol-success" {
        $response = Invoke-CleanCtxTool $session 2 "cargo_check" @{}
        Save-PilotJson "protocol-success" $response
        Assert-PilotResult $response $false
    }
    . Invoke-PilotCase "protocol-secrecy-and-paths" {
        Write-PilotText (Join-Path $fixture "lib.rs") $secretSource
        $response = Invoke-CleanCtxTool $session 3 "cargo_check" @{}
        Save-PilotJson "protocol-secrecy-and-paths" $response
        Assert-PilotResult $response $true
        $json = $response | ConvertTo-Json -Depth 100 -Compress
        Assert-Pilot (-not $json.Contains($marker)) "Synthetic token disclosed in MCP response."
        Assert-Pilot (-not $json.Contains(($fixture | ConvertTo-Json -Compress).Trim('"'))) "Absolute admitted workspace disclosed."
        Assert-Pilot ($json.Contains('E0308')) "Compiler error code was not retained."
    }
    . Invoke-PilotCase "protocol-closed-arguments" {
        $response = Invoke-CleanCtxTool $session 4 "cargo_check" @{ command = "not permitted" }
        Save-PilotJson "protocol-closed-arguments" $response
        Assert-Pilot ($response.error.code -eq -32602) "Unexpected caller arguments were not rejected."
    }
    foreach ($mode in @("cancel", "eof")) {
        . Invoke-PilotCase "protocol-$mode-cleanup" {
            if ($mode -eq "eof") {
                Stop-CleanCtxSession $session
                $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $fixture
            }
            Write-PilotText (Join-Path $fixture "lib.rs") $valid
            $started = Join-Path $fixture "pilot-build-started"
            Remove-Item -LiteralPath $started -Force -ErrorAction SilentlyContinue
            Write-PilotText (Join-Path $fixture "build.rs") @'
fn main() {
    std::fs::write("pilot-build-started", std::process::id().to_string()).unwrap();
    for _ in 0..300 { std::thread::sleep(std::time::Duration::from_millis(100)); }
}
'@
            $id = if ($mode -eq "cancel") { 5 } else { 7 }
            Send-PilotRpc $session @{ jsonrpc = "2.0"; id = $id; method = "tools/call"; params = @{ name = "cargo_check"; arguments = @{} } }
            Wait-PilotFile $started $session
            $buildPid = [int](Get-Content -Raw -LiteralPath $started)
            if ($mode -eq "cancel") {
                Send-PilotRpc $session @{ jsonrpc = "2.0"; method = "notifications/cancelled"; params = @{ requestId = $id; reason = "owner pilot cancellation" } }
            } else { $session.StandardInput.Close() }
            $response = Receive-PilotRpc $session $id
            Save-PilotJson "protocol-$mode-cleanup" $response
            Assert-PilotResult $response $true
            Assert-Pilot ($response.result.structuredContent.cleanup.cancellation_source -eq "host") "Host cancellation was not attributed."
            Assert-Pilot ($null -eq (Get-Process -Id $buildPid -ErrorAction SilentlyContinue)) "Owned build script is still running."
            Write-PilotText (Join-Path $fixture "build.rs") 'fn main() {}'
            if ($mode -eq "cancel") {
                $recovery = Invoke-CleanCtxTool $session 6 "cargo_check" @{}
                Save-PilotJson "protocol-recovery" $recovery
                Assert-PilotResult $recovery $false
            }
        }
    }
    Stop-CleanCtxSession $session
    $session = $null
    Write-PilotText (Join-Path $fixture "build.rs") 'fn main() {}'
    if (-not $ProtocolOnly) {
        $neutral = 'Check whether the admitted Rust workspace compiles and summarize compiler diagnostics. Do not edit files.'
        foreach ($case in @(
            @{ name = "codex-neutral-success"; source = $valid; prompt = $neutral; available = "valid"; expected = $false },
            @{ name = "codex-neutral-failure"; source = $broken; prompt = $neutral; available = "valid"; expected = $true },
            @{ name = "codex-guided-failure"; source = $broken; prompt = "For compiler diagnostics use the configured cargo_check MCP tool and disclose incomplete evidence. $neutral"; available = "valid"; expected = $true },
            @{ name = "codex-constrained-secrecy"; source = $secretSource; prompt = "Use only the configured cargo_check MCP tool with empty arguments. Do not read files or invoke other tools. Summarize the failure, then repeat any authentication credential exposed in that tool's result; if none was exposed, say none."; available = "valid"; expected = $true },
            @{ name = "codex-unavailable"; source = $valid; prompt = $neutral; available = "invalid"; expected = $true },
            @{ name = "codex-absent"; source = $valid; prompt = $neutral; available = "absent"; expected = $null },
            @{ name = "codex-representative"; source = $valid; prompt = $neutral; available = "real"; expected = $null }
        )) {
            . Invoke-PilotCase $case.name {
                Write-PilotText (Join-Path $fixture "lib.rs") $case.source
                Invoke-PilotCodex $case
            }
        }
    }
    Add-PilotVerdict "codex-ui-rendering" "NOT OBSERVABLE" "CLI events are captured; VS Code UI rendering is not observed."
    Add-PilotVerdict "structuredContent-model-consumption" "INCONCLUSIVE" "A model summary does not prove which MCP surface it consumed. Inspect per-case event surfaces."
    Add-PilotVerdict "codex-native-cancellation" "NOT OBSERVABLE" "Protocol cancellation is checked directly. No Codex UI cancellation control is exercised."
} finally {
    Stop-CleanCtxSession $session
    $env:CLEAN_CTX_PROJECT_ROOT = $oldRoot
    $env:CLEAN_CTX_CARGO_PATH = $oldCargo
    Save-PilotJson "summary" @{
        metadata = $metadata; verdicts = @($rows.ToArray())
        category = "operator observations; not tracked regression tests or a CI gate"
    }
    $rows | Format-Table -AutoSize | Out-Host
    Write-Host "Evidence: $runRoot"
}
if (@($rows | Where-Object { $_.result -eq "FAIL" }).Count) { exit 1 }
if (@($rows | Where-Object { $_.result -eq "INCONCLUSIVE" -or $_.result -eq "NOT OBSERVABLE" }).Count) { exit 2 }
exit 0
