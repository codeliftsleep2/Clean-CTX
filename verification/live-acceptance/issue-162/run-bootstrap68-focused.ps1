# Agent MaxHeadRoom: integrated deterministic checkpoint; no Node/build/runtime pilot.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap68-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozenTest = 'src/tests/mcp/angular_bootstrap_reference_issue_162.rs'
    $frozenHash = '6820e89190db1fecb468bc1e12f04c739e9b6ac0e6cfcc5b93ce260502d6245f'
    if ((Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash -ne $frozenHash) { throw 'Original RED bytes changed' }
    if ((Get-FileHash -LiteralPath 'src/tests/mcp/angular_bootstrap_controls_issue_162.rs' -Algorithm SHA256).Hash -ne '36fdb689c1abdbf0937cd5e0785b2d88f7e246610439885b5153f592985ff2b5') { throw 'Scoped RED bytes changed' }
    $formatFiles = @(
        'src/angular_meta/bootstrap_references.rs', 'src/angular_meta/mod.rs', 'src/angular_meta/typed_provider_root.rs',
        'src/ir/typescript_resolution/capture.rs', 'src/ir/typescript_resolution/declaration_facets.rs',
        'src/ir/typescript_resolution/references.rs', 'src/ir/typescript_resolution/invocation_sites.rs',
        'src/ir/typescript_resolution/mod.rs', 'src/tests/mcp/angular_bootstrap_scope_controls_issue_162.rs',
        'src/ir/typescript_resolution/declaration_facets/invocations.rs',
        'src/ir/typescript_resolution/demanded_occurrences.rs', 'src/ir/typescript_resolution/relation_evidence.rs',
        'src/layers/meta/angular_semantic.rs', 'src/layers/meta/canonical_endpoint.rs',
        'src/layers/meta/mod.rs', 'src/layers/meta/semantic.rs',
        'src/mcp/semantic_compilation.rs', 'src/mcp/semantic_compilation/compile.rs',
        'src/mcp/semantic_compilation/retained.rs',
        'src/tests/mcp/angular_template_compatibility.rs', 'src/tests/mcp/ngrx_legacy_transition_compatibility.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @formatFiles *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; full logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash -ne $frozenHash) { throw 'Formatting changed original RED bytes' }
    if ((Get-FileHash -LiteralPath 'src/tests/mcp/angular_bootstrap_controls_issue_162.rs' -Algorithm SHA256).Hash -ne '36fdb689c1abdbf0937cd5e0785b2d88f7e246610439885b5153f592985ff2b5') { throw 'Formatting changed scoped RED bytes' }
    & pwsh -NoProfile -ExecutionPolicy Bypass scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active-file size check failed; full logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; full logs: $logDirectory" }
    foreach ($testFilter in @('issue_162_bootstrap_scope_controls','issue_162_bootstrap_reference','issue_162_bootstrap_controls','angular_template_compatibility','ngrx_legacy_transition')) {
        $testLog = Join-Path $logDirectory "$testFilter.log"
        & cargo test --lib --all-features --quiet $testFilter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 10
        if ($testExit -ne 0) { throw "Tracked tests failed: $testFilter; full log: $testLog" }
    }
    Write-Output "Requested integrated focused commands GREEN; full logs: $logDirectory"
} finally {
    Pop-Location
}
