# Operator-only one tracked warm-currentness RED; existing external safeguards unchanged.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    & rustfmt --edition 2024 --config skip_children=true src/tests/ir/ops_r1_shared_acquisition_cost.rs
    if($LASTEXITCODE -ne 0){throw 'Focused test formatting failed.'}
    & python (Join-Path $PSScriptRoot 'prepare-red.py')
    if($LASTEXITCODE -ne 0){throw 'RED identity preparation failed; no build started.'}
    $execution=Join-Path $PSScriptRoot 'execution'
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='currentness-28-red'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/currentness-28-red-build-01
    if($LASTEXITCODE -ne 0){throw 'RED compilation failed; no test started.'}
    $build=Join-Path $root 'target/ops-r1/currentness-28-red-build-01/build-receipt.json'
    $result=@(& python (Join-Path $execution 'capture.py') --build-receipt $build --case retained-currentness --wall-seconds 60 --memory-mib 512 --stop-wait 5)
    if($result.Count -ne 1){throw 'Unexpected capture output; preserve partial evidence.'}
    $receipt=Get-Content (Join-Path $result[0] 'receipt.json') -Raw|ConvertFrom-Json
    if($receipt.exit_code -ne 101 -or $receipt.status -cne 'TRACKED_TEST_FAILED_REVIEW_REASON' -or -not $receipt.post_source_and_binary_identity_verified){
        throw "Unexpected RED outcome; preserve $($result[0])"
    }
    $stderr=Get-Content (Join-Path $result[0] 'stderr.txt') -Raw
    if(-not $stderr.Contains('one exact retained-currentness constraint per call')){throw "Unrelated failure; preserve $($result[0])"}
    Write-Output "Return exact retained-currentness RED: $($result[0])"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
