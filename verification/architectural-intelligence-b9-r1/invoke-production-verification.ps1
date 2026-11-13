param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot,
    [string]$ExpectedBinarySha256 = 'd91702b1c2a5b8e875b2c937e54b9ed4d5ad6fcbc304d79c9001967aff78fa38'
)
# User-run actual-root reconciliation/stdio. No build and no discovery-probe rerun.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output=[IO.Path]::GetFullPath($OutputRoot)
if(Test-Path -LiteralPath $output){throw 'Use a fresh production verification directory.'}
$binary=Join-Path $root 'target/debug/clean-ctx.exe'
$profile=Join-Path $root '.clean-ctx/pcg-documents/profile.json'
if($ExpectedBinarySha256 -notmatch '^[0-9a-f]{64}$'){throw 'Expected binary identity must be a lowercase SHA-256.'}
if((Get-FileHash $binary).Hash.ToLowerInvariant() -ne $ExpectedBinarySha256){throw 'Production executable identity changed.'}
if((Get-FileHash $profile).Hash.ToLowerInvariant() -ne '0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9'){throw 'Profile changed.'}
New-Item -ItemType Directory -Path $output|Out-Null
$utf8=[Text.UTF8Encoding]::new($false,$true)
function Write-Json([string]$path,$value){[IO.File]::WriteAllText($path,(ConvertTo-Json -InputObject $value -Depth 50),$utf8)}
$driver=Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
$launch=Join-Path $root '.clean-ctx/pcg-documents/authoring'
$baseline=Get-Content (Join-Path $root 'target/pcg-b10/baseline.json') -Raw|ConvertFrom-Json
$queries=[Collections.Generic.List[object]]::new()
$queries.Add(@{workspaceRoot=$root;mode='search';text='PassPipeline'})
$queries.Add(@{workspaceRoot=$root;mode='search'})
foreach($node in $baseline.nodes){$queries.Add(@{workspaceRoot=$root;mode='get';id=$node.id})}
$contexts=Import-Csv (Join-Path $root 'docs/architecture/knowledge-normalization/B9_CONTEXT_COMPLETENESS.csv')
foreach($context in $contexts){$queries.Add(@{workspaceRoot=$root;mode='context';id=$context.id})}
$queries.Add(@{workspaceRoot=$root;mode='related';id='AUTH.CANONICAL_IR';direction='both'})
$queriesPath=Join-Path $output 'graph-requests.json';Write-Json $queriesPath $queries.ToArray()
& $driver -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $queriesPath -OutputRoot (Join-Path $output 'graph') -LaunchDirectory $launch
$units=@{};$refs=[Collections.Generic.List[object]]::new()
for($index=0;$index -lt $baseline.nodes.Count;$index++){
    $p=Get-Content (Join-Path $output "graph/$($index+4).payload.json") -Raw|ConvertFrom-Json
    $unit=$p.units[0];$units[$unit.id]=$unit
    foreach($component in $unit.components){$refs.Add($component.reference)}
}
$pipeline=$units['PIPELINE-001'].components[0].reference
$invalid=$pipeline|ConvertTo-Json -Depth 8|ConvertFrom-Json;$invalid.end=999999
$oldRefs=@(foreach($id in @('AUTH.PCG_DOCUMENTS','AUTH.PCG_GLS','AUTH.PCG_PREAUTHORING')){
    $node=@($baseline.nodes|Where-Object id -eq $id)[0];$c=$node.components[0]
    [pscustomobject]@{id=$node.id;revision=$node.revision;ordinal=$c.ordinal;role=$c.role;label=$c.label;path=$c.path;sourceSha256=$c.source_sha256;start=$c.start;end=$c.end;contentSha256=$c.content_sha256;selector=$units[$id].components[0].selector;historical=$true}
})
$stale=$oldRefs[0]|ConvertTo-Json -Depth 8|ConvertFrom-Json;$stale.historical=$false
$oldDocument=@($baseline.nodes|Where-Object id -eq 'AUTH.PCG_DOCUMENTS')[0]
$acquisitions=@(
    @{workspaceRoot=$root;mode='acquire';references=$refs.ToArray()},
    @{workspaceRoot=$root;mode='acquire';references=@($pipeline)},
    @{workspaceRoot=$root;mode='acquire';references=@($pipeline,$pipeline,$invalid)},
    @{workspaceRoot=$root;mode='acquire';references=$oldRefs},
    @{workspaceRoot=$root;mode='acquire';references=@($stale)},
    $units['PIPELINE-001'].source_request,
    @{workspaceRoot=$root;mode='source';id=$oldDocument.id;revision=$oldDocument.revision;sourceSha256=$oldDocument.source_sha256;historical=$true}
)
$acquisitionPath=Join-Path $output 'acquisition-requests.json';Write-Json $acquisitionPath $acquisitions
& $driver -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $acquisitionPath -OutputRoot (Join-Path $output 'acquisition') -LaunchDirectory $launch
& (Join-Path $PSScriptRoot 'verify-production-capture.ps1') -WorkspaceRoot $root -CaptureRoot $output
