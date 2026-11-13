param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot
)
# User-owned runtimes. Preserves original failures and runs one corrected host turn.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh production/host output directory.' }
$binary = Join-Path $root 'target/debug/clean-ctx.exe'
$profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
$launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
$binaryHash = '74e1be0de109610a35e06c1f369f4662aed0af48c74d44be941fa38a1ac4cd81'
$profileHash = '0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9'
$generation = '80c1195a7a9df253e4221635bd3feb8201d15ac4e0fbc328425e59cbe04b03e5'
if ((Get-FileHash $binary).Hash.ToLowerInvariant() -ne $binaryHash) { throw 'Rebuilt executable identity differs.' }
if ((Get-FileHash $profile).Hash.ToLowerInvariant() -ne $profileHash) { throw 'Profile identity differs.' }
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function Write-Json([string]$path, $value) {
    [IO.File]::WriteAllText($path, (ConvertTo-Json -InputObject $value -Depth 30), $utf8)
}
$driver = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
$requests = @(Get-Content (Join-Path $PSScriptRoot 'knowledge-production-requests.json') -Raw | ConvertFrom-Json)
foreach ($request in $requests) { $request.workspaceRoot = $root }
$requests += [pscustomobject]@{workspaceRoot=$root; mode='search'}
$requestPath = Join-Path $output 'requests.json'
Write-Json $requestPath $requests
$production = Join-Path $output 'production'
& $driver -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $requestPath -OutputRoot $production -LaunchDirectory $launch
$receipt = Get-Content (Join-Path $production 'receipt.json') -Raw | ConvertFrom-Json
if ($receipt.error -or $receipt.queries.Count -ne 13) { throw 'Production request set did not complete.' }
foreach ($query in $receipt.queries) {
    if ($query.result_state -ne 'ready' -or !$query.catalog_current -or !$query.complete -or $query.generation -ne $generation) {
        throw 'Production readiness/currentness/completeness/generation changed.'
    }
}
$catalog = Get-Content (Join-Path $production '1.response.json') -Raw | ConvertFrom-Json
$schema = ($catalog.result.tools | Where-Object name -eq pcg_knowledge_v1).inputSchema
if ($schema.oneOf -or $schema.allOf -or $schema.anyOf -or @($schema.properties.PSObject.Properties).Count -ne 15) {
    throw 'Production descriptor is not the expected flat 15-field object.'
}
if ($schema.additionalProperties -ne $false -or ($schema.required -join ',') -ne 'workspaceRoot,mode') {
    throw 'Production descriptor common requirements changed.'
}
Write-Json (Join-Path $output 'production-input-schema.json') $schema
$last = Get-Content (Join-Path $production '14.response.json') -Raw | ConvertFrom-Json
$all = $last.result.content[0].text | ConvertFrom-Json
$baseline = Get-Content (Join-Path $root 'target/pcg-b10/baseline.json') -Raw | ConvertFrom-Json
if ($all.units.Count -ne 218) { throw 'Identity count changed.' }
$members = @{}
foreach ($unit in $all.units) {
    if ($members.ContainsKey($unit.id)) { throw 'Duplicate returned identity.' }
    $members[$unit.id] = $unit
}
foreach ($node in $baseline.nodes) {
    if (!$members.ContainsKey($node.id) -or $members[$node.id].revision -ne $node.revision) {
        throw "Identity/revision changed: $($node.id)"
    }
}
$actualEdges = @($all.units | ForEach-Object {
    $unit = $_
    foreach ($edge in $unit.declared_relationships) { "$($unit.id)|$($edge.kind)|$($edge.target)" }
} | Sort-Object)
$expectedEdges = @($baseline.edges | ForEach-Object { "$($_.source)|$($_.kind)|$($_.target)" } | Sort-Object)
if ($actualEdges.Count -ne 221 -or (Compare-Object $actualEdges $expectedEdges)) { throw 'Typed relationships changed.' }
Write-Json (Join-Path $output 'membership.json') @{identities=218;relationships=221;revisions='all baseline revisions identical';generation=$generation}
$refusals = @(
    @{name='unknown-query'; args=@{workspaceRoot=$root;mode='search';query='architectural pipeline ordering'}; code=-32602; message='pcg/invalid_request'},
    @{name='hash-mismatch'; args=@{workspaceRoot=$root;mode='source';id='PIPELINE-001';sourceSha256=('0'*64)}; code=-32603; message='pcg/source_revision_changed'}
)
foreach ($case in $refusals) {
    $caseRequests = Join-Path $output "$($case.name)-requests.json"
    Write-Json $caseRequests @($case.args)
    $caseOutput = Join-Path $output $case.name
    try {
        & $driver -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $caseRequests -OutputRoot $caseOutput -LaunchDirectory $launch
    } catch {
        # The existing driver throws on expected protocol errors; verify raw response.
        if (!(Test-Path (Join-Path $caseOutput '2.response.json'))) { throw }
    }
    $response = Get-Content (Join-Path $caseOutput '2.response.json') -Raw | ConvertFrom-Json
    if ($response.error.code -ne $case.code -or $response.error.message -ne $case.message) {
        throw "Expected refusal not preserved: $($case.name)"
    }
    Write-Output "Verified expected refusal: $($case.message)"
}
Write-Output 'Production descriptor, requests, membership and refusals checked; starting one actual-host smoke.'
& (Join-Path $PSScriptRoot 'invoke-host-smoke.ps1') -WorkspaceRoot $root -OutputRoot (Join-Path $output 'host') -ExpectedBinarySha256 $binaryHash
