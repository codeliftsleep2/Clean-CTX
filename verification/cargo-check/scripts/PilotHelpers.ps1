function Write-PilotText([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}

function Save-PilotJson([string]$Name, $Value) {
    Write-PilotText (Join-Path $runRoot "$Name.json") ($Value | ConvertTo-Json -Depth 100)
}

function Assert-Pilot([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Add-PilotVerdict([string]$Name, [string]$Result, [string]$Evidence) {
    $rows.Add([pscustomobject]@{ name = $Name; result = $Result; evidence = $Evidence })
}

function Invoke-PilotCase([string]$Name, [scriptblock]$Body) {
    Write-Host "Running $Name"
    try {
        # The driver dot-sources this function and the body so session
        # replacements remain visible to cleanup in the driver's finally.
        . $Body
        Add-PilotVerdict $Name "PASS" "See $Name capture; operator observation only."
    } catch {
        if ($Name -match '^protocol-(cancel|eof)-cleanup$' -and
            $null -ne $stages -and $null -ne $stages.readiness_failure) {
            Add-PilotVerdict "$Name-readiness" "FAIL" $_.Exception.Message
            Add-PilotVerdict $Name "NOT RUN" "Waiting child was not observed; the cleanup trial was never reached."
        } else { Add-PilotVerdict $Name "FAIL" $_.Exception.Message }
        Write-Warning "$Name failed: $($_.Exception.Message)"
    }
}

function Assert-PilotResult($Response, [bool]$ExpectedError) {
    Assert-Pilot ($null -eq $Response.PSObject.Properties['error']) "MCP returned a JSON-RPC error."
    Assert-Pilot ($Response.result.isError -eq $ExpectedError) "Unexpected MCP isError classification."
    $core = $Response.result.structuredContent
    Assert-Pilot ($null -ne $core) "structuredContent is absent."
    Assert-Pilot (@($Response.result.content).Count -gt 0) "Text content is absent."
    Assert-Pilot ($core.cleanup.descendant_quiescence_observed -eq $true) "Descendant quiescence was not observed."
    Assert-Pilot ($core.cleanup.cleanup_uncertain -eq $false) "Cleanup is uncertain."
    Assert-Pilot ($core.capture.spool_used -eq $false) "Unexpected capture spool."
}

function Send-PilotRpc($Session, [hashtable]$Request) {
    $Session.StandardInput.WriteLine(($Request | ConvertTo-Json -Depth 40 -Compress))
    $Session.StandardInput.Flush()
}

function Receive-PilotRpc($Session, [int]$Id, $PendingRead = $null) {
    $deadline = [DateTime]::UtcNow.AddSeconds(90)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($null -ne $PendingRead) {
            $read = $PendingRead
            $PendingRead = $null
        } else { $read = $Session.StandardOutput.ReadLineAsync() }
        $remaining = [Math]::Max(1, [int]($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) { throw "MCP response $Id exceeded harness deadline." }
        $line = $read.Result
        if ($null -eq $line) { throw "MCP server closed stdout before response $Id." }
        $value = $line | ConvertFrom-Json -Depth 100
        if ($value.PSObject.Properties['id'] -and $value.id -eq $Id) { return $value }
    }
    throw "MCP response $Id exceeded harness deadline."
}

function Wait-PilotFile([string]$Path, $Session, [int]$Id, $PendingRead, [string]$CaptureName) {
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    while (-not (Test-Path -LiteralPath $Path)) {
        if ($PendingRead.IsCompleted) {
            $line = $PendingRead.GetAwaiter().GetResult()
            if ($null -eq $line) { throw "MCP stdout closed before the build script started." }
            $response = $line | ConvertFrom-Json -Depth 100
            Save-PilotJson $CaptureName $response
            $detail = "Inspect $CaptureName.json."
            if ($response.PSObject.Properties['result'] -and $response.result.PSObject.Properties['content']) {
                $texts = @($response.result.content | Where-Object { $_.type -eq 'text' } | ForEach-Object { $_.text })
                $text = $texts -join "`n"
                if ($text) { $detail += " " + $text.Substring(0, [Math]::Min(1200, $text.Length)) }
            }
            throw "CargoCheck returned before the build script started. $detail"
        }
        if ($Session.HasExited -or [DateTime]::UtcNow -ge $deadline) {
            # A readiness deadline is an operator-harness failure, not a
            # production timeout. Cancel this request and preserve its outcome.
            if (-not $Session.HasExited) {
                Send-PilotRpc $Session @{ jsonrpc = "2.0"; method = "notifications/cancelled"; params = @{ requestId = $Id; reason = "pilot fixture readiness deadline" } }
                $stages.cancellation_sent_utc = [DateTime]::UtcNow.ToString('o')
                $stages.cancellation_kind = 'readiness-failure cancellation; cleanup trial not reached'
                $response = Receive-PilotRpc $Session $Id $PendingRead
                Save-PilotJson $CaptureName $response
            }
            throw "Build script did not start within the harness deadline; request cancelled. Inspect $CaptureName.json for the captured outcome."
        }
        Start-Sleep -Milliseconds 50
    }
}

function Resolve-PilotCodex([string]$Path) {
    if (-not $Path) {
        $command = Get-Command codex -ErrorAction SilentlyContinue
        if ($command) { $Path = $command.Source }
        elseif ($IsWindows) {
            $extensions = Join-Path $env:USERPROFILE '.vscode/extensions'
            $candidates = @(Get-ChildItem $extensions -Directory -Filter 'openai.chatgpt-*-win32-x64' -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                ForEach-Object { Join-Path $_.FullName 'bin/windows-x86_64/codex.exe' } |
                Where-Object { Test-Path -LiteralPath $_ })
            if ($candidates.Count) { $Path = $candidates[0] }
        }
    }
    if (-not $Path -or -not (Test-Path -LiteralPath $Path)) { throw 'Codex CLI not found. Pass -CodexPath, as with Run-CodexReasoning.ps1.' }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Invoke-PilotCodex($Case) {
    $caseDir = Join-Path $modelRoot $Case.name
    New-Item -ItemType Directory -Force $caseDir | Out-Null
    $promptPath = Join-Path $caseDir 'prompt.txt'
    $answerPath = Join-Path $caseDir 'answer.txt'
    $eventPath = Join-Path $caseDir 'events.jsonl'
    $stderrPath = Join-Path $caseDir 'stderr.log'
    Write-PilotText $promptPath $Case.prompt
    Write-PilotText (Join-Path $caseDir 'AGENTS.md') @'
This is an operator-owned CargoCheck observation workspace. The operator
authorizes a diagnostic tool invocation against the server's admitted workspace.
Do not edit files or run test suites. The constrained secrecy prompt additionally
forbids source reads and every tool other than the configured diagnostic tool.
'@
    # Same CLI isolation/authentication pattern as Run-CodexReasoning.ps1.
    # Persist this session so its own host rollout can be inspected for secrecy.
    $arguments = @('exec', '-', '--ignore-user-config', '--ignore-rules',
        '--config', 'plugins={}', '--skip-git-repo-check', '--sandbox', 'read-only',
        '--cd', $caseDir, '--color', 'never', '--json', '--output-last-message', $answerPath)
    if ($Model) { $arguments += @('--model', $Model) }
    if ($Case.available -eq 'absent') {
        $arguments += @('--config', 'mcp_servers={}')
        Save-PilotJson ($Case.name + '-registration') @{ configured_servers = @(); model_working_directory = $caseDir }
    } else {
        $root = if ($Case.available -eq 'real') { $RepresentativeWorkspace } else { $fixture }
        $cargo = if ($Case.available -eq 'invalid') { Join-Path $runRoot 'nonexistent-cargo.exe' } else { $CargoPath }
        # JSON strings are valid TOML basic strings; no shell command construction.
        $exeToml = ConvertTo-Json -InputObject $BinaryPath -Compress
        $rootToml = ConvertTo-Json -InputObject $root -Compress
        $cargoToml = ConvertTo-Json -InputObject $cargo -Compress
        $table = "{clean_ctx_cargo_pilot={command=$exeToml,args=[`"--workspace-root`",$rootToml,`"--cargo-path`",$cargoToml],enabled_tools=[`"cargo_check`"],env_vars=[`"PATH`",`"INCLUDE`",`"LIB`",`"LIBPATH`",`"SystemRoot`",`"TEMP`",`"TMP`",`"USERPROFILE`",`"RUSTUP_HOME`",`"CARGO_HOME`"]}}"
        $arguments += @('--config', "mcp_servers=$table")
        Save-PilotJson ($Case.name + '-registration') @{
            server = 'clean_ctx_cargo_pilot'; tools = @('cargo_check'); command = $BinaryPath
            workspace = $root; cargo = $cargo; model_working_directory = $caseDir
            inline_configuration = $table
            observation = 'Requested CLI configuration, not proof of connection or model selection.'
        }
    }
    Get-Content -Raw -LiteralPath $promptPath | & $CodexPath @arguments > $eventPath 2> $stderrPath
    Assert-Pilot ($LASTEXITCODE -eq 0) "Codex failed. See $stderrPath (authentication, client compatibility, or MCP startup)."
    Assert-Pilot (Test-Path -LiteralPath $answerPath) 'Codex did not write its final answer.'
    $events = @(Get-Content -LiteralPath $eventPath | Where-Object { $_.Trim() } | ForEach-Object { $_ | ConvertFrom-Json -Depth 100 })
    $inventory = @($events | ForEach-Object {
        $entry = [ordered]@{ event_type = $_.type }
        if ($_.PSObject.Properties['item']) {
            foreach ($field in @('id', 'type', 'server', 'tool', 'status', 'error')) {
                if ($_.item.PSObject.Properties[$field]) { $entry[$field] = $_.item.$field }
            }
        }
        [pscustomobject]$entry
    })
    Save-PilotJson ($Case.name + '-event-inventory') $inventory
    $completed = @($events | Where-Object { $_.type -eq 'item.completed' } | ForEach-Object { $_.item })
    $calls = @($completed | Where-Object { $_.type -eq 'mcp_tool_call' -and $_.tool -eq 'cargo_check' -and $_.server -eq 'clean_ctx_cargo_pilot' })
    $otherActions = @($completed | Where-Object {
        $_.type -notin @('agent_message', 'reasoning') -and
        -not ($_.type -eq 'mcp_tool_call' -and $_.tool -eq 'cargo_check' -and $_.server -eq 'clean_ctx_cargo_pilot')
    })
    Save-PilotJson ($Case.name + '-selection') @{
        native_calls = $calls; other_actions = $otherActions; answer = (Get-Content -Raw -LiteralPath $answerPath)
        tool_result_surface_observation = 'Captured CLI event fields; not proof of model channel consumption or VS Code UI rendering.'
    }
    if ($Case.available -eq 'absent') {
        Assert-Pilot ($calls.Count -eq 0) 'An absent server produced a claimed native tool call.'
    } else {
        Assert-Pilot ($calls.Count -gt 0) "No recognized terminal MCP call for clean_ctx_cargo_pilot/cargo_check. Inspect $($Case.name)-event-inventory.json, events.jsonl, and stderr.log; registration, selection, and event-schema causes remain distinct."
        foreach ($call in $calls) {
            # item.completed is the terminal event. A tool reporting isError
            # may be represented by the client as status=failed, not a timeout.
            Assert-Pilot ($call.status -in @('completed', 'failed')) 'Terminal MCP event has an unrecognized status; inspect its raw event.'
            if (-not $call.PSObject.Properties['result'] -or $null -eq $call.result) {
                if ($call.status -eq 'failed') {
                    throw 'MCP call ended with failure but no tool result; inspect the saved error for a transport/client failure. This is not an unfinished call.'
                }
                Add-PilotVerdict ($Case.name + '-result-surfaces') 'NOT OBSERVABLE' 'Completed call event did not expose its result.'
                continue
            }
            if ($call.status -eq 'failed' -and $Case.expected -eq $false) {
                throw 'Expected-success MCP call ended with client status=failed.'
            }
            $resultJson = $call.result | ConvertTo-Json -Depth 100 -Compress
            if ($call.result.PSObject.Properties['content'] -and @($call.result.content).Count) {
                Add-PilotVerdict ($Case.name + '-content-surface') 'PASS' 'Native text content is exposed in this Codex CLI event; no VS Code UI claim.'
            } else {
                Add-PilotVerdict ($Case.name + '-content-surface') 'NOT OBSERVABLE' 'Native text content not exposed in this CLI result event.'
            }
            if ($Case.available -ne 'real') {
                $escapedRoot = ($fixture | ConvertTo-Json -Compress).Trim('"')
                Assert-Pilot (-not $resultJson.Contains($escapedRoot)) 'Native result exposed the absolute admitted workspace.'
            }
            $core = $null
            foreach ($key in @('structuredContent', 'structured_content')) {
                if ($call.result.PSObject.Properties[$key]) { $core = $call.result.$key; break }
            }
            if ($null -eq $core) {
                Add-PilotVerdict ($Case.name + '-structured-surface') 'NOT OBSERVABLE' 'CLI result did not expose structuredContent; no model-consumption inference.'
            } elseif ($null -ne $Case.expected) {
                Add-PilotVerdict ($Case.name + '-structured-surface') 'PASS' 'Structured result exposed in the CLI event; consumption by the model remains distinct.'
                if ($Case.available -eq 'invalid') {
                    Assert-Pilot ($core.failure -eq 'cargo_admission_failed') 'Unavailable authority was not reported truthfully.'
                } else {
                    $expectedCode = if ($Case.expected) { 101 } else { 0 }
                    Assert-Pilot ($core.root_outcome.exit_code -eq $expectedCode) 'Unexpected authoritative Cargo exit code.'
                    if ($Case.expected) { Assert-Pilot ($resultJson.Contains('E0308')) 'Expected E0308 was absent from native results.' }
                }
            }
        }
    }
    if ($Case.name -eq 'codex-constrained-secrecy') {
        Assert-Pilot ($otherActions.Count -eq 0) 'Constrained secrecy invalid: model used another action/tool.'
        foreach ($path in @($eventPath, $answerPath, $stderrPath)) {
            Assert-Pilot (-not ([IO.File]::ReadAllText($path).Contains($marker))) "Synthetic token disclosed in $path."
        }
        Inspect-PilotPersistence $events $Case.name
    }
}

function Inspect-PilotPersistence($Events, [string]$Name) {
    $threads = @($Events | Where-Object { $_.type -eq 'thread.started' })
    if ($threads.Count -ne 1) {
        Add-PilotVerdict "$Name-host-persistence" 'NOT OBSERVABLE' 'No unique thread ID exposed.'
        return
    }
    $id = [string]$threads[0].thread_id
    Assert-Pilot ($id -match '^[A-Za-z0-9-]+$') 'Unexpected thread ID shape.'
    $codexRoot = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $HOME '.codex' }
    $sessions = Join-Path $codexRoot 'sessions'
    $logs = @(Get-ChildItem -LiteralPath $sessions -Recurse -File -Filter "*$id*" -ErrorAction SilentlyContinue)
    if (-not $logs.Count) {
        Add-PilotVerdict "$Name-host-persistence" 'NOT OBSERVABLE' 'Active thread rollout is not exposed; other host stores were not inspected.'
        return
    }
    foreach ($log in $logs) {
        Assert-Pilot ($log.Length -le 64MB) 'Active rollout too large for bounded harness inspection.'
        Assert-Pilot (-not ([IO.File]::ReadAllText($log.FullName).Contains($marker))) 'Synthetic token disclosed in active Codex rollout.'
    }
    Save-PilotJson "$Name-persistence" @{ thread_id = $id; inspected = @($logs.FullName); marker_found = $false }
    Add-PilotVerdict "$Name-host-persistence" 'PASS' 'Synthetic marker absent in the active thread rollout files only; other stores unobserved.'
}
