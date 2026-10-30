# Agent MaxHeadRoom: only the newly authored deterministic exit batch.
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
try {
    $logDirectory = Join-Path $PWD ('target/issue-162/lazy-route74-supplement-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_lazy_route_controls_issue_162.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw 'Active sizes failed' }
    & cargo test --lib --all-features --quiet issue_162_lazy_exit_object_child_field_and_root_retirement -- --test-threads=1 *> (Join-Path $logDirectory 'exit.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'exit.log') -Tail 8
    if ($testExit -ne 0) { throw 'Supplement failed' }
    if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'exit.log') -Pattern 'test result: ok\. 1 passed;' -Quiet)) { throw 'Expected one supplemental test' }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw 'fmt failed' }
    Write-Output "Supplement GREEN; binary/final repository/field gates remain. Full logs: $logDirectory"
} finally { Pop-Location }
