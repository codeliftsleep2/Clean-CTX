# Operator-only two targeted witnesses; no production batch change or full gate.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    & rustfmt --edition 2024 --config skip_children=true src/mcp/tool_handlers/query.rs src/tests/mcp/ops_r1_acquisition_fitness.rs src/tests/mcp/ops_r1_query_phase_currentness.rs
    if($LASTEXITCODE -ne 0){throw 'Focused witness formatting failed.'}
    & python (Join-Path $PSScriptRoot 'prepare-red.py')
    if($LASTEXITCODE -ne 0){throw 'Source identity preparation failed; no build started.'}
    $execution=Join-Path $PSScriptRoot 'execution'
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='phase-29-red'
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/phase-29-red-build-01
    if($LASTEXITCODE -ne 0){throw 'Focused compilation failed; no test started.'}
    $build=Join-Path $root 'target/ops-r1/phase-29-red-build-01/build-receipt.json'
    foreach($case in @('phase-edit','public-work')){
        $result=@(& python (Join-Path $execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
        if($result.Count -ne 1){throw 'Unexpected capture output; preserve partial evidence.'}
        $receipt=Get-Content (Join-Path $result[0] 'receipt.json') -Raw|ConvertFrom-Json
        if(-not $receipt.post_source_and_binary_identity_verified){throw "Identity failure; preserve $($result[0])"}
        if($case -eq 'phase-edit'){
            if($receipt.exit_code -ne 0 -or $receipt.status -cne 'TRACKED_TEST_COMPLETED'){throw "Boundary control failed; preserve $($result[0])"}
        } else {
            $stderr=Get-Content (Join-Path $result[0] 'stderr.txt') -Raw
            if($receipt.exit_code -ne 101 -or $receipt.status -cne 'TRACKED_TEST_FAILED_REVIEW_REASON' -or -not $stderr.Contains('one independent preparation phase and one shared consumption validation')){
                throw "Public work witness did not produce intended RED; preserve $($result[0])"
            }
        }
        Write-Output "Return $case evidence: $($result[0])"
    }
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
