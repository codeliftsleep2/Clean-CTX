param(
    [string]$AnswersPath = ""
)

# Deterministic grader for the edit-only task evaluation. No LLM judge, no
# binary invocation: it compares the model's apply_edit answer against the
# task's target + find/replace reference using string/substring checks.
#
# v1 grading is substring-based (proves the model targeted the right method,
# read the original marker, and produced the replacement). Byte-exact
# expectedOldText matching is the v2 refinement (apply_edit round-trip or a
# real TS parser), not yet implemented.

$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$tasks = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\tasks.json") | ConvertFrom-Json)
if (-not $AnswersPath) { $AnswersPath = Join-Path $captures "task-answers.json" }
if (-not (Test-Path -LiteralPath $AnswersPath)) { throw "Missing task answers: $AnswersPath" }
$answers = @(Get-Content -Raw -LiteralPath $AnswersPath | ConvertFrom-Json)

$results = foreach ($task in $tasks) {
    $match = @($answers | Where-Object { $_.task_id -eq $task.id })
    if ($match.Count -ne 1) {
        [ordered]@{ task_id = $task.id; pass = $false; checks = @("missing or ambiguous answer") }
        continue
    }
    $op = $match[0].answer
    $checks = [Collections.Generic.List[string]]::new()

    $targetOk = ([string]$op.target) -ceq $task.target
    if (-not $targetOk) { $checks.Add("target: got '$($op.target)' expected '$($task.target)'") }

    $old = [string]$op.expectedOldText
    $new = [string]$op.newText
    if (-not $old.Contains($task.find)) { $checks.Add("expectedOldText missing find '$($task.find)'") }
    if (-not $new.Contains($task.replace)) { $checks.Add("newText missing replace '$($task.replace)'") }
    if ($new.Contains($task.find)) { $checks.Add("newText still contains find '$($task.find)'") }

    [ordered]@{ task_id = $task.id; pass = ($checks.Count -eq 0); checks = @($checks) }
}

$output = Join-Path $captures "task-grades.json"
$results | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
$results | ConvertTo-Json -Depth 10
$pass = @($results | Where-Object pass).Count
Write-Host "PASS: $pass/$($tasks.Count) edit tasks"
