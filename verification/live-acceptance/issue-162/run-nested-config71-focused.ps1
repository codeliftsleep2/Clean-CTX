# Agent MaxHeadRoom: approved direct named merge-input focused checkpoint.
param([switch]$ResumeAfterReverseControlFailure)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/nested-config71-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = @{
        'angular_nested_config_input_issue_162.rs' = '1390987D2182634A4A5B56A311B18AB3813C0BD8B75246D72955DB6A612F0BFD'
        'angular_nested_config_cycles_issue_162.rs' = '09DE349B1CB6B32AEEB4FE3A4E2A725B585B611DC2DC9E3ED7694EADD8570A8D'
    }
    foreach ($file in $frozen.Keys) {
        if ((Get-FileHash -LiteralPath (Join-Path $repoPath "src/tests/mcp/$file") -Algorithm SHA256).Hash -ne $frozen[$file]) { throw "Frozen test changed: $file" }
    }
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/config_composition.rs src/layers/meta/mod.rs src/tests/mcp/angular_template_compatibility.rs src/tests/mcp/ngrx_legacy_transition_compatibility.rs src/tests/mcp/angular_config_composition_exit_issue_162.rs src/tests/mcp/angular_config_composition_controls_issue_162.rs src/tests/mcp/angular_nested_config_controls_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; logs: $logDirectory" }
    $filters = @('issue_162_nested_config','issue_162_config_composition','angular_template_compatibility','ngrx_legacy_transition')
    if ($ResumeAfterReverseControlFailure) { $filters = @('issue_162_config_composition_exit_api_type_and_input_refusals','angular_template_compatibility','ngrx_legacy_transition') }
    foreach ($filter in $filters) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 10
        if ($testExit -ne 0) { throw "Focused test failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    foreach ($file in $frozen.Keys) {
        if ((Get-FileHash -LiteralPath (Join-Path $repoPath "src/tests/mcp/$file") -Algorithm SHA256).Hash -ne $frozen[$file]) { throw "Frozen test changed: $file" }
    }
    Write-Output "Requested focused checkpoint GREEN; logs: $logDirectory"
} finally { Pop-Location }
