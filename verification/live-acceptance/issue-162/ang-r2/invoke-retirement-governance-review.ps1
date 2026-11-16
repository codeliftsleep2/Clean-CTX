# Operator-owned actual descriptor validation and normal admission-check path.
# No protected-authority authorization is created or submitted by this script.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$utf8 = [Text.UTF8Encoding]::new($false,$true)
$pins = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_RETIREMENT_GOVERNANCE_PINS.json') -Raw -Encoding utf8 | ConvertFrom-Json
Push-Location $root
try {
    if ((& git rev-parse HEAD) -cne $pins.baseline) { throw 'Baseline changed; review candidate again.' }
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if ((Get-FileHash $binary).Hash.ToLowerInvariant() -cne $pins.binary_sha256) { throw 'Verified executable changed.' }
    if ((Get-FileHash $profile).Hash.ToLowerInvariant() -cne $pins.profile_sha256) { throw 'Profile changed.' }
    if ((Get-FileHash $pins.source).Hash.ToLowerInvariant() -cne $pins.candidate_sha256) { throw 'Governed candidate changed.' }
    $out = Join-Path $root ('target/ang-r2/retirement-governance-review-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $descriptor = Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptor 2> (Join-Path $out 'describe-stderr.txt')
    if ($LASTEXITCODE -ne 0) { throw "Actual descriptor checker failed; preserve $out" }
    $value = Get-Content -LiteralPath $descriptor -Raw -Encoding utf8 | ConvertFrom-Json
    if ($value.authorization_recorded -or $value.accepted_publication) { throw 'Descriptor reported an unexpected mutation.' }
    if ($value.descriptor -and $value.descriptor.predecessor_generation -cne $pins.accepted_generation) { throw "Predecessor changed; preserve $out" }
    $requests = Join-Path $out 'context-request.json'
    [IO.File]::WriteAllText($requests,(ConvertTo-Json -InputObject @(@{workspaceRoot=$root;mode='context';id='DOC-270'}) -Depth 20),$utf8)
    & (Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1') `
        -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $requests `
        -OutputRoot (Join-Path $out 'admission-check') -LaunchDirectory (Join-Path $root '.clean-ctx/pcg-documents/authoring')
    [ordered]@{
        category='Actual Rust descriptor and normal document admission checker; no protected grant submitted'
        candidate_sha256=$pins.candidate_sha256;expected_predecessor=$pins.accepted_generation
        descriptor_sha256=(Get-FileHash $descriptor).Hash.ToLowerInvariant()
        digest=$value.digest;diagnostics=$value.diagnostics
    } | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Host "Return complete descriptor, diagnostics and admission-check receipts: $out"
    Write-Host 'A protected authorization blocker requires review of this exact Rust descriptor; no grant was submitted.'
} finally { Pop-Location }
