# Agent MaxHeadRoom: read-only descriptor retry after declaration-fence correction.
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/descriptor-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
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
