# Agent MaxHeadRoom: approved loadComponent field-only verification; unchanged lazy investigation last.
param([switch]$ResumeAfterOwnerHashCorrection)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/lazy-route-field-capture-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_lazy_route_component_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/module_metadata.rs src/tests/mcp/typescript_lazy_route_field_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen probes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    $filters = @('issue_162_lazy_field','angular_lazy_route_component_tests::issue_162_lazy_route_unsupported_siblings_observed','angular_route_component_membership_tests','issue_162_route_component_controls','issue_162_route_component_exit','angular_meta::routing::','issue_162_route_component_capture','issue_162_route_provider','provider_semantics','issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition')
    if ($ResumeAfterOwnerHashCorrection) {
        $filters[0] = 'issue_162_lazy_field_sqlite_before_queries_and_owner_currentness'
    }
    foreach ($filter in $filters) {
        $logName = $filter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Capture/preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet angular_lazy_route_component_tests -- --test-threads=1 *> (Join-Path $logDirectory 'investigation.log')
    $probeExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'investigation.log') -Tail 24
    Write-Output "Investigation exit: $probeExit. Classify prerequisites separately; lazy production/callback authority remain unapproved. Full logs: $logDirectory"
    exit $probeExit
} finally { Pop-Location }
