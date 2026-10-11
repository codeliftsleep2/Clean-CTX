param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [string]$ProviderPath = '/workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp',
    [string]$ExpectedHash = '6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0',
    [switch]$ContinueMatrix
)

# Linux A5 bootstrap-stage harness. -ContinueMatrix adds the safe session,
# routing, recovery and admission-exit cases; mutation/cohort mismatch skip.
# Uses the existing ProcessStartInfo/stdio MCP
# session pattern; strace observes the unmodified production provider boundary.
# No provider wrapper, private cache root, daemon kill or account repair.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (-not $IsLinux) { throw 'This acceptance harness is Linux-only.' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$ProviderPath = (Resolve-Path $ProviderPath).Path
if ((Get-FileHash $ProviderPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedHash) {
    throw 'A5 BLOCKED — ACCEPTANCE ARTIFACT CHANGED'
}
if ([Environment]::GetEnvironmentVariable('CBM_CACHE_DIR')) {
    throw 'Normal-account acceptance requires no CBM_CACHE_DIR override.'
}
# Fail preflight before provider execution if a turn-scoped platform write
# grant expired. Do not create the actual provider directory manually.
$cacheParent = Join-Path $HOME '.cache'
$probe = Join-Path $cacheParent ('a5-writability-' + [guid]::NewGuid())
try { [IO.File]::WriteAllText($probe, ''); [IO.File]::Delete($probe) }
catch { throw "Normal account cache is not writable: $($_.Exception.Message)" }
$runRoot = Join-Path $repo ('target/cbm-a5/' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
$primary = Join-Path $runRoot 'primary'
$additional = Join-Path $runRoot 'additional'
$sessions = [Collections.Generic.List[object]]::new()
$observations = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow

function Write-A5Text([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}
function Save-A5([string]$Name, $Value) {
    Write-A5Text (Join-Path $runRoot "$Name.json") ($Value | ConvertTo-Json -Depth 100)
}
function Assert-A5([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Check-A5Budget($Session) {
    if ([DateTime]::UtcNow -ge $script:deadline) { throw '90-second case observation deadline exceeded.' }
    if ((Test-Path $Session.Trace) -and (Get-Item $Session.Trace).Length -gt 12MB) {
        throw '12 MiB trace observation budget exceeded.'
    }
}
function Invoke-A5Rpc($Session, [string]$Method, [hashtable]$Params) {
    Check-A5Budget $Session
    $script:rpcId++
    $request = @{jsonrpc='2.0';id=$script:rpcId;method=$Method;params=$Params}
    $Session.Process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 40 -Compress))
    $Session.Process.StandardInput.Flush()
    while ($true) {
        $read = $Session.Process.StandardOutput.ReadLineAsync()
        while (-not $read.Wait(50)) { Check-A5Budget $Session }
        $line = $read.Result
        if ($null -eq $line) { throw "Clean-CTX stdout closed before $Method response." }
        Assert-A5 ($line.Length -le 4MB) 'Clean response exceeded evidence frame budget.'
        $response = $line | ConvertFrom-Json -Depth 100
        if ($response.PSObject.Properties['id'] -and $response.id -eq $script:rpcId) {
            $observations.Add(@{session=$Session.Name;request=$request;response=$response;utc=[DateTime]::UtcNow})
            Save-A5 'rpc' @($observations.ToArray())
            return $response
        }
    }
}
function Invoke-A5Tool($Session, [string]$Name, [hashtable]$Arguments) {
    Invoke-A5Rpc $Session 'tools/call' @{name=$Name;arguments=$Arguments}
}
function Start-A5Session([string]$Name, [bool]$AbortAdmission = $false) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = '/usr/bin/strace'
    $trace = Join-Path $runRoot "$Name.trace"
    foreach ($arg in @('-f','-ttt','-s','65536','-e','trace=process,chdir,write','-o',$trace,$BinaryPath)) {
        $info.ArgumentList.Add($arg)
    }
    $info.WorkingDirectory = $primary
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardInputEncoding = [Text.UTF8Encoding]::new($false)
    $info.StandardOutputEncoding = [Text.UTF8Encoding]::new($false)
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    Assert-A5 ($process.Start()) 'Unable to start traced Clean-CTX.'
    $session = [pscustomobject]@{Name=$Name;Process=$process;Trace=$trace;Stderr=$process.StandardError.ReadToEndAsync()}
    $sessions.Add($session)
    if ($AbortAdmission) {
        while ($true) {
            Check-A5Budget $session
            # Find the actual traced Clean parent from /proc.
            $clean = @(Get-A5Processes | Where-Object {
                $_.exe -eq $BinaryPath -and
                ([IO.File]::ReadAllText("/proc/$($_.pid)/status") -match "PPid:\s+$($process.Id)\s")
            })
            if ($clean.Count -eq 1) {
                $children = @(Get-A5Processes | Where-Object {
                    $_.exe -eq $ProviderPath -and $_.cmd.Trim() -eq $ProviderPath -and
                    ([IO.File]::ReadAllText("/proc/$($_.pid)/status") -match "PPid:\s+$($clean[0].pid)\s")
                })
                if ($children.Count -eq 1) {
                    Save-A5 'case9-owned-front-end' $children[0]
                    [Diagnostics.Process]::GetProcessById($children[0].pid).Kill()
                    break
                }
            }
            Start-Sleep -Milliseconds 10
        }
    }
    $init = Invoke-A5Rpc $session 'initialize' @{
        protocolVersion='2024-11-05';capabilities=@{};clientInfo=@{name='cbm-a5-acceptance';version='1.0.0'}
    }
    Assert-A5 (-not $init.PSObject.Properties['error']) 'Clean initialization failed.'
    $process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}')
    $process.StandardInput.Flush()
    return $session
}
function Stop-A5Session($Session) {
    if (-not $Session.Process.HasExited) {
        $Session.Process.StandardInput.Close()
        # EOF is the normal Clean shutdown path. A tracing process may outlive
        # Clean while observing the provider-owned daemon: detach only strace.
        if (-not $Session.Process.WaitForExit(5000)) { $Session.Process.Kill() }
    }
    if ($Session.Stderr.Wait(5000)) {
        $text = $Session.Stderr.Result
        Assert-A5 ($text.Length -le 1MB) 'Stderr evidence exceeded 1 MiB.'
        Write-A5Text (Join-Path $runRoot ($Session.Name + '.stderr')) $text
    }
}
function Get-A5Processes {
    $found = @()
    foreach ($directory in Get-ChildItem /proc -Directory -ErrorAction SilentlyContinue) {
        if ($directory.Name -notmatch '^\d+$') { continue }
        try {
            $exe = [IO.File]::ResolveLinkTarget((Join-Path $directory.FullName 'exe'), $true).FullName
            if ($exe -notin @($BinaryPath,$ProviderPath)) { continue }
            $cmd = [IO.File]::ReadAllText((Join-Path $directory.FullName 'cmdline')).Replace([char]0,' ')
            $cwd = [IO.File]::ResolveLinkTarget((Join-Path $directory.FullName 'cwd'),$true).FullName
            $found += @{pid=[int]$directory.Name;exe=$exe;cmd=$cmd;cwd=$cwd}
        } catch { continue }
    }
    return $found
}

New-Item -ItemType Directory -Force $primary,$additional | Out-Null
# Repository boundaries keep provider Git-root discovery inside owned fixtures.
foreach ($root in @($primary,$additional)) {
    & git -C $root init --quiet
    Assert-A5 ($LASTEXITCODE -eq 0) 'Fixture Git initialization failed.'
}
Write-A5Text (Join-Path $primary 'probe.ts') 'export function a5PrimaryProbe(): number { return 42; }'
Write-A5Text (Join-Path $additional 'probe.ts') 'export function a5AdditionalProbe(): string { return "additional"; }'
Write-A5Text (Join-Path $primary '.clean-ctx.json') (@{
    cbm=@{binary_path=$ProviderPath};additional_roots=@($additional)
} | ConvertTo-Json -Depth 20)
Save-A5 'preflight' @{
    provider=$ProviderPath;provider_sha256=$ExpectedHash;clean_binary=$BinaryPath
    clean_sha256=(Get-FileHash $BinaryPath).Hash;primary=$primary;additional=$additional
    processes=@(Get-A5Processes);cache_override_present=([bool][Environment]::GetEnvironmentVariable('CBM_CACHE_DIR'))
    normal_cache_exists=(Test-Path (Join-Path $HOME '.cache/codebase-memory-mcp'))
}
$result = 'INCOMPLETE'
try {
    $script:deadline = [DateTime]::UtcNow.AddSeconds(90)
    $a = Start-A5Session 'A'
    $status = Invoke-A5Tool $a 'get_cbm_status' @{}
    Save-A5 'case1-status' $status
    Save-A5 'case1-processes' @(Get-A5Processes)
    # Save evidence before interpreting status; failure stops the matrix.
    Assert-A5 ($status.result._meta.cbm_status -eq 'available') 'Provider did not reach available status; inspect admission trace and stderr.'
    $native = Invoke-A5Tool $a 'provide_code_context' @{workspaceRoot=$primary;filePath='probe.ts'}
    Save-A5 'case1-native' $native
    $result = 'BOOTSTRAP OBSERVED — REMAINING CASES REQUIRE TRACE REVIEW'
    if ($ContinueMatrix) {
        $script:deadline = [DateTime]::UtcNow.AddSeconds(90)
        $before = @(Get-A5Processes)
        Save-A5 'case3-before' $before
        $b = Start-A5Session 'B'
        Save-A5 'case3-after' @(Get-A5Processes)
        foreach ($s in @($a,$b)) {
            $projects = Invoke-A5Tool $s 'cbm_proxy' @{cbm_tool='list_projects';parameters=@{}}
            Save-A5 ('case4-projects-' + $s.Name) $projects
            Assert-A5 (-not $projects.PSObject.Properties['error']) 'Project request returned a protocol error.'
            Assert-A5 (-not ($projects.result.PSObject.Properties['isError'] -and $projects.result.isError)) 'Project request failed.'
        }
        $script:deadline = [DateTime]::UtcNow.AddSeconds(90)
        foreach ($item in @(@{root=$primary;symbol='a5PrimaryProbe'},@{root=$additional;symbol='a5AdditionalProbe'})) {
            $slug = $item.root.Trim('/').Replace('/','-')
            $answer = Invoke-A5Tool $a 'cbm_proxy' @{
                cbm_tool='search_graph';parameters=@{project=$slug;name_pattern=$item.symbol;format='json';limit=5}
            }
            Save-A5 ('case5-' + $item.symbol) $answer
            Assert-A5 (($answer | ConvertTo-Json -Depth 100 -Compress).Contains($item.symbol)) 'Explicit project routing lacked fixture evidence.'
        }
        Stop-A5Session $b
        Save-A5 'case6-after-retirement' @(Get-A5Processes)
        Save-A5 'case6-sibling' (Invoke-A5Tool $a 'cbm_proxy' @{cbm_tool='list_projects';parameters=@{}})
        $script:deadline = [DateTime]::UtcNow.AddSeconds(90)
        $clean = @(Get-A5Processes | Where-Object {$_.exe -eq $BinaryPath})
        Assert-A5 ($clean.Count -eq 1) 'Recovery fixture must have exactly one remaining owned Clean session.'
        $frontends = @(Get-A5Processes | Where-Object {
            $_.exe -eq $ProviderPath -and $_.cmd.Trim() -eq $ProviderPath -and
            ([IO.File]::ReadAllText("/proc/$($_.pid)/status") -match "PPid:\s+$($clean[0].pid)\s")
        })
        Assert-A5 ($frontends.Count -eq 1) 'Direct frontend ownership not uniquely established.'
        Save-A5 'case7-before' @(Get-A5Processes)
        [Diagnostics.Process]::GetProcessById($frontends[0].pid).Kill()
        $recovery = Invoke-A5Tool $a 'cbm_proxy' @{cbm_tool='list_projects';parameters=@{}}
        Save-A5 'case7-result' $recovery
        Save-A5 'case7-after' @(Get-A5Processes)
        Assert-A5 (-not $recovery.PSObject.Properties['error']) 'Recovery returned a protocol error.'
        Assert-A5 (-not ($recovery.result.PSObject.Properties['isError'] -and $recovery.result.isError)) 'Recovery did not return a usable result.'
        Save-A5 'case7-status' (Invoke-A5Tool $a 'get_cbm_status' @{})
        $script:deadline = [DateTime]::UtcNow.AddSeconds(90)
        $c = Start-A5Session 'C' $true
        $admission = Invoke-A5Tool $c 'get_cbm_status' @{}
        Save-A5 'case9-status' $admission
        Assert-A5 ($admission.result._meta.cbm_status -ne 'available') 'Killed admission became usable.'
        Save-A5 'case9-native' (Invoke-A5Tool $c 'provide_code_context' @{workspaceRoot=$primary;filePath='probe.ts'})
        Save-A5 'case9-sibling' (Invoke-A5Tool $a 'cbm_proxy' @{cbm_tool='list_projects';parameters=@{}})
        $result = 'MATRIX OBSERVATIONS CAPTURED — VERDICTS REQUIRE TRACE REVIEW'
    }
} catch {
    $result = 'FAIL: ' + $_.Exception.Message
    Write-Warning $result
} finally {
    foreach ($session in $sessions) { Stop-A5Session $session }
    Save-A5 'postflight' @{
        result=$result;provider_sha256=(Get-FileHash $ProviderPath).Hash.ToLowerInvariant()
        processes=@(Get-A5Processes);utc=[DateTime]::UtcNow
    }
    Write-Host "A5 evidence: $runRoot"
    Write-Host $result
}
if ($result.StartsWith('FAIL:')) { exit 1 }
