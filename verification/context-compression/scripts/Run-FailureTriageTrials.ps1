param([string]$Model = "", [string]$CodexPath = "")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$baseline = Join-Path $captures "reasoning-results-codex-corrected.json"
$matrix = Get-Content -Raw (Join-Path $definitionRoot "investigation\rerun-matrix-v1.json") | ConvertFrom-Json -Depth 20
if (-not (Test-Path $baseline)) { throw "Missing corrected baseline: $baseline" }
$caseIds = @($matrix.category_a_cases) + @($matrix.category_b_cases)

for ($trial = 1; $trial -le $matrix.trials_per_case; $trial++) {
    $trialPath = Join-Path $captures "reasoning-results-codex-triage-v2-$trial.json"
    if (-not (Test-Path $trialPath)) {
        $rows = @(Get-Content -Raw $baseline | ConvertFrom-Json -Depth 100)
        foreach ($row in $rows | Where-Object { $caseIds -contains $_.case_id }) {
            $row.actual_model_answer = $null
            $row.pass = $null
            $row.failure_categories = @()
            $row.notes = "TRIAGE TRIAL $trial"
        }
        $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $trialPath
    }
    $args = @{ ResultsPath = $trialPath; NoFailSummary = $true }
    if ($Model) { $args.Model = $Model }
    if ($CodexPath) { $args.CodexPath = $CodexPath }
    & (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @args
    Write-Host "Completed triage trial $trial"
}
