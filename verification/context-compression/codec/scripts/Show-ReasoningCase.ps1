param(
    [Parameter(Mandatory = $true)][string]$CaseId
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$resultsPath = Join-Path $repositoryRoot "target\context-compression-verification\captures\reasoning-results.json"
if (-not (Test-Path $resultsPath)) { throw "Missing reasoning worksheet. Run Prepare-ReasoningWorksheet.ps1 first." }

$rows = @(Get-Content -Raw $resultsPath | ConvertFrom-Json -Depth 100)
$row = @($rows | Where-Object case_id -eq $CaseId)
if ($row.Count -ne 1) { throw "Expected one case '$CaseId', found $($row.Count)" }
if (-not (Test-Path $row[0].control_full_capture)) { throw "Missing CONTROL-FULL capture: $($row[0].control_full_capture)" }

# Deliberately emit only the registered model-visible payload and scored question.
Get-Content -Raw $row[0].control_full_capture
Write-Output ""
Write-Output $row[0].question
