# Operator-owned ANG-R2 descriptor capture. This script never submits a grant.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$pins = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_CANDIDATE_PINS.json') -Raw | ConvertFrom-Json
Push-Location $root
try {
    if ((& git rev-parse HEAD) -cne $pins.baseline) { throw 'Baseline changed; review the candidate again.' }
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    foreach ($pin in $pins.files) {
        $actual = (Get-FileHash -LiteralPath (Join-Path $root $pin.path) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $pin.candidate_sha256) { throw "Candidate drift: $($pin.path)" }
    }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne $pins.binary_sha256) { throw 'Binary drift.' }
    if ((Get-FileHash -LiteralPath $profile).Hash.ToLowerInvariant() -cne $pins.profile_sha256) { throw 'Profile drift.' }
    $out = Join-Path $root ('target/ang-r2/governance-review-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $report = Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $report 2> (Join-Path $out 'stderr.txt')
    if ($LASTEXITCODE -ne 0) { throw "Describe failed. Preserve $out" }
    $value = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    if (-not $value.digest -or -not $value.descriptor) { throw "No exact descriptor. Inspect $out" }
    if ($value.authorization_recorded -or $value.accepted_publication) { throw 'Unexpected publication/grant reported.' }
    if ($value.descriptor.predecessor_generation -cne $pins.accepted_generation) { throw "Predecessor changed. Inspect $out" }
    $codes = @($value.diagnostics | ForEach-Object { $_.code })
    if ($codes.Count -ne 1 -or $codes[0] -cne 'E_GOVERNANCE_AUTHORIZATION_REQUIRED') {
        throw "Unexpected diagnostics. Preserve $out"
    }
    [ordered]@{
        category = 'Operator descriptor capture; no authorization or admission'
        digest = $value.digest
        descriptor_sha256 = (Get-FileHash -LiteralPath $report).Hash.ToLowerInvariant()
        expected_affected_authorities = @('AUTH.ANGULAR_RESOLVER', 'AUTH.ANGULAR_CANONICAL_RELATIONS', 'AUTH.TOOL_WORKFLOW')
        accepted_predecessor_generation = $pins.accepted_generation
    } | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Host "Descriptor captured: $out"
    Write-Host "Exact candidate digest: $($value.digest)"
    Write-Host 'Return the complete descriptor for review. No authorization submitted or graph admitted.'
} finally { Pop-Location }
