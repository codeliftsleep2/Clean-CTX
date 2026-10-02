param(
    [string]$BinaryPath,
    [switch]$KeepWorkspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $repoRoot 'target\debug\clean-ctx.exe'
}
$BinaryPath = (Resolve-Path $BinaryPath).Path
$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-csharp-ctor-live-" + [guid]::NewGuid())
[System.IO.Directory]::CreateDirectory($workspace) | Out-Null
$process = $null
$nextId = 0

function Write-Utf8File {
    param([string]$Path, [string]$Content)
    [System.IO.File]::WriteAllText($Path, $Content, [System.Text.UTF8Encoding]::new($false))
}

function Invoke-McpRequest {
    param([string]$Method, [hashtable]$Params)
    $script:nextId += 1
    $requestId = $script:nextId
    $request = @{
        jsonrpc = '2.0'; id = $requestId; method = $Method; params = $Params
    } | ConvertTo-Json -Compress -Depth 30
    $script:process.StandardInput.WriteLine($request)
    $script:process.StandardInput.Flush()
    while (-not $script:process.HasExited) {
        $line = $script:process.StandardOutput.ReadLine()
        if ($null -eq $line) { break }
        try { $response = $line | ConvertFrom-Json } catch { continue }
        if ($response.id -eq $requestId) { return $response }
    }
    throw "MCP server exited before responding to request $requestId ($Method)."
}

function Invoke-Tool {
    param([string]$Name, [hashtable]$Arguments)
    Invoke-McpRequest -Method 'tools/call' -Params @{ name = $Name; arguments = $Arguments }
}

function Assert-NoError {
    param($Response, [string]$Scenario)
    if ($null -ne $Response.PSObject.Properties['error']) {
        throw "$Scenario returned an MCP error: $($Response.error | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Get-Structured {
    param($Response, [string]$Scenario)
    Assert-NoError $Response $Scenario
    if ($null -eq $Response.result.structuredContent) {
        throw "$Scenario omitted structuredContent."
    }
    $Response.result.structuredContent
}

function Compile-File {
    param([string]$Path)
    $response = Invoke-Tool 'workspace_query' @{
        type = 'entities_in_file'; file_path = $Path
        workspaceRoot = $workspace; fidelity = 'high'
    }
    [void](Get-Structured $response "compile $Path")
}

function Query-Edges {
    param([string]$Type, [string]$EntityType, [string]$Name)
    $response = Invoke-Tool 'workspace_query' @{
        type = $Type; domain = 'builtin'; entity_type = $EntityType
        name = $Name; workspaceRoot = $workspace
    }
    Get-Structured $response "$Type builtin/$EntityType/$Name"
}

function Assert-Coverage {
    param(
        $Result,
        [string]$Status,
        [bool]$IdentityIndexed,
        [bool]$CapabilityEstablished,
        [string]$Scenario
    )
    if ($Result.coverage.status -ne $Status -or
        [bool]$Result.coverage.identity_indexed -ne $IdentityIndexed -or
        [bool]$Result.coverage.capability_established -ne $CapabilityEstablished -or
        [bool]$Result.coverage.source_complete) {
        throw "$Scenario returned misleading coverage: $($Result.coverage | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Test-ConstructorEdge {
    param($Edge, [string]$Subject, [string]$ExpectedFile)
    $subjectFile = [string]$Edge.subject.file
    $objectFile = [string]$Edge.object.file
    return $Edge.relation -eq 'HasConstructorParameterType' -and
        $Edge.subject.domain -eq 'builtin' -and
        $Edge.subject.entity_type -eq 'Class' -and
        $Edge.subject.name -eq $Subject -and
        $Edge.object.domain -eq 'builtin' -and
        $Edge.object.entity_type -eq 'TypeRef' -and
        $Edge.object.name -eq 'IFooService' -and
        $Edge.layer -eq 'builtin' -and
        $subjectFile.Equals($ExpectedFile, [System.StringComparison]::OrdinalIgnoreCase) -and
        $objectFile.Equals($ExpectedFile, [System.StringComparison]::OrdinalIgnoreCase)
}

function Assert-ForwardConstructorEdge {
    param($Result, [string]$Subject, [string]$ExpectedFile)
    $matches = @($Result.edges | Where-Object {
        Test-ConstructorEdge $_ $Subject $ExpectedFile
    })
    if ($matches.Count -ne 1) {
        throw "Expected exactly one constructor type edge for $Subject, found $($matches.Count): $($Result | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Assert-NoConstructorEdge {
    param($Result, [string]$Subject)
    $matches = @($Result.edges | Where-Object {
        $_.relation -eq 'HasConstructorParameterType' -and
        $_.subject.domain -eq 'builtin' -and
        $_.subject.entity_type -eq 'Class' -and
        $_.subject.name -eq $Subject -and
        $_.object.domain -eq 'builtin' -and
        $_.object.entity_type -eq 'TypeRef' -and
        $_.object.name -eq 'IFooService'
    })
    if ($matches.Count -ne 0) {
        throw "Unexpected constructor type edge for ${Subject}: $($matches | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Edge-Key {
    param($Edge)
    @(
        $Edge.relation,
        $Edge.subject.domain, $Edge.subject.entity_type, $Edge.subject.name, $Edge.subject.file,
        $Edge.object.domain, $Edge.object.entity_type, $Edge.object.name, $Edge.object.file,
        $Edge.layer
    ) -join '|'
}

try {
    $interfacePath = Join-Path $workspace 'IFooService.cs'
    $barPath = Join-Path $workspace 'BarController.cs'
    $bazPath = Join-Path $workspace 'BazService.cs'
    $attributedPath = Join-Path $workspace 'AttributedConsumer.cs'
    $implementsPath = Join-Path $workspace 'ImplementsOnly.cs'
    $methodPath = Join-Path $workspace 'MethodOnly.cs'

    Write-Utf8File $interfacePath @'
public interface IFooService
{
}
'@
    Write-Utf8File $barPath @'
public sealed class BarController
{
    public BarController(IFooService fooService)
    {
    }
}
'@
    Write-Utf8File $bazPath @'
public sealed class BazService
{
    public BazService(IFooService fooService)
    {
    }
}
'@
    Write-Utf8File $attributedPath @'
public sealed class AttributedConsumer
{
    public AttributedConsumer([FromServices] IFooService fooService)
    {
    }
}
'@
    Write-Utf8File $implementsPath @'
public sealed class ImplementsOnly : IFooService
{
}
'@
    Write-Utf8File $methodPath @'
public sealed class MethodOnly
{
    public void Handle(IFooService fooService)
    {
    }
}
'@

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $BinaryPath
    $startInfo.WorkingDirectory = $workspace
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $false
    $startInfo.CreateNoWindow = $true
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) { throw "Failed to start $BinaryPath" }

    $tools = Invoke-McpRequest 'tools/list' @{}
    Assert-NoError $tools 'tools/list'
    $workspaceTools = @($tools.result.tools | Where-Object { $_.name -eq 'workspace_query' })
    if ($workspaceTools.Count -ne 1) {
        throw 'tools/list did not expose exactly one workspace_query tool.'
    }
    Write-Host 'PASS: live tools/list exposes workspace_query.'

    foreach ($path in @($interfacePath, $barPath, $bazPath, $attributedPath, $implementsPath, $methodPath)) {
        Compile-File $path
    }

    $bogus = Query-Edges 'reverse_edges' 'DefinitelyUnsupported' 'DefinitelyMissing'
    if ($bogus.count -ne 0) {
        throw "A bogus exact identity returned edges: $($bogus | ConvertTo-Json -Compress -Depth 20)"
    }
    Assert-Coverage $bogus 'identity_not_indexed' $false $false 'bogus exact identity'
    Write-Host 'PASS: a bogus exact identity reports identity_not_indexed instead of absence.'

    $unsupportedInterface = Query-Edges 'reverse_edges' 'Interface' 'IFooService'
    if ($unsupportedInterface.count -ne 0) {
        throw "The unsupported Interface consumer shape returned edges: $($unsupportedInterface | ConvertTo-Json -Compress -Depth 20)"
    }
    Assert-Coverage $unsupportedInterface 'capability_not_established' $true $false 'Interface consumer shape'
    Write-Host 'PASS: a real Interface identity reports capability_not_established for consumer lookup.'

    $reverse = Query-Edges 'reverse_edges' 'TypeRef' 'IFooService'
    $reverseEdges = @($reverse.edges)
    if ($reverseEdges.Count -ne 3) {
        throw "Expected exactly three constructor consumers, found $($reverseEdges.Count): $($reverse | ConvertTo-Json -Compress -Depth 20)"
    }
    $expectedConsumers = @{
        BarController = $barPath
        BazService = $bazPath
        AttributedConsumer = $attributedPath
    }
    foreach ($edge in $reverseEdges) {
        $subject = [string]$edge.subject.name
        if (-not $expectedConsumers.ContainsKey($subject)) {
            throw "Unexpected constructor consumer '$subject'."
        }
        if (-not (Test-ConstructorEdge $edge $subject $expectedConsumers[$subject])) {
            throw "Malformed constructor consumer relationship: $($edge | ConvertTo-Json -Compress -Depth 20)"
        }
    }
    foreach ($subject in $expectedConsumers.Keys) {
        if (@($reverseEdges | Where-Object { $_.subject.name -eq $subject }).Count -ne 1) {
            throw "Expected exactly one reverse edge for $subject."
        }
    }
    Assert-Coverage $reverse 'established_indexed_capability' $true $true 'TypeRef consumer shape'
    Write-Host 'PASS: reverse_edges(TypeRef/IFooService) includes the attributed C# constructor consumer.'

    $barForward = Query-Edges 'forward_edges' 'Class' 'BarController'
    Assert-ForwardConstructorEdge $barForward 'BarController' $barPath
    Write-Host 'PASS: forward_edges(Class/BarController) exposes its constructor parameter type dependency.'

    $bazForward = Query-Edges 'forward_edges' 'Class' 'BazService'
    Assert-ForwardConstructorEdge $bazForward 'BazService' $bazPath
    Write-Host 'PASS: forward_edges(Class/BazService) exposes its constructor parameter type dependency.'

    $attributedForward = Query-Edges 'forward_edges' 'Class' 'AttributedConsumer'
    Assert-ForwardConstructorEdge $attributedForward 'AttributedConsumer' $attributedPath
    Write-Host 'PASS: [FromServices] metadata does not pollute the constructor TypeRef identity.'

    $methodForward = Query-Edges 'forward_edges' 'Class' 'MethodOnly'
    Assert-NoConstructorEdge $methodForward 'MethodOnly'
    Assert-Coverage $methodForward 'established_indexed_capability' $true $true 'unused Class forward capability'
    Write-Host 'PASS: an ordinary C# method parameter does not become a constructor dependency.'

    $batch = Get-Structured (Invoke-Tool 'workspace_query' @{
        workspaceRoot = $workspace
        queries = @(
            @{ id = 'bogus'; type = 'reverse_edges'; domain = 'builtin'; entity_type = 'DefinitelyUnsupported'; name = 'DefinitelyMissing' },
            @{ id = 'unsupported'; type = 'reverse_edges'; domain = 'builtin'; entity_type = 'Interface'; name = 'IFooService' },
            @{ id = 'positive'; type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'IFooService' },
            @{ id = 'zero'; type = 'forward_edges'; domain = 'builtin'; entity_type = 'Class'; name = 'MethodOnly' }
        )
    }) 'exact-identity coverage batch'
    $batchResults = @($batch.results)
    if ($batchResults.Count -ne 4 -or @($batchResults | Where-Object { $_.status -ne 'ok' }).Count -ne 0) {
        throw "Coverage batch did not preserve four successful item outcomes: $($batch | ConvertTo-Json -Compress -Depth 30)"
    }
    Assert-Coverage $batchResults[0].result 'identity_not_indexed' $false $false 'batched bogus identity'
    Assert-Coverage $batchResults[1].result 'capability_not_established' $true $false 'batched unsupported capability'
    Assert-Coverage $batchResults[2].result 'established_indexed_capability' $true $true 'batched positive capability'
    Assert-Coverage $batchResults[3].result 'established_indexed_capability' $true $true 'batched genuine zero'
    Write-Host 'PASS: batch results preserve independent exact-identity coverage.'

    $implementsForward = Query-Edges 'forward_edges' 'Class' 'ImplementsOnly'
    Assert-NoConstructorEdge $implementsForward 'ImplementsOnly'
    Write-Host 'PASS: implementing an interface does not masquerade as constructor consumption.'

    $firstKeys = @($reverseEdges | ForEach-Object { Edge-Key $_ } | Sort-Object)
    $repeated = Query-Edges 'reverse_edges' 'TypeRef' 'IFooService'
    $repeatedKeys = @($repeated.edges | ForEach-Object { Edge-Key $_ } | Sort-Object)
    if (($firstKeys -join "`n") -ne ($repeatedKeys -join "`n")) {
        throw "Repeated constructor-consumer lookup changed its semantic answer: $($repeated | ConvertTo-Json -Compress -Depth 20)"
    }
    Write-Host 'PASS: repeated constructor-consumer lookup is semantically stable.'

    Write-Utf8File $barPath @'
public sealed class BarController
{
    public BarController()
    {
    }
}
'@
    Compile-File $barPath
    $afterReplacement = Query-Edges 'reverse_edges' 'TypeRef' 'IFooService'
    $replacementEdges = @($afterReplacement.edges)
    if ($replacementEdges.Count -ne 2 -or
        @($replacementEdges | Where-Object { Test-ConstructorEdge $_ 'BazService' $bazPath }).Count -ne 1 -or
        @($replacementEdges | Where-Object { Test-ConstructorEdge $_ 'AttributedConsumer' $attributedPath }).Count -ne 1) {
        throw "Recompilation did not replace the stale constructor dependency: $($afterReplacement | ConvertTo-Json -Compress -Depth 20)"
    }
    if (@($replacementEdges | Where-Object { $_.subject.name -eq 'BarController' }).Count -ne 0) {
        throw 'BarController remained in reverse_edges after its constructor dependency was removed.'
    }
    Write-Host 'PASS: recompilation removes a stale C# constructor dependency from workspace_query.'

    Write-Host 'PASS: live MCP C# constructor-dependency verification completed.'
}
finally {
    if ($null -ne $process) {
        try { $process.StandardInput.Close() } catch {}
        if (-not $process.HasExited) {
            $process.Kill()
            if (-not $process.WaitForExit(5000)) {
                Write-Warning 'MCP server did not exit within five seconds; workspace cleanup may be deferred.'
            }
        }
        $process.Dispose()
    }
    if ($KeepWorkspace) {
        Write-Host "Retained live verification workspace: $workspace"
    }
    elseif (Test-Path -LiteralPath $workspace) {
        $removed = $false
        foreach ($attempt in 1..5) {
            try {
                Remove-Item -LiteralPath $workspace -Recurse -Force
                $removed = $true
                break
            }
            catch {
                if ($attempt -lt 5) { Start-Sleep -Milliseconds 200 }
            }
        }
        if (-not $removed) { Write-Warning "Could not remove live verification workspace: $workspace" }
    }
}
