# Operator-owned read-only descriptor capture. Never submits authorization.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $expected = '4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f'
    if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne $expected) { throw 'Executable identity changed.' }
    $out = Join-Path $root ('target/pcg-b11/bootstrap-review-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $report = Join-Path $out 'descriptor.json'
    $profile = (Resolve-Path -LiteralPath (Join-Path $root '.clean-ctx/pcg-documents/profile.json')).Path
    & $binary pcg documents describe --profile $profile 1> $report 2> (Join-Path $out 'stderr.txt')
    if ($LASTEXITCODE -ne 0) {
        $detail = Get-Content -LiteralPath (Join-Path $out 'stderr.txt') -Raw
        throw "Descriptor capture failed: $detail Preserve $out"
    }
    $value = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    if (-not $value.digest -or -not $value.descriptor) { throw 'Missing exact candidate descriptor.' }
    if ($value.authorization_recorded -or $value.accepted_publication) { throw 'Unexpected mutation reported.' }
    $codes = @($value.diagnostics | ForEach-Object { $_.code })
    if ($codes.Count -ne 1 -or $codes[0] -cne 'E_GOVERNANCE_BOOTSTRAP_REQUIRED') {
        throw "Unexpected diagnostics; preserve $out"
    }
    Write-Host "Review captured: $out"
    Write-Host "Exact candidate digest: $($value.digest)"
    Write-Host 'No authorization submitted. No accepted graph publication created.'
} finally { Pop-Location }
