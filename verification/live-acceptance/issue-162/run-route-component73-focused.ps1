# Agent MaxHeadRoom: approved generation-73 canonical eager-route verification.
param([switch]$ResumeAfterControlCorrections, [switch]$ResumeAfterGenerationControls)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-component73-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_route_component_membership_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/route_objects.rs src/angular_meta/route_providers.rs src/angular_meta/route_components.rs src/angular_meta/routing.rs src/angular_meta/routing/structured.rs src/angular_meta/mod.rs src/layers/meta/angular_semantic.rs src/layers/meta/mod.rs src/ir/typescript_resolution/relation_evidence.rs src/tests/mcp/angular_route_component_controls_issue_162.rs src/tests/mcp/angular_route_component_lifecycle_issue_162.rs src/tests/mcp/angular_route_component_support_issue_162.rs src/tests/mcp/angular_template_compatibility.rs src/tests/mcp/ngrx_legacy_transition_compatibility.rs src/tests/mcp/ngrx_legacy_discriminant.rs src/tests/mcp/ngrx_legacy_object_dispatch.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen probes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    $filters = if ($ResumeAfterGenerationControls) {
        @('angular_meta::routing::','issue_162_route_provider','provider_semantics',
          'issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition')
    } elseif ($ResumeAfterControlCorrections) {
        @('issue_162_route_component_controls_selective_legacy_and_lazy_origin',
          'issue_162_route_component_controls_sqlite_before_queries_and_stale_repair',
          'issue_162_route_component_controls_unchanged_owner_barrel_api_repair',
          'issue_162_route_component_capture','angular_template_compatibility','ngrx_legacy',
          'angular_meta::routing::','issue_162_route_provider','provider_semantics',
          'issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition')
    } else {
        @('angular_route_component_membership_tests','issue_162_route_component_controls','issue_162_route_component_capture','angular_template_compatibility','ngrx_legacy','angular_meta::routing::','issue_162_route_provider','provider_semantics','issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition')
    }
    foreach ($filter in $filters) {
        $logName = $filter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Focused/preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed; logs: $logDirectory" }
    Write-Output "Generation-73 focused Rust/lint GREEN. Binary and full repository/field gates remain open. Full logs: $logDirectory"
} finally { Pop-Location }
