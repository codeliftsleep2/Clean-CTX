param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path,
    [switch]$KeepWorkspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")

$outputRoot = Join-Path $RepositoryRoot "target\provide-context-batch-verification"
$captures = Join-Path $outputRoot "captures"
$workspace = Join-Path ([IO.Path]::GetTempPath()) ("clean-ctx-context-batch-live-" + [guid]::NewGuid())
$fixtures = Join-Path $RepositoryRoot "verification\context-batch\fixtures"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Missing production binary. Run: cargo build --all-features --bin clean-ctx"
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path

$changeMarkers = @(
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\core\provide.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\core\provide\batch.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_handlers\core\provide\evaluate.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tool_schemas.rs"),
    (Join-Path $RepositoryRoot "src\mcp\tools.rs")
)
$binaryTime = (Get-Item -LiteralPath $BinaryPath).LastWriteTimeUtc
$newerMarker = $changeMarkers | Where-Object {
    (Get-Item -LiteralPath $_).LastWriteTimeUtc -gt $binaryTime
} | Select-Object -First 1
if ($null -ne $newerMarker) {
    throw "Production binary is stale relative to $newerMarker. Run: cargo build --all-features --bin clean-ctx"
}

New-Item -ItemType Directory -Force -Path $workspace, $captures | Out-Null
Copy-Item -LiteralPath (Join-Path $fixtures "overview.ts") -Destination $workspace
Copy-Item -LiteralPath (Join-Path $fixtures "target.ts") -Destination $workspace
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

function Cache-Hits {
    param($Session, [int]$Id)
    $response = Invoke-CleanCtxTool $Session $Id "context_stats" @{ format = "json" }
    Assert-NoRpcError $response "context_stats"
    $dashboard = ([string]$response.result.content[0].text | ConvertFrom-Json -Depth 100)
    return [int]$dashboard.cache.hits
}

$batchArguments = @{
    workspaceRoot = $workspace
    tokenizer = "o200k"
    files = @(
        @{ id = "overview"; filePath = "overview.ts"; intent = "overview" },
        @{ id = "missing"; filePath = "missing.ts"; intent = "debug" },
        @{
            id = "target"; filePath = "target.ts"; intent = "edit"
            focusMethods = @("ContextTarget.update")
        }
    )
}
$session = $null

try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace

    $tools = Invoke-CleanCtxRpc $session 1 "tools/list" @{}
    Assert-NoRpcError $tools "tools/list"
    $tool = @($tools.result.tools | Where-Object { $_.name -eq "provide_code_context" })
    Assert-True ($tool.Count -eq 1) "tools/list did not expose exactly one provide_code_context tool."
    $batchBranch = @($tool[0].inputSchema.oneOf | Where-Object {
        @($_.required) -contains "files"
    })
    Assert-True ($batchBranch.Count -eq 1) "provide_code_context omitted its batch request branch."
    Assert-True ($batchBranch[0].properties.files.maxItems -eq 8) "Batch schema omitted the eight-item cap."
    Assert-True ($tool[0].outputSchema.properties.results.type -eq "array") "Output schema omitted ordered results."
    Write-Host "PASS: live tools/list exposes the provide-code-context batch contract."

    $first = Invoke-CleanCtxTool $session 2 "provide_code_context" $batchArguments
    Save-Capture "batch-first" $first
    $firstResults = Structured-Results $first "first batch"
    Assert-True ($firstResults.Count -eq 3) "First batch did not return exactly three outcomes."
    Assert-True (($firstResults.id -join ",") -eq "overview,missing,target") "Batch did not preserve IDs and order."
    Assert-True ($firstResults[0].status -eq "ok") "Overview item failed."
    Assert-True ($firstResults[0].content_index -eq 0) "Overview content index was not zero."
    Assert-True ($firstResults[1].status -eq "error") "Missing file did not produce an isolated error."
    Assert-True ($firstResults[1].error.code -eq -32602) "Missing file used the wrong error classification."
    Assert-True ($null -eq $firstResults[1].PSObject.Properties["content_index"]) "Failed item fabricated a content index."
    Assert-True ($firstResults[2].status -eq "ok") "Focused Edit item failed."
    Assert-True ($firstResults[2].content_index -eq 1) "Focused Edit content index did not skip the failure."
    Assert-True (@($first.result.content).Count -eq 2) "Batch content did not contain exactly the two success blocks."
    Assert-True ([string]$first.result.content[1].text -match "return value \+ 1") "Focused Edit block omitted the exact target body."
    Write-Host "PASS: mixed modes preserve order, isolate failure, and correlate exact content indexes."

    $single = Invoke-CleanCtxTool $session 3 "provide_code_context" @{
        workspaceRoot = $workspace
        tokenizer = "o200k"
        filePath = "target.ts"
        intent = "edit"
        focusMethods = @("ContextTarget.update")
    }
    Save-Capture "single-edit" $single
    Assert-NoRpcError $single "single focused Edit"
    Assert-True -Condition (
        [string]$first.result.content[1].text -ceq [string]$single.result.content[0].text
    ) -Message "Batch focused Edit text differs from the legacy single response."
    Write-Host "PASS: successful focused Edit content is byte-identical to the legacy single call."

    $hitsBeforeRepeat = Cache-Hits $session 4
    $second = Invoke-CleanCtxTool $session 5 "provide_code_context" $batchArguments
    Save-Capture "batch-repeat" $second
    $secondResults = Structured-Results $second "repeated batch"
    $hitsAfterRepeat = Cache-Hits $session 6
    Assert-True -Condition (
        ($first.result.content | ConvertTo-Json -Depth 100 -Compress) -ceq
        ($second.result.content | ConvertTo-Json -Depth 100 -Compress)
    ) -Message "Repeated batch changed model-visible content."
    Assert-True -Condition (
        ($firstResults | ConvertTo-Json -Depth 100 -Compress) -ceq
        ($secondResults | ConvertTo-Json -Depth 100 -Compress)
    ) -Message "Repeated batch changed structured item semantics."
    Assert-True ($hitsAfterRepeat -gt $hitsBeforeRepeat) "Repeated batch did not increase the prompt-cache hit count."
    Write-Host "PASS: repeated execution preserves semantics and reuses the batch cache identity."

    $report = [ordered]@{
        verified_at_utc = [DateTime]::UtcNow.ToString("o")
        binary = $BinaryPath
        workspace = $workspace
        ordered_ids = @($firstResults.id)
        isolated_error_code = $firstResults[1].error.code
        edit_content_matches_single = $true
        cache_hits_before_repeat = $hitsBeforeRepeat
        cache_hits_after_repeat = $hitsAfterRepeat
    }
    Save-Capture "verification-report" $report
    Write-Host "PASS: live MCP provide-code-context batch verification completed."
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
