$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$original = Join-Path $captures "reasoning-results-codex.json"
$corrected = Join-Path $captures "reasoning-results-codex-corrected.json"
$repairs = @(Get-Content -Raw (Join-Path $definitionRoot "reasoning\repairs-v2.json") | ConvertFrom-Json -Depth 100)
if (-not (Test-Path $original)) { throw "Missing preserved original run: $original" }
$rows = @(Get-Content -Raw $original | ConvertFrom-Json -Depth 100)
if ($rows.Count -ne 36) { throw "Original run must contain 36 cases" }
foreach ($repair in $repairs) {
    $row = @($rows | Where-Object case_id -eq $repair.case_id)
    if ($row.Count -ne 1) { throw "Repair case not found exactly once: $($repair.case_id)" }
    $row[0].question = $repair.question
    $row[0].exact_expected_oracle = $repair.expected
    $row[0].actual_model_answer = $null
    $row[0].pass = $null
    $row[0].failure_categories = @()
    $row[0].notes = "RERUN REQUIRED: $($repair.defect)"
}
$rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $corrected
Write-Host "Prepared corrected lineage with $($repairs.Count) rerun cases: $corrected"
