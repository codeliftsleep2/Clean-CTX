# Agent MaxHeadRoom: one major integrated rebuilt-binary checkpoint.
param([string]$ExpectedExistingBinarySha256, [ValidateSet('all','modules-only','after-modules')][string]$Selection = 'all')
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/load-children75-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'check-syntax.ps1') *> (Join-Path $logDirectory 'syntax.log')
    if ($LASTEXITCODE -ne 0) { throw "Syntax failed; logs: $logDirectory" }
    $binaryPath = Join-Path $repoPath 'target/debug/clean-ctx.exe'
    if ($ExpectedExistingBinarySha256) {
        if ($ExpectedExistingBinarySha256 -notmatch '^[a-fA-F0-9]{64}$' -or (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $ExpectedExistingBinarySha256) {
            throw 'Existing binary hash mismatch; rebuild without ExpectedExistingBinarySha256'
        }
        Write-Output "Reusing exact rebuilt binary SHA-256: $ExpectedExistingBinarySha256"
    } else {
        # Force this final target afresh when starting a new binary checkpoint.
        $buildMarker = 'issue162-children75-' + (Get-Date -Format 'yyyyMMddHHmmssfff')
        & cargo rustc --all-features --bin clean-ctx -- -C "metadata=$buildMarker" *> (Join-Path $logDirectory 'build.log')
        if ($LASTEXITCODE -ne 0) { throw "Build failed; logs: $logDirectory" }
    }
    $binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash
    $checkpointInputsHash = $null
    $suiteReceipts = @()
    $selectedSuites = switch ($Selection) {
        'modules-only' { @('load-children-modules') }
        'after-modules' { @('load-children-routes','lazy-route-components','route-providers') }
        default { @('load-children-modules','load-children-routes','lazy-route-components','route-providers') }
    }
    foreach ($suiteName in $selectedSuites) {
        if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $binaryHash) {
            throw "Binary changed between suites; logs: $logDirectory"
        }
        $suiteLog = Join-Path $logDirectory "$suiteName.log"
        & node verification/live-acceptance/issue_162_live_acceptance.mjs $binaryPath $suiteName *> $suiteLog
        $suiteExit = $LASTEXITCODE
        Get-Content -LiteralPath $suiteLog -Tail 5
        if ($suiteExit -ne 0) { throw "Suite failed: $suiteName; full log: $suiteLog" }
        $summaryLine = Select-String -LiteralPath $suiteLog -Pattern 'summary: (.+)$' | Select-Object -Last 1
        if (-not $summaryLine) { throw "Suite summary missing: $suiteName" }
        $summaryDirectory = $summaryLine.Matches[0].Groups[1].Value.Trim()
        $summaryPath = Join-Path $summaryDirectory 'report.json'
        $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
        if ($summary.status -ne 'PASS' -or $summary.binary.sha256 -ne $binaryHash.ToLowerInvariant()) { throw "Suite receipt mismatch: $suiteName" }
        if ($null -eq $checkpointInputsHash) { $checkpointInputsHash = $summary.inputsHash }
        if ($summary.inputsHash -ne $checkpointInputsHash) { throw "Recorded inputs changed between suites: $suiteName" }
        $suiteReceipts += [pscustomobject]@{ suite = $suiteName; summary = $summaryPath; sha256 = (Get-FileHash -LiteralPath $summaryPath -Algorithm SHA256).Hash }
    }
    [pscustomobject]@{ agent = 'Agent MaxHeadRoom'; category = 'operator binary/stdio; not Rust/CI or final field'; selection = $Selection; binary = $binaryPath; binaryHash = $binaryHash; inputsHash = $checkpointInputsHash; suites = $suiteReceipts } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logDirectory 'checkpoint.json') -Encoding utf8NoBOM
    Write-Output "Selected checkpoint commands PASS ($Selection); full logs: $logDirectory"
} finally { Pop-Location }
