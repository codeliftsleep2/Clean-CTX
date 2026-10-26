# Agent MaxHeadRoom: tracked Rust investigation, not binary/live acceptance.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath 'target/issue-162'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'route-provider-compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compile failed; see target/issue-162/route-provider-compile.log' }
    & cargo test --lib --all-features --quiet issue_162_route_provider -- --test-threads=1 *> (Join-Path $logDirectory 'route-provider-red.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'route-provider-red.log') -Tail 28
    Write-Output "Test exit: $testExit; complete output: target/issue-162/route-provider-red.log"
    if ($testExit -ne 0) { throw 'Tracked controls failed; inspect the full log to classify the failure.' }
} finally {
    Pop-Location
}
