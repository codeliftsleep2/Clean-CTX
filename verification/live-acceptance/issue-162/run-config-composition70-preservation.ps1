# Agent MaxHeadRoom: supplemental exit plus pending preservation, not focused repetition.
param([switch]$ResumeAfterRefusalFailure)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/config-composition70-preserve-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_config_composition_exit_issue_162.rs src/tests/mcp/angular_config_composition_controls_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    $filters = @('issue_162_config_composition_exit','ir::typescript_resolution::callable_scope_tests::','ir::typescript_resolution::type_query_tests::','issue_162_provider','issue_162_view_provider','issue_162_string','issue_162_module','provider_semantics','issue_162_route_provider','issue_162_bootstrap_reference','issue_162_bootstrap_inline','durable_semantic_restore')
    if ($ResumeAfterRefusalFailure) { $filters[0] = 'issue_162_config_composition_exit_api_type_and_input_refusals' }
    foreach ($filter in $filters) {
        $logName = $filter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 8
        if ($testExit -ne 0) { throw "Tests failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Fmt failed; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    Write-Output "Selected exit/preservation and new-test lint GREEN; logs: $logDirectory"
} finally { Pop-Location }
