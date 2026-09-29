$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$resultsPath = Join-Path $repositoryRoot "target\context-compression-verification\captures\schema-v5-reasoning-results.json"
if (-not (Test-Path $resultsPath)) { throw "Missing reasoning worksheet. Run Prepare-ReasoningWorksheet.ps1 first." }

@(Get-Content -Raw $resultsPath | ConvertFrom-Json -Depth 100) |
    Select-Object case_id, fixture, fidelity, intent, focus_mode, production_operation, pass |
    Format-Table -AutoSize
