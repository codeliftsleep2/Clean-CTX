param([string]$Model = "", [string]$CodexPath = "", [switch]$Restart)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$templatePath = Join-Path $captures "reasoning-results.json"
if (-not (Test-Path $templatePath)) { throw "Missing reasoning worksheet: $templatePath" }

$caseIds = @(
    "overloads--high-refactor",
    "behavior-facts--high-refactor",
    "unresolved-honesty--high-refactor",
    "body-byte-fidelity--edit-all-crlf"
)
$rows = @(
    Get-Content -Raw $templatePath | ConvertFrom-Json -Depth 100 |
        Where-Object { $caseIds -contains $_.case_id }
)
if ($rows.Count -ne $caseIds.Count) {
    throw "Expected $($caseIds.Count) bounded smoke cases, found $($rows.Count)"
}
foreach ($row in $rows) {
    $candidate = Join-Path $captures "$($row.capture)\compact-a2.txt"
    if (-not (Test-Path $candidate)) { throw "Missing A2 capture: $candidate" }
    if (-not (Get-Content -Raw $candidate).StartsWith("// COMPACT-A A2")) {
        throw "Stale or invalid A2 capture: $candidate"
    }
    $row.control_full_capture = $candidate
    $row.actual_model_answer = $null
    $row.pass = $null
    $row.failure_categories = @()
    $row.notes = "BOUNDED COMPACT-A2 FILE-LOCAL HIGH-RISK SMOKE"
    $row.exact_expected_oracle = switch ($row.case_id) {
        "overloads--high-refactor" {
            "Alpha owns two run methods: M11 and M13. High fidelity carries no exact bodies, so mode.exact_body_method_ids and the body section are empty."
        }
        "dependency-injection--high-refactor" {
            "The framework edge is angular Service Alpha Injects angular Service Repository in the angular layer. Its subject and object files independently equal the semantic-rich.ts source path. Core injection_occurrences for C6 are empty; do not invent duplicates or assign C3 to Alpha."
        }
        "behavior-facts--high-refactor" {
            "Report emitted non-empty families only: M8 PRIVATE and CTOR(C6,M8); M16 ASYNC, control_flow await/await, side_effect async, execution_context async; M18 data_flow reads/observable; M20 PRIVATE. Preserve IDs and nested arrays. Do not claim unreported families are absent."
        }
        "framework-edge-provenance--high-refactor" {
            "Subject angular Service Alpha, relation Injects, object angular Service Repository, layer angular. Subject.file and object.file independently contain the same semantic-rich.ts source path."
        }
        "body-byte-fidelity--edit-all-crlf" {
            "The exact editable IDs are M4, M8, M11, M13, M15, M16, M18, M20, M23, and M26. Their body text, CRLF/UTF-8 bytes, body_start, and body_end are authoritative and must not be normalized or reconstructed."
        }
        default { $row.exact_expected_oracle }
    }
}

$resultsPath = Join-Path $captures "reasoning-results-compact-a2-smoke.json"
if ($Restart -or -not (Test-Path $resultsPath)) {
    $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}
$arguments = @{
    ResultsPath = $resultsPath
    ExpectedCaseCount = $caseIds.Count
    NoFailSummary = $true
    Restart = $Restart
    UsePreparedResults = $true
}
if ($Model) { $arguments.Model = $Model }
if ($CodexPath) { $arguments.CodexPath = $CodexPath }
& (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @arguments

Write-Host "COMPACT-A2 bounded file-local smoke results: $resultsPath"
