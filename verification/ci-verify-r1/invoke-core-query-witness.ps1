# Operator-owned proposed budget RED witness. No production cache change.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/core-query-witness-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $source='src/tests/ir/ci_runtime_core_query.rs'
    & rustfmt --edition 2024 --config skip_children=true $source
    if($LASTEXITCODE -ne 0){throw 'Witness formatting failed'}
    $selection='ir::pipeline::core::ci_runtime_query_tests::ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query'
    $argumentsForRun=@('test','--lib','--all-features',$selection,'--','--exact','--test-threads=1','--nocapture')
    $log=Join-Path $out 'tracked-test.log'
    & cargo @argumentsForRun *> $log
    $code=$LASTEXITCODE
    $pins=@($source,'src/ir/pipeline/core.rs','src/compression/capture_pipeline.rs','src/ir/calls.rs','Cargo.toml','Cargo.lock')
    foreach($path in $pins){
        $target=Join-Path $out ('source/'+$path)
        New-Item -ItemType Directory -Force -Path (Split-Path $target)|Out-Null
        Copy-Item -LiteralPath $path -Destination $target
    }
    @{head=(git rev-parse HEAD);category='Proposed warm-query budget RED investigation; not approved production cache implementation';command='cargo';arguments=$argumentsForRun;exitCode=$code;log_sha256=(Get-FileHash $log).Hash;sources=@($pins|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})}|ConvertTo-Json -Depth 8|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath $log -Tail 55
    Write-Output "Return exact witness receipt and output: $out"
    if($code -eq 0){Write-Output 'Proposed budget GREEN; investigate prior query initialization and premise.'}
    elseif($code -eq 101 -and (Select-String -LiteralPath $log -Pattern 'CI_RUNTIME_CORE_QUERY seconds=.*parses=1 cursors=1 query_compilations=1' -Quiet) -and (Select-String -LiteralPath $log -SimpleMatch -Pattern '0 passed; 1 failed;' -Quiet)){
        Write-Output 'Expected behavioral RED observed: one new query compilation after warming identical binary-owned query. This does not authorize a production cache extension.'
    } else {throw "Unexpected failure; preserve $out"}
} finally {Pop-Location}
