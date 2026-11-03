# Agent MaxHeadRoom: one bounded rebuilt-binary suite; local Rust gates already GREEN.
param([string]$ExpectedExistingBinarySha256)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-guard77-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'check-syntax.ps1') *> (Join-Path $logDirectory 'syntax.log')
    if ($LASTEXITCODE -ne 0) { throw "Syntax failed; logs: $logDirectory" }
    $binaryPath = Join-Path $repoPath 'target/debug/clean-ctx.exe'
    if ($ExpectedExistingBinarySha256) {
        if ($ExpectedExistingBinarySha256 -notmatch '^[a-fA-F0-9]{64}$' -or (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $ExpectedExistingBinarySha256) { throw 'Existing binary mismatch; start checkpoint without reuse parameter' }
        Write-Output "Reusing exact rebuilt binary SHA-256: $ExpectedExistingBinarySha256"
    } else {
        $buildMarker = 'issue162-guard77-' + (Get-Date -Format 'yyyyMMddHHmmssfff')
        & cargo rustc --all-features --bin clean-ctx -- -C "metadata=$buildMarker" *> (Join-Path $logDirectory 'build.log')
        if ($LASTEXITCODE -ne 0) { throw "Build failed; logs: $logDirectory" }
    }
    $binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash
    Write-Output "Binary SHA-256: $binaryHash"
    $suiteLog = Join-Path $logDirectory 'route-guard-kinds.log'
    & node verification/live-acceptance/issue_162_live_acceptance.mjs $binaryPath route-guard-kinds *> $suiteLog
    $suiteExit = $LASTEXITCODE
    Get-Content -LiteralPath $suiteLog -Tail 5
    if ($suiteExit -ne 0) { throw "Suite failed; full log: $suiteLog; binary SHA-256: $binaryHash" }
    $summaryLine = Select-String -LiteralPath $suiteLog -Pattern 'summary: (.+)$' | Select-Object -Last 1
    if (-not $summaryLine) { throw 'Suite summary missing' }
    $summaryPath = Join-Path ($summaryLine.Matches[0].Groups[1].Value.Trim()) 'report.json'
    $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
    if ($summary.status -ne 'PASS' -or $summary.binary.sha256 -ne $binaryHash.ToLowerInvariant()) { throw 'Summary binary/status mismatch' }
    $suite = Get-Content -LiteralPath $summary.suites[0].report -Raw | ConvertFrom-Json
    if ($suite.suite -ne 'route-guard-kinds' -or $suite.status -ne 'PASS' -or $suite.checks.Count -ge 500 -or $suite.sessions.Count -ne 3 -or @($suite.sessions | Where-Object status -ne 'FINISHED').Count -ne 0 -or $suite.errors.Count -ne 0) { throw 'Suite receipt/check/session mismatch' }
    if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $binaryHash) { throw 'Binary changed during checkpoint' }
    [pscustomobject]@{ agent = 'Agent MaxHeadRoom'; category = 'operator binary/stdio; not Rust/CI or final field'; commandHash = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash; binary = $binaryPath; binaryHash = $binaryHash; inputsHash = $summary.inputsHash; summary = $summaryPath; summaryHash = (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash; checks = $suite.checks.Count; sessions = $suite.sessions.Count } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'checkpoint.json') -Encoding utf8NoBOM
    Write-Output "Generation-77 binary checkpoint PASS; full logs: $logDirectory"
} finally { Pop-Location }
