$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$oracles = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\oracles.json") | ConvertFrom-Json)
$workspaceOps = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\workspace-query-oracles.json") | ConvertFrom-Json)
$failures = [Collections.Generic.List[string]]::new()

if (-not (Test-Path -LiteralPath $captures)) {
    throw "No captures directory exists. Run Capture-Baselines.ps1 and Capture-WorkspaceQuery.ps1 before verification."
}

function Require([bool]$condition, [string]$message) { if (-not $condition) { $failures.Add($message) } }

# File and lifecycle lanes must be the rendered SCHEMA-v5 presentation captured
# as content.txt (the actual content[0].text), not the codec CONTROL-FULL JSON
# and not raw source.
foreach ($oracle in $oracles | Where-Object { $_.lane -ne "workspace" }) {
    foreach ($capture in @($oracle.captures)) {
        $path = Join-Path $captures "$capture\content.txt"
        Require (Test-Path -LiteralPath $path) "${capture}: missing content.txt"
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $text = Get-Content -Raw $path
        if ($capture -eq "delta-flow-delta") {
            Require ($text.StartsWith([char]0x0394 + " delta for")) "delta-flow-delta: content.txt is not the delta summary"
        } else {
            Require ($text.StartsWith("// SCHEMA v5")) "${capture}: content.txt is not a rendered SCHEMA-v5 presentation"
        }
    }
}

# Workspace lane: each op must have a captured model-visible response.
foreach ($op in $workspaceOps) {
    $path = Join-Path $captures "workspace-query\$($op.id).txt"
    Require (Test-Path -LiteralPath $path) "workspace-query/$($op.id): missing capture"
}

$result = [ordered]@{ pass = ($failures.Count -eq 0); failure_count = $failures.Count; failures = @($failures) }
$result | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM (Join-Path $captures "schema-v5-verification-result.json")
if ($failures.Count) { $failures | ForEach-Object { Write-Host "FAIL: $_" -ForegroundColor Red }; exit 1 }
Write-Host "PASS: SCHEMA-v5 presentation and workspace-query captures verified (operator evidence, not CI proof)."
