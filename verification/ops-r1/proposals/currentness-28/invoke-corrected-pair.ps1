# Operator-only exact corrected-fixture RED then GREEN. No full gates or field test.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$out=Join-Path $root ('target/ops-r1/currentness-28-corrected-'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$prepare=Join-Path $PSScriptRoot 'prepare-corrected.py'
$execution=Join-Path $PSScriptRoot 'execution'
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    & rustfmt --edition 2024 --config skip_children=true src/tests/ir/ops_r1_shared_acquisition_cost.rs
    if($LASTEXITCODE -ne 0){throw 'Corrected test formatting failed.'}
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='currentness-28-corrected-red'
    & python $prepare red $out
    if($LASTEXITCODE -ne 0){throw "Corrected RED identity failed; preserve $out"}
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/currentness-28-corrected-red-build-01
    if($LASTEXITCODE -ne 0){throw "Corrected RED compilation failed; preserve $out"}
    $build=Join-Path $root 'target/ops-r1/currentness-28-corrected-red-build-01/build-receipt.json'
    $result=@(& python (Join-Path $execution 'capture.py') --build-receipt $build --case retained-currentness --wall-seconds 60 --memory-mib 512 --stop-wait 5)
    if($result.Count -ne 1){throw 'Unexpected RED capture output.'}
    $receipt=Get-Content (Join-Path $result[0] 'receipt.json') -Raw|ConvertFrom-Json
    $stderr=Get-Content (Join-Path $result[0] 'stderr.txt') -Raw
    if($receipt.exit_code -ne 101 -or $receipt.status -cne 'TRACKED_TEST_FAILED_REVIEW_REASON' -or -not $receipt.post_source_and_binary_identity_verified -or -not $stderr.Contains('one exact retained-currentness constraint per call')){
        throw "Corrected RED failed for an unexpected reason; preserve $out"
    }
    & python $prepare green $out $result[0]
    if($LASTEXITCODE -ne 0){throw "GREEN preparation failed; preserve $out"}
    $env:OPS_R1_GREEN_SOURCE='currentness-28-corrected-green'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/currentness-28-corrected-green-build-01
    if($LASTEXITCODE -ne 0){throw "Corrected GREEN compilation failed; preserve $out"}
    $build=Join-Path $root 'target/ops-r1/currentness-28-corrected-green-build-01/build-receipt.json'
    $cases=@('retained-currentness','seal','qualification','cost-constraints','directory-work','fitness',
        'seed_is_not_sibling_disk_authority','divergent_overlay_and_private_cache_are_not_overwritten',
        'failed_partial_and_poisoned_leader_do_not_grant_or_poison',
        'precached_unaccounted_values_never_seal_from_nonempty_witnesses',
        'source_and_package_changes_invalidate_proof','config_roots_and_route_changes_stay_independent',
        'same_config_different_automatic_type_lookup_stays_distinct','typescript_persistence','resolver_positive')
    $receipts=@()
    foreach($case in $cases){
        $result=@(& python (Join-Path $execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
        $code=$LASTEXITCODE
        if($result.Count -ne 1){throw "Unexpected GREEN output; preserve $out"}
        $path=Join-Path $result[0] 'receipt.json'
        $receipt=Get-Content $path -Raw|ConvertFrom-Json
        $receipts += [ordered]@{case=$case;receipt=$path;sha256=(Get-FileHash $path).Hash.ToLowerInvariant();status=$receipt.status}
        ConvertTo-Json -InputObject $receipts -Depth 8|Set-Content (Join-Path $out 'green-receipts.json') -Encoding utf8NoBOM
        if($code -ne 0 -or $receipt.status -cne 'TRACKED_TEST_COMPLETED' -or -not $receipt.post_source_and_binary_identity_verified){
            throw "Affected GREEN failed: $case; preserve $out"
        }
    }
    Write-Output "Return corrected exact RED/GREEN and affected controls: $out"
} finally {
    if(Test-Path (Join-Path $out 'candidate-record.json')){
        & python $prepare restore $out
        if($LASTEXITCODE -ne 0){Write-Error "Candidate restoration needs review; preserve $out"}
    }
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
