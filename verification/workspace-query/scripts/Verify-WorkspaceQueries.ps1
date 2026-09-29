$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\workspace-query-verification\captures"
if (-not (Test-Path $captures)) { throw "Run Capture-WorkspaceQueries.ps1 first" }

function Response([string]$name) {
    $path = Join-Path $captures "$name.json"
    if (-not (Test-Path $path)) { throw "Missing capture: $path" }
    return Get-Content -Raw $path | ConvertFrom-Json -Depth 100
}
function Require([bool]$condition, [string]$message) {
    if (-not $condition) { throw "FAIL: $message" }
}
function Text($response) { return [string]$response.result.content[0].text }
function Structured($response) { return $response.result.structuredContent }

$find = Response "find-entities"
$forward = Response "forward-edges"
$reverse = Response "reverse-edges"
$inFile = Response "entities-in-file"
$transitive = Response "transitive-dependencies"
$cycle = Response "has-cycle"
$negative = Response "negative-reverse-edges"

Require (@((Structured $find).entities | Where-Object { $_.domain -eq "angular" -and $_.entity_type -eq "Service" -and $_.name -eq "Alpha" }).Count -gt 0) "find_entities omitted angular Service Alpha"
Require (@((Structured $forward).edges | Where-Object { $_.relation -eq "Injects" -and $_.subject.name -eq "Alpha" -and $_.object.name -eq "Repository" }).Count -gt 0) "forward_edges omitted Alpha Injects Repository"
Require (@((Structured $reverse).edges | Where-Object { $_.relation -eq "Injects" -and $_.subject.name -eq "Alpha" -and $_.object.name -eq "Repository" }).Count -gt 0) "reverse_edges omitted Alpha Injects Repository"
Require (@((Structured $inFile).entities | Where-Object { $_.name -eq "Alpha" }).Count -gt 0) "entities_in_file omitted Alpha"
Require (@((Structured $transitive).dependencies | Where-Object { $_.Count -eq 3 -and $_[0] -eq "angular" -and $_[1] -eq "Service" -and $_[2] -eq "Repository" }).Count -gt 0) "transitive_dependencies omitted angular Service Repository"
Require ((Structured $cycle).has_cycle -eq $false) "acyclic fixture reported a cycle"
Require ((Structured $negative).count -eq 0) "negative reverse_edges was not empty"

$rows = @(
    @{ query="find_entities"; text=(Text $find); structured_count=(Structured $find).count; content_names_facts=((Text $find) -match 'Alpha') },
    @{ query="forward_edges"; text=(Text $forward); structured_count=(Structured $forward).count; content_names_facts=((Text $forward) -match 'Alpha|Repository|Injects') },
    @{ query="reverse_edges"; text=(Text $reverse); structured_count=(Structured $reverse).count; content_names_facts=((Text $reverse) -match 'Alpha|Repository|Injects') },
    @{ query="entities_in_file"; text=(Text $inFile); structured_count=(Structured $inFile).count; content_names_facts=((Text $inFile) -match 'Alpha') },
    @{ query="transitive_dependencies"; text=(Text $transitive); structured_count=(Structured $transitive).count; content_names_facts=((Text $transitive) -match 'Repository') },
    @{ query="has_cycle"; text=(Text $cycle); structured_count=$null; content_names_facts=$true },
    @{ query="negative_reverse_edges"; text=(Text $negative); structured_count=(Structured $negative).count; content_names_facts=$false }
)
$report = [ordered]@{
    structured_graph_facts_verified = $true
    content_contains_positive_semantic_records = @($rows | Where-Object { $_.query -ne "has_cycle" -and $_.content_names_facts }).Count -gt 0
    host_visibility_still_requires_trace = $true
    rows = $rows
}
$reportPath = Join-Path (Split-Path $captures) "visibility-report.json"
[IO.File]::WriteAllText($reportPath, ($report | ConvertTo-Json -Depth 20), [Text.UTF8Encoding]::new($false))
Write-Host "PASS: structured workspace-query facts match the controlled production graph."
Write-Host "Visibility report: $reportPath"
