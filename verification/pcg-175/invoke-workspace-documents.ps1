param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$Profile,
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$Requests,
    [Parameter(Mandatory)][string]$OutputRoot,
    [Parameter(Mandatory)][string]$LaunchDirectory,
    [string]$TokenCounter
)
# Explicit production document queries, not a fixture/corpus importer or a test.
# Uses the same bounded JSON-RPC/stdio lifecycle as run-documents.ps1.
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$workspace = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$profilePath = (Resolve-Path -LiteralPath $Profile).Path
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$launchPath = (Resolve-Path -LiteralPath $LaunchDirectory).Path
$outputPath = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $outputPath) { throw 'Use a fresh output root; receipts are never overwritten' }
$requestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Requests).Path)
$queries = @($utf8.GetString($requestBytes) | ConvertFrom-Json -AsHashtable)
if ($queries.Count -eq 0) { throw 'Supply explicit document queries' }
$settings = [IO.File]::ReadAllText($profilePath, $utf8) | ConvertFrom-Json -AsHashtable
if ([IO.Path]::GetFullPath($settings.workspace_root) -ine $workspace) { throw 'Profile workspace mismatch' }
foreach ($query in $queries) {
    if ($query.workspaceRoot -ine $workspace) { throw 'Every query must declare the actual workspaceRoot' }
}
[IO.Directory]::CreateDirectory($outputPath) | Out-Null
[IO.File]::WriteAllBytes((Join-Path $outputPath 'queries.json'), $requestBytes)
function Write-Json([string]$Name, $Value) {
    [IO.File]::WriteAllText((Join-Path $outputPath $Name), ($Value | ConvertTo-Json -Depth 100), $utf8)
}
$binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
$profileHash = (Get-FileHash -LiteralPath $profilePath -Algorithm SHA256).Hash.ToLowerInvariant()
$counterPath = if ($TokenCounter) { (Resolve-Path -LiteralPath $TokenCounter).Path } else { $null }
$counterHash = if ($counterPath) { (Get-FileHash -LiteralPath $counterPath -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = $binaryPath
$info.WorkingDirectory = $launchPath
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardInput = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
$info.StandardInputEncoding = $utf8
$info.StandardOutputEncoding = $utf8
$info.StandardErrorEncoding = $utf8
# Process configuration is isolated; document trust/scope is the actual workspace profile.
$info.Environment['CLEAN_CTX_PROJECT_ROOT'] = $launchPath
$info.Environment['CLEAN_CTX_PCG_DOCUMENT_PROFILE'] = $profilePath
$null = $info.Environment.Remove('CLEAN_CTX_PCG_PROFILE')
$process = [Diagnostics.Process]::new()
$process.StartInfo = $info
$stderr = $null
$failure = $null
$results = [Collections.Generic.List[object]]::new()
function Rpc([int]$Id, [string]$Method, $Params) {
    $request = @{jsonrpc='2.0'; id=$Id; method=$Method; params=$Params}
    Write-Json "$Id.request.json" $request
    $process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 100 -Compress))
    $read = $process.StandardOutput.ReadLineAsync()
    if (-not $read.Wait(30000)) { throw "MCP read timed out at request $Id" }
    $line = $read.GetAwaiter().GetResult()
    if ($null -eq $line) { throw "MCP stdout closed at request $Id" }
    [IO.File]::WriteAllText((Join-Path $outputPath "$Id.response.json"), $line, $utf8)
    $response = $line | ConvertFrom-Json -AsHashtable
    if ($response.id -ne $Id) { throw 'Unexpected RPC response ID' }
    if ($response.ContainsKey('error')) { throw ($response.error | ConvertTo-Json -Compress) }
    if ($Method -eq 'tools/call') {
        $prefix = '{"jsonrpc":"2.0","id":' + $Id + ',"result":'
        if (-not $line.StartsWith($prefix) -or -not $line.EndsWith('}')) { throw 'Unexpected exact-byte PCG framing' }
        $rawResult = $line.Substring($prefix.Length, $line.Length - $prefix.Length - 1)
        [IO.File]::WriteAllText((Join-Path $outputPath "$Id.serialized-result.json"), $rawResult, $utf8)
    }
    return $response
}
try {
    if (-not $process.Start()) { throw 'Production binary launch failed' }
    $stderr = $process.StandardError.ReadToEndAsync()
    $null = Rpc 0 'initialize' @{protocolVersion='2025-11-25'; capabilities=@{}; clientInfo=@{name='maxheadroom-production-authoring'; version='1'}}
    $process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}')
    $catalog = Rpc 1 'tools/list' @{}
    if (@($catalog.result.tools | Where-Object {$_.name -ceq 'pcg_documents_v1'}).Count -ne 1) { throw 'Document tool unavailable in production stdio' }
    $id = 2
    foreach ($query in $queries) {
        $response = Rpc $id 'tools/call' @{name='pcg_documents_v1'; arguments=$query}
        $payload = $response.result.content[0].text | ConvertFrom-Json -AsHashtable
        Write-Json "$id.payload.json" $payload
        $tokens = $null
        if ($counterPath) {
            $tokens = [int](& $counterPath count $query.tokenizer (Join-Path $outputPath "$id.serialized-result.json"))
            if ($LASTEXITCODE -ne 0) { throw 'Existing Rust tokenizer helper failed' }
        }
        $results.Add(@{request=$id; mode=$query.mode; node=$query.id; result_state=$payload.result_state; catalog_current=$payload.catalog_current; generation=$payload.generation; documents=@($payload.documents).Count; context=$payload.context; serialized_tokens=$tokens; budget=$payload.budget; diagnostics=$payload.diagnostics})
        $id++
    }
} catch {
    $failure = $_.Exception.Message
    throw
} finally {
    $exitCode = $null
    if ($stderr) {
        $process.StandardInput.Close()
        if (-not $process.WaitForExit(10000)) { $process.Kill(); $process.WaitForExit(); $failure = 'Production process required termination' }
        $exitCode = $process.ExitCode
        [IO.File]::WriteAllText((Join-Path $outputPath 'stderr.txt'), $stderr.GetAwaiter().GetResult(), $utf8)
    }
    Write-Json 'receipt.json' @{workflow='actual-workspace pcg_documents_v1 production stdio'; workspace=$workspace; profile=$profilePath; binary=$binaryPath; binary_sha256=$binaryHash; profile_sha256=$profileHash; token_counter=$counterPath; token_counter_sha256=$counterHash; process_exit_code=$exitCode; error=$failure; queries=$results.ToArray()}
    $process.Dispose()
}
if ($failure -or $exitCode -ne 0) { throw "Production stdio failed: $failure / exit $exitCode" }
Write-Output "Production authoring receipt: $(Join-Path $outputPath 'receipt.json')"
