param([string]$WorkspaceRoot = (Get-Location).Path)
# Operator-owned raw-provider diagnostic. No index request or daemon management.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$binary = (Get-Command codebase-memory-mcp.exe -CommandType Application -ErrorAction Stop).Source
$output = Join-Path $root ('target/pcg-b9-r1/cbm-error-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function Save-Json([string]$Name, $Value) {
    [IO.File]::WriteAllText((Join-Path $output "$Name.json"), ($Value | ConvertTo-Json -Depth 50), $utf8)
}
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = $binary
$start.WorkingDirectory = $root
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardInput = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.StandardOutputEncoding = $utf8
$start.StandardErrorEncoding = $utf8
$process = [Diagnostics.Process]::new()
$process.StartInfo = $start
$stderrTask = $null
$script:requestId = 0
$script:frameId = 0
function Invoke-RawRpc([string]$Method, [hashtable]$Parameters) {
    $script:requestId++
    $id = $script:requestId
    $request = @{jsonrpc='2.0';id=$id;method=$Method;params=$Parameters}
    Save-Json "request-$id" $request
    $process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 30 -Compress))
    $process.StandardInput.Flush()
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while ($true) {
        $read = $process.StandardOutput.ReadLineAsync()
        $remaining = [Math]::Max(1, [int]($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) { throw "Raw provider timed out on request $id ($Method)." }
        $line = $read.Result
        if ($null -eq $line) { throw "Raw provider closed stdout on request $id." }
        $script:frameId++
        [IO.File]::WriteAllText((Join-Path $output "frame-$script:frameId.txt"), $line, $utf8)
        $frame = $line | ConvertFrom-Json
        if ($frame.PSObject.Properties['id'] -and $frame.id -eq $id) { return $frame }
        if ([DateTime]::UtcNow -ge $deadline) { throw 'Provider notification stream exhausted request deadline.' }
    }
}
try {
    Save-Json 'identity' @{binary=$binary;sha256=(Get-FileHash $binary).Hash.ToLowerInvariant();root=$root;purpose='raw missing-project error; no indexing';timeout_seconds=30}
    if (-not $process.Start()) { throw 'Provider frontend did not start.' }
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $initialized = Invoke-RawRpc 'initialize' @{protocolVersion='2024-11-05';capabilities=@{};clientInfo=@{name='clean-ctx-b9-r1-diagnostic';version='1'}}
    Save-Json 'initialize-response' $initialized
    if ($initialized.PSObject.Properties['error']) { throw 'Provider rejected initialization.' }
    $process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}')
    $process.StandardInput.Flush()
    $listed = Invoke-RawRpc 'tools/list' @{}
    Save-Json 'tools-list' $listed
    $response = Invoke-RawRpc 'tools/call' @{name='query_graph';arguments=@{
        query='MATCH (n:Function) WHERE n.in_degree = 0 AND n.is_entry_point = false RETURN n.name, n.file_path'
        project='definitely-not-a-real-project-xyz';format='json'
    }}
    Save-Json 'missing-project-response' $response
    Save-Json 'outcome' @{status='response_captured';scope='raw provider only; not a passing Clean-CTX regression'}
} catch {
    Save-Json 'outcome' @{status='failed';message=$_.Exception.Message;scope='raw provider diagnostic'}
    throw
} finally {
    if ($process.Id -and -not $process.HasExited) { $process.Kill(); $null = $process.WaitForExit(5000) }
    if ($null -ne $stderrTask -and $stderrTask.Wait(5000)) {
        [IO.File]::WriteAllText((Join-Path $output 'stderr.txt'), $stderrTask.Result, $utf8)
    }
    $process.Dispose()
    Write-Output "Raw provider diagnostic evidence: $output"
}
