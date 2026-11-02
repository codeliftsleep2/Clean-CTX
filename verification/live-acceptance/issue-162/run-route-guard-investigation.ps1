# Agent MaxHeadRoom: bounded canActivate investigation; no semantic repair.
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
try {
    $logDirectory = Join-Path $PWD ('target/issue-162/route-guard-investigation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_route_guard_membership_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw 'Active sizes failed' }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compilation failed; not behavioral RED' }
    & cargo test --lib --all-features --quiet issue_162_route_guard_ -- --test-threads=1 *> (Join-Path $logDirectory 'investigation.log')
    $probeExit = $LASTEXITCODE
    Select-String -LiteralPath (Join-Path $logDirectory 'investigation.log') -Pattern 'panicked at|^test result:|^    mcp::' | ForEach-Object { $_.Line }
    Write-Output "Investigation exit=$probeExit; classify field/array/reference prerequisites before association RED. Logs: $logDirectory"
    if ($probeExit -ne 0) { throw 'Investigation has failures; return logs for classification' }
} finally { Pop-Location }
