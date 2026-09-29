# Replicate the unchanged combined angular-method pair after one candidate-only
# fabricated (+1) suffix and one clean full-gate restart. Makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$templatePath = Join-Path $captures "schema-vnext-combined-reasoning-template.json"

if (-not (Test-Path -LiteralPath $templatePath)) {
    throw "Missing combined template. Run Prepare-CombinedReasoningWorksheet.ps1"
}

$template = @(Get-Content -Raw -LiteralPath $templatePath | ConvertFrom-Json -Depth 30)
$sourceRows = @($template | Where-Object {
    $_.case_id -in @(
        'combined-angular-method-baseline',
        'combined-angular-method-candidate'
    )
})
if ($sourceRows.Count -ne 2) {
    throw "Expected the unchanged baseline/candidate angular-method pair"
}

$rows = [Collections.Generic.List[object]]::new()
foreach ($replicate in 1..3) {
    foreach ($source in $sourceRows) {
        $variant = if ($source.case_id.EndsWith('-baseline')) {
            'baseline'
        } else {
            'candidate'
        }
        $row = [ordered]@{}
        foreach ($property in $source.PSObject.Properties) {
            $row[$property.Name] = $property.Value
        }
        $row.case_id = "combined-angular-method-r$replicate-$variant"
        $row.capture = "$($source.capture)-replicate-$replicate"
        $row.actual_model_answer = $null
        $row.pass = $null
        $row.failure_categories = @()
        $row.notes = $null
        $rows.Add($row)
    }
}

if ($rows.Count -ne 6 -or ($rows.case_id | Sort-Object -Unique).Count -ne 6) {
    throw "Replication worksheet must contain six unique cases"
}

$output = Join-Path $captures "schema-vnext-combined-method-replication-template.json"
$rows | ConvertTo-Json -Depth 30 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-combined-method-replication-template.json (6 cases; zero model calls)"
