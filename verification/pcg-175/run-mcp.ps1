param(
    [Parameter(Mandatory)][string]$Binary,
    [string]$InputReceipt = 'target/pcg-175/m4a-verify/probe-input.json',
    [string]$OutputRoot = 'target/pcg-175/m4a-verify/stdio'
)
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$inputPath = (Resolve-Path -LiteralPath $InputReceipt).Path
$inputBytes = [IO.File]::ReadAllBytes($inputPath)
$fixture = $utf8.GetString($inputBytes) | ConvertFrom-Json -AsHashtable
$outputPath = [IO.Path]::GetFullPath($OutputRoot)
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null
$binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
$reports = @()
foreach ($enabled in @($true, $false)) {
    $label = if ($enabled) { 'enabled' } else { 'disabled' }
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $binaryPath
    $start.WorkingDirectory = $fixture.workspace
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardInputEncoding = $utf8
    $start.StandardOutputEncoding = $utf8
    $start.StandardErrorEncoding = $utf8
    $start.Environment['CLEAN_CTX_PROJECT_ROOT'] = $fixture.workspace
    if ($enabled) { $start.Environment['CLEAN_CTX_PCG_PROFILE'] = $fixture.profile }
    else { $null = $start.Environment.Remove('CLEAN_CTX_PCG_PROFILE') }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    $status = 'failed'
    $failure = $null
    $exitCode = $null
    $stdout = ''
    $stderr = ''
    $started = $false
    $stdoutTask = $null
    $stderrTask = $null
    $stdoutLines = [Text.StringBuilder]::new()
    try {
        if (-not $process.Start()) { throw 'MCP process did not start' }
        $started = $true
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $initialize = @{jsonrpc='2.0';id=0;method='initialize';params=@{
            protocolVersion='2025-11-25';capabilities=@{};clientInfo=@{name='pcg-m4a-probe';version='1'}
        }}
        $process.StandardInput.WriteLine(($initialize | ConvertTo-Json -Depth 16 -Compress))
        $initTask = $process.StandardOutput.ReadLineAsync()
        if (-not $initTask.Wait(60000)) { throw 'MCP initialize timed out' }
        $initLine = $initTask.GetAwaiter().GetResult()
        $null = $stdoutLines.AppendLine($initLine)
        $init = $initLine | ConvertFrom-Json -AsHashtable
        if ($init.id -ne 0 -or $init.result.serverInfo.name -cne 'clean-ctx') { throw 'MCP initialization failed' }
        $process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}')
        $requests = @(
            @{jsonrpc='2.0';id=1;method='tools/list'},
            @{jsonrpc='2.0';id=2;method='tools/call';params=@{name='pcg_query_v1';arguments=$fixture.arguments}},
            @{jsonrpc='2.0';id=3;method='not_a_method'},
            @{jsonrpc='2.0';id=4;method='tools/call';params=@{name='list_sessions';arguments=@{}}}
        )
        foreach ($request in $requests) {
            $process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 32 -Compress))
        }
        # Keep stdin open until every response arrives; EOF is session shutdown.
        for ($responseIndex = 0; $responseIndex -lt $requests.Count; $responseIndex++) {
            $lineTask = $process.StandardOutput.ReadLineAsync()
            if (-not $lineTask.Wait(60000)) { throw 'Bounded MCP response timed out' }
            $line = $lineTask.GetAwaiter().GetResult()
            if ($null -eq $line) { throw 'Unexpected MCP stdout EOF' }
            $null = $stdoutLines.AppendLine($line)
        }
        $process.StandardInput.Close()
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        if (-not $process.WaitForExit(60000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw 'Bounded MCP probe timed out'
        }
        $exitCode = $process.ExitCode
        $stdout = $stdoutLines.ToString() + $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        if ($exitCode -ne 0) { throw 'MCP process failed' }
        $lines = @($stdout -split '\r?\n' | Where-Object { $_.Length -gt 0 })
        $responses = @{}
        $queryLine = $null
        foreach ($line in $lines) {
            $response = $line | ConvertFrom-Json -AsHashtable
            $responses[[int]$response.id] = $response
            if ($response.id -eq 2) { $queryLine = $line }
        }
        if ($responses.Count -ne 5) { throw 'Missing or unexpected MCP responses' }
        $tools = @($responses[1].result.tools)
        $pcg = @($tools | Where-Object { $_.name -eq 'pcg_query_v1' })
        $legacy = @($tools | Where-Object { $_.name -ne 'pcg_query_v1' })
        if (($legacy | ConvertTo-Json -Depth 100 -Compress) -cne ($fixture.legacy_tools | ConvertTo-Json -Depth 100 -Compress)) {
            throw 'Legacy tool descriptors changed'
        }
        if ($enabled) {
            if ($pcg.Count -ne 1) { throw 'Enabled PCG descriptor missing' }
            $prefix = '{"jsonrpc":"2.0","id":2,"result":'
            if (-not $queryLine.StartsWith($prefix, [StringComparison]::Ordinal) -or -not $queryLine.EndsWith('}')) {
                throw 'Unexpected PCG transport envelope'
            }
            $actual = $queryLine.Substring($prefix.Length, $queryLine.Length - $prefix.Length - 1)
            if (-not [String]::Equals($actual, [string]$fixture.expected_result, [StringComparison]::Ordinal)) {
                throw 'Delivered result differs from metered M3 bytes'
            }
        } else {
            if ($pcg.Count -ne 0 -or $responses[2].error.code -ne -32601) { throw 'Disabled PCG boundary changed' }
        }
        if ($responses[3].error.code -ne -32601) { throw 'Legacy method dispatch changed' }
        if ($responses[4].result.content[0].text -cne 'Persistence not enabled') { throw 'Legacy tool dispatch changed' }
        $status = 'succeeded'
    } catch { $failure = $_.Exception.Message }
    finally {
        if ($started -and -not $process.HasExited) { $process.Kill($true); $process.WaitForExit() }
        if ($stdoutTask) { $stdout = $stdoutLines.ToString() + $stdoutTask.GetAwaiter().GetResult() }
        else { $stdout = $stdoutLines.ToString() }
        if ($stderrTask) { $stderr = $stderrTask.GetAwaiter().GetResult() }
        [IO.File]::WriteAllText((Join-Path $outputPath "$label.stdout.jsonl"), $stdout, $utf8)
        [IO.File]::WriteAllText((Join-Path $outputPath "$label.stderr.txt"), $stderr, $utf8)
        $reports += @{mode=$label;status=$status;exit_code=$exitCode;error=$failure}
        [IO.File]::WriteAllText((Join-Path $outputPath 'receipt.json'), (@{binary_sha256=$binaryHash;input_sha256=(Get-FileHash -LiteralPath $inputPath).Hash.ToLowerInvariant();cases=$reports} | ConvertTo-Json -Depth 16), $utf8)
        $process.Dispose()
    }
    if ($status -ne 'succeeded') { throw $failure }
}
if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $binaryHash) { throw 'Binary changed during probe' }
Write-Output "M4a stdio receipt: $(Join-Path $outputPath 'receipt.json')"
