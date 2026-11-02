# Agent MaxHeadRoom: approved four remaining guard demand names only verification; unchanged probes last.
param([switch]$ResumeAtLint)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-guard-kinds-field-capture-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_route_guard_kinds_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/module_metadata.rs src/tests/mcp/typescript_route_guard_field_support_issue_162.rs src/tests/mcp/typescript_guard_kinds_field_support_issue_162.rs src/tests/mcp/typescript_guard_kinds_field_issue_162.rs src/tests/mcp/typescript_guard_kinds_field_lifecycle_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen probes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    if (-not $ResumeAtLint) {
        & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
        if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    }
    # Representative affected contracts, not every prior campaign/lifecycle suite.
    $filters = @('issue_162_guard_kinds_field_','issue_162_guard_semantic_','issue_162_guard_controls_other_kinds_same_target_and_outside_legacy','issue_162_guard_controls_broad_runtime_values_partial_slots_and_no_fallback','issue_162_route_provider_direct_membership','issue_162_route_component_direct','issue_162_lazy_semantic_direct','issue_162_children_semantic_module_direct','issue_162_children_semantic_routes_direct','issue_162_module_relations_','issue_162_bootstrap_controls_independent_local_scoped_and_unsupported_arguments','issue_162_legacy_bootstrap_direct','issue_162_config_composition_direct','issue_162_application_config_provider_direct','issue_162_bootstrap_inline_provider_direct','provider_semantics')
    if ($ResumeAtLint) { $filters = @() }
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
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_guard_kind_ -- --test-threads=1 --nocapture *> (Join-Path $logDirectory 'investigation.log')
    $probeExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'investigation.log') -Tail 24
    Write-Output "Investigation exit: $probeExit. Classify prerequisites separately; Return each field's earliest remaining prerequisite. Success certifies acquisition only; candidate guard-kind semantics remain unapproved. Full logs: $logDirectory"
    exit $probeExit
} finally { Pop-Location }
