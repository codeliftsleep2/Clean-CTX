# Operator-owned bounded feature-isolation controls, not the full repository gate.
param([switch]$IsolatedOnly)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/feature-isolation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
$savedColor=$env:CARGO_TERM_COLOR
$env:CARGO_TERM_COLOR='never'
Push-Location $root
try {
    $witness='src/tests/ir/ci_runtime_core_query.rs'
    if((Get-FileHash $witness).Hash.ToLowerInvariant() -cne 'bdc4c262d8b5f799e7c5fa4de44cdd16a662d493124d173a9cbab11f125387b3'){throw 'Original RED witness source changed'}
    $files=@(git ls-files --modified --others --exclude-standard -- '*.rs'|Where-Object{$_ -cne $witness})
    & rustfmt --edition 2024 --config skip_children=true @files
    if($LASTEXITCODE -ne 0){throw 'Focused formatting failed'}
    $controls=@(
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_complete_definitions_match_frozen_composition',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_complete_queries_preserve_captures_and_ir',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_custom_invalid_and_mismatched_queries_stay_dynamic',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_disabled_query_features_refuse',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_core_concurrent_worker',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_core_concurrent_first_access_is_process_cold',
        'gitdiff::engine::feature_isolation_tests::ci_runtime_feature_template_dispatch_matches_availability',
        'dotnet_meta::efcore::tests::tests::test_extracts_dbcontext'
    )
    $native=@(
        'ir::pipeline::core::native_feature_evidence_tests::ci_runtime_native_typescript_evidence_preserves_source_ownership',
        'angular_meta::inject_evidence::angular_inject_evidence_tests::angular_inject_evidence_direct_descendant_and_token_roles'
    )
    $all=@($controls)+@($native)+@(
        'ir::pipeline::core::ci_runtime_query_tests::ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query',
        'compression::capture_pipeline::tests::registered_static_query_compiles_once_across_observations'
    )
    $pins=@(@($files)+$witness|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    $runs=[Collections.Generic.List[object]]::new()
    $failed=$false
    # Both implicated single-language configurations are collected before stopping.
    $configurations=@('csharp','typescript')
    if(-not $IsolatedOnly){$configurations+=@('angular','dotnet','all-features')}
    foreach($feature in $configurations){
        if($feature -eq 'angular' -and $failed){break}
        if($feature -eq 'all-features'){
            $selected=$all
            $argumentsForRun=@('test','--lib','--all-features','--','--exact','--test-threads=1','--nocapture')+$selected
        } else {
            $selected=$controls
            if($feature -in @('typescript','angular')){$selected=@($controls)+@($native)}
            $argumentsForRun=@('test','--lib','--no-default-features','--features',$feature,'--','--exact','--test-threads=1','--nocapture')+$selected
        }
        $log=Join-Path $out ($feature+'.log')
        $clock=[Diagnostics.Stopwatch]::StartNew()
        & cargo @argumentsForRun *> $log
        $code=$LASTEXITCODE
        $clock.Stop()
        $summary='test result: ok. '+$selected.Count+' passed; 0 failed; 0 ignored;'
        $complete=($code -eq 0 -and (Select-String -LiteralPath $log -SimpleMatch -Pattern $summary -Quiet))
        $warnings=[bool](Select-String -LiteralPath $log -Pattern '^warning:' -Quiet)
        $runs.Add(@{feature=$feature;arguments=$argumentsForRun;selection=$selected;exitCode=$code;complete_selection=$complete;warnings=$warnings;seconds=$clock.Elapsed.TotalSeconds;log_sha256=(Get-FileHash $log).Hash})
        @{category='Focused tracked feature-isolation correctness; not full CI';head=(git rev-parse HEAD);sources=$pins;runs=$runs.ToArray()}|ConvertTo-Json -Depth 12|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 55
        Write-Output "Return receipts from $out"
        if(-not $complete -or $warnings){$failed=$true}
    }
    if($failed){throw "Feature-isolation controls failed; preserve $out"}
    foreach($pin in $pins){if((Get-FileHash $pin.path).Hash -cne $pin.sha256){throw "Source drift: $($pin.path)"}}
    if(-not $IsolatedOnly){
        & (Join-Path $root 'target/debug/clean-ctx.exe') pcg documents describe --profile (Join-Path $root '.clean-ctx/pcg-documents/profile.json') 1> (Join-Path $out 'descriptor.json') 2> (Join-Path $out 'descriptor-stderr.txt')
        if($LASTEXITCODE -ne 0){throw "Read-only B11 descriptor failed; preserve $out"}
        Write-Output "Return exact B11 descriptor for owner review: $out"
    }
} finally {$env:CARGO_TERM_COLOR=$savedColor;Pop-Location}
