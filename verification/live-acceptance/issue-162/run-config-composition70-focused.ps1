# Agent MaxHeadRoom: approved config composition deterministic checkpoint.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/config-composition70-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = Join-Path $repoPath 'src/tests/mcp/angular_config_composition_issue_162.rs'
    $expectedHash = '0F67F5EAE4711B5DED2A3087CCEA8CCDF05B421D373B5944F4061B64D3332029'
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Frozen RED changed' }
    $formatFiles = @(
        'src/angular_meta/config_composition.rs', 'src/angular_meta/bootstrap_references.rs', 'src/angular_meta/mod.rs',
        'src/ir/typescript_resolution/declaration_facets.rs', 'src/ir/typescript_resolution/demanded_occurrences.rs',
        'src/ir/typescript_resolution/relation_evidence.rs', 'src/layers/meta/semantic.rs',
        'src/layers/meta/angular_semantic.rs', 'src/layers/meta/mod.rs', 'src/mcp/semantic_compilation/retained.rs',
        'src/mcp/semantic_compilation.rs', 'src/tests/mcp/angular_config_composition_controls_issue_162.rs',
        'src/tests/mcp/angular_template_compatibility.rs', 'src/tests/mcp/ngrx_legacy_transition_compatibility.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @formatFiles *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active size check failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; logs: $logDirectory" }
    foreach ($filter in @('issue_162_config_composition','issue_162_application_config_provider','issue_162_bootstrap_controls','angular_template_compatibility','ngrx_legacy_transition')) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 9
        if ($testExit -ne 0) { throw "Focused tests failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Frozen RED changed' }
    Write-Output "Requested scoped config checkpoint GREEN; logs: $logDirectory"
} finally { Pop-Location }
