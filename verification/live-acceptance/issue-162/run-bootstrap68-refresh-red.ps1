# Agent MaxHeadRoom: deterministic refresh-discovery regression; operator owned.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap68-refresh-red-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_bootstrap_controls_issue_162.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_bootstrap_controls_refresh_rediscovers_unchanged_barrel_caller -- --test-threads=1 *> (Join-Path $logDirectory 'test.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'test.log') -Tail 25
    Write-Output "Investigation exit: $testExit; inspect prerequisite assertions and final forward multiset before classifying RED. Logs: $logDirectory"
} finally { Pop-Location }
