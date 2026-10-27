# Agent MaxHeadRoom: integrated deterministic checkpoint; operator execution only.
param([switch]$ResumeAfterInlineControlsFailure)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/bootstrap-inline69-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozenFile = Join-Path $repoPath 'src/tests/mcp/angular_bootstrap_inline_provider_issue_162.rs'
    $expectedHash = '89F0BC03A8C51EEB3F4FCD97AF426E66910000F367B4AEF6A18EC31A911C516D'
    if ((Get-FileHash -LiteralPath $frozenFile -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Original RED test bytes changed' }
    $formatFiles = @(
        'src/angular_meta/bootstrap_inline_providers.rs', 'src/angular_meta/bootstrap_references.rs',
        'src/angular_meta/provider_imported.rs', 'src/angular_meta/provider_position.rs',
        'src/angular_meta/provider_pair_authority.rs', 'src/angular_meta/mod.rs',
        'src/layers/meta/mod.rs', 'src/mcp/semantic_compilation.rs',
        'src/tests/mcp/angular_bootstrap_inline_controls_issue_162.rs',
        'src/tests/mcp/angular_template_compatibility.rs', 'src/tests/mcp/ngrx_legacy_transition_compatibility.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @formatFiles *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; logs: $logDirectory" }
    $filters = @('issue_162_bootstrap_inline_provider', 'issue_162_bootstrap_inline_controls', 'issue_162_bootstrap_controls', 'angular_template_compatibility', 'ngrx_legacy_transition')
    if ($ResumeAfterInlineControlsFailure) { $filters = $filters[1..4] }
    foreach ($filter in $filters) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 8
        if ($testExit -ne 0) { throw "Focused tests failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    if ((Get-FileHash -LiteralPath $frozenFile -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Original RED test bytes changed' }
    Write-Output "Requested focused checkpoint GREEN; logs: $logDirectory"
} finally { Pop-Location }
