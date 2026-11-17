# Operator-owned bounded verification, not full CI.
param([switch]$RemainingOnly)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$out=Join-Path $root ('target/issue-162/restore-validator-green-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $files=@(git ls-files --modified --others --exclude-standard -- '*.rs')
    & rustfmt --edition 2024 --config skip_children=true @files
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    $selection=@(
        'mcp::semantic_compilation::angular_application_config_controls_tests::issue_162_application_config_provider_complete_sqlite_same_name_identity_restore'
        'mcp::semantic_compilation::angular_bootstrap_controls_tests::issue_162_bootstrap_controls_complete_sqlite_index_restore_and_multiplicity'
        'mcp::semantic_compilation::angular_bootstrap_inline_controls_tests::issue_162_bootstrap_inline_controls_sqlite_complete_before_query_and_multiplicity'
        'mcp::semantic_compilation::angular_config_composition_controls_tests::issue_162_config_composition_controls_sqlite_before_query_and_stale_repair'
        'mcp::semantic_compilation::angular_functional_lifecycle_tests::issue_162_functional_lifecycle_restart_retains_exact_authority_and_endpoints'
        'mcp::semantic_compilation::angular_legacy_bootstrap_controls_tests::issue_162_legacy_controls_sqlite_restore_before_public_query'
        'mcp::semantic_compilation::angular_nested_config_controls_tests::issue_162_nested_config_controls_sqlite_before_queries_and_owned_assertion'
        'mcp::semantic_compilation::angular_public_entry_lifecycle_tests::issue_162_public_entry_lifecycle_exact_restart_precedes_reacquisition'
        'mcp::semantic_compilation::typescript_declared_return_member_tests::issue_162_declared_return_member_sqlite_stale_dependency_and_repair'
        'mcp::tool_handlers::query::tests_issue_157_dependencies::issue_157_durable_projection_retains_and_validates_contributors'
        'mcp::semantic_compilation::restore_registration_boundary_tests::issue_162_unchanged_bootstrap_durable_registration_boundary'
        'mcp::semantic_compilation::restore_registration_config_matrix_tests::issue_162_restore_registration_config_preservation_matrix'
        'mcp::compatibility::validator::tests::compatible_context_yields_typed_canonical_and_semantic_evidence'
        'mcp::compatibility::validator::tests::source_and_fidelity_mismatches_are_distinct'
        'mcp::compatibility::validator::tests::configuration_mismatches_are_scoped_to_canonical_or_semantic_authority'
        'mcp::compatibility::validator::tests::producer_set_comparison_rejects_addition_and_removal_bidirectionally'
        'mcp::compatibility::validator::tests::every_missing_legacy_component_has_a_distinct_diagnostic'
        'mcp::compatibility::validator::tests::centralized_failure_taxonomy_keeps_non_identity_rejections_distinct'
        'mcp::compatibility::validator::tests::durable_load_classifies_malformed_identity_as_physical_schema_failure'
        'mcp::compatibility::validator::tests::durable_load_classifies_cross_artifact_mismatch_as_structural_incoherence'
        'mcp::compatibility::validator::tests::validation_uses_immutable_input_and_leaves_untrusted_state_unchanged'
    )
    if($selection.Count -ne 21){throw 'Expected exactly 21 focused tests'}
    if($RemainingOnly){
        $selection=@($selection|Where-Object{$_ -match 'restore_registration_config_matrix_tests|issue_157_durable_projection_retains_and_validates_contributors'})
        if($selection.Count -ne 2){throw 'Expected exactly two remaining tests'}
    }
    $log=Join-Path $out 'tracked-tests.log'
    & cargo test --lib --all-features -- --exact --test-threads=1 @selection *> $log
    $code=$LASTEXITCODE
    [ordered]@{head=(git rev-parse HEAD);command='cargo test --lib --all-features -- --exact --test-threads=1 <receipt selection>';rustc=(& rustc --version);cargo=(& cargo --version);rustflags=$env:RUSTFLAGS;selection=$selection;exitCode=$code;log_sha256=(Get-FileHash $log).Hash;files=@($files|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath $log -Tail 100
    Write-Output "Return all diagnostic receipts: $out"
    if($code -ne 0){throw "Focused failure; preserve $out"}
    $expected='^test result: ok\. '+$selection.Count+' passed; 0 failed; 0 ignored;'
    if(-not (Select-String -LiteralPath $log -Pattern $expected -Quiet)){throw 'Incomplete selection'}
} finally {Pop-Location}
