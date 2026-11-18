# Operator-owned final metadata guards and actual admission. No Cargo, commit or push.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/publication-guards-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
$review=Join-Path $root 'target/ci-verify-r1/delivery-review-core-query'
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    foreach($pin in @(Get-Content (Join-Path $review 'candidate-sources.json') -Raw|ConvertFrom-Json)) {
        if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw "Candidate source changed: $($pin.path)"}
    }
    foreach($step in @(
        @{name='utf8';arguments=@('-NoProfile','-File','scripts/check-utf8.ps1')},
        @{name='active-size';arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')},
        @{name='actual-admission';arguments=@('-NoProfile','-File','verification/ci-verify-r1/invoke-admission.ps1','-ReviewDirectory',$review,'-ExpectedDigest','b6ee555e27f4bdffd456c80b1f64d0664ae9dca9cfc32672b71570fa8957d737')}
    )) {
        $log=Join-Path $out ($step.name+'.log')
        $argumentsForRun=$step.arguments
        & pwsh @argumentsForRun *> $log
        $code=$LASTEXITCODE
        @{name=$step.name;arguments=$argumentsForRun;exitCode=$code;log_sha256=(Get-FileHash -LiteralPath $log).Hash}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $out ($step.name+'-receipt.json')) -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 12
        if($code -ne 0){throw "$($step.name) failed; preserve $out"}
    }
    Write-Output "Return final metadata guard/admission receipts: $out"
} finally {Pop-Location}
