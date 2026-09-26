# Prepare paired baseline/A1 reasoning cases for the single-path experiment.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$recordsPath = Join-Path $captures "schema-vnext-a1-records.json"

if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-a1-records.json. Run Measure-A1SinglePath.ps1"
}

function Get-PathMapping([string]$capture, [string]$variant, [string]$payloadPath) {
    if (-not (Test-Path -LiteralPath $payloadPath)) {
        throw "${capture}/${variant}: missing payload $payloadPath"
    }
    $payload = [IO.File]::ReadAllText($payloadPath, [Text.Encoding]::UTF8)
    $matches = [regex]::Matches(
        $payload,
        '(?m)^  (?<alias>α\d+) = (?<path>[^\r\n]+)(?=\r?$)'
    )
    if ($matches.Count -ne 1) {
        throw "${capture}/${variant}: expected one PATHMAP entry, found $($matches.Count)"
    }
    return [pscustomobject]@{
        alias = $matches[0].Groups['alias'].Value
        path = $matches[0].Groups['path'].Value
    }
}

$records = @(Get-Content -Raw -LiteralPath $recordsPath | ConvertFrom-Json -Depth 20)
$lowRows = @($records |
    Where-Object { $_.tokenizer -eq 'o200k' -and $_.fidelity -eq 'low' -and $_.focus_mode -eq 'none' } |
    Sort-Object language)
if ($lowRows.Count -ne 3) {
    throw "Expected one Low A1 record for each of three languages, found $($lowRows.Count)"
}

$rows = @()
foreach ($record in $lowRows) {
    $capture = [string]$record.capture
    $language = [string]$record.language
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-a1-single-path.txt"
    $baselineMapping = Get-PathMapping $capture "baseline" $baselinePath
    $candidateMapping = Get-PathMapping $capture "candidate" $candidatePath
    if ($baselineMapping.alias -cne $candidateMapping.alias -or
        $baselineMapping.path -cne $candidateMapping.path) {
        throw "${capture}: A1 changed the authoritative alias-to-path mapping"
    }

    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $baselinePath },
        [pscustomobject]@{ name = 'candidate'; payload = $candidatePath }
    )) {
        $rows += [ordered]@{
            case_id = "a1-pathmap-$($variant.name)-$language"
            task_family = "a1-pathmap"
            lane = "file"
            fixture = $capture
            fidelity = "low"
            intent = "overview"
            focus_mode = "none"
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "a1-candidate" }
            capture = "$capture-$($variant.name)"
            control_full_capture = $variant.payload
            question = "Which exact source path does the final file alias $($candidateMapping.alias) identify, and which section is authoritative for that mapping?"
            exact_expected_oracle = "$($candidateMapping.alias) identifies the exact path '$($candidateMapping.path)'. The trailing §PATHMAP section is authoritative."
            zero_tolerance = @(
                "wrong or missing exact path",
                "path inferred from the alias instead of read from §PATHMAP"
            )
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

if ($rows.Count -ne 6) { throw "Expected six paired A1 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 6) {
    throw "A1 reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-a1-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-a1-reasoning-template.json (6 paired cases; zero model calls)"
