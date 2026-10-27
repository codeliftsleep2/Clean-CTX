# Agent MaxHeadRoom: scoped tracked preservation and lint; operator execution only.
param([string]$StartAt = '')
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$filters = @(
    'ir::typescript_resolution::lexical_site_tests::',
    'ir::typescript_resolution::pattern_binding_tests::',
    'ir::typescript_resolution::iteration_binding_tests::',
    'ir::typescript_resolution::callable_scope_tests::',
    'ir::typescript_resolution::namespace_lexical_tests::',
    'ir::typescript_resolution::namespace_scoped_use_tests::',
    'ir::typescript_resolution::type_query_tests::',
    'ir::typescript_resolution::type_query_boundary_tests::',
    'ir::typescript_resolution::type_occurrence_tests::',
    'issue_162_provider', 'issue_162_view_provider', 'issue_162_string',
    'issue_162_module', 'provider_semantics',
    'issue_162_application_config_provider', 'issue_162_route_provider',
    'issue_162_functional', 'issue_162_inject', 'issue_162_constructor',
    'durable_semantic_restore', 'core::restore::compatibility_tests',
    'core::delta::fidelity_persistence_tests'
)
# Resume explicitly at the failed filter; earlier logs remain separate evidence.
# Use StartAt=lint only after the selected tests have completed successfully.
if ($StartAt -eq 'lint') { $filters = @() }
elseif ($StartAt) {
    $startIndex = [Array]::IndexOf($filters, $StartAt)
    if ($startIndex -lt 0) { throw "Unknown StartAt filter: $StartAt" }
    $filters = $filters[$startIndex..($filters.Length - 1)]
}
Push-Location $repoPath
try {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $logDirectory = Join-Path $repoPath "target/issue-162/bootstrap68-preserve-$stamp"
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    $filters | Set-Content -LiteralPath (Join-Path $logDirectory 'selected-filters.txt') -Encoding utf8NoBOM
    Write-Output "Full logs: $logDirectory"
    foreach ($testFilter in $filters) {
        $logName = $testFilter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $testFilter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 8
        if ($testExit -ne 0) { throw "Tracked tests failed: $testFilter; full log: $testLog" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) {
            throw "No passing tests selected: $testFilter; full log: $testLog"
        }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; full log: $logDirectory/fmt.log" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; full log: $logDirectory/clippy.log" }
    Write-Output "Selected scoped preservation and lint commands GREEN; full logs: $logDirectory"
} finally {
    Pop-Location
}
