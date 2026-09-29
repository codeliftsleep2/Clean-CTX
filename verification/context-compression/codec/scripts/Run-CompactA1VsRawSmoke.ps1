param([string]$Model = "", [string]$CodexPath = "", [switch]$Restart)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$tokenRecordsPath = Join-Path $captures "compact-a2-token-records.json"
if (-not (Test-Path $tokenRecordsPath)) { throw "Run Measure-CompactA.ps1 first" }

$definitions = @(
    [ordered]@{
        id = "ownership-overloads"
        capture = "high-refactor"
        question = "Which declarations named run exist, which typed owner contains each, and which declarations form one overload family?"
        oracle = "Alpha owns two run overloads. Beta and InheritanceProbe each own a distinct run method. Runner is an interface that declares run. Do not merge equal display names across typed owners."
        zero = @("wrong owner", "overload collapse", "class/interface confusion")
    },
    [ordered]@{
        id = "calls-order-spread"
        capture = "high-refactor"
        question = "In Alpha's string run implementation, report the written calls in order, preserving repeated calls and spread evidence without inventing resolved targets."
        oracle = "The relevant written sequence contains lookup(value), lookup(value), external(...items), and repository.find(value), in that order. The two lookup occurrences remain distinct and external uses spread. Do not claim a canonical callee resolution that the payload does not support."
        zero = @("deduplicated call", "wrong order", "spread lost", "fabricated resolution")
    }
)

$tokenRecords = @(Get-Content -Raw $tokenRecordsPath | ConvertFrom-Json -Depth 30)
$rows = [Collections.Generic.List[object]]::new()
foreach ($definition in $definitions) {
    $candidate = Join-Path $captures "$($definition.capture)\compact-a2.txt"
    $raw = Join-Path $captures "$($definition.capture)\raw-source.txt"
    if (-not (Test-Path $candidate)) { throw "Missing A2 payload: $candidate" }
    if (-not (Test-Path $raw)) { throw "Missing capture-time raw payload: $raw" }
    $economical = @($tokenRecords | Where-Object {
        $_.capture -eq $definition.capture -and $_.economical_vs_raw -eq $true
    }).Count
    if (-not $economical) {
        throw "$($definition.capture): A2 did not clear the raw-source economics screen; model calls are not justified"
    }
    foreach ($representation in @("compact-a2", "raw-source")) {
        $rows.Add([ordered]@{
            case_id = "$($definition.id)--$representation"
            task_family = $definition.id
            fixture = "semantic-rich.ts"
            fidelity = "high"
            intent = "refactor"
            focus_mode = "ignored"
            production_operation = "paired raw/A2 smoke"
            capture = $definition.capture
            control_full_capture = if ($representation -eq "compact-a2") { $candidate } else { $raw }
            question = $definition.question
            exact_expected_oracle = $definition.oracle
            zero_tolerance = $definition.zero
            model = "codex"
            model_version = if ($Model) { $Model } else { "configured default" }
            sampling_settings = "fresh ephemeral invocation; paired representation"
            actual_model_answer = $null
            pass = $null
            failure_categories = @()
            notes = "PAIRED A2 VERSUS CAPTURE-TIME RAW SOURCE"
        })
    }
}

$resultsPath = Join-Path $captures "reasoning-results-compact-a2-vs-raw.json"
if ($Restart -or -not (Test-Path $resultsPath)) {
    $rows | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $resultsPath
}
$arguments = @{
    ResultsPath = $resultsPath
    ExpectedCaseCount = $rows.Count
    NoFailSummary = $true
    Restart = $Restart
    UsePreparedResults = $true
}
if ($Model) { $arguments.Model = $Model }
if ($CodexPath) { $arguments.CodexPath = $CodexPath }
& (Join-Path $PSScriptRoot "Run-CodexReasoning.ps1") @arguments
Write-Host "Paired A2/raw bounded smoke results: $resultsPath"
