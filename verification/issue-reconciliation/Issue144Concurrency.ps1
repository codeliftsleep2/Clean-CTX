# Issue #144 live concurrent-request transport helper. The caller owns the
# session, deadline, evidence path, timing collection, and reverse-edge ceiling.
function Invoke-ConcurrentReversePair(
    $Session,
    [string]$Target,
    [string]$Evidence
) {
    Assert-B (-not $Session.Terminal) "Session $($Session.Name) is terminal."
    $pending = @{}
    foreach ($label in @('first', 'second')) {
        $script:rpcId++
        $id = $script:rpcId
        $request = @{
            jsonrpc = '2.0'
            id = $id
            method = 'tools/call'
            params = @{
                name = 'workspace_query'
                arguments = @{
                    workspaceRoot = $primary
                    type = 'reverse_edges'
                    domain = 'builtin'
                    entity_type = 'TypeRef'
                    name = $Target
                }
            }
        }
        Save-B ("$($Session.Name)-request-$id") $request
        $pending[[string]$id] = $label
        $Session.Process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 40 -Compress))
    }

    # Both frames must be outstanding before either response is consumed.
    $Session.Process.StandardInput.Flush()
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    $responses = @{}
    $notification = 0
    while ($pending.Count -gt 0) {
        $read = $Session.Process.StandardOutput.ReadLineAsync()
        $remaining = [Math]::Max(1, [int]($script:deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) {
            $Session.Terminal = $true
            throw "Concurrent MCP response deadline exceeded in $($Session.Name); inspect saved requests and stderr."
        }
        $line = $read.Result
        if ($null -eq $line) {
            $Session.Terminal = $true
            throw "Session $($Session.Name) closed stdout during concurrent requests."
        }
        Assert-B ([Text.Encoding]::UTF8.GetByteCount($line) -le 8MB) 'Harness response capture exceeded 8 MiB.'
        $response = $line | ConvertFrom-Json
        if (-not $response.PSObject.Properties['id']) {
            $notification++
            Save-B ("$Evidence-notification-$notification") $response
            continue
        }
        $responseId = [string]$response.id
        Assert-B ($pending.ContainsKey($responseId)) "Unexpected concurrent response id $responseId."
        $label = $pending[$responseId]
        Save-B ("$($Session.Name)-frame-$responseId") $response
        $toolResult = Get-BResult $response
        Assert-B ([bool]$toolResult.PSObject.Properties['structuredContent']) 'workspace_query omitted structuredContent.'
        $responses[$label] = $toolResult.structuredContent
        $pending.Remove($responseId)
    }
    $stopwatch.Stop()
    $timings.Add(@{
        evidence = $Evidence
        seconds = $stopwatch.Elapsed.TotalSeconds
        ceiling_seconds = $MaxReverseSeconds
        concurrent_requests = 2
    })
    Assert-B ($stopwatch.Elapsed.TotalSeconds -le $MaxReverseSeconds) (
        "$Evidence exceeded the concurrent reverse-edge latency ceiling: " +
        "$($stopwatch.Elapsed.TotalSeconds.ToString('F3'))s > $($MaxReverseSeconds.ToString('F3'))s."
    )
    $pair = [pscustomobject]@{
        first = $responses['first']
        second = $responses['second']
        seconds = $stopwatch.Elapsed.TotalSeconds
    }
    Save-B $Evidence $pair
    return $pair
}
