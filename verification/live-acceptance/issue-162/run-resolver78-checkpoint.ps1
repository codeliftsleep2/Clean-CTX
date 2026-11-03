# Agent MaxHeadRoom: approved Phase B, gated on exact Phase A source identity.
param([Parameter(Mandatory=$true)][string]$PhaseAReceipt)
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $phase=Get-Content -LiteralPath $PhaseAReceipt -Raw | ConvertFrom-Json
    if($phase.status -ne 'PASS' -or $phase.results.Count -ne 8 -or @($phase.results | Where-Object exitCode -ne 0).Count -ne 0){throw 'Phase A must have eight passing tracked results'}
    foreach($item in $phase.files){if((Get-FileHash -LiteralPath $item.path -Algorithm SHA256).Hash -ne $item.sha256){throw "Phase A source changed: $($item.path)"}}
    $logDirectory=Join-Path $repoPath ('target/issue-162/resolver78-binary-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Phase B logs: $logDirectory"
    & node --check verification/live-acceptance/issue-162/route-resolver78.mjs *> (Join-Path $logDirectory 'syntax.log')
    if($LASTEXITCODE -ne 0){throw 'Resolver suite syntax failed'}
    & node --check verification/live-acceptance/issue-162/suites/route-resolver78.mjs *> (Join-Path $logDirectory 'entry-syntax.log')
    if($LASTEXITCODE -ne 0){throw 'Resolver entry syntax failed'}
    $marker='issue162-resolver78-'+(Get-Date -Format 'yyyyMMddHHmmssfff')
    & cargo rustc --all-features --bin clean-ctx -- -C "metadata=$marker" *> (Join-Path $logDirectory 'build.log')
    if($LASTEXITCODE -ne 0){throw "Build failed: $logDirectory"}
    $binary=Join-Path $repoPath 'target/debug/clean-ctx.exe'
    $binaryHash=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash
    & node verification/live-acceptance/issue_162_live_acceptance.mjs $binary route-resolver78 *> (Join-Path $logDirectory 'suite.log')
    $suiteExit=$LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'suite.log') -Tail 8
    if($suiteExit -ne 0){throw "Binary suite failed: $logDirectory"}
    $summaryLine=Select-String -LiteralPath (Join-Path $logDirectory 'suite.log') -Pattern 'summary: (.+)$' | Select-Object -Last 1
    if(-not $summaryLine){throw 'Missing suite summary'}
    $summaryPath=Join-Path ($summaryLine.Matches[0].Groups[1].Value.Trim()) 'report.json'
    $summary=Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
    if($summary.status -ne 'PASS' -or $summary.binary.sha256 -ne $binaryHash.ToLowerInvariant()){throw 'Summary binary/status mismatch'}
    $suite=Get-Content -LiteralPath $summary.suites[0].report -Raw | ConvertFrom-Json
    if($suite.suite -ne 'route-resolver78' -or $suite.status -ne 'PASS' -or $suite.sessions.Count -ne 3 -or @($suite.sessions | Where-Object status -ne 'FINISHED').Count -ne 0 -or $suite.errors.Count -ne 0){throw 'Incomplete sessions or failed suite'}
    $scenarios=@($suite.checks | Where-Object {$_.name -like 'scenario/*'})
    if($scenarios.Count -ne 26 -or @($scenarios | Where-Object pass -ne $true).Count -ne 0){throw 'Missing/failed required scenarios'}
    foreach($item in $phase.files){if((Get-FileHash -LiteralPath $item.path).Hash -ne $item.sha256){throw "Source changed during checkpoint: $($item.path)"}}
    if((Get-FileHash -LiteralPath $binary).Hash -ne $binaryHash){throw 'Binary changed during checkpoint'}
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='operator binary/stdio; not Rust/CI/field';status='PASS';phaseAReceipt=(Resolve-Path -LiteralPath $PhaseAReceipt).Path;phaseAHash=(Get-FileHash -LiteralPath $PhaseAReceipt).Hash;sourceFiles=$phase.files;binarySha256=$binaryHash;commandHash=(Get-FileHash -LiteralPath $PSCommandPath).Hash;summary=$summaryPath;summaryHash=(Get-FileHash -LiteralPath $summaryPath).Hash;fixtureSha256=([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes(($suite.fixture | ConvertTo-Json -Depth 20 -Compress)))));checks=$suite.checks.Count;scenarios=$scenarios.Count;sessions=$suite.sessions} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Phase B PASS: $logDirectory"
} finally {Pop-Location}
