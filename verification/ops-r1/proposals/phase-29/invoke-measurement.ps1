# One matched currentness comparison; no full gates or repeated growth campaign.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$isolated=Join-Path $root 'target/ops-r1/m3-isolated-23'
$out=Join-Path $root ('target/ops-r1/phase-29-measurement-'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$savedTarget=$env:CARGO_TARGET_DIR
$savedVersion=$env:OPS_R1_GREEN_SOURCE
$growth=Join-Path $root 'verification/ops-r1/proposals/shared-acquisition/performance-25/run-growth.py'
Push-Location $root
try {
    $baseline=Join-Path $isolated 'target/ops-r1/currentness-28-unobserved-candidate-build-01/build-receipt.json'
    & python $growth currentness-28-unobserved-candidate $baseline directories-scripts-64 2>&1 |
        Tee-Object -FilePath (Join-Path $out 'baseline-output.txt')
    if($LASTEXITCODE -ne 0){throw "Matched baseline stopped; preserve $out. No automatic retry."}
    & python (Join-Path $PSScriptRoot 'prepare-measurement.py')
    if($LASTEXITCODE -ne 0){throw "Measurement source preparation failed; preserve $out"}
    $execution=Join-Path $isolated 'verification/ops-r1/proposals/shared-acquisition/execution'
    $env:CARGO_TARGET_DIR=Join-Path $isolated 'target'
    $env:OPS_R1_GREEN_SOURCE='phase-29-performance'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/phase-29-measurement-build-01
    if($LASTEXITCODE -ne 0){throw "Measurement compilation failed; preserve $out"}
    $build=Join-Path $isolated 'target/ops-r1/phase-29-measurement-build-01/build-receipt.json'
    & python $growth phase-29-performance $build directories-scripts-64 2>&1 |
        Tee-Object -FilePath (Join-Path $out 'candidate-output.txt')
    if($LASTEXITCODE -ne 0){throw "Candidate measurement stopped; preserve $out. No automatic retry."}
    Write-Output "Return matched currentness observations: $out"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
