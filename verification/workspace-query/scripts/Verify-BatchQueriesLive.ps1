param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path,
    [switch]$KeepWorkspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")

$outputRoot = Join-Path $RepositoryRoot "target\workspace-query-batch-verification"
$captures = Join-Path $outputRoot "captures"
$workspace = Join-Path ([IO.Path]::GetTempPath()) ("clean-ctx-batch-live-" + [guid]::NewGuid())
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Missing production binary. Run: cargo build --all-features --bin clean-ctx"
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path

$changeMarkers = @(
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\query.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\query\batch.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\query\content.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_schemas.rs")
)
$binaryTime = (Get-Item -LiteralPath $BinaryPath).LastWriteTimeUtc
$newerMarker = $changeMarkers | Where-Object {
    (Get-Item -LiteralPath $_).LastWriteTimeUtc -gt $binaryTime
} | Select-Object -First 1
if ($null -ne $newerMarker) {
    throw "Production binary is stale relative to $newerMarker. Run: cargo build --all-features --bin clean-ctx"
}

New-Item -ItemType Directory -Force -Path $workspace, $captures | Out-Null
[IO.File]::WriteAllText(
    (Join-Path $workspace "graph.ts"),
    @'
import { Injectable } from "@angular/core";

@Injectable()
export class Repository {}

@Injectable()
export class Alpha {
  constructor(private repository: Repository) {}
}
'@,
    [Text.UTF8Encoding]::new($false)
)
$config = @{
    additional_roots = @()
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText(
    (Join-Path $workspace ".clean-ctx.json"),
    $config,
    [Text.UTF8Encoding]::new($false)
)

function Save-Capture {
    param([string]$Name, $Value)
    [IO.File]::WriteAllText(
        (Join-Path $captures "$Name.json"),
        ($Value | ConvertTo-Json -Depth 100),
        [Text.UTF8Encoding]::new($false)
    )
}

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Assert-NoRpcError {
    param($Response, [string]$Scenario)
    if ($null -ne $Response.PSObject.Properties["error"]) {
        throw "$Scenario returned JSON-RPC error: $($Response.error | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Structured-Results {
    param($Response, [string]$Scenario)
    Assert-NoRpcError $Response $Scenario
    Assert-True ($null -ne $Response.result.structuredContent) "$Scenario omitted structuredContent."
    return @($Response.result.structuredContent.results)
}

function Semantic-Json {
    param($Value)
    $copy = $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -AsHashtable
    if ($copy.ContainsKey("discovery")) { $copy.Remove("discovery") }
    return ($copy | ConvertTo-Json -Depth 100 -Compress)
}

function Assert-NoRepeatedDiscoveryWork {
    param($Item)
    $discoveryProperty = $Item.result.PSObject.Properties["discovery"]
    if ($null -eq $discoveryProperty) { return }
    $discovery = $discoveryProperty.Value
    Assert-True ($null -eq $discovery.PSObject.Properties["discovered"]) "Repeated batch reported newly discovered candidates."
    Assert-True ($null -eq $discovery.PSObject.Properties["compiled"]) "Repeated batch reported newly compiled candidates."
}

$shared = @{ workspaceRoot = $workspace }
$items = @(
    @{ id = "find-alpha"; type = "find_entities"; name = "Alpha" },
    @{
        id = "forward-alpha"; type = "forward_edges"; name = "Alpha"
        domain = "angular"; entity_type = "Service"
    },
    @{
        id = "reverse-repository"; type = "reverse_edges"; name = "Repository"
        domain = "angular"; entity_type = "Service"
    },
    @{ id = "invalid"; type = "find_entities" }
)
$batchArguments = @{ workspaceRoot = $workspace; queries = $items }
$session = $null

try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace

    $tools = Invoke-CleanCtxRpc $session 1 "tools/list" @{}
    Assert-NoRpcError $tools "tools/list"
    $tool = @($tools.result.tools | Where-Object { $_.name -eq "workspace_query" })
    Assert-True ($tool.Count -eq 1) "tools/list did not expose exactly one workspace_query tool."
    $batchBranch = @($tool[0].inputSchema.oneOf | Where-Object {
        @($_.required) -contains "queries"
    })
    Assert-True ($batchBranch.Count -eq 1) "workspace_query inputSchema omitted the batch request branch."
    Assert-True ($tool[0].outputSchema.properties.results.type -eq "array") "workspace_query outputSchema omitted ordered batch results."
    $nameSchema = $tool[0].inputSchema.properties.name
    Assert-True ($nameSchema.type -eq "string") "workspace_query name schema is not singular string input."
    Assert-True ($nameSchema.minLength -eq 1) "workspace_query name schema permits an empty string."
    Assert-True ([string]$nameSchema.description -match "top-level queries") "workspace_query name schema does not direct multi-name callers to queries."
    Write-Host "PASS: live tools/list exposes the batch input and output contract."

    $first = Invoke-CleanCtxTool $session 2 "workspace_query" $batchArguments
    Save-Capture "batch-first" $first
    $firstResults = Structured-Results $first "first batch"
    Assert-True ($firstResults.Count -eq 4) "First batch did not return exactly four outcomes."
    Assert-True (($firstResults.id -join ",") -eq "find-alpha,forward-alpha,reverse-repository,invalid") "First batch did not preserve input order and IDs."
    Assert-True (@($firstResults[0..2] | Where-Object { $_.status -ne "ok" }).Count -eq 0) "A valid batch item failed."
    Assert-True ($firstResults[0].result.count -ge 1) "find_entities returned no Alpha occurrence."
    Assert-True ($firstResults[1].result.count -ge 1) "forward_edges returned no Alpha dependency."
    Assert-True ($firstResults[2].result.count -ge 1) "reverse_edges returned no Repository consumer."
    Assert-True ($firstResults[3].status -eq "error") "Invalid item did not produce an isolated error."
    Assert-True ($firstResults[3].error.code -eq -32602) "Invalid item used the wrong error classification."
    Assert-True ($null -eq $firstResults[3].PSObject.Properties["result"]) "Invalid item fabricated a semantic result."
    $content = [string]$first.result.content[0].text
    Assert-True $content.StartsWith("// WORKSPACE-QUERY-BATCH v1; structuredContent remains authoritative") "Batch content used the wrong envelope."
    Write-Host "PASS: heterogeneous results are ordered and one invalid item is isolated."

    $second = Invoke-CleanCtxTool $session 3 "workspace_query" $batchArguments
    Save-Capture "batch-repeat" $second
    $secondResults = Structured-Results $second "repeated batch"
    foreach ($item in $secondResults[0..2]) { Assert-NoRepeatedDiscoveryWork $item }
    for ($index = 0; $index -lt 3; $index += 1) {
        Assert-True ((Semantic-Json $firstResults[$index].result) -eq (Semantic-Json $secondResults[$index].result)) "Repeated batch changed semantic result at index $index."
    }
    Write-Host "PASS: repeated execution reuses discovery completion without changing semantics."

    $singleArguments = @(
        @{ type = "find_entities"; name = "Alpha"; workspaceRoot = $workspace },
        @{
            type = "forward_edges"; name = "Alpha"; workspaceRoot = $workspace
            domain = "angular"; entity_type = "Service"
        },
        @{
            type = "reverse_edges"; name = "Repository"; workspaceRoot = $workspace
            domain = "angular"; entity_type = "Service"
        }
    )
    for ($index = 0; $index -lt $singleArguments.Count; $index += 1) {
        $single = Invoke-CleanCtxTool $session (4 + $index) "workspace_query" $singleArguments[$index]
        Save-Capture "single-$index" $single
        Assert-NoRpcError $single "single query $index"
        $singleResult = $single.result.structuredContent
        Assert-True ((Semantic-Json $firstResults[$index].result) -eq (Semantic-Json $singleResult)) "Batch item $index differs from its equivalent single query."
    }
    Write-Host "PASS: successful batch items match equivalent legacy single queries."

    $malformed = Invoke-CleanCtxTool $session 7 "workspace_query" @{
        type = "reverse_edges"
        name = @("MethodA", "MethodB")
        workspaceRoot = $workspace
    }
    Save-Capture "invalid-array-name" $malformed
    Assert-True ($null -ne $malformed.PSObject.Properties["error"]) "Array-valued name was not rejected."
    Assert-True ($malformed.error.code -eq -32602) "Array-valued name used the wrong error classification."
    Assert-True ([string]$malformed.error.message -match "non-empty string") "Array-valued name was not reported as a type error."
    Assert-True ([string]$malformed.error.message -match "queries") "Array-valued name error omitted the supported batch form."
    Write-Host "PASS: array-valued name is rejected and points callers to queries."

    $report = [ordered]@{
        verified_at_utc = [DateTime]::UtcNow.ToString("o")
        binary = $BinaryPath
        workspace = $workspace
        ordered_ids = @($firstResults.id)
        repeated_discovery_reused = $true
        single_query_semantics_matched = $true
        invalid_array_name_rejected = $true
        isolated_error_code = $firstResults[3].error.code
    }
    Save-Capture "verification-report" $report
    Write-Host "PASS: live MCP workspace-query batch verification completed."
    Write-Host "Captures: $captures"
}
finally {
    Stop-CleanCtxSession $session
    if ($KeepWorkspace) {
        Write-Host "Retained live verification workspace: $workspace"
    }
    elseif (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
}
