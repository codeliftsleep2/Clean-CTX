# Agent MaxHeadRoom: continue remaining checkpoint steps; no test reruns.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/checkpoint-continuation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Write-Output "CI-VERIFY-R1 continuation: $out"
Push-Location $root
try {
    function Invoke-Step([string]$Name,[string]$Executable,[string[]]$Arguments){
        $log=Join-Path $out ($Name+'.log')
        $started=Get-Date
        & $Executable @Arguments *> $log
        $code=$LASTEXITCODE
        [ordered]@{name=$Name;executable=$Executable;arguments=$Arguments;exitCode=$code;seconds=((Get-Date)-$started).TotalSeconds;log_sha256=(Get-FileHash $log).Hash}|
            ConvertTo-Json -Depth 5|Set-Content (Join-Path $out ($Name+'-receipt.json')) -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 20
        if($code -ne 0){throw "$Name failed; preserve $out"}
    }
    Invoke-Step 'case-helper-format' 'rustfmt' @('--edition','2024','--config','skip_children=true','src/tests/compression/core_ir_registration.rs')
    Invoke-Step 'measurement-registration-only' 'pwsh' @('-NoProfile','-File','verification/ci-verify-r1/invoke-measurements.ps1','-ListOnly')
    Invoke-Step 'root-library-tests-clippy' 'cargo' @('clippy','--lib','--tests','--all-features','--','-D','warnings')
    Invoke-Step 'utf8' 'pwsh' @('-NoProfile','-File','scripts/check-utf8.ps1')
    Invoke-Step 'active-size' 'pwsh' @('-NoProfile','-File','scripts/check-file-sizes.ps1')
    $pins=@(git ls-files --modified --others --exclude-standard|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    $pins|ConvertTo-Json -Depth 4|Set-Content (Join-Path $out 'candidate-sources.json') -Encoding utf8NoBOM
    $binary=Join-Path $root 'target/debug/clean-ctx.exe'
    $profile=Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    $baseline=Get-Content (Join-Path $root 'target/ci-verify-r1/authority-20261117-132802-523/baseline.json') -Raw|ConvertFrom-Json
    if((Get-FileHash $binary).Hash.ToLowerInvariant() -cne $baseline.executable_sha256 -or (Get-FileHash $profile).Hash.ToLowerInvariant() -cne $baseline.profile_sha256){throw 'Production authoring binary/profile changed; inspect actual identities'}
    $descriptor=Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptor 2> (Join-Path $out 'descriptor-stderr.txt')
    if($LASTEXITCODE -ne 0){throw "Actual B11 descriptor/check failed; preserve $out"}
    $value=Get-Content -LiteralPath $descriptor -Raw|ConvertFrom-Json
    if(-not $value.digest -or -not $value.descriptor -or $value.authorization_recorded -or $value.accepted_publication){throw "Invalid read-only descriptor; preserve $out"}
    foreach($pin in $pins){if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw "Candidate source drift: $($pin.path)"}}
    Write-Output "Return checkpoint receipts and exact descriptor for review: $out"
    Write-Output 'No authorization, publication, empirical campaign, commit or push performed.'
} finally {Pop-Location}
