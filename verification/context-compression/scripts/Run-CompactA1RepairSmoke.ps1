param(
    [string]$Model = "",
    [string]$CodexPath = "",
    [switch]$Restart,
    [switch]$DiOnly
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$templatePath = Join-Path $captures "reasoning-results.json"
if (-not (Test-Path $templatePath)) { throw "Missing reasoning worksheet: $templatePath" }

$oracles = [ordered]@{
    "dependency-injection--high-refactor" = "The framework edge is angular Service Alpha Injects angular Service Repository in the angular layer. Its subject and object files independently equal the semantic-rich.ts source path. Core injection_occurrences for C6 are empty; do not invent duplicates or assign C3 to Alpha."
    "behavior-facts--high-refactor" = "Report emitted non-empty families only: M8 PRIVATE and CTOR(C6,M8); M16 ASYNC, control_flow await/await, side_effect async, execution_context async; M18 data_flow reads/observable; M20 PRIVATE. Preserve IDs and nested arrays. Do not claim unreported families are absent."
    "framework-edge-provenance--high-refactor" = "Subject angular Service Alpha, relation Injects, object angular Service Repository, layer angular. Subject.file and object.file independently contain the same semantic-rich.ts source path."
}
$selectedIds = if ($DiOnly) { @("dependency-injection--high-refactor") } else { @($oracles.Keys) }
$rows = @(
    Get-Content -Raw $templatePath | ConvertFrom-Json -Depth 100 |
        Where-Object { $selectedIds -contains $_.case_id }
)
if ($rows.Count -ne $selectedIds.Count) {
    throw "Expected $($selectedIds.Count) repair cases, found $($rows.Count)"
}
foreach ($row in $rows) {
    $candidate = Join-Path $captures "$($row.capture)\compact-a1.txt"
    if (-not (Test-Path $candidate)) { throw "Missing A1 capture: $candidate" }
    $row.control_full_capture = $candidate
    $row.exact_expected_oracle = $oracles[$row.case_id]
    $row.actual_model_answer = $null
    $row.pass = $null
    $row.failure_categories = @()
    $row.notes = "COMPACT-A1 LOCALITY REPAIR SMOKE"
}

$lane = if ($DiOnly) { "compact-a1-di-repair-smoke" } else { "compact-a1-repair-smoke" }
$resultsPath = Join-Path $captures "reasoning-results-$lane.json"
if ($Restart -or -not (Test-Path $resultsPath)) {
    $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}
$arguments = @{
    ResultsPath = $resultsPath
    ExpectedCaseCount = $selectedIds.Count
    NoFailSummary = $true
    Restart = $Restart
    UsePreparedResults = $true
}
if ($Model) { $arguments.Model = $Model }
if ($CodexPath) { $arguments.CodexPath = $CodexPath }
& (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @arguments

Write-Host "COMPACT-A1 repair smoke results: $resultsPath"
