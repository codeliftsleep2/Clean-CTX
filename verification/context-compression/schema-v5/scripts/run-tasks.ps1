param(
    [string]$Model = "",
    [string]$CodexPath = "",
    [ValidateSet("codex", "deepseek")][string]$ModelRunner = "codex"
)

$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$workingRoot = Join-Path $repositoryRoot "target\context-compression-verification\task-reasoning"
$tasks = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\tasks.json") | ConvertFrom-Json)

if ($ModelRunner -eq "codex") {
    if (-not $CodexPath) {
        $command = Get-Command codex -ErrorAction SilentlyContinue
        if ($command) { $CodexPath = $command.Source }
        else {
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
    if (-not $CodexPath -or -not (Test-Path $CodexPath)) { throw "Codex CLI was not found. Pass its full path with -CodexPath." }
}

function Invoke-Codex([string]$promptPath, [string]$outputPath) {
    $stderrPath = "$outputPath.stderr.log"
    $arguments = @(
        "exec", "-",
        "--ephemeral", "--ignore-user-config", "--ignore-rules",
        "--config", "mcp_servers={}", "--config", "plugins={}",
        "--skip-git-repo-check", "--sandbox", "read-only",
        "--cd", $workingRoot, "--color", "never",
        "--output-last-message", $outputPath
    )
    if ($Model) { $arguments += @("--model", $Model) }
    Get-Content -Raw $promptPath | & $CodexPath @arguments 2> $stderrPath
    if ($LASTEXITCODE -ne 0) { throw "Codex invocation failed for $promptPath" }
    if (-not (Test-Path $outputPath)) { throw "Codex did not write $outputPath" }
}

function Invoke-DeepSeek([string]$promptPath, [string]$outputPath) {
    $apiKey = $env:CLINE02_LOCAL_ENV_API_KEY
    if (-not $apiKey) { throw "CLINE02_LOCAL_ENV_API_KEY environment variable is not set" }
    $modelId = if ($Model) { $Model } else { "cline-pass/deepseek-v4-pro" }
    $prompt = Get-Content -Raw $promptPath
    $body = [ordered]@{ model = $modelId; messages = @(@{ role = "user"; content = $prompt }); stream = $false }
    $headers = @{ Authorization = "Bearer $apiKey" }
    $response = Invoke-RestMethod -Uri "https://api.cline.bot/api/v1/chat/completions" -Method Post -Headers $headers -ContentType "application/json" -Body ($body | ConvertTo-Json -Depth 10)
    $root = if ($null -ne $response.data) { $response.data } else { $response }
    [IO.File]::WriteAllText($outputPath, $root.choices[0].message.content, [Text.UTF8Encoding]::new($false))
}

function Invoke-Model([string]$promptPath, [string]$outputPath) {
    if ($ModelRunner -eq "deepseek") { Invoke-DeepSeek $promptPath $outputPath }
    else { Invoke-Codex $promptPath $outputPath }
}

New-Item -ItemType Directory -Force $workingRoot | Out-Null
$answers = foreach ($task in $tasks) {
    $payloadPath = Join-Path $captures "$($task.capture)\content.txt"
    if (-not (Test-Path -LiteralPath $payloadPath)) { throw "Missing capture: $payloadPath" }
    $payload = Get-Content -Raw -LiteralPath $payloadPath

    $prompt = @"
You are a code-editing assistant working from a Clean-CTX SCHEMA-v5 context
payload. Use only the supplied payload. $($task.instruction)

MODEL-VISIBLE PAYLOAD
$payload
"@
    $promptPath = Join-Path $workingRoot "$($task.id)-prompt.txt"
    $answerPath = Join-Path $workingRoot "$($task.id)-answer.txt"
    [IO.File]::WriteAllText($promptPath, $prompt, [Text.UTF8Encoding]::new($false))
    Write-Host "Evaluating $($task.id)"
    Invoke-Model $promptPath $answerPath

    $raw = (Get-Content -Raw -LiteralPath $answerPath).Trim()
    if ($raw.StartsWith('```')) { $raw = $raw -replace '^```[a-zA-Z]*\s*', ''; $raw = $raw -replace '\s*```\s*$', '' }
    $op = $null
    try { $op = $raw | ConvertFrom-Json -Depth 20 } catch { $op = $null }
    if ($null -eq $op) { $op = [ordered]@{ target = ""; expectedOldText = ""; newText = "" } }
    [ordered]@{ task_id = $task.id; answer = $op }
}

$output = Join-Path $captures "task-answers.json"
$answers | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote $output ($($answers.Count) task answers)"
