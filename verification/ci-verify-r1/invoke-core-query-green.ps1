# Operator-owned bounded correctness/feature checks and read-only B11 descriptor.
# Never authorizes, admits, commits, pushes or executes full CI.
param([switch]$AllFeaturesOnly,[switch]$RemainingOnly)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$savedColor=$env:CARGO_TERM_COLOR
$env:CARGO_TERM_COLOR='never'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/core-query-green-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $witness='src/tests/ir/ci_runtime_core_query.rs'
    $expected='bdc4c262d8b5f799e7c5fa4de44cdd16a662d493124d173a9cbab11f125387b3'
    if((Get-FileHash $witness).Hash.ToLowerInvariant() -cne $expected){throw 'Exact RED witness source changed'}
    $files=@(git ls-files --modified --others --exclude-standard -- '*.rs'|Where-Object{$_ -cne $witness})
    & rustfmt --edition 2024 --config skip_children=true @files
    if($LASTEXITCODE -ne 0){throw 'Focused formatting failed'}
    $tests=@(
        'ir::pipeline::core::ci_runtime_query_tests::ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query',
        'compression::capture_pipeline::tests::registered_static_query_compiles_once_across_observations',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_complete_definitions_match_frozen_composition',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_complete_queries_preserve_captures_and_ir',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_custom_invalid_and_mismatched_queries_stay_dynamic',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_disabled_query_features_refuse',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_core_concurrent_worker',
        'compression::capture_pipeline::core_ir_registration_tests::ci_runtime_core_concurrent_first_access_is_process_cold'
    )
    $allFeatureExpected=8
    if($RemainingOnly){$tests=@($tests|Where-Object{$_ -like '*::ci_runtime_complete_definitions_match_frozen_composition'});$allFeatureExpected=1}
    $runs=@(@{name='all-features';expected=$allFeatureExpected;arguments=(@('test','--lib','--all-features','--','--exact','--test-threads=1','--nocapture')+$tests)})
    if(-not $AllFeaturesOnly){
        foreach($feature in @('csharp','typescript')){
            $runs+=@{name=('only-'+$feature);expected=6;arguments=@('test','--lib','--no-default-features','--features',$feature,'compression::capture_pipeline::core_ir_registration_tests::','--','--test-threads=1','--nocapture')}
        }
    }
    $receipts=[Collections.Generic.List[object]]::new()
    foreach($run in $runs){
        $log=Join-Path $out ($run.name+'.log')
        $argumentsForRun=$run.arguments
        & cargo @argumentsForRun *> $log
        $code=$LASTEXITCODE
        $receipts.Add(@{name=$run.name;arguments=$argumentsForRun;exitCode=$code;log_sha256=(Get-FileHash $log).Hash})
        @{category='Focused tracked correctness and explicit restricted-feature checks; not CI';head=(git rev-parse HEAD);runs=$receipts.ToArray();sources=@(@($files)+$witness|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})}|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 45
        Write-Output "Preserve and return receipts: $out"
        if($code -ne 0){throw "$($run.name) failed; preserve $out"}
        if(Select-String -LiteralPath $log -Pattern '^warning:' -Quiet){throw 'Compiler warnings; preserve capture'}
        $summary='test result: ok. '+$run.expected+' passed; 0 failed; 0 ignored;'
        if(-not (Select-String -LiteralPath $log -SimpleMatch -Pattern $summary -Quiet)){throw 'Unexpected or empty selection'}
    }
    if((Get-FileHash $witness).Hash.ToLowerInvariant() -cne $expected){throw 'Exact RED witness changed during checks'}
    if(-not $AllFeaturesOnly){
        # Existing production governance command; describe is read-only, not authorize.
        $binary=Join-Path $root 'target/debug/clean-ctx.exe'
        $profile=Join-Path $root '.clean-ctx/pcg-documents/profile.json'
        & $binary pcg documents describe --profile $profile 1> (Join-Path $out 'descriptor.json') 2> (Join-Path $out 'descriptor-stderr.txt')
        if($LASTEXITCODE -ne 0){throw "B11 descriptor/check failed; preserve $out"}
        Write-Output "Return exact B11 descriptor for review: $out"
    }
} finally {$env:CARGO_TERM_COLOR=$savedColor;Pop-Location}
