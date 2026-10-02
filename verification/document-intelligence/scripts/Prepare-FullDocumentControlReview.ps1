param(
    [Parameter(Mandatory = $true)][string]$RunPath,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$benchmarkRoot = Join-Path $repositoryRoot "verification\document-intelligence"

function Resolve-InputPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
    return [IO.Path]::GetFullPath((Join-Path $repositoryRoot $Path))
}

function Add-QuotedText([Text.StringBuilder]$Builder, [string]$Text) {
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line.Length -eq 0) { [void]$Builder.AppendLine(">") }
        else { [void]$Builder.AppendLine("> $line") }
    }
}

$resolvedRunPath = Resolve-InputPath $RunPath
if (-not (Test-Path -LiteralPath $resolvedRunPath -PathType Leaf)) {
    throw "Control run not found: $resolvedRunPath"
}
$schemaText = Get-Content -Raw -LiteralPath (Join-Path $benchmarkRoot "full-document-control-capture.schema.json")
$runText = Get-Content -Raw -LiteralPath $resolvedRunPath
if (-not ($runText | Test-Json -Schema $schemaText -ErrorAction Stop)) {
    throw "Control run does not satisfy its JSON Schema"
}
$run = $runText | ConvertFrom-Json -Depth 100
if ($run.status -notin @("captured", "complete")) {
    throw "Review requires a captured or complete run; found $($run.status)"
}

$manifest = Get-Content -Raw -LiteralPath (Join-Path $benchmarkRoot "benchmark-manifest.json") |
    ConvertFrom-Json -Depth 100
$oracles = Get-Content -Raw -LiteralPath (Join-Path $benchmarkRoot "full-document-control-oracles.json") |
    ConvertFrom-Json -Depth 100
if ([int]$run.manifestVersion -ne [int]$manifest.version) {
    throw "Run and manifest versions do not match"
}
if ([int]$run.oracleVersion -ne [int]$oracles.version) {
    throw "Run and oracle versions do not match"
}

$tasksById = @{}
foreach ($task in $manifest.tasks) {
    if ($tasksById.ContainsKey([string]$task.id)) { throw "Duplicate manifest task: $($task.id)" }
    $tasksById[[string]$task.id] = $task
}
$oraclesById = @{}
foreach ($oracle in $oracles.oracles) {
    if ($oraclesById.ContainsKey([string]$oracle.task)) { throw "Duplicate oracle task: $($oracle.task)" }
    $oraclesById[[string]$oracle.task] = $oracle
}

$runId = [string]$run.run.id
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $repositoryRoot "target\document-intelligence\control-runs\$runId-review.md"
}
else {
    $OutputPath = Resolve-InputPath $OutputPath
}
if (Test-Path -LiteralPath $OutputPath) {
    throw "Review worksheet already exists: $OutputPath"
}

$builder = [Text.StringBuilder]::new()
[void]$builder.AppendLine("# Full-Document Control Human Review")
[void]$builder.AppendLine()
[void]$builder.AppendLine("- Run: ``$runId``")
[void]$builder.AppendLine("- Repository revision: ``$($run.run.repositoryRevision)``")
[void]$builder.AppendLine("- Model: ``$($run.run.model)``")
[void]$builder.AppendLine("- Reasoning effort: ``$($run.run.parameters.reasoningEffort)``")
[void]$builder.AppendLine("- Capture status: ``$($run.status)``")
[void]$builder.AppendLine("- Completed tasks: $($run.summary.completed)/$($run.summary.total)")
[void]$builder.AppendLine()
[void]$builder.AppendLine("The reviewer assigns verdicts after comparing each answer with the oracle and explicit requirements. This worksheet does not grade answers automatically.")

foreach ($capture in $run.tasks) {
    $taskId = [string]$capture.task
    if (-not $tasksById.ContainsKey($taskId)) { throw "Run contains unknown task: $taskId" }
    if (-not $oraclesById.ContainsKey($taskId)) { throw "No oracle for task: $taskId" }
    $task = $tasksById[$taskId]
    $oracle = $oraclesById[$taskId]

    $requirements = [ordered]@{}
    foreach ($name in @(
        "required_evidence",
        "required_claims",
        "required_structural_facts",
        "required_diagnostics",
        "required_heading_path",
        "required_behavior",
        "expected",
        "semantic_preservation"
    )) {
        $property = $task.PSObject.Properties[$name]
        if ($null -ne $property) { $requirements[$name] = $property.Value }
    }

    [void]$builder.AppendLine()
    [void]$builder.AppendLine("## $taskId")
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("- Operation: ``$($task.operation)``")
    [void]$builder.AppendLine("- Capture status: ``$($capture.status)``")
    [void]$builder.AppendLine("- Retry count: $($capture.output.retryCount)")
    [void]$builder.AppendLine("- Latency: $($capture.output.latencyMs) ms")
    [void]$builder.AppendLine("- Usage: input=$($capture.output.usage.inputTokens), output=$($capture.output.usage.outputTokens), total=$($capture.output.usage.totalTokens)")
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("### Question")
    [void]$builder.AppendLine()
    Add-QuotedText $builder ([string]$task.question)
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("### Captured answer")
    [void]$builder.AppendLine()
    Add-QuotedText $builder ([string]$capture.output.answer)
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("### Human-reviewed oracle")
    [void]$builder.AppendLine()
    Add-QuotedText $builder ([string]$oracle.answer)
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("### Explicit requirements")
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("````json")
    [void]$builder.AppendLine(($requirements | ConvertTo-Json -Depth 100))
    [void]$builder.AppendLine("````")
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("### Human verdict")
    [void]$builder.AppendLine()
    [void]$builder.AppendLine("- Verdict: [ ] pass  [ ] partial  [ ] fail  [ ] ambiguous")
    [void]$builder.AppendLine("- Reviewer:")
    [void]$builder.AppendLine("- Reviewed at:")
    [void]$builder.AppendLine("- Oracle comparison:")
    [void]$builder.AppendLine("- Notes:")
}

$outputDirectory = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
[IO.File]::WriteAllText($OutputPath, $builder.ToString(), [Text.UTF8Encoding]::new($false))
Write-Host "Prepared human-review worksheet: $OutputPath"
Write-Host "Tasks: $($run.tasks.Count); verdicts remain unassigned"
