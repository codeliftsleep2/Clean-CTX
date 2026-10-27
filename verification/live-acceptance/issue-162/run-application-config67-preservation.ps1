# Agent MaxHeadRoom: broader tracked preservation; no binary/Node execution.
param([switch]$ResumeAfterBaseline, [switch]$ResumeAfterDurableFailure, [switch]$ResumeAfterCompatibilityFailure, [switch]$ResumeAfterAcknowledgementFailure)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $logDirectory = Join-Path $repoPath "target/issue-162/application-config67-preserve-$stamp"
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    # The preceding 33 config/route/compatibility passes remain valid: no production edits.
    $filters = @(
        'issue_162_provider', 'issue_162_view_provider', 'issue_162_string',
        'issue_162_module', 'provider_semantics', 'compress',
        'baseline_publication', 'core::provide::publication_race_tests',
        'semantic_publication_recovery', 'tests_publication_retry',
        'tests_hydration_fidelity', 'durable_semantic_restore',
        'core::restore::compatibility_tests', 'core::delta::fidelity_persistence_tests'
    )
    if ($ResumeAfterAcknowledgementFailure) {
        & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/delta_fidelity_persistence.rs src/tests/mcp/delta_acknowledgement_support.rs src/mcp/tool_handlers/core/delta_apply.rs *> (Join-Path $logDirectory 'acknowledgement-control-format.log')
        if ($LASTEXITCODE -ne 0) { throw "Acknowledgement control formatting failed; logs: $logDirectory" }
        $filters = @(
            'committed_delta_acknowledgement_survives_missing_source',
            'uncommitted_delta_missing_source_preserves_pending_state'
        )
    } elseif ($ResumeAfterCompatibilityFailure) {
        & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/durable_restore_compatibility.rs *> (Join-Path $logDirectory 'restore-control-format.log')
        if ($LASTEXITCODE -ne 0) { throw "Restore control formatting failed; logs: $logDirectory" }
        $filters = @(
            'restore_rejects_csharp_semantic_projection_generation_mismatch',
            'restore_rejects_csharp_snapshot_missing_projection_generation',
            'restore_rejects_current_producer_when_persisted_set_lacks_it',
            'core::delta::fidelity_persistence_tests'
        )
    } elseif ($ResumeAfterDurableFailure) {
        $filters = @(
            'generated_delta_persists_and_restores_complete_target_edges',
            'core::restore::compatibility_tests', 'core::delta::fidelity_persistence_tests'
        )
    } elseif ($ResumeAfterBaseline) { $filters = $filters[7..($filters.Length - 1)] }
    foreach ($testFilter in $filters) {
        $logName = $testFilter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $testFilter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 8
        if ($testExit -ne 0) { throw "Tracked tests failed: $testFilter; full log: $testLog" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) {
            throw "No passing tests selected: $testFilter; full log: $testLog"
        }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; full log: $logDirectory/fmt.log" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; full log: $logDirectory/clippy.log" }
    Write-Output "Requested scoped preservation and lint commands GREEN; full logs: $logDirectory"
} finally {
    Pop-Location
}
