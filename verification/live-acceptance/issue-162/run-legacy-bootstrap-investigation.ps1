# Agent MaxHeadRoom: investigation only; prerequisite failures are not semantic RED.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/legacy-bootstrap-investigation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_legacy_bootstrap_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_legacy_bootstrap -- --test-threads=1 *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'tests.log') -Tail 24
    Write-Output "Investigation exit: $testExit. Inspect stage markers: factory/receiver acquisition failures are prerequisites, not intended legacy-module-association RED. Full logs: $logDirectory"
    exit $testExit
} finally { Pop-Location }
