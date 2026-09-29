$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$variants = Join-Path $repositoryRoot "target\context-compression-verification\organization-experiments"
$manifest = Get-Content -Raw (Join-Path $definitionRoot "investigation\organization-experiments-v1.json") | ConvertFrom-Json -Depth 100

function Semantic-Json([string]$path, [bool]$removeOrganization) {
    $text = Get-Content -Raw $path
    $start = $text.IndexOf("{")
    $end = $text.IndexOf("`n§PATHMAP")
    if ($start -lt 0 -or $end -lt 0) { throw "Bad framing: $path" }
    $value = $text.Substring($start, $end - $start) | ConvertFrom-Json -Depth 100
    if ($removeOrganization) { $value.PSObject.Properties.Remove("_organization") }
    return $value | ConvertTo-Json -Depth 100 -Compress
}

foreach ($experiment in $manifest.experiments) {
    $source = Join-Path $captures "$($experiment.capture)\control-full.txt"
    $variant = Join-Path $variants "$($experiment.id).txt"
    if (-not (Test-Path $source) -or -not (Test-Path $variant)) { throw "Missing source or variant for $($experiment.id)" }
    if ((Semantic-Json $source $false) -cne (Semantic-Json $variant $true)) { throw "$($experiment.id): semantic facts changed" }
    $variantText = Get-Content -Raw $variant
    if ($variantText -notmatch '"_organization"') { throw "$($experiment.id): missing organization index" }
    if ($variantText -match '(classes|interfaces|semantic_edges)\[\d+') {
        throw "$($experiment.id): organization metadata contains a fragile array-index path"
    }
}
Write-Host "PASS: deterministic CONTROL-FULL fact equality and experiment wiring validated."
