param([string]$WorkspaceRoot = (Get-Location).Path)
# Operator-owned first activity: reference-first graphs, then explicit acquisition.
# No build, tests, provider repair, independent evaluation or full gate.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = Join-Path $root ('target/pcg-b11/authority-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
$binary = Join-Path $root 'target/debug/clean-ctx.exe'
$profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
$expectedBinary = 'a555a0ad5374b7bc1b4db4a391ea0e389158880ac7d6c4c210c81f8f708266e0'
$expectedProfile = '0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9'
if ((Get-FileHash $binary).Hash.ToLowerInvariant() -ne $expectedBinary) { throw 'Baseline executable changed; preserve and report its identity.' }
if ((Get-FileHash $profile).Hash.ToLowerInvariant() -ne $expectedProfile) { throw 'Baseline profile changed; inspect before proceeding.' }
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function Save-Json([string]$Name, $Value) {
    [IO.File]::WriteAllText((Join-Path $output $Name), (ConvertTo-Json -InputObject $Value -Depth 60), $utf8)
}
$ids = @('AUTH.PCG_GLS','AUTH.PCG_DOCUMENTS','AUTH.PCG_PREAUTHORING',
    'AUTH.PCG_ENROLLED_V1','AUTH.PCG_ACQUISITION_V1','AUTH.PCG_PROJECTION_V1',
    'AUTH.PCG_MCP_V1','AUTH.FINAL_VERIFICATION')
$queries = @($ids | ForEach-Object { @{workspaceRoot=$root;mode='context';id=$_} })
foreach ($term in @('admission','reconciliation','lifecycle','identity')) {
    $queries += @{workspaceRoot=$root;mode='search';text=$term}
}
Save-Json 'graph-requests.json' $queries
Save-Json 'baseline.json' @{
    branch=(& git -C $root branch --show-current);head=(& git -C $root rev-parse HEAD)
    executable_sha256=$expectedBinary;profile_sha256=$expectedProfile
    historical_counts=@{identities=218;relationships=221;components=326}
    historical_generation='684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70'
    qualification='Historical graph identity must be compared with actual returned state.'
}
$driver = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
$launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
& $driver -WorkspaceRoot $root -Profile $profile -Binary $binary `
    -Requests (Join-Path $output 'graph-requests.json') -OutputRoot (Join-Path $output 'graph') -LaunchDirectory $launch
$references = [Collections.Generic.List[object]]::new()
$seen = [Collections.Generic.HashSet[string]]::new()
$graphs = @()
for ($index=0; $index -lt $queries.Count; $index++) {
    $payload = Get-Content (Join-Path $output "graph/$($index+2).payload.json") -Raw | ConvertFrom-Json
    if (-not $payload.complete -or -not $payload.catalog_current -or $payload.result_state -ne 'ready') {
        throw "Authority request $index is not complete/current/ready; preserve all receipts."
    }
    $graphs += @{request=$queries[$index];generation=$payload.generation;coverage=$payload.catalog_coverage;units=@($payload.units.id)}
    # Acquire authority closures first; search matches remain discovery evidence
    # until their applicability and transitive authority have been inspected.
    if ($index -ge $ids.Count) { continue }
    foreach ($unit in $payload.units) {
        foreach ($component in $unit.components) {
            $key = "$($unit.id)|$($unit.revision)|$($component.ordinal)"
            if ($seen.Add($key)) { $references.Add($component.reference) }
        }
    }
}
if (-not $references.Count) { throw 'No exact authority component references returned.' }
Save-Json 'graph-summary.json' $graphs
Save-Json 'acquisition-requests.json' @(@{workspaceRoot=$root;mode='acquire';references=$references.ToArray()})
& $driver -WorkspaceRoot $root -Profile $profile -Binary $binary `
    -Requests (Join-Path $output 'acquisition-requests.json') -OutputRoot (Join-Path $output 'acquisition') -LaunchDirectory $launch
$acquired = Get-Content (Join-Path $output 'acquisition/2.payload.json') -Raw | ConvertFrom-Json
if (-not $acquired.complete -or -not $acquired.catalog_current -or $acquired.result_state -ne 'ready') {
    throw 'Authority acquisition incomplete; preserve receipts and report the actual limitation.'
}
Write-Output "B11 authority discovery/acquisition captured: $output"
