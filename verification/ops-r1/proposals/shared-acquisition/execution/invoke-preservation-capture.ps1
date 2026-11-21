# Proposed operator handoff only. Execution requires separate approval.
# Reuses the pinned executable and existing monitor; no Cargo, source edits or retries.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
$savedStage=$env:OPS_R1_STAGE
Push-Location $root
try {
    $env:OPS_R1_STAGE='green'
    $manifest='verification/ops-r1/proposals/shared-acquisition/test-selections.json'
    $adapter='verification/ops-r1/proposals/shared-acquisition/execution/capture.py'
    $build='target/ops-r1/shared-green-amendment-15-build-reviewed/build-receipt.json'
    if((Get-FileHash -LiteralPath $manifest).Hash.ToLowerInvariant() -cne '2b8c04a0e4d10f5413095a04a5b87cb4416e53e7afc8457254f0cd52726505fa'){throw 'Test selection manifest changed; no child started.'}
    if((Get-FileHash -LiteralPath $adapter).Hash.ToLowerInvariant() -cne '57cf1977854f9df112899c9d0d6ca06a151a76145f12ad19ae9ce221e304e0f4'){throw 'Reviewed adapter changed; no child started.'}
    if((Get-FileHash -LiteralPath $build).Hash.ToLowerInvariant() -cne '7d9c66f917e730f0f0efd01ab60c42c778b6b639b9399276aff3ddac67db4a58'){throw 'Reviewed build receipt changed; no child started.'}
    $cases=@((Get-Content -LiteralPath $manifest -Raw|ConvertFrom-Json)|Where-Object {$_.case -cne 'fitness'})
    if($cases.Count -ne 25){throw 'Unexpected selection count; no child started.'}
    $number=0
    foreach($case in $cases){
        $number++
        Write-Output "[$number/25] $($case.identity)"
        & python $adapter --build-receipt $build --case $case.case --wall-seconds 60 --memory-mib 512 --stop-wait 5
        if($LASTEXITCODE -ne 0){throw "Stopped after $($case.case); preserve its printed evidence folder. No later case or retry started."}
    }
    Write-Output 'All 25 selected tracked tests completed. Review individual receipts; no CI/admission or broader safety certification implied.'
} finally {
    $env:OPS_R1_STAGE=$savedStage
    Pop-Location
}
