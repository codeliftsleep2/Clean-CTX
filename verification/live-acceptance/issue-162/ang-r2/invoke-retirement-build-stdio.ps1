# Operator-owned current scoped lint/build/guards followed by pinned synthetic stdio.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $before = @(Get-ChildItem 'target/ang-r2' -Directory -Filter 'registration-build-*' | Select-Object -ExpandProperty FullName)
    & (Join-Path $PSScriptRoot 'invoke-registration-build.ps1')
    $builds = @(Get-ChildItem 'target/ang-r2' -Directory -Filter 'registration-build-*' | Where-Object { $_.FullName -notin $before })
    if ($builds.Count -ne 1 -or -not (Test-Path (Join-Path $builds[0].FullName 'binary.json'))) { throw 'Expected exactly one newly completed build capture.' }
    $out = Join-Path $root ('target/ang-r2/retirement-production-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & node 'verification/live-acceptance/issue-162/ang-r2/registration-stdio.mjs' $builds[0].FullName 1> (Join-Path $out 'stdout.txt') 2> (Join-Path $out 'stderr.txt')
    $code = $LASTEXITCODE
    $watch.Stop()
    [ordered]@{
        category='Operator production binary/stdio; not Rust test, CI or Claude field acceptance'
        command=('node verification/live-acceptance/issue-162/ang-r2/registration-stdio.mjs ' + $builds[0].FullName)
        build_receipt_directory=$builds[0].FullName;exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds
        binary_manifest_sha256=(Get-FileHash (Join-Path $builds[0].FullName 'binary.json')).Hash.ToLowerInvariant()
        stdout_sha256=(Get-FileHash (Join-Path $out 'stdout.txt')).Hash.ToLowerInvariant()
        stderr_sha256=(Get-FileHash (Join-Path $out 'stderr.txt')).Hash.ToLowerInvariant()
    } | ConvertTo-Json | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $out 'stdout.txt')
    if ($code -ne 0) { throw "Production stdio failed; preserve $out" }
    Write-Host "Return build, stdio and wrapper receipts: $out. Governed admission and CI remain separate."
} finally { Pop-Location }
