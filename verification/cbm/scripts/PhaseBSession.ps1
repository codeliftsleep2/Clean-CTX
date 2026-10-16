# Reuses the ProcessStartInfo/stdio ownership pattern from McpSession.ps1 and A5.
function Write-BText([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}
function Assert-B([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Save-B([string]$Name, $Value) {
    Write-BText (Join-Path $runRoot "$Name.json") ($Value | ConvertTo-Json -Depth 100)
}
function Invoke-BRpc($Session, [string]$Method, [hashtable]$Params) {
    Assert-B (-not $Session.Terminal) "Session $($Session.Name) is terminal."
    $script:rpcId++
    $request = @{jsonrpc='2.0';id=$script:rpcId;method=$Method;params=$Params}
    Save-B ("$($Session.Name)-request-$rpcId") $request
    $Session.Process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 40 -Compress))
    $Session.Process.StandardInput.Flush()
    while ($true) {
        $read = $Session.Process.StandardOutput.ReadLineAsync()
        $remaining = [Math]::Max(1, [int]($script:deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) {
            $Session.Terminal = $true
            throw "MCP response deadline exceeded in $($Session.Name)/$Method; inspect the saved request and stderr."
        }
        $line = $read.Result
        if ($null -eq $line) { $Session.Terminal=$true; throw "Session $($Session.Name) closed stdout." }
        Assert-B ([Text.Encoding]::UTF8.GetByteCount($line) -le 8MB) 'Harness response capture exceeded 8 MiB.'
        $response = $line | ConvertFrom-Json
        Save-B ("$($Session.Name)-frame-$rpcId") $response
        if ($response.PSObject.Properties['id'] -and $response.id -eq $rpcId) { return $response }
        Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Notification stream exhausted the case deadline.'
    }
}
function Invoke-BTool($Session, [string]$Name, [hashtable]$Arguments) {
    Invoke-BRpc $Session 'tools/call' @{name=$Name;arguments=$Arguments}
}
function Test-BIndexConflict($Response) {
    $message=$null
    if ($Response.PSObject.Properties['error']) {
        $message=[string]$Response.error.message
    } elseif ($Response.PSObject.Properties['result'] -and
        $Response.result.PSObject.Properties['structuredContent'] -and
        $Response.result.structuredContent.PSObject.Properties['error']) {
        $message=[string]$Response.result.structuredContent.error
    }
    return $null -ne $message -and
        $message.EndsWith('another index operation for this project is active with different options')
}
function Invoke-BCleanIndexRepository($Session, [string]$RepoPath) {
    do {
        $response=Invoke-BTool $Session 'cbm_proxy' @{
            cbm_tool='index_repository';parameters=@{repo_path=$RepoPath;mode='fast'}
        }
        $conflict=Test-BIndexConflict $response
        if ($conflict) {
            Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Clean-CTX provider auto-index did not yield before the case deadline.'
            Start-Sleep -Milliseconds 250
        }
    } while ($conflict)
    return Get-BBody (Get-BResult $response)
}
function Invoke-BRawTool([string]$Name, [hashtable]$Arguments) {
    Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool=$Name;parameters=$Arguments}
}
function Invoke-BReadyTool($Session, [string]$Name, [hashtable]$Arguments) {
    # An operator startup gate, not a model polling loop or implicit job start.
    # Every attempt is captured separately; only the explicit temporary flag
    # permits a bounded retry under this case's existing observation deadline.
    do {
        $response=Invoke-BTool $Session $Name $Arguments
        $waiting=$response.PSObject.Properties['result'] -and $response.result.PSObject.Properties['_meta'] -and
            $response.result._meta.PSObject.Properties['still_indexing'] -and $response.result._meta.still_indexing
        if ($waiting) {
            Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Clean startup/index gate exceeded case deadline.'
            Start-Sleep -Milliseconds 250
        }
    } while ($waiting)
    return $response
}
function Get-BResult($Response) {
    Assert-B (-not $Response.PSObject.Properties['error']) "RPC failure: $($Response | ConvertTo-Json -Depth 100 -Compress)"
    Assert-B ($null -ne $Response.PSObject.Properties['result']) 'Missing MCP result.'
    Assert-B (-not ($Response.result.PSObject.Properties['isError'] -and $Response.result.isError)) "Tool failure: $($Response.result | ConvertTo-Json -Depth 100 -Compress)"
    return $Response.result
}
function Get-BBody($Result) {
    # B4 unwraps only sole unannotated text. Additional provider channels remain
    # a compact JSON result envelope in the existing Clean text channel.
    $body = $Result.content[0].text | ConvertFrom-Json
    if ($body.PSObject.Properties['content']) { return ($body.content[0].text | ConvertFrom-Json) }
    return $body
}
function Get-BTextBody($Result) {
    $text=[string]$Result.content[0].text
    try {
        $body=$text | ConvertFrom-Json
        if ($body.PSObject.Properties['content']) { return [string]$body.content[0].text }
    } catch { # Intentional non-JSON provider text remains verbatim.
    }
    return $text
}
function Start-BSession([string]$Name, [string]$Executable, [string]$Root) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$Executable; $info.WorkingDirectory=$Root
    $info.UseShellExecute=$false; $info.RedirectStandardInput=$true
    $info.RedirectStandardOutput=$true; $info.RedirectStandardError=$true
    # Windows PowerShell 5.1 runs on .NET Framework, whose ProcessStartInfo
    # lacks StandardInputEncoding. PowerShell 7/.NET exposes both properties.
    # Preserve explicit UTF-8 where supported without preventing the shared
    # verification harnesses from starting on the older maintained host.
    $utf8=[Text.UTF8Encoding]::new($false)
    if ($info.PSObject.Properties['StandardInputEncoding']) {
        $info.StandardInputEncoding=$utf8
    }
    if ($info.PSObject.Properties['StandardOutputEncoding']) {
        $info.StandardOutputEncoding=$utf8
    }
    $process=[Diagnostics.Process]::new(); $process.StartInfo=$info
    Assert-B ($process.Start()) "Could not launch $Executable"
    $session=[pscustomobject]@{Name=$Name;Process=$process;Stderr=$process.StandardError.ReadToEndAsync();Terminal=$false}
    $sessions.Add($session)
    $null=Get-BResult (Invoke-BRpc $session 'initialize' @{
        protocolVersion='2024-11-05';capabilities=@{};clientInfo=@{name='cbm-phase-b-verification';version='1.0.0'}
    })
    $process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}')
    $process.StandardInput.Flush()
    return $session
}
function Stop-BSession($Session) {
    if (-not $Session.Process.HasExited) {
        $Session.Process.StandardInput.Close()
        if (-not $Session.Process.WaitForExit(5000)) { $Session.Process.Kill() }
    }
    if ($Session.Stderr.Wait(5000)) {
        Assert-B ($Session.Stderr.Result.Length -le 4MB) 'Stderr capture exceeded harness budget.'
        Write-BText (Join-Path $runRoot "$($Session.Name).stderr.log") $Session.Stderr.Result
    }
    $Session.Process.Dispose()
}
function Invoke-BCase([string]$Phase,[string]$Name,[scriptblock]$Action) {
    $script:deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
    Write-Host "$Phase / $Name"
    try { . $Action; $verdicts.Add(@{phase=$Phase;case=$Name;result='PASS'}) }
    catch { $verdicts.Add(@{phase=$Phase;case=$Name;result='FAIL';evidence=$_.Exception.Message}); Write-Warning $_.Exception.Message }
    Save-B 'verdicts' @($verdicts.ToArray())
}
function Get-BGroupedRows($Body) {
    Assert-B ($Body.qn_rule -eq 'qn = qn_prefix == "" ? name : qn_prefix + "." + name') 'Unexpected QN rule.'
    $rows=[Collections.Generic.List[object]]::new()
    foreach ($group in $Body.groups) {
        foreach ($row in $group.rows) {
            Assert-B ($row.Count -eq $Body.cols.Count) 'Grouped row width mismatch.'
            $item=@{}
            for ($i=0;$i -lt $Body.cols.Count;$i++) { $item[$Body.cols[$i]]=$row[$i] }
            $item.qualified_name=if ($group.qn_prefix -eq '') {$item.name} else {"$($group.qn_prefix).$($item.name)"}
            if ($group.PSObject.Properties['file']) { $item.file_path=$group.file }
            $rows.Add($item)
        }
    }
    return ,$rows.ToArray()
}
function Get-BExactQuery([string]$Project,[string]$Query) {
    # Raw provider-shaped oracle through the sole Clean-CTX session; same
    # immutable args, emitted opaque cursors, no offset fallback or stale restart.
    $toolArgs=@{project=$Project;query=$Query;format='json';max_rows=17}
    $rows=[Collections.Generic.List[object]]::new();$seen=[Collections.Generic.HashSet[string]]::new()
    $columns=$null;$pages=0;$bytes=0
    do {
        $body=Get-BBody (Get-BResult (Invoke-BRawTool 'query_graph' $toolArgs))
        $pages++;$bytes+=([Text.Encoding]::UTF8.GetByteCount(($body | ConvertTo-Json -Depth 100 -Compress)))
        Assert-B ($bytes -le 4MB) 'Provider oracle exceeded 4 MiB.'
        if ($null -eq $columns) {$columns=@($body.columns)}
        Assert-B (($columns | ConvertTo-Json -Compress) -eq (@($body.columns) | ConvertTo-Json -Compress)) 'Provider column drift.'
        foreach ($row in $body.rows) { $rows.Add($row) }
        Assert-B ($body.total_relation -eq 'eq') 'Provider exact query total unavailable.'
        if ($body.has_more) {
            Assert-B ($body.PSObject.Properties['next_cursor'] -and $body.next_cursor -and $body.returned -gt 0) 'Missing/nonprogressing opaque cursor.'
            Assert-B ($seen.Add($body.next_cursor)) 'Repeated provider cursor.'
            $toolArgs.cursor=$body.next_cursor
        }
    } while ($body.has_more)
    Assert-B (-not $body.truncated -and $rows.Count -eq $body.total) 'Provider oracle remains partial.'
    return @{columns=$columns;rows=$rows.ToArray();pages=$pages;total=$body.total}
}

function Convert-BStable($Value) {
    if ($null -eq $Value) { return $null }
    if ($Value -is [Collections.IDictionary]) {
        $ordered=[ordered]@{}
        foreach ($key in @($Value.Keys | Sort-Object)) { $ordered[$key]=Convert-BStable $Value[$key] }
        return $ordered
    }
    if ($Value -is [pscustomobject]) {
        $ordered=[ordered]@{}
        foreach ($property in @($Value.PSObject.Properties | Sort-Object Name)) { $ordered[$property.Name]=Convert-BStable $property.Value }
        return $ordered
    }
    if ($Value -is [array]) {
        $items=[Collections.Generic.List[object]]::new()
        foreach ($item in $Value) { $items.Add((Convert-BStable $item)) }
        return ,$items.ToArray()
    }
    return $Value
}
function Get-BExactTrace([string]$Project,[string]$Function) {
    $toolArgs=@{project=$Project;function_name=$Function;direction='both';depth=3;limit=17;format='json'}
    $rows=[Collections.Generic.List[object]]::new();$seen=[Collections.Generic.HashSet[string]]::new()
    $pages=0;$bytes=0
    do {
        $body=Get-BBody (Get-BResult (Invoke-BRawTool 'trace_path' $toolArgs))
        $pages++;$bytes+=[Text.Encoding]::UTF8.GetByteCount(($body | ConvertTo-Json -Depth 100 -Compress))
        Assert-B ($bytes -le 4MB) 'Trace oracle exceeded 4 MiB.'
        foreach ($row in (Get-BGroupedRows $body.callees)) { $rows.Add($row) }
        Assert-B ($body.callees_total_relation -eq 'eq') 'Trace oracle total is lower-bound.'
        $more=$body.PSObject.Properties['has_more'] -and $body.has_more
        if ($more) {
            Assert-B ($body.PSObject.Properties['next_cursor'] -and $body.next_cursor) 'Missing trace cursor.'
            Assert-B ($seen.Add($body.next_cursor)) 'Trace cursor repeated.'
            $toolArgs.cursor=$body.next_cursor
        }
    } while ($more)
    Assert-B (-not ($body.PSObject.Properties['truncated'] -and $body.truncated) -and $rows.Count -eq $body.callees_total) 'Trace oracle remains partial.'
    return @{rows=$rows.ToArray();pages=$pages}
}
