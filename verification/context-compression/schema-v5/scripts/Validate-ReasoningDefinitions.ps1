$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$oracles = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\oracles.json") | ConvertFrom-Json -Depth 100)
$workspaceOps = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\workspace-query-oracles.json") | ConvertFrom-Json -Depth 100)
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
    if ($oracle.lane -eq "workspace") {
        # Workspace lane expands to one workspace_query case per oracle, not per
        # capture; the captures field documents the related scenarios only.
        $caseIds.Add("$($oracle.id)--workspace-query")
    } else {
        foreach ($capture in @($oracle.captures)) {
            $caseId = "$($oracle.id)--$capture"
            $caseIds.Add($caseId)
            $scenario = Scenario-For $capture
            if ($scenario.Count -ne 1) { Fail "${caseId}: expected one production scenario, found $($scenario.Count)"; continue }
            if ($scenario[0].expect -ne "success") { Fail "${caseId}: reasoning capture is not a successful production scenario" }
        }
    }
}

if ($caseIds.Count -ne 30) { Fail "expected exactly 30 executable schema-v5 reasoning cases, found $($caseIds.Count)" }
$duplicates = @($caseIds | Group-Object | Where-Object Count -gt 1)
foreach ($duplicate in $duplicates) { Fail "duplicate case id: $($duplicate.Name)" }

$workspaceOracleIds = @($oracles | Where-Object lane -eq "workspace" | ForEach-Object { $_.id } | Sort-Object)
$workspaceOpIds = @($workspaceOps | ForEach-Object { $_.id } | Sort-Object)
if (($workspaceOracleIds -join ",") -ne ($workspaceOpIds -join ",")) {
    Fail "workspace-lane oracle ids must pair 1:1 with workspace-query op ids"
}

if ($failures.Count) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}
Write-Host "PASS: 30 unique successful schema-v5 reasoning cases; 6 workspace-query ops paired."
