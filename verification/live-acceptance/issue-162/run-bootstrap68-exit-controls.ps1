# Agent MaxHeadRoom: test-only exit strengthening; operator execution only.
param([switch]$ResumeAfterFilesystemFailure)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $logDirectory = Join-Path $repoPath "target/issue-162/bootstrap68-exit-$stamp"
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_bootstrap_controls_issue_162.rs src/mcp/tool_handlers/hydration/bootstrap.rs src/mcp/tool_handlers/hydration.rs src/mcp/tool_handlers/hydration/filesystem.rs src/mcp/tool_handlers/query/edges.rs src/tests/mcp/workspace_query_9.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    if (-not $ResumeAfterFilesystemFailure) {
    & cargo test --lib --all-features --quiet issue_162_bootstrap_controls -- --test-threads=1 *> (Join-Path $logDirectory 'controls.log')
    $testExit = $LASTEXITCODE
    Get-Content (Join-Path $logDirectory 'controls.log') -Tail 10
    if ($testExit -ne 0) { throw "Exit controls failed; logs: $logDirectory" }
    if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'controls.log') -Pattern 'test result: ok\. 8 passed; 0 failed; 0 ignored;' -Quiet)) {
        throw "Expected all eight integrated controls; logs: $logDirectory"
    }
    }
    $filters = @('refresh_workspace', 'tests_hydration_discovery_cache', 'tests_filesystem_discovery_cache', 'issue_162_application_config_provider', 'issue_162_route_provider')
    if ($ResumeAfterFilesystemFailure) { $filters = $filters[2..4] }
    foreach ($filter in $filters) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        if ($LASTEXITCODE -ne 0) { throw "Preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty preservation filter: $filter" }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Fmt failed; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    Write-Output "Eight integrated exit controls and lint GREEN; full logs: $logDirectory"
} finally { Pop-Location }
