param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot,
    [string[]]$Probe
)
# User-owned operational discovery sessions. Never a frozen B7 A/B execution.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh discovery output directory.' }
$questionsPath = Join-Path $PSScriptRoot 'discovery-questions.csv'
$questions = @(Import-Csv -LiteralPath $questionsPath)
if ($Probe) {
    foreach ($id in $Probe) { if ($id -notin $questions.probe) { throw "Unknown probe: $id" } }
    $questions = @($questions | Where-Object probe -in $Probe)
}
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$preamble = @'
You are conducting a read-only engineering assessment in this repository.
Follow its normal engineering-agent instructions and choose your own discovery
strategy. Use actual available tools and source evidence, without editing files,
running tests/builds, launching runtimes through the shell or changing configuration.
Normal reconciliation performed by an authorized existing MCP tool is permitted.
Do not inspect evaluator packages, operator assessments, previous run outputs,
answer keys or scoring material. Those are not engineering sources for this task.

Answer the engineering question below. State the sources actually consulted,
applicable constraints, verification standing, uncertainty and your recommended
engineering disposition. Retain failed searches and distinguish direct matches
from related authority. Do not claim that source-reported tests ran in this session.
Do not seek operator coaching. This is an operational discovery probe, not an
independent decision-quality experiment or permission to implement a change.

Engineering question:
'@
$manifest = [ordered]@{
    purpose='ten operational discovery probes; not frozen B7 or paired evaluation'
    questions_sha256=(Get-FileHash -LiteralPath $questionsPath).Hash.ToLowerInvariant()
    selected_probes=@($questions.probe)
    session_policy='fresh ephemeral host per question; memory injection disabled; no inherited conversation supplied'
    isolation_limit='grader/operator filesystem denial is not established; no independent-evaluation eligibility claim'
    retry_policy='no silent retries or coaching; preserve incorrect answers and operational failures; stop batch on nonzero host exit'
    result_policy='exit 0 is session completion only; semantic results require transcript review'
}
[IO.File]::WriteAllText((Join-Path $output 'manifest.json'), ($manifest | ConvertTo-Json -Depth 6), $utf8)
foreach ($question in $questions) {
    $promptPath = Join-Path $output "$($question.probe)-prompt.txt"
    [IO.File]::WriteAllText($promptPath, ($preamble + "`n" + $question.question), $utf8)
    Write-Output "Starting $($question.probe); preserve its first result without coaching."
    & (Join-Path $PSScriptRoot 'invoke-host-smoke.ps1') -WorkspaceRoot $root `
        -OutputRoot (Join-Path $output $question.probe) `
        -ExpectedBinarySha256 '74e1be0de109610a35e06c1f369f4662aed0af48c74d44be941fa38a1ac4cd81' `
        -TaskPromptFile $promptPath -Purpose "B10 $($question.probe) operational discovery; not independent evaluation"
}
Write-Output "Discovery sessions finished: $output. Outcomes await transcript review."
