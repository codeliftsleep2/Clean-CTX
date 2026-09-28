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
$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-cbm-duplicate-" + [guid]::NewGuid())
$source = Join-Path $workspace 'src'
[System.IO.Directory]::CreateDirectory($source) | Out-Null
$evidenceRoot = Join-Path $repoRoot 'target\cbm-duplicate-trace-verification'
[System.IO.Directory]::CreateDirectory($evidenceRoot) | Out-Null
$evidencePath = Join-Path $evidenceRoot ((Get-Date -Format 'yyyyMMdd-HHmmss') + '.json')
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

function Require-TransportSuccess {
    param($Response, [string]$Scenario)
    if ($null -ne $Response.PSObject.Properties['error']) {
        throw "$Scenario returned a JSON-RPC error: $($Response.error | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Test-ToolError {
    param($Response)
    $property = $Response.result.PSObject.Properties['isError']
    return $null -ne $property -and $property.Value -eq $true
}

function Tool-ErrorFlag {
    param($Response)
    if (Test-ToolError $Response) { return 'true' }
    return 'false'
}

function Require-AmbiguousTraceSource {
    param($Response, [string[]]$ExpectedIds, [string]$Scenario)
    if ($null -eq $Response.PSObject.Properties['error'] -or $Response.error.code -ne -32602) {
        throw "$Scenario did not return the required -32602 ambiguity error: $($Response | ConvertTo-Json -Compress -Depth 30)"
    }
    if (-not $Response.error.message.Contains('ambiguous')) {
        throw "$Scenario did not identify ambiguity explicitly."
    }
    $actual = @($Response.error.data.candidates | ForEach-Object { $_.id } | Sort-Object)
    $expected = @($ExpectedIds | Sort-Object)
    if (($actual -join "`n") -ne ($expected -join "`n")) {
        throw "$Scenario returned the wrong canonical candidates: $($Response.error.data.candidates | ConvertTo-Json -Compress -Depth 20)"
    }
}

function Invoke-GraphTrace {
    param([string]$From, [string]$To)
    Invoke-Tool 'graph_trace' @{ from = $From; to = $To; project = $workspace }
}

function Invoke-ProxyTrace {
    param([string]$FunctionName, [string]$Project)
    Invoke-Tool 'cbm_proxy' @{
        cbm_tool = 'trace_path'
        parameters = @{
            function_name = $FunctionName
            direction = 'outbound'
            depth = 3
            project = $Project
        }
    }
}

try {
    Write-Utf8File (Join-Path $workspace 'Cargo.toml') @'
[package]
name = "duplicate-trace-probe"
version = "0.1.0"
edition = "2024"
'@
    Write-Utf8File (Join-Path $source 'lib.rs') @'
pub mod alpha;
pub mod beta;
'@
    Write-Utf8File (Join-Path $source 'alpha.rs') @'
pub fn duplicate_probe() {
    alpha_leaf();
}

pub fn alpha_leaf() {}
'@
    Write-Utf8File (Join-Path $source 'beta.rs') @'
pub fn duplicate_probe() {
    beta_leaf();
}

pub fn beta_leaf() {}
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
    Require-TransportSuccess $tools 'tools/list'
    foreach ($required in @('index_repository', 'graph_search', 'graph_trace', 'cbm_proxy')) {
        if (@($tools.result.tools | Where-Object { $_.name -eq $required }).Count -ne 1) {
            throw "tools/list did not expose '$required'."
        }
    }
    Write-Host 'PASS: live server exposes every required CBM surface.'

    $index = Invoke-Tool 'index_repository' @{ repo_path = $workspace; mode = 'full' }
    Require-TransportSuccess $index 'index_repository'
    if (Test-ToolError $index) {
        throw "CBM rejected fixture indexing: $($index | ConvertTo-Json -Compress -Depth 20)"
    }
    Write-Host 'PASS: controlled duplicate-name repository was submitted to CBM indexing.'

    $search = Invoke-Tool 'graph_search' @{
        query = 'duplicate_probe'
        name_pattern = '.*duplicate_probe.*'
        project = $workspace
    }
    Require-TransportSuccess $search 'graph_search duplicate_probe'
    if (Test-ToolError $search) {
        throw "CBM search failed: $($search | ConvertTo-Json -Compress -Depth 20)"
    }
    $duplicates = @($search.result.structuredContent.nodes | Where-Object {
        $_.name -eq 'duplicate_probe'
    })
    if ($duplicates.Count -ne 2) {
        throw "Expected exactly two discovered duplicate_probe functions, found $($duplicates.Count): $($search | ConvertTo-Json -Compress -Depth 30)"
    }
    $canonical = @($duplicates | ForEach-Object { $_.id } | Sort-Object -Unique)
    if ($canonical.Count -ne 2) {
        throw 'CBM did not expose two distinct canonical identities for the duplicate functions.'
    }
    Write-Host 'PASS: graph_search discovered two distinct canonical duplicate_probe identities.'

    $alphaSearch = Invoke-Tool 'graph_search' @{
        query = 'alpha_leaf'; name_pattern = '.*alpha_leaf.*'; project = $workspace
    }
    $betaSearch = Invoke-Tool 'graph_search' @{
        query = 'beta_leaf'; name_pattern = '.*beta_leaf.*'; project = $workspace
    }
    Require-TransportSuccess $alphaSearch 'graph_search alpha_leaf'
    Require-TransportSuccess $betaSearch 'graph_search beta_leaf'
    $alphaLeaf = @($alphaSearch.result.structuredContent.nodes | Where-Object { $_.name -eq 'alpha_leaf' })[0].id
    $betaLeaf = @($betaSearch.result.structuredContent.nodes | Where-Object { $_.name -eq 'beta_leaf' })[0].id
    if ([string]::IsNullOrWhiteSpace($alphaLeaf) -or [string]::IsNullOrWhiteSpace($betaLeaf)) {
        throw 'Canonical leaf identities were not discoverable.'
    }

    $alphaDuplicate = @($duplicates | Where-Object { $_.file -match 'alpha\.rs$' })[0].id
    $betaDuplicate = @($duplicates | Where-Object { $_.file -match 'beta\.rs$' })[0].id
    if ([string]::IsNullOrWhiteSpace($alphaDuplicate) -or [string]::IsNullOrWhiteSpace($betaDuplicate)) {
        throw 'Could not associate canonical duplicate identities with alpha.rs and beta.rs.'
    }
    $projectSlug = $alphaDuplicate -replace '\.src\.alpha\.duplicate_probe$', ''
    if ([string]::IsNullOrWhiteSpace($projectSlug) -or
        -not $betaDuplicate.StartsWith("$projectSlug.", [System.StringComparison]::Ordinal)) {
        throw 'Could not derive one shared canonical CBM project slug from discovered identities.'
    }

    $wrapper = [ordered]@{
        bare_to_alpha = Invoke-GraphTrace 'duplicate_probe' $alphaLeaf
        bare_to_beta = Invoke-GraphTrace 'duplicate_probe' $betaLeaf
        canonical_alpha = Invoke-GraphTrace $alphaDuplicate $alphaLeaf
        canonical_beta = Invoke-GraphTrace $betaDuplicate $betaLeaf
        crossed_alpha_to_beta = Invoke-GraphTrace $alphaDuplicate $betaLeaf
        crossed_beta_to_alpha = Invoke-GraphTrace $betaDuplicate $alphaLeaf
    }
    $proxy = [ordered]@{
        bare = Invoke-ProxyTrace 'duplicate_probe' $projectSlug
        canonical_alpha = Invoke-ProxyTrace $alphaDuplicate $projectSlug
        canonical_beta = Invoke-ProxyTrace $betaDuplicate $projectSlug
    }

    Require-AmbiguousTraceSource $wrapper.bare_to_alpha @($alphaDuplicate, $betaDuplicate) 'graph_trace bare source'
    Require-AmbiguousTraceSource $wrapper.bare_to_beta @($alphaDuplicate, $betaDuplicate) 'graph_trace bare source with alternate target'
    Require-AmbiguousTraceSource $proxy.bare @($alphaDuplicate, $betaDuplicate) 'cbm_proxy trace_path bare source'
    Require-TransportSuccess $wrapper.canonical_alpha 'graph_trace canonical alpha source'
    Require-TransportSuccess $wrapper.canonical_beta 'graph_trace canonical beta source'
    if ($wrapper.canonical_alpha.result.structuredContent.count -ne 1 -or
        $wrapper.canonical_beta.result.structuredContent.count -ne 1) {
        throw 'Canonical graph_trace controls did not retain their distinct edges.'
    }
    Require-TransportSuccess $proxy.canonical_alpha 'cbm_proxy canonical alpha source'
    Require-TransportSuccess $proxy.canonical_beta 'cbm_proxy canonical beta source'

    $evidence = [ordered]@{
        recorded_at = (Get-Date).ToString('o')
        binary = $BinaryPath
        fixture = $workspace
        discovered = [ordered]@{
            duplicate_nodes = $duplicates
            project_slug = $projectSlug
            alpha_leaf = $alphaLeaf
            beta_leaf = $betaLeaf
        }
        wrapper_graph_trace = $wrapper
        proxy_trace_path = $proxy
    }
    [System.IO.File]::WriteAllText(
        $evidencePath,
        ($evidence | ConvertTo-Json -Depth 50),
        [System.Text.UTF8Encoding]::new($false)
    )

    Write-Host 'PASS: graph_trace rejects an ambiguous bare source with both canonical candidates.'
    Write-Host 'PASS: cbm_proxy(trace_path) enforces the same ambiguity boundary.'
    Write-Host 'PASS: canonical graph_trace and proxy identities retain their direct trace paths.'
    Write-Host "Evidence written to: $evidencePath"
    Write-Host 'PASS: live MCP duplicate trace identity-resolution verification completed.'
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
        Write-Host "Retained fixture workspace: $workspace"
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
        if (-not $removed) { Write-Warning "Could not remove fixture workspace: $workspace" }
    }
}
