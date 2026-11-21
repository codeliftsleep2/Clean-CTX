# Operator-owned OPS-R1 census, guards, read-only B11 descriptor and actual admission attempt.
# No Cargo/build/test, authorization, commit, push, CI dispatch or repair implementation.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
if($PSVersionTable.PSVersion.Major -lt 7){throw 'Use PowerShell 7 or later.'}
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $root
try {
    $out=Join-Path $root ('target/ops-r1/checkpoint-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $out -Force|Out-Null
    $binary=Join-Path $root 'target/debug/clean-ctx.exe'
    $profile=Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne 'f03052a27e2a916ddf52dfb0a18deb246b387c386c04584f8dc8d8115c3ff4cd'){throw 'Production authoring binary changed; inspect identity. No rebuild is requested.'}
    $generator=Join-Path $PSScriptRoot 'census.py'
    $summary=Join-Path $root 'verification/ops-r1/test-census-index.json'
    $reuse=$false
    if(Test-Path -LiteralPath $summary){
        $old=Get-Content -LiteralPath $summary -Raw|ConvertFrom-Json
        $reuse=$old.generator_sha256 -ceq (Get-FileHash $generator).Hash.ToLowerInvariant() -and $old.head -ceq (git rev-parse HEAD)
        if($reuse){
            $archive=Join-Path $root $old.generated_archive
            $reuse=(Test-Path -LiteralPath $archive) -and (Get-FileHash $archive).Hash.ToLowerInvariant() -ceq $old.generated_archive_sha256
        }
    }
    if(-not $reuse){
        & python $generator 1> (Join-Path $out 'census-stdout.txt') 2> (Join-Path $out 'census-stderr.txt')
        if($LASTEXITCODE -ne 0){throw "Offline census failed; preserve $out"}
    }
    Copy-Item -LiteralPath $summary -Destination (Join-Path $out 'census-summary.json')
    foreach($guard in @('check-utf8.ps1','check-file-sizes.ps1')){
        & pwsh -NoProfile -File (Join-Path $root "scripts/$guard") 1> (Join-Path $out "$guard-stdout.txt") 2> (Join-Path $out "$guard-stderr.txt")
        if($LASTEXITCODE -ne 0){throw "$guard failed; preserve $out"}
    }
    $pins=@(git ls-files --modified --others --exclude-standard|Sort-Object -Unique|ForEach-Object{
        [ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}
    })
    ConvertTo-Json -InputObject $pins -Depth 5|Set-Content (Join-Path $out 'candidate-sources.json') -Encoding utf8NoBOM
    $descriptor=Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptor 2> (Join-Path $out 'descriptor-stderr.txt')
    if($LASTEXITCODE -ne 0){throw "Actual descriptor/checker failed; preserve $out"}
    $value=Get-Content -LiteralPath $descriptor -Raw|ConvertFrom-Json
    if($value.authorization_recorded -or $value.accepted_publication){throw "Unexpected descriptor mutation reported; preserve $out"}
    if(-not $value.digest -or -not $value.descriptor){
        $details=(@($value.diagnostics|ForEach-Object{"$($_.code): $($_.message) [$($_.path)]"}) -join '; ')
        throw "Descriptor refused by actual checker: $details. Preserve $out"
    }
    $request=Join-Path $out 'contexts.json'
    # One root context includes the declared DOC-270/VERIFY-001 dependency closure.
    # A refusal needs no repeated equivalent context calls.
    $requests=@(@{workspaceRoot=$root;mode='context';id='OPS-001'})
    ConvertTo-Json -InputObject @($requests) -Depth 10|Set-Content $request -Encoding utf8NoBOM
    $admission='REFUSED_OR_INCOMPLETE'
    try {
        & (Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1') -WorkspaceRoot $root -Binary $binary -Profile $profile -LaunchDirectory (Join-Path $root '.clean-ctx/pcg-documents/authoring') -Requests $request -OutputRoot (Join-Path $out 'contexts')
        $payloads=@(Get-ChildItem (Join-Path $out 'contexts') -Filter '*.payload.json')
        if($payloads.Count -eq 1){
            $ready=@($payloads|ForEach-Object{Get-Content -LiteralPath $_.FullName -Raw|ConvertFrom-Json}|Where-Object{$_.result_state -ceq 'ready' -and $_.complete -and $_.catalog_current})
            if($ready.Count -eq 1){
                $ids=@($ready[0].units|ForEach-Object{$_.id})
                if(@('OPS-001','DOC-270','VERIFY-001'|Where-Object{$_ -notin $ids}).Count -eq 0){
                    $admission='CONTEXTS_READY_SOURCE_ACQUISITION_NOT_YET_VERIFIED'
                }
            }
        }
    } catch {
        $_.ToString()|Set-Content (Join-Path $out 'admission-error.txt') -Encoding utf8NoBOM
    }
    foreach($pin in $pins){if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw "Candidate drift: $($pin.path)"}}
    [ordered]@{
        agent='Agent MaxHeadRoom';category='Owner checkpoint evidence, not repair or enforcement implementation'
        descriptor_digest=$value.digest;descriptor_sha256=(Get-FileHash $descriptor).Hash
        admission_standing=$admission;census_reused=$reuse
        binary_sha256=(Get-FileHash $binary).Hash;profile_sha256=(Get-FileHash $profile).Hash
        diagnostic_codes=@($value.diagnostics|ForEach-Object{$_.code})
        authorization_submitted=$false;commit_or_push_performed=$false
    }|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Return checkpoint receipts and exact descriptor: $out"
    Write-Output "Admission standing: $admission. No protected authorization was submitted."
} finally {Pop-Location}
