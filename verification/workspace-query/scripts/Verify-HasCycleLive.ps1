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
$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-cycle-live-" + [guid]::NewGuid())
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
        jsonrpc = '2.0'
        id = $requestId
        method = $Method
        params = $Params
    } | ConvertTo-Json -Compress -Depth 20
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

function Assert-NoError {
    param($Response, [string]$Scenario)
    $errorProperty = $Response.PSObject.Properties['error']
    if ($null -ne $errorProperty -and $null -ne $errorProperty.Value) {
        throw "$Scenario returned an MCP error: $($Response.error | ConvertTo-Json -Compress -Depth 10)"
    }
}

function Invoke-WorkspaceQuery {
    param([hashtable]$Arguments)
    Invoke-McpRequest -Method 'tools/call' -Params @{
        name = 'workspace_query'
        arguments = $Arguments
    }
}

function Compile-IntoIndex {
    param([string]$FilePath)
    $response = Invoke-WorkspaceQuery -Arguments @{
        type = 'entities_in_file'
        file_path = $FilePath
        workspaceRoot = $workspace
        fidelity = 'high'
    }
    Assert-NoError -Response $response -Scenario "compile $FilePath"
}

function Invoke-HasCycle {
    $response = Invoke-WorkspaceQuery -Arguments @{
        type = 'has_cycle'
        kind = 'dependency'
        workspaceRoot = $workspace
    }
    Assert-NoError -Response $response -Scenario 'has_cycle'
    return $response
}

try {
    $callA = Join-Path $workspace 'call-a.ts'
    $callB = Join-Path $workspace 'call-b.ts'
    $serviceA = Join-Path $workspace 'alpha.service.ts'
    $serviceB = Join-Path $workspace 'beta.service.ts'

    Write-Utf8File $callA @'
export class CallA {
  alpha(): void { beta(); }
}
'@
    Write-Utf8File $callB @'
export class CallB {
  beta(): void { alpha(); }
}
'@
    Write-Utf8File $serviceA @'
import { Injectable } from '@angular/core';
@Injectable()
export class AlphaService {
  constructor(private beta: BetaService) {}
}
'@
    Write-Utf8File $serviceB @'
import { Injectable } from '@angular/core';
@Injectable()
export class BetaService {
  constructor(private alpha: AlphaService) {}
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

    $toolList = Invoke-McpRequest -Method 'tools/list' -Params @{}
    Assert-NoError $toolList 'tools/list'
    $workspaceTool = @($toolList.result.tools | Where-Object { $_.name -eq 'workspace_query' })
    if ($workspaceTool.Count -ne 1) { throw 'tools/list did not expose exactly one workspace_query tool.' }
    if (@($workspaceTool[0].inputSchema.properties.kind.enum) -notcontains 'dependency') {
        throw 'workspace_query schema did not expose kind=dependency.'
    }
    Write-Host 'PASS: live tools/list exposes dependency cycle kind.'

    $untouched = Invoke-HasCycle
    $untouchedResult = $untouched.result.structuredContent
    if ($untouchedResult.has_cycle -ne $false -or @($untouchedResult.cycle).Count -ne 0) {
        throw 'A first-touch has_cycle query must not compile the workspace implicitly.'
    }
    if ($untouchedResult.coverage.status -ne 'indexed_evidence_only' -or
        $untouchedResult.coverage.source_complete -ne $false) {
        throw 'A no-cycle first-touch response did not disclose index-only coverage.'
    }
    Write-Host 'PASS: first-touch has_cycle is index-only and performs no implicit workspace compilation.'

    Compile-IntoIndex $callA
    Compile-IntoIndex $callB
    $excluded = Invoke-HasCycle
    if ($excluded.result.structuredContent.has_cycle -ne $false) {
        throw 'A native Calls loop must not count as a dependency cycle.'
    }
    Write-Host 'PASS: a live native Calls loop is excluded from dependency-cycle semantics.'

    Compile-IntoIndex $serviceA
    Compile-IntoIndex $serviceB
    $positive = Invoke-HasCycle
    $result = $positive.result.structuredContent
    if ($result.has_cycle -ne $true) { throw 'The live Injects loop was not detected.' }
    $cycle = @($result.cycle)
    if ($cycle.Count -ne 2) { throw "Expected a two-step witness, found $($cycle.Count)." }
    if (@($cycle | Where-Object { $_.relation -ne 'Injects' }).Count -ne 0) {
        throw 'The live witness contains a relation other than Injects.'
    }
    if ($cycle[0].object.name -ne $cycle[1].subject.name -or
        $cycle[1].object.name -ne $cycle[0].subject.name) {
        throw 'The live witness is not a closed ordered path.'
    }
    if (@($cycle | Where-Object { [string]::IsNullOrWhiteSpace($_.asserting_file) }).Count -ne 0) {
        throw 'A live witness step is missing asserting-file provenance.'
    }
    Write-Host 'PASS: a scoped live Injects cycle returns a closed two-step witness with provenance.'
    Write-Host 'PASS: live MCP has_cycle witness verification completed.'
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
