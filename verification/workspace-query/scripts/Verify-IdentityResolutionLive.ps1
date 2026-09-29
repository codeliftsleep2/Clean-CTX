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
$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-identity-live-" + [guid]::NewGuid())
$feature = Join-Path $workspace 'feature'
$other = Join-Path $workspace 'other'
[System.IO.Directory]::CreateDirectory($feature) | Out-Null
[System.IO.Directory]::CreateDirectory($other) | Out-Null
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

function Compile-File {
    param([string]$Path)
    $response = Invoke-Tool 'workspace_query' @{
        type = 'entities_in_file'; file_path = $Path
        workspaceRoot = $workspace; fidelity = 'high'
    }
    Assert-NoError $response "compile $Path"
}

function Query {
    param([hashtable]$Arguments)
    $Arguments.workspaceRoot = $workspace
    Invoke-Tool 'workspace_query' $Arguments
}

function Structured {
    param($Response, [string]$Scenario)
    Assert-NoError $Response $Scenario
    if ($null -eq $Response.result.structuredContent) {
        throw "$Scenario omitted structuredContent."
    }
    $Response.result.structuredContent
}

function Edge-Key {
    param($Edge)
    "$($Edge.relation)|$($Edge.subject.domain)|$($Edge.subject.entity_type)|$($Edge.subject.name)|$($Edge.object.domain)|$($Edge.object.entity_type)|$($Edge.object.name)"
}

try {
    $consumer = Join-Path $feature 'consumer.component.ts'
    $middle = Join-Path $feature 'middle.service.ts'
    $calls = Join-Path $feature 'calls.ts'
    $duplicate = Join-Path $other 'duplicate.component.ts'
    $ambiguous = Join-Path $other 'shared.service.ts'

    Write-Utf8File $consumer @'
import { Component } from '@angular/core';
@Component({ selector: 'live-consumer', template: '' })
export class LiveConsumer {
  constructor(private middle: MiddleService) {}
}
'@
    Write-Utf8File $middle @'
import { Injectable } from '@angular/core';
@Injectable()
export class MiddleService {
  constructor(private end: EndService) {}
}
'@
    Write-Utf8File $calls @'
export class CallProbe {
  run(): void { step(); }
  step(): void { finish(); }
  finish(): void {}
}
'@
    Write-Utf8File $duplicate @'
import { Component } from '@angular/core';
@Component({ selector: 'duplicate-consumer', template: '' })
export class LiveConsumer {
  constructor(private audit: AuditService) {}
}
'@
    Write-Utf8File $ambiguous @'
import { Injectable } from '@angular/core';
@Injectable()
export class LiveConsumer {
  constructor(private alternate: AlternateService) {}
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
    $schema = @($tools.result.tools | Where-Object { $_.name -eq 'workspace_query' })[0].inputSchema
    if ($null -eq $schema.properties.domain -or $null -eq $schema.properties.entity_type) {
        throw 'workspace_query schema omitted optional identity filters.'
    }
    Write-Host 'PASS: live tools/list exposes optional workspace identity filters.'

    Compile-File $consumer
    Compile-File $middle
    Compile-File $calls

    $bare = Structured (Query @{ type = 'forward_edges'; name = 'run' }) 'unique bare-name query'
    $exact = Structured (Query @{
        type = 'forward_edges'; name = 'run'
        domain = 'builtin'; entity_type = 'Method'
    }) 'fully qualified query'
    if ($bare.resolved_identity.domain -ne 'builtin' -or
        $bare.resolved_identity.entity_type -ne 'Method') {
        throw 'The unique bare name resolved to the wrong semantic identity.'
    }
    $bareKeys = @($bare.edges | ForEach-Object { Edge-Key $_ } | Sort-Object)
    $exactKeys = @($exact.edges | ForEach-Object { Edge-Key $_ } | Sort-Object)
    if (($bareKeys -join "`n") -ne ($exactKeys -join "`n")) {
        throw 'Bare-name forward_edges did not match the fully qualified result.'
    }
    Write-Host 'PASS: a unique bare name matches the fully qualified forward_edges result.'

    $reverse = Structured (Query @{ type = 'reverse_edges'; name = 'finish' }) 'bare reverse query'
    if ($reverse.resolved_identity.entity_type -ne 'Method' -or $reverse.count -lt 1) {
        throw 'Bare-name reverse_edges did not resolve the method target.'
    }
    $transitive = Structured (Query @{
        type = 'transitive_dependencies'; name = 'LiveConsumer'; depth = 0
        domain = 'angular'; entity_type = 'Component'
    }) 'filtered transitive query'
    if ($transitive.count -lt 2) {
        throw 'Identity-resolved transitive_dependencies did not traverse the dependency chain.'
    }
    Write-Host 'PASS: reverse_edges resolves a unique bare name and transitive_dependencies accepts resolved identity input.'

    Compile-File $duplicate
    $repeated = Structured (Query @{
        type = 'forward_edges'; name = 'LiveConsumer'
        domain = 'angular'; entity_type = 'Component'
    }) 'repeated identity query'
    if ($repeated.count -lt 2) {
        throw 'Repeated physical occurrences of one identity were lost or treated as ambiguity.'
    }
    Write-Host 'PASS: repeated physical occurrences of one semantic identity are not ambiguous.'

    Compile-File $ambiguous
    $ambiguousResponse = Query @{ type = 'forward_edges'; name = 'LiveConsumer' }
    if ($ambiguousResponse.error.code -ne -32602 -or
        @($ambiguousResponse.error.data.candidates).Count -lt 2) {
        throw "Distinct identities did not return bounded disambiguation candidates: $($ambiguousResponse | ConvertTo-Json -Compress -Depth 20)"
    }
    $filtered = Structured (Query @{
        type = 'forward_edges'; name = 'LiveConsumer'; entity_type = 'Service'
    }) 'partial identity filter'
    if ($filtered.resolved_identity.entity_type -ne 'Service') {
        throw 'The partial entity_type filter did not resolve the intended candidate.'
    }
    Write-Host 'PASS: ambiguity is explicit and a partial identity filter disambiguates it.'

    $missing = Query @{ type = 'forward_edges'; name = 'DefinitelyMissing' }
    if ($missing.error.code -ne -32602 -or @($missing.error.data.candidates).Count -ne 0) {
        throw 'A missing name did not return explicit not-found candidates=[].'
    }
    Write-Host 'PASS: a missing name returns an explicit not-found error.'

    $scoped = Structured (Query @{
        type = 'forward_edges'; name = 'LiveConsumer'; entity_type = 'Component'
        withinPath = $feature
    }) 'withinPath-scoped identity query'
    $outside = @($scoped.edges | Where-Object {
        [string]::IsNullOrWhiteSpace($_.subject.file) -or
        -not $_.subject.file.StartsWith($feature, [System.StringComparison]::OrdinalIgnoreCase)
    })
    if ($scoped.resolved_identity.entity_type -ne 'Component' -or
        $scoped.count -lt 1 -or $outside.Count -ne 0) {
        throw "withinPath did not constrain identity resolution and returned occurrences: $($scoped | ConvertTo-Json -Compress -Depth 20)"
    }
    Write-Host 'PASS: withinPath constrains both identity resolution and returned occurrences.'
    Write-Host 'PASS: live MCP workspace-query identity-resolution verification completed.'
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
