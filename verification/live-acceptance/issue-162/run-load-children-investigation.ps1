# Agent MaxHeadRoom: investigation probes only; operator owns execution.
param([switch]$ResumeModuleFixtureCorrection)
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
try {
    $logDirectory = Join-Path $PWD ('target/issue-162/load-children-investigation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_load_children_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw 'Active sizes failed' }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compilation failed; not behavioral RED' }
    $filter = if ($ResumeModuleFixtureCorrection) { "issue_162_load_children_module_" } else { "issue_162_load_children_" }
    & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> (Join-Path $logDirectory 'investigation.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'investigation.log') -Tail 30
    Write-Output "Investigation exit=$testExit; inspect earliest prerequisite, not assumed semantic RED. Logs: $logDirectory"
    if ($testExit -ne 0) { throw 'Investigation has failures; return logs for classification' }
} finally { Pop-Location }
