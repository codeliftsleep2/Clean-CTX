param([string]$Model = "", [string]$CodexPath = "")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$baseline = Join-Path $captures "reasoning-results-codex-corrected.json"
$productionCapture = Join-Path $captures "high-refactor\control-full.txt"
if (-not (Test-Path $baseline)) { throw "Missing corrected baseline: $baseline" }
if (-not (Test-Path $productionCapture)) { throw "Missing production capture: $productionCapture" }

$payload = Get-Content -Raw $productionCapture
if ($payload -notmatch '^// CONTROL-FULL v2') {
    throw "High-refactor capture is stale (expected CONTROL-FULL v2). Rebuild target/debug/clean-ctx.exe, then rerun Capture-Baselines.ps1. Cargo test and Clippy do not refresh the runnable binary."
}
if ($payload -notmatch '"schema"\s*:\s*"clean-ctx/control-full-navigation"') {
    throw "High-refactor capture does not contain production navigation"
}

$caseIds = @(
    "dependency-injection--high-refactor",
    "behavior-facts--high-refactor"
)
$rows = @(Get-Content -Raw $baseline | ConvertFrom-Json -Depth 100)
foreach ($row in $rows | Where-Object { $caseIds -contains $_.case_id }) {
    $row.control_full_capture = $productionCapture
    $row.actual_model_answer = $null
    $row.pass = $null
    $row.failure_categories = @()
    $row.notes = "CONTROL-FULL v2 PRESENTATION REPAIR SMOKE"
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$resultsPath = Join-Path $captures "reasoning-results-control-full-v2-smoke-$stamp.json"
$rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
$arguments = @{ ResultsPath = $resultsPath; NoFailSummary = $true }
if ($Model) { $arguments.Model = $Model }
if ($CodexPath) { $arguments.CodexPath = $CodexPath }
& (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @arguments

Write-Host "CONTROL-FULL v2 smoke results: $resultsPath"
