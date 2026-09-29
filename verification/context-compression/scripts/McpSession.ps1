function Start-CleanCtxSession {
    param([string]$BinaryPath, [string]$WorkingDirectory)
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $BinaryPath
    $info.WorkingDirectory = $WorkingDirectory
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    # Let diagnostics flow to the operator terminal. Redirecting without an
    # asynchronous drain can deadlock a long capture when the stderr pipe fills.
    $info.RedirectStandardError = $false
    $info.CreateNoWindow = $true
    $utf8 = [Text.UTF8Encoding]::new($false, $true)
    $info.StandardInputEncoding = $utf8
    $info.StandardOutputEncoding = $utf8
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    if (-not $process.Start()) { throw "Unable to start $BinaryPath" }
    $null = Invoke-CleanCtxRpc $process -1 "initialize" @{
        protocolVersion = "2024-11-05"
        capabilities = @{}
        clientInfo = @{ name = "context-compression-verification"; version = "1.0.0" }
    }
    $notification = @{
        jsonrpc = "2.0"; method = "notifications/initialized"; params = @{}
    } | ConvertTo-Json -Depth 10 -Compress
    $process.StandardInput.WriteLine($notification)
    $process.StandardInput.Flush()
    return $process
}

function Invoke-CleanCtxRpc {
    param($Session, [int]$Id, [string]$Method, [hashtable]$Params)
    $request = @{
        jsonrpc = "2.0"; id = $Id; method = $Method; params = $Params
    } | ConvertTo-Json -Depth 40 -Compress
    $Session.StandardInput.WriteLine($request)
    $Session.StandardInput.Flush()
    while (-not $Session.HasExited) {
        $line = $Session.StandardOutput.ReadLine()
        if ($null -eq $line) { break }
        try { $value = $line | ConvertFrom-Json -Depth 100 } catch { continue }
        if ($value.jsonrpc -eq "2.0" -and $value.id -eq $Id) { return $value }
    }
    throw "MCP server ended before response $Id (see stderr above)"
}

function Invoke-CleanCtxTool {
    param($Session, [int]$Id, [string]$Tool, [hashtable]$Arguments)
    Invoke-CleanCtxRpc $Session $Id "tools/call" @{ name = $Tool; arguments = $Arguments }
}

function Stop-CleanCtxSession {
    param($Session)
    if ($Session -and -not $Session.HasExited) {
        $Session.StandardInput.Close()
        if (-not $Session.WaitForExit(3000)) { $Session.Kill() }
    }
    if ($Session) { $Session.Dispose() }
}
