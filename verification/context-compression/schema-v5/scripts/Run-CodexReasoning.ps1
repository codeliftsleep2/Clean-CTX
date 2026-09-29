param(
    [string]$Model = "",
    [string]$CodexPath = "",
    [string]$TemplatePath = "",
    [string]$ResultsPath = "",
    [int]$ExpectedCaseCount = 30,
    [ValidateSet("codex", "deepseek")][string]$ModelRunner = "codex",
    [switch]$UsePreparedResults,
    [switch]$NoFailSummary,
    [switch]$Restart
)

$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
if (-not $ResultsPath) { $ResultsPath = Join-Path $captures "schema-v5-reasoning-results-codex.json" }
$resultsPath = $ResultsPath
$resultLane = [IO.Path]::GetFileNameWithoutExtension($ResultsPath)
if (-not $resultLane) { $resultLane = "schema-v5-reasoning-results-codex" }
$workingRoot = Join-Path $repositoryRoot "target\context-compression-verification\codex-reasoning\$resultLane"
$scoreSchema = Join-Path $definitionRoot "score.schema.json"
$reasoningInstructions = Get-Content -Raw (Join-Path $definitionRoot "schema-v5\REASONING_INSTRUCTIONS.md")
if (-not $TemplatePath) {
    $TemplatePath = Join-Path $captures "schema-v5-reasoning-results.json"
}

if ($ModelRunner -eq "codex") {
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
}
if (-not (Test-Path $templatePath)) { & (Join-Path $PSScriptRoot "Prepare-ReasoningWorksheet.ps1") }
if (-not (Test-Path $templatePath)) { throw "Reasoning worksheet was not created" }

if ($Restart -and (Test-Path $workingRoot)) {
    $expectedRoot = [IO.Path]::GetFullPath(
        (Join-Path $repositoryRoot "target\context-compression-verification\codex-reasoning")
    )
    $resolvedWorkingRoot = [IO.Path]::GetFullPath($workingRoot)
    if (-not $resolvedWorkingRoot.StartsWith("$expectedRoot\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to clear reasoning state outside $expectedRoot"
    }
    Remove-Item -LiteralPath $resolvedWorkingRoot -Recurse -Force
}
New-Item -ItemType Directory -Force $workingRoot | Out-Null
if (($Restart -or -not (Test-Path $resultsPath)) -and -not $UsePreparedResults) {
    $template = @(Get-Content -Raw $templatePath | ConvertFrom-Json -Depth 100)
    foreach ($row in $template) {
        $row.model = $ModelRunner
        $row.model_version = if ($Model) { $Model } else { "configured default" }
        $row.sampling_settings = "fresh ephemeral invocation; read-only isolated working directory"
    }
    $template | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}
if (-not (Test-Path $resultsPath)) { throw "Prepared results do not exist: $resultsPath" }

$rows = @(Get-Content -Raw $resultsPath | ConvertFrom-Json -Depth 100)
if ($rows.Count -ne $ExpectedCaseCount) {
    throw "Expected $ExpectedCaseCount Codex cases, found $($rows.Count)"
}

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

# Routes the selected model through Cline's OpenAI-compatible API, billed to the
# Cline account (ClinePass / usage-billing). Uses the Cline API key.
function Invoke-DeepSeek([string]$promptPath, [string]$outputPath, [string]$schemaPath = "") {
    $apiKey = $env:CLINE02_LOCAL_ENV_API_KEY
    if (-not $apiKey) { throw "CLINE02_LOCAL_ENV_API_KEY environment variable is not set" }
    $modelId = if ($Model) { $Model } else { "cline-pass/deepseek-v4-pro" }
    $prompt = Get-Content -Raw $promptPath
    $messages = @(@{ role = "user"; content = $prompt })
    $body = [ordered]@{ model = $modelId; messages = $messages; stream = $false }
    # No response_format here: the reasoning model returns markdown-fenced JSON,
    # which is stripped by the score parser instead of relying on structured output.
    $json = $body | ConvertTo-Json -Depth 10
    $headers = @{ Authorization = "Bearer $apiKey" }
    $response = Invoke-RestMethod -Uri "https://api.cline.bot/api/v1/chat/completions" -Method Post -Headers $headers -ContentType "application/json" -Body $json
    $root = if ($null -ne $response.data) { $response.data } else { $response }
    $content = $root.choices[0].message.content
    [IO.File]::WriteAllText($outputPath, $content, [Text.UTF8Encoding]::new($false))
    if (-not (Test-Path $outputPath)) { throw "Cline API did not write $outputPath" }
}

function Invoke-Model([string]$promptPath, [string]$outputPath, [string]$schemaPath = "") {
    if ($ModelRunner -eq "deepseek") {
        Invoke-DeepSeek $promptPath $outputPath $schemaPath
    } else {
        Invoke-Codex $promptPath $outputPath $schemaPath
    }
}

for ($index = 0; $index -lt $rows.Count; $index++) {
    $row = $rows[$index]
    if ($null -ne $row.pass -and -not [string]::IsNullOrWhiteSpace($row.actual_model_answer)) {
        Write-Host "[$($index + 1)/$ExpectedCaseCount] $($row.case_id): already recorded"
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
$reasoningInstructions

MODEL-VISIBLE PAYLOAD
$payload

QUESTION
$($row.question)
"@
    [IO.File]::WriteAllText($answerPrompt, $answerText, [Text.UTF8Encoding]::new($false))
    if (Test-Path $answerPath) {
        Write-Host "[$($index + 1)/$ExpectedCaseCount] $($row.case_id): reusing completed answer"
    } else {
        Write-Host "[$($index + 1)/$ExpectedCaseCount] $($row.case_id): evaluating"
        Invoke-Model $answerPrompt $answerPath
    }
    $answer = Get-Content -Raw $answerPath

    $scoreText = @"
Score the MODEL ANSWER strictly against the EXPECTED ORACLE and ZERO-TOLERANCE
conditions. Do not inspect files or invoke tools. Return ONLY a single JSON object
with exactly these three fields and nothing else:

  "pass": <boolean>
  "failure_categories": <array of strings>
  "notes": <string>

Pass only when the answer is semantically correct and violates no zero-tolerance
condition. Select failure categories only from the output schema. Do NOT add a
numeric "score" field or any other key. Do NOT wrap the JSON in markdown fences.

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
    Write-Host "[$($index + 1)/$ExpectedCaseCount] $($row.case_id): scoring"
    $score = $null
    for ($attempt = 1; $attempt -le 3 -and $null -eq $score; $attempt++) {
        if ($attempt -eq 1) {
            Invoke-Model $scorePrompt $scorePath $scoreSchema
        } else {
            $retryPrompt = $scoreText + "`n`nYOUR PREVIOUS RESPONSE WAS NOT VALID JSON. Return ONLY the JSON object with exactly the three fields, no markdown fences, no commentary, no numeric score."
            [IO.File]::WriteAllText($scorePrompt, $retryPrompt, [Text.UTF8Encoding]::new($false))
            Invoke-Model $scorePrompt $scorePath $scoreSchema
        }
        $scoreRaw = (Get-Content -Raw $scorePath).Trim()
        if ($scoreRaw.StartsWith('```')) {
            $scoreRaw = $scoreRaw -replace '^```[a-zA-Z]*\s*', ''
            $scoreRaw = $scoreRaw -replace '\s*```\s*$', ''
        }
        try {
            $score = $scoreRaw | ConvertFrom-Json -Depth 20
        } catch {
            $score = $null
        }
    }
    if ($null -eq $score) {
        $row.actual_model_answer = $answer
        $row.pass = $false
        $row.failure_categories = @("other semantic mismatch")
        $row.notes = "score JSON parse failed after retries (harness)"
        $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
        continue
    }

    $row.actual_model_answer = $answer
    $row.pass = [bool]$score.pass
    $row.failure_categories = @($score.failure_categories)
    $row.notes = [string]$score.notes
    $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}

$evaluator = ([IO.Path]::GetFileNameWithoutExtension($resultsPath) -replace '^schema-v5-reasoning-results-', '')
& (Join-Path $PSScriptRoot "Summarize-ReasoningResults.ps1") `
    -ResultsPath $resultsPath `
    -Evaluator $evaluator `
    -ExpectedCaseCount $ExpectedCaseCount `
    -NoFail:$NoFailSummary
