# One repeated-request tracked witness through the existing external monitor.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/ops_r1_acquisition_fitness.rs
    if($LASTEXITCODE -ne 0){throw 'Focused formatting failed.'}
    & python (Join-Path $PSScriptRoot 'prepare.py')
    if($LASTEXITCODE -ne 0){throw 'Source reconciliation failed; no compilation started.'}
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='retention-30'
    $execution=Join-Path $PSScriptRoot 'execution'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/retention-30-build-01
    if($LASTEXITCODE -ne 0){throw 'Focused compilation failed; preserve its receipt.'}
    $build=Join-Path $root 'target/ops-r1/retention-30-build-01/build-receipt.json'
    & python (Join-Path $execution 'capture.py') --build-receipt $build --case repeated-requests --wall-seconds 60 --memory-mib 512 --stop-wait 5
    if($LASTEXITCODE -ne 0){throw 'Repeated-request witness stopped or failed; preserve partial evidence. No automatic retry.'}
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
