$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$oracles = @(Get-Content -Raw (Join-Path $definitionRoot "reasoning\oracles.json") | ConvertFrom-Json -Depth 100)
$transport = @(Get-Content -Raw (Join-Path $definitionRoot "reasoning\transport-assertions.json") | ConvertFrom-Json -Depth 100)
$scenarios = @(Get-Content -Raw (Join-Path $definitionRoot "expected\scenarios.json") | ConvertFrom-Json -Depth 100)
$failures = [Collections.Generic.List[string]]::new()

function Fail([string]$message) { $failures.Add($message) }
function Scenario-For([string]$capture) {
    $id = if ($capture -like "delta-flow-*") { "delta-flow" } else { $capture }
    return @($scenarios | Where-Object id -eq $id)
}

$caseIds = [Collections.Generic.List[string]]::new()
foreach ($oracle in $oracles) {
    if ([string]::IsNullOrWhiteSpace($oracle.id)) { Fail "oracle has no id" }
    if ([string]::IsNullOrWhiteSpace($oracle.question)) { Fail "$($oracle.id): missing question" }
    if ([string]::IsNullOrWhiteSpace($oracle.expected)) { Fail "$($oracle.id): missing exact expected oracle" }
    foreach ($capture in @($oracle.captures)) {
        $caseId = "$($oracle.id)--$capture"
        $caseIds.Add($caseId)
        $scenario = Scenario-For $capture
        if ($scenario.Count -ne 1) { Fail "${caseId}: expected one production scenario, found $($scenario.Count)"; continue }
        if ($scenario[0].expect -ne "success") { Fail "${caseId}: reasoning capture is not a successful production scenario" }
        if ($scenario[0].fidelity -eq "verbatim") { Fail "${caseId}: verbatim capture used as semantic CONTROL-FULL" }
        $fixture = Join-Path $definitionRoot "fixtures\$($scenario[0].fixture)"
        $generatedFixture = $scenario[0].fixture -eq "semantic-rich-crlf.ts"
        if (-not $generatedFixture -and -not (Test-Path $fixture)) { Fail "${caseId}: missing tracked fixture $fixture" }
    }
}

if ($caseIds.Count -ne 36) { Fail "expected exactly 36 executable reasoning cases, found $($caseIds.Count)" }
$duplicates = @($caseIds | Group-Object | Where-Object Count -gt 1)
foreach ($duplicate in $duplicates) { Fail "duplicate case id: $($duplicate.Name)" }

$expectedTransport = @("edit-focus-ambiguous", "edit-focus-invalid")
if ($transport.Count -ne 2) { Fail "expected exactly two transport assertions, found $($transport.Count)" }
foreach ($id in $expectedTransport) {
    $entry = @($transport | Where-Object id -eq $id)
    $scenario = @($scenarios | Where-Object id -eq $id)
    if ($entry.Count -ne 1) { Fail "${id}: missing or duplicate transport assertion" }
    if ($scenario.Count -ne 1 -or $scenario[0].expect -ne "error") { Fail "${id}: missing registered error scenario" }
    if (@($caseIds | Where-Object { $_ -like "*--$id" }).Count -ne 0) { Fail "${id}: transport error leaked into reasoning manifest" }
}

if ($failures.Count) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}
Write-Host "PASS: 36 unique successful reasoning cases; two separate focus-error transport assertions."
