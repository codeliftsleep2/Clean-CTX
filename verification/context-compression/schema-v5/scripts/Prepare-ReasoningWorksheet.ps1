$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
New-Item -ItemType Directory -Force $captures | Out-Null
$oracles = Get-Content -Raw (Join-Path $definitionRoot "schema-v5\oracles.json") | ConvertFrom-Json
$scenarios = Get-Content -Raw (Join-Path $definitionRoot "expected\scenarios.json") | ConvertFrom-Json

function Get-Scenario([string]$capture) {
    $scenarioId = if ($capture -like "delta-flow-*") { "delta-flow" } else { $capture }
    $scenario = @($scenarios | Where-Object id -eq $scenarioId)
    if ($scenario.Count -ne 1) { throw "Capture '$capture' maps to $($scenario.Count) scenarios" }
    return $scenario[0]
}

$rows = foreach ($oracle in $oracles) {
    if ($oracle.lane -eq "workspace") {
        [ordered]@{
            case_id = "$($oracle.id)--workspace-query"
            task_family = $oracle.id
            lane = "workspace"
            fixture = "cross-file"
            fidelity = ""
            intent = $null
            focus_mode = "ignored"
            production_operation = "workspace_query"
            capture = "workspace-query"
            control_full_capture = Join-Path $captures "workspace-query\$($oracle.id).txt"
            question = $oracle.question
            exact_expected_oracle = $oracle.expected
            zero_tolerance = @($oracle.zeroTolerance)
            model = $null
            model_version = $null
            sampling_settings = $null
            actual_model_answer = $null
            pass = $null
            failure_categories = @()
            notes = $null
        }
    } else {
        # File and lifecycle lanes feed the SCHEMA-v5 presentation captured as
        # content.txt (the actual content[0].text, preserved for every scenario).
        foreach ($capture in $oracle.captures) {
            $payloadFile = "content.txt"
            $scenario = Get-Scenario $capture
            [ordered]@{
                case_id = "$($oracle.id)--$capture"
                task_family = $oracle.id
                lane = $oracle.lane
                fixture = $scenario.fixture
                fidelity = $scenario.fidelity
                intent = if ($scenario.intent) { $scenario.intent } else { $null }
                focus_mode = $scenario.focusMode
                production_operation = $scenario.operation
                capture = $capture
                control_full_capture = Join-Path $captures "$capture\$payloadFile"
                question = $oracle.question
                exact_expected_oracle = $oracle.expected
                zero_tolerance = @($oracle.zeroTolerance)
                model = $null
                model_version = $null
                sampling_settings = $null
                actual_model_answer = $null
                pass = $null
                failure_categories = @()
                notes = $null
            }
        }
    }
}

if ($rows.Count -ne 30) { throw "Expected exactly 30 schema-v5 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 30) { throw "Schema-v5 reasoning case IDs are not unique" }

$output = Join-Path $captures "schema-v5-reasoning-results.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote $output ($($rows.Count) executable cases; no aggregate score)"
