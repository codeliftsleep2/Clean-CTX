param(
    [string]$Model = "",
    [string]$CodexPath = "",
    [string]$ResultsPath = "",
    [switch]$NoFailSummary,
    [switch]$Restart
)

$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
if (-not $ResultsPath) { $ResultsPath = Join-Path $captures "reasoning-results-codex.json" }
$resultsPath = $ResultsPath
$resultLane = [IO.Path]::GetFileNameWithoutExtension($ResultsPath)
if (-not $resultLane) { $resultLane = "reasoning-results-codex" }
$workingRoot = Join-Path $repositoryRoot "target\context-compression-verification\codex-reasoning\$resultLane"
$scoreSchema = Join-Path $definitionRoot "reasoning\score.schema.json"
$templatePath = Join-Path $captures "reasoning-results.json"

if (-not $CodexPath) {
    $command = Get-Command codex -ErrorAction SilentlyContinue
    if ($command) {
        $CodexPath = $command.Source
    } else {
        $extensionRoot = Join-Path $env:USERPROFILE ".vscode\extensions"
        $candidates = @(
            Get-ChildItem $extensionRoot -Directory -Filter "openai.chatgpt-*-win32-x64" -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                ForEach-Object { Join-Path $_.FullName "bin\windows-x86_64\codex.exe" } |
                Where-Object { Test-Path $_ }
        )
        if ($candidates.Count) { $CodexPath = $candidates[0] }
    }
}
if (-not $CodexPath -or -not (Test-Path $CodexPath)) {
    throw "Codex CLI was not found. Pass its full path with -CodexPath."
}
if (-not (Test-Path $templatePath)) { & (Join-Path $PSScriptRoot "Prepare-ReasoningWorksheet.ps1") }
if (-not (Test-Path $templatePath)) { throw "Reasoning worksheet was not created" }

New-Item -ItemType Directory -Force $workingRoot | Out-Null
if ($Restart -or -not (Test-Path $resultsPath)) {
    $template = @(Get-Content -Raw $templatePath | ConvertFrom-Json -Depth 100)
    foreach ($row in $template) {
        $row.model = "codex"
        $row.model_version = if ($Model) { $Model } else { "configured default" }
        $row.sampling_settings = "fresh ephemeral invocation; read-only isolated working directory"
    }
    $template | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}

$rows = @(Get-Content -Raw $resultsPath | ConvertFrom-Json -Depth 100)
if ($rows.Count -ne 36) { throw "Expected 36 Codex cases, found $($rows.Count)" }

function Invoke-Codex([string]$promptPath, [string]$outputPath, [string]$schemaPath = "") {
    $stderrPath = "$outputPath.stderr.log"
    $arguments = @(
        "exec", "-",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--config", "mcp_servers={}",
        "--config", "plugins={}",
        "--skip-git-repo-check",
        "--sandbox", "read-only",
        "--cd", $workingRoot,
        "--color", "never",
        "--output-last-message", $outputPath
    )
    if ($Model) { $arguments += @("--model", $Model) }
    if ($schemaPath) { $arguments += @("--output-schema", $schemaPath) }
    Get-Content -Raw $promptPath | & $CodexPath @arguments 2> $stderrPath
    $exitCode = $LASTEXITCODE
    $stderrLines = if (Test-Path $stderrPath) { @(Get-Content $stderrPath) } else { @() }
    $transportNoise = 'rmcp::transport::worker: worker quit with fatal: Transport channel closed, when Client\(HttpRequest\(HttpRequest\("error decoding response body"\)\)\)'
    $meaningfulStderr = @($stderrLines | Where-Object { $_ -notmatch $transportNoise })
    if ($meaningfulStderr.Count) { $meaningfulStderr | Write-Host }
    if ($exitCode -ne 0) {
        if ($stderrLines.Count) { $stderrLines | Write-Error }
        throw "Codex invocation failed for $promptPath"
    }
    if (-not (Test-Path $outputPath)) { throw "Codex did not write $outputPath" }
}

for ($index = 0; $index -lt $rows.Count; $index++) {
    $row = $rows[$index]
    if ($null -ne $row.pass -and -not [string]::IsNullOrWhiteSpace($row.actual_model_answer)) {
        Write-Host "[$($index + 1)/36] $($row.case_id): already recorded"
        continue
    }
    if (-not (Test-Path $row.control_full_capture)) { throw "Missing capture: $($row.control_full_capture)" }

    $safeId = $row.case_id -replace '[^A-Za-z0-9_.-]', '_'
    $caseDir = Join-Path $workingRoot $safeId
    New-Item -ItemType Directory -Force $caseDir | Out-Null
    $answerPrompt = Join-Path $caseDir "answer-prompt.txt"
    $answerPath = Join-Path $caseDir "answer.txt"
    $scorePrompt = Join-Path $caseDir "score-prompt.txt"
    $scorePath = Join-Path $caseDir "score.json"
    $payload = Get-Content -Raw $row.control_full_capture

    $answerText = @"
You are evaluating a semantic context representation. Use only the supplied
MODEL-VISIBLE PAYLOAD. Do not inspect files, invoke tools, or use repository
state. Answer the QUESTION directly. Preserve canonical IDs, uncertainty,
duplicates, and ordering exactly as supported by the payload. Filter records by
any requested canonical ID before collecting rows. Treat occurrence indices and
nested arrays as semantic. Report every explicitly requested field. Written
unresolved callees are not resolved declarations. Distinguish file IDs, entity
IDs, and absent files. Do not infer lifecycle facts not stated in the payload.

MODEL-VISIBLE PAYLOAD
$payload

QUESTION
$($row.question)
"@
    [IO.File]::WriteAllText($answerPrompt, $answerText, [Text.UTF8Encoding]::new($false))
    if (Test-Path $answerPath) {
        Write-Host "[$($index + 1)/36] $($row.case_id): reusing completed answer"
    } else {
        Write-Host "[$($index + 1)/36] $($row.case_id): evaluating"
        Invoke-Codex $answerPrompt $answerPath
    }
    $answer = Get-Content -Raw $answerPath

    $scoreText = @"
Score the MODEL ANSWER strictly against the EXPECTED ORACLE and ZERO-TOLERANCE
conditions. Do not inspect files or invoke tools. Return only the required JSON.
Pass only when the answer is semantically correct and violates no zero-tolerance
condition. Select failure categories only from the output schema.

QUESTION
$($row.question)

EXPECTED ORACLE
$($row.exact_expected_oracle)

ZERO-TOLERANCE CONDITIONS
$($row.zero_tolerance | ConvertTo-Json -Compress)

MODEL ANSWER
$answer
"@
    [IO.File]::WriteAllText($scorePrompt, $scoreText, [Text.UTF8Encoding]::new($false))
    Write-Host "[$($index + 1)/36] $($row.case_id): scoring"
    Invoke-Codex $scorePrompt $scorePath $scoreSchema
    $score = Get-Content -Raw $scorePath | ConvertFrom-Json -Depth 20

    $row.actual_model_answer = $answer
    $row.pass = [bool]$score.pass
    $row.failure_categories = @($score.failure_categories)
    $row.notes = [string]$score.notes
    $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}

$evaluator = ([IO.Path]::GetFileNameWithoutExtension($resultsPath) -replace '^reasoning-results-', '')
& (Join-Path $PSScriptRoot "Summarize-ReasoningResults.ps1") -ResultsPath $resultsPath -Evaluator $evaluator -NoFail:$NoFailSummary
