# Agent MaxHeadRoom: supplemental approval controls only; no broad test repetition.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap-inline69-exit-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_bootstrap_inline_exit_issue_162.rs src/tests/mcp/angular_bootstrap_inline_controls_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_bootstrap_inline_exit -- --test-threads=1 *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'tests.log') -Tail 12
    if ($testExit -ne 0) { throw "Exit controls failed; logs: $logDirectory" }
    if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'tests.log') -Pattern 'test result: ok\. 3 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Expected three exit controls; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Fmt failed; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    Write-Output "Supplemental controls and new-test lint GREEN; logs: $logDirectory"
} finally { Pop-Location }
