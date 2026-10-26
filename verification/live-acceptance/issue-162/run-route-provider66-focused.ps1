# Agent MaxHeadRoom: operator-owned deterministic checkpoint; no Node/runtime pilot.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath 'target/issue-162'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'route-provider66-compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compile failed; see target/issue-162/route-provider66-compile.log' }
    & cargo test --lib --all-features --quiet issue_162_route_provider -- --test-threads=1 *> (Join-Path $logDirectory 'route-provider66-focused.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'route-provider66-focused.log') -Tail 28
    if ($testExit -ne 0) { throw 'Focused controls failed; see target/issue-162/route-provider66-focused.log' }
    Write-Output 'Focused route-provider checkpoint GREEN; logs retained under target/issue-162/route-provider66-*'
} finally {
    Pop-Location
}
