param(
    [Parameter(Mandatory = $true)][string]$RunPath,
    [Parameter(Mandatory = $true)][string]$Model,
    [ValidateSet("none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra")]
    [string]$ReasoningEffort = "low",
    [string]$CodexPath,
    [switch]$ValidateOnly
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captureRoot = [IO.Path]::GetFullPath(
    (Join-Path $repositoryRoot "target\document-intelligence\control-runs")
).TrimEnd('\')
$resolvedRunPath = [IO.Path]::GetFullPath(
    $(if ([IO.Path]::IsPathRooted($RunPath)) { $RunPath } else { Join-Path $repositoryRoot $RunPath })
)
if (-not $resolvedRunPath.StartsWith("$captureRoot\", [StringComparison]::OrdinalIgnoreCase)) {
    throw "RunPath must remain under $captureRoot"
}
if (-not (Test-Path -LiteralPath $resolvedRunPath -PathType Leaf)) {
    throw "Prepared run not found: $resolvedRunPath"
}
if ([string]::IsNullOrWhiteSpace($Model)) {
    throw "Pass the exact Codex model ID with -Model"
}

if (-not $CodexPath) {
    $command = Get-Command codex -ErrorAction SilentlyContinue
    if ($command) { $CodexPath = $command.Source }
}
if (-not $CodexPath -or -not (Test-Path -LiteralPath $CodexPath -PathType Leaf)) {
    throw "Codex CLI was not found. Pass its full path with -CodexPath."
}

$benchmarkRoot = Join-Path $repositoryRoot "verification\document-intelligence"
$schemaPath = Join-Path $benchmarkRoot "full-document-control-capture.schema.json"
$schemaText = Get-Content -Raw -LiteralPath $schemaPath
$captureText = Get-Content -Raw -LiteralPath $resolvedRunPath
if (-not ($captureText | Test-Json -Schema $schemaText -ErrorAction Stop)) {
    throw "Prepared run does not satisfy its JSON Schema"
}
$capture = $captureText | ConvertFrom-Json -Depth 100
if ($capture.status -notin @("prepared", "in_progress")) {
    throw "Codex execution requires a prepared or in-progress run; found $($capture.status)"
}
if ($capture.protocol.transport -cne "concatenated") {
    throw "Codex execution requires concatenated transport"
}
if (-not [bool]$capture.protocol.oracleHiddenDuringCapture) {
    throw "Capture does not assert oracle isolation"
}
if ($capture.run.model -and [string]$capture.run.model -cne $Model) {
    throw "Prepared model '$($capture.run.model)' does not match requested model '$Model'"
}
if ($capture.run.parameters.reasoningEffort -and
    [string]$capture.run.parameters.reasoningEffort -cne $ReasoningEffort) {
    throw "Prepared reasoning effort '$($capture.run.parameters.reasoningEffort)' does not match '$ReasoningEffort'"
}

$versionOutput = @(& $CodexPath --version 2>&1)
if ($LASTEXITCODE -ne 0) { throw "Unable to read Codex CLI version" }
$clientVersion = [string]($versionOutput | Where-Object { $_ -match 'codex-cli' } | Select-Object -Last 1)
if ([string]::IsNullOrWhiteSpace($clientVersion)) { $clientVersion = "unknown" }

$runId = [string]$capture.run.id
if ($runId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
    throw "Run contains an unsafe ID: $runId"
}
$artifactRoot = Join-Path $captureRoot $runId
$isolationRoot = Join-Path ([IO.Path]::GetTempPath()) "clean-ctx-document-control\$runId"
if ($ValidateOnly) {
    $pending = @($capture.tasks | Where-Object status -eq "pending").Count
    Write-Host "Codex control validation passed: $resolvedRunPath"
    Write-Host "Model: $Model; reasoning: $ReasoningEffort; client: $clientVersion; pending tasks: $pending"
    exit 0
}

if (Test-Path -LiteralPath $isolationRoot) {
    throw "Isolation directory already exists; remove it or use a new run ID: $isolationRoot"
}
New-Item -ItemType Directory -Force -Path $artifactRoot, $isolationRoot | Out-Null

$utf8 = [Text.UTF8Encoding]::new($false)
function Get-TextSha256([string]$Text) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        return (([BitConverter]::ToString($algorithm.ComputeHash($utf8.GetBytes($Text)))) -replace '-', '').ToLowerInvariant()
    }
    finally { $algorithm.Dispose() }
}

function Save-Capture {
    $json = $capture | ConvertTo-Json -Depth 100
    if (-not ($json | Test-Json -Schema $schemaText -ErrorAction Stop)) {
        throw "Updated run does not satisfy its JSON Schema"
    }
    $temporaryPath = "$resolvedRunPath.tmp"
    [IO.File]::WriteAllText($temporaryPath, "$json`n", $utf8)
    Move-Item -LiteralPath $temporaryPath -Destination $resolvedRunPath -Force
}

function Invoke-CodexTask([object]$Task, [string]$TaskDirectory) {
    $answerPath = Join-Path $TaskDirectory "answer.txt"
    $tracePath = Join-Path $TaskDirectory "trace.jsonl"
    $stderrPath = Join-Path $TaskDirectory "stderr.log"
    $promptPath = Join-Path $TaskDirectory "prompt.txt"
    [IO.File]::WriteAllText($promptPath, [string]$Task.prompt.transportText, $utf8)

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $CodexPath
    $startInfo.WorkingDirectory = $isolationRoot
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in @(
        "exec", "-", "--json", "--ephemeral", "--ignore-user-config", "--ignore-rules",
        "--config", "mcp_servers={}", "--config", "plugins={}",
        "--config", "model_reasoning_effort=`"$ReasoningEffort`"",
        "--skip-git-repo-check", "--sandbox", "read-only", "--cd", $isolationRoot,
        "--color", "never", "--model", $Model, "--output-last-message", $answerPath
    )) {
        [void]$startInfo.ArgumentList.Add($argument)
    }

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    if (-not $process.Start()) { throw "Failed to start Codex for $($Task.task)" }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $process.StandardInput.Write([string]$Task.prompt.transportText)
    $process.StandardInput.Close()
    $process.WaitForExit()
    $stopwatch.Stop()
    $stdout = $stdoutTask.GetAwaiter().GetResult()
    $stderr = $stderrTask.GetAwaiter().GetResult()
    [IO.File]::WriteAllText($tracePath, $stdout, $utf8)
    [IO.File]::WriteAllText($stderrPath, $stderr, $utf8)

    $events = [Collections.Generic.List[object]]::new()
    foreach ($line in $stdout -split "`r?`n") {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        try { $events.Add(($line | ConvertFrom-Json -Depth 100)) }
        catch { throw "Codex emitted invalid JSONL for $($Task.task)" }
    }
    $nonAnswerItemEvents = @($events | Where-Object {
        $_.PSObject.Properties["item"] -and
        $_.item.type -notin @("reasoning", "agent_message")
    })
    $usageEvent = $events | Where-Object type -eq "turn.completed" | Select-Object -Last 1
    $answer = if (Test-Path -LiteralPath $answerPath) {
        Get-Content -Raw -LiteralPath $answerPath
    }
    else { $null }

    return [ordered]@{
        exitCode = $process.ExitCode
        latencyMs = [long]$stopwatch.ElapsedMilliseconds
        answer = $answer
        toolEvents = $nonAnswerItemEvents.Count
        inputTokens = if ($usageEvent) { $usageEvent.usage.input_tokens } else { $null }
        outputTokens = if ($usageEvent) { $usageEvent.usage.output_tokens } else { $null }
        totalTokens = if ($usageEvent) { $usageEvent.usage.total_tokens } else { $null }
        stderr = $stderr.Trim()
    }
}

$capture.status = "in_progress"
$capture.run.provider = "OpenAI"
$capture.run.model = $Model
$capture.run.reportedModelVersion = $Model
$capture.run.parameters.reasoningEffort = $ReasoningEffort
$capture.run.client = "Codex CLI"
$capture.run.clientVersion = $clientVersion
$capture.run.executor = "Run-FullDocumentControlCodex.ps1"
if ($null -eq $capture.run.startedAt) { $capture.run.startedAt = [DateTimeOffset]::UtcNow.ToString("o") }
Save-Capture

for ($index = 0; $index -lt $capture.tasks.Count; $index++) {
    $task = $capture.tasks[$index]
    if ($task.status -ne "pending") {
        Write-Host "[$($index + 1)/$($capture.tasks.Count)] $($task.task): already terminal"
        continue
    }
    if (-not $task.prompt.transportText -or
        (Get-TextSha256 ([string]$task.prompt.transportText)) -cne [string]$task.prompt.transportSha256) {
        throw "Transport prompt missing or hash-mismatched for $($task.task)"
    }

    $taskDirectory = Join-Path $artifactRoot ([string]$task.task)
    New-Item -ItemType Directory -Force -Path $taskDirectory | Out-Null
    Write-Host "[$($index + 1)/$($capture.tasks.Count)] $($task.task): running isolated Codex capture"
    $result = Invoke-CodexTask -Task $task -TaskDirectory $taskDirectory
    $task.output.answer = $result.answer
    $task.output.latencyMs = $result.latencyMs
    $task.output.finishReason = if ($result.exitCode -eq 0) { "process_exit_0" } else { "process_error" }
    $task.output.truncated = $false
    $task.output.usage.source = if ($null -ne $result.totalTokens) { "provider_reported" } else { "unavailable" }
    $task.output.usage.inputTokens = $result.inputTokens
    $task.output.usage.outputTokens = $result.outputTokens
    $task.output.usage.totalTokens = $result.totalTokens
    if ($result.exitCode -ne 0) {
        $task.status = "error"
        $task.output.error = "Codex exited $($result.exitCode): $($result.stderr)"
    }
    elseif ($result.toolEvents -ne 0) {
        $task.status = "error"
        $task.output.error = "Control invalidated because Codex emitted $($result.toolEvents) tool event(s)"
    }
    elseif ([string]::IsNullOrWhiteSpace([string]$result.answer)) {
        $task.status = "error"
        $task.output.error = "Codex returned no final answer"
    }
    else {
        $task.status = "completed"
        $task.output.error = $null
    }
    Save-Capture
}

$capture.summary.pending = @($capture.tasks | Where-Object status -eq "pending").Count
$capture.summary.completed = @($capture.tasks | Where-Object status -eq "completed").Count
$capture.summary.skipped = @($capture.tasks | Where-Object status -eq "skipped").Count
$capture.summary.error = @($capture.tasks | Where-Object status -eq "error").Count
if ($capture.summary.pending -eq 0) {
    $capture.status = "captured"
    $capture.run.completedAt = [DateTimeOffset]::UtcNow.ToString("o")
}
Save-Capture
Write-Host "Codex capture finished: completed=$($capture.summary.completed), errors=$($capture.summary.error), pending=$($capture.summary.pending)"
