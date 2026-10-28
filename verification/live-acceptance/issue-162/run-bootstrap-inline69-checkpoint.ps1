# Agent MaxHeadRoom: one major integrated rebuilt-binary checkpoint.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap-inline69-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'check-syntax.ps1') *> (Join-Path $logDirectory 'syntax.log')
    if ($LASTEXITCODE -ne 0) { throw "Syntax failed; logs: $logDirectory" }
    & cargo build --all-features --bin clean-ctx *> (Join-Path $logDirectory 'build.log')
    if ($LASTEXITCODE -ne 0) { throw "Build failed; logs: $logDirectory" }
    $binaryPath = Join-Path $repoPath 'target/debug/clean-ctx.exe'
    $binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash
    foreach ($suiteName in @('bootstrap-inline-providers','bootstrap-references','application-config-providers')) {
        if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $binaryHash) {
            throw "Binary changed between suites; logs: $logDirectory"
        }
        $suiteLog = Join-Path $logDirectory "$suiteName.log"
        & node verification/live-acceptance/issue_162_live_acceptance.mjs $binaryPath $suiteName *> $suiteLog
        $suiteExit = $LASTEXITCODE
        Get-Content -LiteralPath $suiteLog -Tail 5
        if ($suiteExit -ne 0) { throw "Suite failed: $suiteName; full log: $suiteLog" }
    }
    Write-Output "Integrated checkpoint commands PASS; full logs: $logDirectory"
} finally { Pop-Location }
