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

# File and lifecycle lanes must contain the actual model-visible content. That
# is normally SCHEMA-v5, but the production economics boundary may explicitly
# select byte-exact raw source when the complete candidate is not smaller.
foreach ($oracle in $oracles | Where-Object { $_.lane -ne "workspace" }) {
    foreach ($capture in @($oracle.captures)) {
        $path = Join-Path $captures "$capture\content.txt"
        Require (Test-Path -LiteralPath $path) "${capture}: missing content.txt"
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $text = Get-Content -Raw $path
        if ($capture -eq "delta-flow-delta") {
            Require ($text.StartsWith([char]0x0394 + " delta for")) "delta-flow-delta: content.txt is not the delta summary"
            continue
        }
        if ($text.StartsWith("// SCHEMA v5")) { continue }

        $responsePath = Join-Path $captures "$capture\response.json"
        Require (Test-Path -LiteralPath $responsePath) "${capture}: non-SCHEMA content has no response metadata"
        if (-not (Test-Path -LiteralPath $responsePath)) { continue }
        $response = Get-Content -Raw -LiteralPath $responsePath | ConvertFrom-Json -Depth 100
        $contentKind = if ($null -ne $response.result._meta.content_kind) {
            [string]$response.result._meta.content_kind
        } else {
            [string]$response.result.content_kind
        }
        Require ($contentKind -eq "raw_passthrough") "${capture}: non-SCHEMA content is not declared raw_passthrough"
        if ($contentKind -ne "raw_passthrough") { continue }

        $rawPath = Join-Path $captures "$capture\raw-source.txt"
        Require (Test-Path -LiteralPath $rawPath) "${capture}: raw_passthrough has no raw-source.txt"
        if (Test-Path -LiteralPath $rawPath) {
            $contentBytes = [Convert]::ToBase64String([IO.File]::ReadAllBytes($path))
            $rawBytes = [Convert]::ToBase64String([IO.File]::ReadAllBytes($rawPath))
            Require ($contentBytes -ceq $rawBytes) "${capture}: raw_passthrough is not byte-exact"
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
