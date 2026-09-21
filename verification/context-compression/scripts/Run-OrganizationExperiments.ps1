param([string]$Model = "", [string]$CodexPath = "")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$variantRoot = Join-Path $repositoryRoot "target\context-compression-verification\organization-experiments"
$baseline = Join-Path $captures "reasoning-results-codex-corrected.json"
$manifest = Get-Content -Raw (Join-Path $definitionRoot "investigation\organization-experiments-v1.json") | ConvertFrom-Json -Depth 100
if (-not (Test-Path $baseline)) { throw "Missing corrected baseline: $baseline" }

& (Join-Path $PSScriptRoot "Prepare-OrganizationExperiments.ps1")
& (Join-Path $PSScriptRoot "Validate-ControlFullExperiments.ps1")

foreach ($experiment in $manifest.experiments) {
    for ($trial = 1; $trial -le $manifest.trials_per_variant; $trial++) {
        $trialPath = Join-Path $captures "reasoning-results-$($experiment.id)-trial-$trial.json"
        if (-not (Test-Path $trialPath)) {
            $rows = @(Get-Content -Raw $baseline | ConvertFrom-Json -Depth 100)
            $row = @($rows | Where-Object case_id -eq $experiment.case_id)
            if ($row.Count -ne 1) { throw "Case not found exactly once: $($experiment.case_id)" }
            $row[0].control_full_capture = Join-Path $variantRoot "$($experiment.id).txt"
            $row[0].actual_model_answer = $null
            $row[0].pass = $null
            $row[0].failure_categories = @()
            $row[0].notes = "ORGANIZATION EXPERIMENT $($experiment.id), TRIAL $trial"
            $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $trialPath
        }
        $args = @{ ResultsPath = $trialPath; NoFailSummary = $true }
        if ($Model) { $args.Model = $Model }
        if ($CodexPath) { $args.CodexPath = $CodexPath }
        & (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @args
    }
}
