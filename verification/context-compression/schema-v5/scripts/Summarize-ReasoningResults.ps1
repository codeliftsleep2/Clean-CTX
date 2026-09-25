param(
    [string]$ResultsPath = "",
    [string]$Evaluator = "unspecified",
    [int]$ExpectedCaseCount = 30,
    [switch]$NoFail
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
if (-not $ResultsPath) { $ResultsPath = Join-Path $captures "schema-v5-reasoning-results.json" }
if (-not (Test-Path $ResultsPath)) { throw "Missing reasoning worksheet: $ResultsPath" }

$rows = @(Get-Content -Raw $ResultsPath | ConvertFrom-Json -Depth 100)
if ($rows.Count -ne $ExpectedCaseCount) {
    throw "Expected $ExpectedCaseCount cases, found $($rows.Count)"
}
$unrun = @($rows | Where-Object { $null -eq $_.pass -or [string]::IsNullOrWhiteSpace($_.actual_model_answer) })
$failed = @($rows | Where-Object { $_.pass -eq $false })
$passed = @($rows | Where-Object { $_.pass -eq $true })
$critical = @($failed | Where-Object { $_.failure_categories.Count -gt 0 })

$summary = [ordered]@{
    evaluator = $Evaluator
    results_path = (Resolve-Path $ResultsPath).Path
    required = $ExpectedCaseCount
    passed = $passed.Count
    failed = $failed.Count
    unrun = $unrun.Count
    gate_pass = ($passed.Count -eq $ExpectedCaseCount -and $failed.Count -eq 0 -and $unrun.Count -eq 0)
    failures = @($failed | ForEach-Object { [ordered]@{ case_id = $_.case_id; categories = @($_.failure_categories); notes = $_.notes } })
    unrun_case_ids = @($unrun | ForEach-Object { $_.case_id })
}
$suffix = if ($Evaluator -eq "unspecified") { "" } else { "-$Evaluator" }
$output = Join-Path $captures "schema-v5-reasoning-summary$suffix.json"
$summary | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
$summary | ConvertTo-Json -Depth 20
if (-not $summary.gate_pass -and -not $NoFail) { exit 1 }
