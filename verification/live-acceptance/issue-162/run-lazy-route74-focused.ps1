# Agent MaxHeadRoom: approved generation-74 lazy production verification.
param([switch]$ResumeAfterLoadChildrenControlCorrection)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/lazy-route74-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = @('src/tests/mcp/angular_lazy_route_semantic_issue_162.rs','src/tests/mcp/angular_lazy_route_component_issue_162.rs')
    $beforeHashes = @($frozen | ForEach-Object { (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash })
    $formatFiles = @('src/angular_meta/route_lazy_components.rs','src/angular_meta/mod.rs','src/angular_meta/routing.rs','src/layers/meta/angular_semantic.rs','src/ir/typescript_resolution/relation_evidence.rs','src/layers/meta/mod.rs','src/tests/mcp/angular_template_compatibility.rs','src/tests/mcp/ngrx_legacy_transition_compatibility.rs','src/tests/mcp/ngrx_legacy_discriminant.rs','src/tests/mcp/ngrx_legacy_object_dispatch.rs','src/tests/mcp/typescript_lazy_route_field_issue_162.rs','src/tests/mcp/angular_lazy_route_controls_issue_162.rs','src/tests/mcp/angular_lazy_route_lifecycle_issue_162.rs','src/mcp/semantic_compilation.rs')
    & rustfmt --edition 2024 --config skip_children=true @formatFiles *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    for ($i=0; $i -lt $frozen.Count; $i++) { if ((Get-FileHash -LiteralPath $frozen[$i] -Algorithm SHA256).Hash -ne $beforeHashes[$i]) { throw 'Frozen probes changed' } }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    $filters = @('issue_162_lazy_semantic','angular_lazy_route_component_tests','issue_162_lazy_controls','issue_162_lazy_selection','issue_162_lazy_field','issue_162_route_component','angular_meta::routing::','angular_template_compatibility','ngrx_legacy','issue_162_route_provider','provider_semantics','issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition','issue_162_application_config','issue_162_bootstrap_inline')
    if ($ResumeAfterLoadChildrenControlCorrection) {
        $filters = @('issue_162_lazy_controls_outside_ownership_and_load_children_preserved') + $filters[3..($filters.Count - 1)]
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
    Write-Output "Generation-74 focused Rust/lint GREEN; approval map and binary/field gates remain. Full logs: $logDirectory"
} finally { Pop-Location }
