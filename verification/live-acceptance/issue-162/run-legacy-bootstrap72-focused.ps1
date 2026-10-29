# Agent MaxHeadRoom: approved generation-72 focused/preservation checkpoint.
param([switch]$ResumeAfterGenerationControls)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/legacy-bootstrap72-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozenTest = 'src/tests/mcp/angular_legacy_bootstrap_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/legacy_bootstrap.rs src/angular_meta/bootstrap_references.rs src/angular_meta/mod.rs src/ir/typescript_resolution/capture.rs src/ir/typescript_resolution/relation_evidence.rs src/layers/meta/semantic.rs src/layers/meta/mod.rs src/mcp/semantic_compilation/retained.rs src/mcp/semantic_compilation.rs src/tests/mcp/angular_legacy_bootstrap_controls_issue_162.rs src/tests/mcp/angular_template_compatibility.rs src/tests/mcp/ngrx_legacy_transition_compatibility.rs src/tests/mcp/ngrx_legacy_discriminant.rs src/tests/mcp/ngrx_legacy_object_dispatch.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen regression changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; logs: $logDirectory" }
    $filters = @('issue_162_legacy','issue_162_declared_return_member','angular_template_semantics_increment','ngrx_legacy','issue_162_bootstrap_controls','issue_162_config_composition','issue_162_bootstrap_inline')
    if ($ResumeAfterGenerationControls) {
        $filters = @('discriminant_authority_advances_only_angular_semantics','import_refinement_advances_only_angular_semantics','issue_162_bootstrap_controls','issue_162_config_composition','issue_162_bootstrap_inline')
    }
    foreach ($filter in $filters) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Focused checkpoint failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Fmt failed; logs: $logDirectory" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    Write-Output "Generation-72 requested focused/preservation/lint GREEN; full logs: $logDirectory"
} finally { Pop-Location }
