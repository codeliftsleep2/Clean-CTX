# Agent MaxHeadRoom: major integrated checkpoint after deterministic preservation.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-provider66-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'check-syntax.ps1') *> (Join-Path $logDirectory 'syntax.log')
    if ($LASTEXITCODE -ne 0) { throw "Syntax preflight failed; see $logDirectory/syntax.log" }
    & cargo build --all-features --bin clean-ctx *> (Join-Path $logDirectory 'build.log')
    if ($LASTEXITCODE -ne 0) { throw "Build failed; see $logDirectory/build.log" }
    foreach ($suiteName in @('route-providers','provider-imported-owners')) {
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
