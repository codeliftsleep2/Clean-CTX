# Operator-owned focused activation and catalog preservation capture. No build or tests.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function Save($Path, $Value) {
    [IO.File]::WriteAllText($Path, (ConvertTo-Json -InputObject $Value -Depth 100), $utf8)
}
function Same($Before, $After, $Label) {
    if ((ConvertTo-Json -InputObject $Before -Depth 100 -Compress) -cne
        (ConvertTo-Json -InputObject $After -Depth 100 -Compress)) { throw "$Label changed." }
}
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    if ((Get-FileHash $binary).Hash.ToLowerInvariant() -cne '4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f') { throw 'Executable changed.' }
    $review = Get-Content 'target/pcg-b11/bootstrap-review-20261114-220707-840/descriptor.json' -Raw | ConvertFrom-Json
    $digest = '4115d4b02491b3259c5165b429ea210fe5e0e3c2e53dadacaab3724a9d13d20e'
    if ($review.digest -cne $digest) { throw 'Review identity changed.' }
    $profile = (Resolve-Path '.clean-ctx/pcg-documents/profile.json').Path
    $settings = Get-Content $profile -Raw | ConvertFrom-Json
    $out = Join-Path $root ('target/pcg-b11/bootstrap-publication-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $helper = Join-Path $PSScriptRoot 'capture-production.py'
    & python $helper snapshot $settings.database_path (Join-Path $out 'catalog-before.json')
    if ($LASTEXITCODE -ne 0) { throw 'Before snapshot failed.' }
    $before = Get-Content (Join-Path $out 'catalog-before.json') -Raw | ConvertFrom-Json
    if ($before.generation -cne $review.descriptor.predecessor_generation -or $before.governance_publications.Count -ne 0) { throw 'Predecessor changed or governance already active; preserve receipt.' }
    if (@($before.authorization_grants | Where-Object { $_[0] -ceq $digest }).Count -ne 1) { throw 'Exact operator grant missing.' }
    Save (Join-Path $out 'requests.json') @(@{workspaceRoot=$root;mode='get';id='AUTH.PCG_DOCUMENTS'})
    & (Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1') `
        -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests (Join-Path $out 'requests.json') `
        -OutputRoot (Join-Path $out 'publication') -LaunchDirectory (Join-Path $root '.clean-ctx/pcg-documents/authoring')
    & python $helper snapshot $settings.database_path (Join-Path $out 'catalog-after.json')
    if ($LASTEXITCODE -ne 0) { throw 'After snapshot failed.' }
    $after = Get-Content (Join-Path $out 'catalog-after.json') -Raw | ConvertFrom-Json
    foreach ($field in 'members','relationships','historical_revisions','historical_revision_rows','historical_sources','authorization_grants','components') {
        Same $before.$field $after.$field $field
    }
    if ($after.governance_publications.Count -ne 1) { throw 'Expected exactly one governance publication.' }
    $publication = $after.governance_publications[0]
    if ($publication[0] -cne $after.generation -or $publication[1] -cne $review.descriptor.proposed_state_digest -or
        $publication[2] -cne $digest -or $publication[3] -cne $before.generation) { throw 'Published governance binding mismatch.' }
    if ($after.publication_history.Count -ne $before.publication_history.Count + 1) { throw 'Unexpected publication history growth.' }
    for ($i=0; $i -lt $before.publication_history.Count; $i++) {
        Same $before.publication_history[$i] $after.publication_history[$i] 'historical publication'
    }
    Save (Join-Path $out 'result.json') @{result='passed';generation=$after.generation;authorization_digest=$digest;identities=$after.members.Count;relationships=$after.relationships.Count;components=$after.components;governance_active=$true;historical_state_preserved=$true}
    Write-Host "Focused bootstrap publication verified: $out"
} finally { Pop-Location }
