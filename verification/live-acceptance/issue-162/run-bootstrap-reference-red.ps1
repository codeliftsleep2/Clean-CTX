# Agent MaxHeadRoom: investigation only; no production semantics or binary run.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap-reference-red-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_bootstrap_reference_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_bootstrap_reference_native_api_refusal_controls -- --test-threads=1 *> (Join-Path $logDirectory 'prerequisites.log')
    if ($LASTEXITCODE -ne 0) { throw "Native prerequisite control failed; no intended association RED; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_bootstrap_reference -- --test-threads=1 --skip issue_162_bootstrap_reference_native_api_refusal_controls *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'tests.log') -Tail 20
    Write-Output "Investigation exit: $testExit. Intended RED requires stage=bootstrap-association after all native prerequisites. Full logs: $logDirectory"
    exit $testExit
} finally {
    Pop-Location
}
