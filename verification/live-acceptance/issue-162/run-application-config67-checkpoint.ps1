# Agent MaxHeadRoom: major integrated checkpoint after deterministic preservation.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/application-config67-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_application_config_controls_issue_162.rs *> (Join-Path $logDirectory 'config-refusal-format.log')
    if ($LASTEXITCODE -ne 0) { throw "Control formatting failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_application_config_provider_membership_forgery_and_serde_refusal -- --test-threads=1 *> (Join-Path $logDirectory 'config-refusal.log')
    if ($LASTEXITCODE -ne 0) { throw "Expanded refusal control failed; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'check-syntax.ps1') *> (Join-Path $logDirectory 'syntax.log')
    if ($LASTEXITCODE -ne 0) { throw "Syntax preflight failed; see $logDirectory/syntax.log" }
    & cargo build --all-features --bin clean-ctx *> (Join-Path $logDirectory 'build.log')
    if ($LASTEXITCODE -ne 0) { throw "Build failed; see $logDirectory/build.log" }
    foreach ($suiteName in @('application-config-providers','route-providers','provider-imported-owners')) {
        $suiteLog = Join-Path $logDirectory "$suiteName.log"
        & node verification/live-acceptance/issue_162_live_acceptance.mjs ./target/debug/clean-ctx.exe $suiteName *> $suiteLog
        $suiteExit = $LASTEXITCODE
        Get-Content -LiteralPath $suiteLog -Tail 4
        if ($suiteExit -ne 0) { throw "Suite failed: $suiteName; full log: $suiteLog" }
    }
    Write-Output "Checkpoint commands PASS; full logs: $logDirectory"
} finally {
    Pop-Location
}
