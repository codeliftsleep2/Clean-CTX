# Operator-only compile and exact bounded RED capture. No production changes.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../../..'))
$isolated = Join-Path $root 'target/ops-r1/m3-isolated-23'
$execution = Join-Path $isolated 'verification/ops-r1/proposals/shared-acquisition/execution'
$savedTarget = $env:CARGO_TARGET_DIR
$savedVersion = $env:OPS_R1_GREEN_SOURCE
$savedStage = $env:OPS_R1_STAGE
$env:CARGO_TARGET_DIR = Join-Path $isolated 'target'
$env:OPS_R1_GREEN_SOURCE = 'performance-25-directory-red'
$env:OPS_R1_STAGE = 'green'
Push-Location $root
try {
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/performance-25-directory-red-build-01
    if ($LASTEXITCODE -ne 0) { throw 'RED build failed; no test started.' }
    $build = Join-Path $isolated 'target/ops-r1/performance-25-directory-red-build-01/build-receipt.json'
    $result = @(& python (Join-Path $execution 'capture.py') --build-receipt $build --case directory-work --wall-seconds 60 --memory-mib 512 --stop-wait 5)
    if ($result.Count -ne 1) { throw 'Unexpected capture output; preserve partial evidence.' }
    $receipt = Get-Content -LiteralPath (Join-Path $result[0] 'receipt.json') -Raw | ConvertFrom-Json
    if ($receipt.exit_code -ne 101 -or $receipt.status -cne 'TRACKED_TEST_FAILED_REVIEW_REASON' -or -not $receipt.post_source_and_binary_identity_verified) {
        throw "Unexpected RED outcome; preserve $($result[0])"
    }
    Write-Output "Return actual assertion and RED receipt for review: $($result[0])"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR = $savedTarget
    $env:OPS_R1_GREEN_SOURCE = $savedVersion
    $env:OPS_R1_STAGE = $savedStage
}
