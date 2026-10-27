# Agent MaxHeadRoom: operator-owned tracked investigation; no binary/Node pilot.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath 'target/issue-162'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'application-config-provider-compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compile failed; see target/issue-162/application-config-provider-compile.log' }
    & cargo test --lib --all-features --quiet issue_162_application_config_provider -- --test-threads=1 *> (Join-Path $logDirectory 'application-config-provider-red.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'application-config-provider-red.log') -Tail 28
    Write-Output "Test exit: $testExit; full output: target/issue-162/application-config-provider-red.log"
    if ($testExit -ne 0) { throw 'Controls failed; inspect the full log before classifying RED evidence.' }
} finally {
    Pop-Location
}
