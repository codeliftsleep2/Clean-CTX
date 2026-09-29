param(
    [Parameter(Mandatory = $true)][string]$CaseId,
    [Parameter(Mandatory = $true)][string]$AnswerPath,
    [Parameter(Mandatory = $true)][ValidateSet("pass", "fail")][string]$Result,
    [Parameter(Mandatory = $true)][string]$Model,
    [Parameter(Mandatory = $true)][string]$ModelVersion,
    [Parameter(Mandatory = $true)][string]$SamplingSettings,
    [ValidateSet("identity error", "ownership error", "fabricated resolved call target", "missing/incorrect call occurrence", "arity/spread error", "DI error", "provenance error", "duplicate/order loss", "unsupported claim", "wrong source-escalation decision", "wrong edit target", "source/body corruption", "other semantic mismatch")]
    [string[]]$FailureCategory = @(),
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$resultsPath = Join-Path $repositoryRoot "target\context-compression-verification\captures\reasoning-results.json"
if (-not (Test-Path $resultsPath)) { throw "Missing reasoning worksheet. Run Prepare-ReasoningWorksheet.ps1 first." }
if (-not (Test-Path $AnswerPath)) { throw "Missing answer file: $AnswerPath" }
if ($Result -eq "pass" -and $FailureCategory.Count -ne 0) { throw "Passing cases cannot have failure categories" }
if ($Result -eq "fail" -and $FailureCategory.Count -eq 0) { throw "Failing cases require at least one failure category" }

$rows = @(Get-Content -Raw $resultsPath | ConvertFrom-Json -Depth 100)
$matches = @(0..($rows.Count - 1) | Where-Object { $rows[$_].case_id -eq $CaseId })
if ($matches.Count -ne 1) { throw "Expected one case '$CaseId', found $($matches.Count)" }
$row = $rows[$matches[0]]
$row.actual_model_answer = Get-Content -Raw $AnswerPath
$row.pass = ($Result -eq "pass")
$row.failure_categories = @($FailureCategory)
$row.model = $Model
$row.model_version = $ModelVersion
$row.sampling_settings = $SamplingSettings
$row.notes = $Notes
$rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
Write-Host "Recorded $Result for $CaseId"
