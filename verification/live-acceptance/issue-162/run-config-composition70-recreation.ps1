# Agent MaxHeadRoom: one corrected lifecycle control; operator-owned.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/config-composition70-recreation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_config_composition_exit_issue_162.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_config_composition_exit_independent_caller_result_retirement -- --test-threads=1 *> (Join-Path $logDirectory 'test.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'test.log') -Tail 10
    if ($testExit -ne 0) { throw "Recreation test failed; logs: $logDirectory" }
    if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'test.log') -Pattern 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Expected one targeted test; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "New-test lint failed; logs: $logDirectory" }
    Write-Output "Recreation control and lint GREEN; logs: $logDirectory"
} finally { Pop-Location }
