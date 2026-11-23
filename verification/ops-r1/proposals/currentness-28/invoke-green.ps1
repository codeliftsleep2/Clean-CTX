# Operator-only affected currentness regressions; no full gates or timing campaign.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$out=Join-Path $root ('target/ops-r1/currentness-28-green-'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    & rustfmt --edition 2024 --config skip_children=true src/ir/typescript_resolution/host.rs src/workspace/index/coverage.rs src/workspace/index/resolver_registrations.rs src/mcp/semantic_compilation/retained.rs
    if($LASTEXITCODE -ne 0){throw 'Scoped formatting failed.'}
    & python (Join-Path $PSScriptRoot 'prepare-green.py')
    if($LASTEXITCODE -ne 0){throw 'GREEN identity preparation failed; no build started.'}
    $execution=Join-Path $PSScriptRoot 'execution'
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='currentness-28-green'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/currentness-28-green-build-01
    if($LASTEXITCODE -ne 0){throw "Build failed; preserve $out"}
    $build=Join-Path $root 'target/ops-r1/currentness-28-green-build-01/build-receipt.json'
    $receipts=@()
    $cases=@('retained-currentness','seal','qualification','cost-constraints','directory-work','fitness',
        'seed_is_not_sibling_disk_authority','divergent_overlay_and_private_cache_are_not_overwritten',
        'failed_partial_and_poisoned_leader_do_not_grant_or_poison',
        'precached_unaccounted_values_never_seal_from_nonempty_witnesses',
        'source_and_package_changes_invalidate_proof','config_roots_and_route_changes_stay_independent',
        'same_config_different_automatic_type_lookup_stays_distinct','typescript_persistence','resolver_positive')
    foreach($case in $cases){
        $result=@(& python (Join-Path $execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
        $code=$LASTEXITCODE
        if($result.Count -ne 1){throw "Unexpected capture output; preserve $out"}
        $path=Join-Path $result[0] 'receipt.json'
        $receipt=Get-Content $path -Raw|ConvertFrom-Json
        $receipts += [ordered]@{case=$case;receipt=$path;sha256=(Get-FileHash $path).Hash.ToLowerInvariant();status=$receipt.status}
        $receipts|ConvertTo-Json -Depth 8|Set-Content (Join-Path $out 'receipts.json') -Encoding utf8NoBOM
        if($code -ne 0 -or $receipt.status -cne 'TRACKED_TEST_COMPLETED' -or -not $receipt.post_source_and_binary_identity_verified){
            throw "Affected regression failed: $case; preserve $out"
        }
        Write-Host "PASS $case"
    }
    Write-Output "Return affected currentness receipts: $out"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
