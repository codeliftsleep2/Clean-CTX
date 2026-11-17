# Explicit empirical workflow. No new threshold or architectural acceptance grant.
param([switch]$ListOnly)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/measurements-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out | Out-Null
Push-Location $root
try {
    $config=Get-Content (Join-Path $PSScriptRoot 'membership.json') -Raw|ConvertFrom-Json
    $items=@(Get-Content (Join-Path $PSScriptRoot $config.measurements_path) -Raw|ConvertFrom-Json)
    if(-not $items.Count -or $items.Count -ne $config.expected_measurements){throw 'Empty/incomplete measurement selection'}
    $listing=Join-Path $out 'registration.log'
    & cargo test --lib --all-features -- --list *> $listing
    if($LASTEXITCODE -ne 0){throw "Measurement registration failed; preserve $out"}
    $text=[IO.File]::ReadAllText($listing)
    foreach($item in $items){
        $pattern='(?m)^'+[regex]::Escape($item.name)+': test\r?$'
        if([regex]::Matches($text,$pattern).Count -ne 1){throw "Missing/duplicate registered measurement: $($item.name)"}
    }
    $selection=@($items|ForEach-Object{$_.name})
    $paths=@(git ls-files --cached --others --exclude-standard -- src proxy Cargo.toml Cargo.lock verification/ci-verify-r1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $pins=@($paths|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    [ordered]@{
        agent='Agent MaxHeadRoom';head=(git rev-parse HEAD);files=$pins;selection=$selection
        features='all-features';mode='empirical; original fixtures, warmups, rotations and samples'
        list_only=[bool]$ListOnly;scope='No automatic performance acceptance; historical acceptance obligations remain applicable.'
    }|ConvertTo-Json -Depth 6|Set-Content (Join-Path $out 'configuration.json') -Encoding utf8NoBOM
    if($ListOnly){Write-Output "Registered $($selection.Count) exact measurement assets: $out";return}
    $log=Join-Path $out 'measurements.log'
    & cargo test --lib --all-features -- --ignored --exact --test-threads=1 --nocapture @selection *> $log
    $code=$LASTEXITCODE
    $summary=[regex]::Match([IO.File]::ReadAllText($log),'(?m)^test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
    $passed=$code -eq 0 -and $summary.Success -and [int]$summary.Groups[1].Value -eq $selection.Count
    [ordered]@{exitCode=$code;passed=$passed;category='Empirical measurement execution, not general performance certification';log_sha256=(Get-FileHash $log).Hash}|
        ConvertTo-Json|Set-Content (Join-Path $out 'result.json') -Encoding utf8NoBOM
    if(-not $passed){Get-Content -LiteralPath $log -Tail 100;throw "Measurement workflow failed; preserve $out"}
    Write-Output "Empirical assets executed successfully: $out"
} finally {Pop-Location}
