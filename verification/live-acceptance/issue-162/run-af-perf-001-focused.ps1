# Agent MaxHeadRoom: narrow AF gate; no full acceptance or binary checkpoint.
param([switch]$ResumeAtLint, [switch]$DelegatedProviders)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/af-perf-001-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $paths = @('src/angular_meta/admission_work_metrics.rs','src/angular_meta/mod.rs','src/angular_meta/route_resolvers.rs','src/ir/typescript_resolution/relation_evidence.rs','src/ir/typescript_resolution/host.rs','src/mcp/tool_handlers/query.rs','src/mcp/semantic_compilation.rs','src/lib.rs','src/tests/architecture/semantic_work_static.rs','src/tests/mcp/angular_admission_lifecycle_af_perf_001.rs','src/tests/mcp/angular_resolver_replay_p0_issue_162.rs','src/angular_meta/delegated_work_metrics.rs','src/angular_meta/bootstrap_references.rs','src/angular_meta/application_config_providers.rs','src/angular_meta/config_composition.rs','src/angular_meta/provider_imported.rs','src/angular_meta/provider_pair_authority.rs','src/angular_meta/provider_position.rs','src/tests/architecture/delegated_provider_work_af_perf_001.rs')
    & rustfmt --edition 2024 --config skip_children=true @paths *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size gate failed: $logDirectory" }
    if (-not $ResumeAtLint) {
        & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1 *> (Join-Path $logDirectory 'fitness.log')
        if ($LASTEXITCODE -ne 0) { Get-Content -LiteralPath (Join-Path $logDirectory 'fitness.log') -Tail 55; throw "Fitness failed: $logDirectory" }
        $filters = @('issue_162_guard77_controls_cross_kind_admission_and_named_json_compatibility','issue_162_guard77_controls_missing_competing_crossed_retained_proof','issue_162_guard77_controls_partial_broad_values_duplicate_fields_and_owned_no_fallback','issue_162_resolver78_controls_exact_admission_forged_and_borrowed','issue_162_resolver78_controls_broad_partial_duplicate_keys_and_owned_no_fallback')
        if ($DelegatedProviders) {
            $filters = @('issue_162_bootstrap_inline_controls_composition_strategy_and_refusal_siblings','issue_162_bootstrap_inline_controls_missing_crossed_field_entry_and_serde_refusal','issue_162_application_config_provider_native_roots_refusals_and_position_policy','issue_162_application_config_provider_membership_forgery_and_serde_refusal','issue_162_config_composition_controls_native_and_result_input_refusals','issue_162_config_composition_controls_missing_crossed_proof_and_serde','issue_162_provider_imported_composition_mixed_strategies_alias_cycles_and_refusals','issue_162_provider_imported_composition_source_hash_span_and_occurrence_refusal')
        }
        foreach ($filter in $filters) {
            $testLog = Join-Path $logDirectory ($filter + '.log')
            & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
            if ($LASTEXITCODE -ne 0) { throw "Preservation failed: $testLog" }
            if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored;' -Quiet)) { throw "Missing required test: $testLog" }
        }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logDirectory 'encoding.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding failed: $logDirectory" }
    Write-Output "Focused fitness/preservation/lint complete: $logDirectory. Full generation-78 and binary acceptance remain separate."
} finally { Pop-Location }
