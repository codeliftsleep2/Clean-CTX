# Agent MaxHeadRoom: approved investigation only; no production repair.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap-inline-provider-red-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_bootstrap_inline_provider_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; no behavioral RED; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_bootstrap_inline_provider -- --test-threads=1 *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'tests.log') -Tail 30
    Write-Output "Investigation exit: $testExit. Intended RED requires stage=inline-membership after native/structure/provider prerequisites. Full logs: $logDirectory"
} finally { Pop-Location }
