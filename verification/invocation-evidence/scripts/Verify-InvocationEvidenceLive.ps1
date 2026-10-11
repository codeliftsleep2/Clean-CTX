[CmdletBinding()]
param(
    [string]$BinaryPath = '',
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path,
    [ValidateRange(5, 300)][int]$RequestTimeoutSeconds = 90,
    [ValidateSet('all','0','1','2','3','4','5','6','7')][string]$Phase = 'all',
    [switch]$KeepWorkspace
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# Reuse the established initialize/initialized, tool invocation and shutdown pattern.
. (Join-Path $RepositoryRoot 'verification/context-compression/scripts/McpSession.ps1')

# Same RPC protocol, with an operator-facing deadline and notification-safe IDs.
# Start-CleanCtxSession also uses this function for its initialize request.
function Invoke-CleanCtxRpc {
    param($Session, [int]$Id, [string]$Method, [hashtable]$Params)
    if ($Method -eq 'initialize') { $script:session = $Session }
    $Session.StandardInput.WriteLine((@{
        jsonrpc = '2.0'; id = $Id; method = $Method; params = $Params
    } | ConvertTo-Json -Depth 40 -Compress))
    $Session.StandardInput.Flush()
    $deadline = [DateTime]::UtcNow.AddSeconds($RequestTimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $read = $Session.StandardOutput.ReadLineAsync()
        $remaining = [Math]::Max(1, [int]($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) { throw "$Method request $Id exceeded ${RequestTimeoutSeconds}s; see server stderr above." }
        if ($null -eq $read.Result) { throw "MCP server closed stdout before $Method response $Id." }
        try { $value = $read.Result | ConvertFrom-Json -Depth 100 } catch { continue }
        if ($value.PSObject.Properties['id'] -and $value.id -eq $Id) { return $value }
    }
    throw "$Method request $Id exceeded its deadline."
}

if (-not $BinaryPath) {
    $name = if ($IsWindows) { 'clean-ctx.exe' } else { 'clean-ctx' }
    $BinaryPath = Join-Path $RepositoryRoot "target/debug/$name"
}
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw 'Build the production binary first: cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4'
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$binaryTime = (Get-Item -LiteralPath $BinaryPath).LastWriteTimeUtc
$changedSources = @(Get-ChildItem (Join-Path $RepositoryRoot 'src') -Recurse -File -Filter '*.rs' |
    Where-Object { $_.LastWriteTimeUtc -gt $binaryTime })
if ($changedSources.Count) { throw 'Production binary is stale. Rebuild with the command above.' }
$stamp = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$runRoot = Join-Path $RepositoryRoot "target/invocation-evidence-verification/$stamp"
$captures = Join-Path $runRoot 'captures'
$workspace = Join-Path ([IO.Path]::GetTempPath()) ("clean-ctx-invocation-live-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force $workspace, $captures | Out-Null
$fixtureRoot = Join-Path $RepositoryRoot 'verification/invocation-evidence/fixtures'
Copy-Item (Join-Path $fixtureRoot '*') $workspace
$session = $null
$script:nextId = 0
$rows = [Collections.Generic.List[object]]::new()
$script:planned = 0
$script:selected = 0

function Write-Utf8([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}
function Save-Capture([string]$Name, $Value) {
    Write-Utf8 (Join-Path $captures "$Name.json") ($Value | ConvertTo-Json -Depth 100)
}
function Assert-Live([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Invoke-Case([string]$Name, [scriptblock]$Body) {
    $script:planned += 1
    $casePhase = if ($Name -match '^phase([0-7])-') { $Matches[1] } else {
        switch -Wildcard ($Name) {
            'typescript-*' { '2' }; 'csharp-*' { '3' }; 'rxjs-*' { '5' }
            '*config' { '5' }; 'changed-*' { '6' }; 'angular-*' { '0' }
            default { '7' }
        }
    }
    if ($Phase -ne 'all' -and $Phase -ne $casePhase) { return }
    $script:selected += 1
    Write-Host "Running $Name (phase $casePhase)"
    try {
        & $Body
        $rows.Add([pscustomobject]@{ name = $Name; phase = $casePhase; status = 'PASS'; evidence = 'See captures; live stdio observation.' })
        Write-Host "PASS: $Name"
    } catch {
        $rows.Add([pscustomobject]@{ name = $Name; phase = $casePhase; status = 'FAIL'; evidence = $_.Exception.Message })
        Write-Warning "$Name failed: $($_.Exception.Message)"
    }
}
function Invoke-Live([string]$Name, [string]$Tool, [hashtable]$Arguments, [switch]$AllowError) {
    $script:nextId += 1
    Save-Capture "$Name-request" @{ tool = $Tool; arguments = $Arguments }
    $response = Invoke-CleanCtxTool $session $script:nextId $Tool $Arguments
    Save-Capture $Name $response
    if (-not $AllowError) {
        Assert-Live (-not $response.PSObject.Properties['error']) "$Name returned a JSON-RPC error."
        Assert-Live ($null -ne $response.PSObject.Properties['result']) "$Name has no result."
        $errorFlag = $response.result.PSObject.Properties['isError']
        Assert-Live (-not $errorFlag -or -not $errorFlag.Value) "$Name returned an MCP tool error."
        Assert-Live (@($response.result.content).Count -gt 0) "$Name has no model-facing content."
    }
    return $response
}
function Compress([string]$Name, [string]$File, [string]$Fidelity = 'high') {
    Invoke-Live $Name 'compress_code_context' @{
        workspaceRoot = $workspace; filePath = (Join-Path $workspace $File)
        fidelity = $Fidelity; encoding = 'named'; tokenizer = 'o200k'
    }
}
function Pipes($Response) {
    Assert-Live ($null -ne $Response.result.PSObject.Properties['pretty']) 'Named canonical projection missing.'
    @($Response.result.pretty.ir | Where-Object { $_[0] -eq 'TYPE' -and $_[1] -eq '@pipeRx' } |
        ForEach-Object { [string]$_[2] })
}
function Calls([string]$Name, [string]$File, [string]$Owner, [string]$Method = 'Run', [string]$Kind = 'class', $Parameters = $null) {
    $methodSelector = @{ name = $Method }
    if ($null -ne $Parameters) { $methodSelector.parameters = @($Parameters) }
    $reply = Invoke-Live $Name 'workspace_query' @{
        type = 'calls_in_file'; workspaceRoot = $workspace; filePath = (Join-Path $workspace $File)
        owner = @{ kind = $Kind; name = $Owner }; method = $methodSelector
    }
    Assert-Live ($null -ne $reply.result.PSObject.Properties['structuredContent']) 'Calls structuredContent missing.'
    # Verify the mirrored model-facing result, not only the side channel.
    $text = [string]$reply.result.content[0].text
    $start = $text.IndexOf('{')
    Assert-Live ($start -ge 0) 'Calls text omitted its JSON envelope.'
    $envelope = $text.Substring($start) | ConvertFrom-Json -Depth 100
    Assert-Live ($envelope.query.type -eq 'calls_in_file') 'Wrong model-facing query identity.'
    Assert-Live (($envelope.result | ConvertTo-Json -Depth 100 -Compress) -ceq
        ($reply.result.structuredContent | ConvertTo-Json -Depth 100 -Compress)) 'Calls text/structured results disagree.'
    return $reply.result.structuredContent
}
function Flat-Calls($Answer) { @($Answer.overloads | ForEach-Object { $_.calls }) }
function Assert-Call($Calls, [string]$Name, [int]$Count, [int]$Arguments, [bool]$Spread = $false) {
    $matches = @($Calls | Where-Object { $_.callee_written -eq $Name })
    Assert-Live ($matches.Count -eq $Count) "Expected $Count '$Name' calls, got $($matches.Count)."
    foreach ($call in $matches) {
        Assert-Live ($call.explicit_argument_count -eq $Arguments) "$Name argument count changed."
        Assert-Live ($call.has_spread -eq $Spread) "$Name spread evidence changed."
        Assert-Live (-not $call.PSObject.Properties['resolved_target']) 'Fabricated call resolution.'
        Assert-Live (-not $call.PSObject.Properties['span']) 'Private occurrence fields leaked into canonical calls.'
    }
}
function Reset-Workspace([string]$Name) {
    $reply = Invoke-Live $Name 'refresh_workspace' @{ workspaceRoot = $workspace }
    Assert-Live ($reply.result.structuredContent.refreshed -eq $true) 'Refresh was not confirmed.'
}
function Write-Config([bool]$Enabled = $true, [int]$MinOperators = 2) {
    Write-Utf8 (Join-Path $workspace '.clean-ctx.json') (@{
        additional_roots = @(); auto_delta = $false
        cbm = @{ enabled = $false; auto_launch = $false }; persistence = @{ enabled = $false }
        meta_layers = @{ angular = @{ rxjs = @{ enabled = $Enabled; min_pipe_operators = $MinOperators } } }
    } | ConvertTo-Json -Depth 20)
}

try {
    Write-Config
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
    $inventory = Invoke-CleanCtxRpc $session -2 'tools/list' @{}
    Save-Capture 'tools-list' $inventory
    foreach ($tool in @('compress_code_context', 'provide_code_context', 'workspace_query', 'refresh_workspace')) {
        Assert-Live (@($inventory.result.tools | Where-Object { $_.name -eq $tool }).Count -eq 1) "Required live tool missing: $tool"
    }
    Save-Capture 'environment' @{
        binary = $BinaryPath; binary_sha256 = (Get-FileHash -LiteralPath $BinaryPath -Algorithm SHA256).Hash
        branch_commit = (& git -C $RepositoryRoot rev-parse HEAD); powershell = $PSVersionTable.PSVersion.ToString()
        working_tree = @(& git -C $RepositoryRoot status --porcelain)
        platform = [Environment]::OSVersion.ToString(); workspace = $workspace
    }
    . (Join-Path $PSScriptRoot 'Verify-PhaseBoundaries.ps1')
    . (Join-Path $PSScriptRoot 'Verify-CrossLayerBoundaries.ps1')
    Invoke-Case 'rxjs-high-structure-and-unicode' {
        $reply = Compress 'rxjs-high' 'Reactive.ts'
        $pipes = @(Pipes $reply)
        Assert-Live ($pipes.Count -eq 3) 'Expected exactly three real chains; comments/strings must be excluded.'
        Assert-Live ($pipes[0] -match '(?s)^stream\$\s*=.*map.*switchMap.*tap') 'Initializer/operator ordering changed.'
        Assert-Live ($pipes[1] -match '(?s)^unicode\$\s*=.*map.*tap') 'UTF-8 summary chain missing.'
        Assert-Live ($pipes[2] -match '(?s)^getOther\s*=.*map.*tap') 'Method ownership changed.'
        Assert-Live (-not (($pipes -join "`n").Contains('fake'))) 'Lexical decoy produced a chain.'
    }
    Invoke-Case 'rxjs-low-medium-fidelity' {
        $low = @(Pipes (Compress 'rxjs-low' 'Reactive.ts' 'low'))
        $medium = @(Pipes (Compress 'rxjs-medium' 'Reactive.ts' 'medium'))
        Assert-Live ($low.Count -eq 3 -and $medium.Count -eq 3) 'Fidelity changed chain discovery.'
        Assert-Live ($low[0] -eq 'stream$') 'Low fidelity should retain only owner.'
        Assert-Live ($medium[0] -match 'map.*switchMap.*tap') 'Medium operator ordering changed.'
    }
    Invoke-Case 'typescript-canonical-calls' {
        $answer = Calls 'ts-calls' 'Reactive.ts' 'Reactive' 'run'
        $calls = @(Flat-Calls $answer)
        Assert-Live ($calls.Count -eq 3) 'Unsupported dynamic target/decoys/field initializers fabricated calls.'
        Assert-Call $calls 'Save' 1 2 $true
        Assert-Call $calls 'Lookup' 2 1
        Assert-Live (($calls.occurrence -join ',') -eq '0,1,2') 'Repeated occurrences lost deterministic ordering.'
    }
    Invoke-Case 'csharp-generics-named-nesting-and-overloads' {
        $answer = Calls 'cs-calls' 'Owners.cs' 'Alpha'
        Assert-Live ($answer.overload_count -eq 2) 'C# overloads collapsed.'
        $calls = @(Flat-Calls $answer)
        Assert-Live ($calls.Count -eq 7) 'C# calls fabricated or omitted.'
        Assert-Call $calls 'Save' 1 2
        Assert-Call $calls 'Lookup' 2 1
        Assert-Call $calls 'Create' 1 0
        Assert-Call $calls 'Load' 1 1
        Assert-Call $calls 'First' 1 0
        Assert-Call $calls 'Numeric' 1 1
        $selected = Calls 'cs-overload-selected' 'Owners.cs' 'Alpha' 'Run' 'class' @('int value')
        Assert-Live ($selected.count -eq 1) 'Explicit overload selection failed.'
        Assert-Call @(Flat-Calls $selected) 'Numeric' 1 1
    }
    Invoke-Case 'csharp-file-owner-interface-isolation' {
        Assert-Call @(Flat-Calls (Calls 'cs-beta' 'Owners.cs' 'Beta')) 'OnlyBeta' 1 0
        Assert-Call @(Flat-Calls (Calls 'cs-peer' 'Peer.cs' 'Alpha')) 'OnlyPeer' 1 0
        $interface = Calls 'cs-interface' 'Owners.cs' 'IRunnable' 'Run' 'interface'
        Assert-Live ($interface.overload_count -eq 1 -and $interface.count -eq 0) 'Interface declaration fabricated implementation calls.'
    }
    Invoke-Case 'provide-context-live-projection' {
        $reply = Invoke-Live 'provide-reactive' 'provide_code_context' @{
            workspaceRoot = $workspace; filePath = (Join-Path $workspace 'Reactive.ts')
            intent = 'refactor'; fidelity = 'high'; tokenizer = 'o200k'
        }
        Assert-Live ($reply.result._meta.fidelity -eq 'high') 'Provided context used the wrong fidelity.'
        $providedText = [string]$reply.result.content[0].text
        if ($reply.result._meta.content_kind -eq 'raw_passthrough') {
            Assert-Live ($providedText -ceq [IO.File]::ReadAllText((Join-Path $workspace 'Reactive.ts'))) 'Raw passthrough changed source.'
        } else {
            Assert-Live ($providedText.Contains('stream$')) 'Provided context lost reactive owner.'
            Assert-Live ($providedText.Contains('switchMap')) 'Provided context lost operator interpretation.'
        }
        $csharp = Invoke-Live 'provide-csharp' 'provide_code_context' @{
            workspaceRoot = $workspace; filePath = (Join-Path $workspace 'Owners.cs'); fidelity = 'high'
        }
        Assert-Live ($csharp.result._meta.fidelity -eq 'high') 'C# context fidelity changed.'
        Assert-Live (([string]$csharp.result.content[0].text).Contains('Alpha')) 'C# provided context lost its class.'
    }
    foreach ($runner in @('karma', 'vitest')) {
        Invoke-Case "angular-$runner-testbed-preservation" {
            $file = "account.$runner.spec.ts"
            $reply = Compress "$runner-spec" $file
            $aliases = @($reply.result.pretty.ir | Where-Object { $_[0] -eq 'TYPE' })
            foreach ($marker in @('@describe', '@it', '@testBed')) {
                Assert-Live (@($aliases | Where-Object { $_[1] -eq $marker }).Count -gt 0) "$runner lost $marker metadata."
            }
            if ($runner -eq 'vitest') {
                Assert-Live (@($aliases | Where-Object { $_[1] -eq '@spy' -and $_[2] -eq 'callback' }).Count -eq 1) 'Vitest vi.fn spy marker missing.'
            }
            $edges = Invoke-Live "$runner-tests-edges" 'workspace_query' @{
                type = 'forward_edges'; workspaceRoot = $workspace
                domain = 'angular'; entity_type = 'TestArtifact'; name = $file; relation = 'Tests'
            }
            $tests = @($edges.result.structuredContent.edges | Where-Object { $_.relation -eq 'Tests' })
            Assert-Live ($tests.Count -eq 1 -and $tests[0].object.name -eq 'AccountComponent') "$runner TestBed linkage was lost or fabricated."
        }
    }
    Invoke-Case 'changed-source-empty-delete-restore' {
        $path = Join-Path $workspace 'Reactive.ts'
        $original = [IO.File]::ReadAllText($path)
        Write-Utf8 $path ($original.Replace(' . pipe /* written boundary */ ', '.skip').Replace('.pipe(', '.skip('))
        Assert-Live (@(Pipes (Compress 'edited-no-pipes' 'Reactive.ts')).Count -eq 0) 'Same-session edit retained stale chains.'
        Write-Utf8 $path "export class Reactive { run() {} }"
        Assert-Live ((Calls 'empty-calls' 'Reactive.ts' 'Reactive' 'run').count -eq 0) 'Empty method retained stale calls.'
        Write-Utf8 $path ''
        $empty = Compress 'empty-source' 'Reactive.ts'
        Assert-Live (-not (($empty | ConvertTo-Json -Depth 100).Contains('@pipeRx'))) 'Empty source retained stale markers.'
        Remove-Item -LiteralPath $path
        Reset-Workspace 'refresh-deleted'
        $missing = Invoke-Live 'deleted-file' 'compress_code_context' @{ filePath = $path; workspaceRoot = $workspace } -AllowError
        Assert-Live ($null -ne $missing.PSObject.Properties['error']) 'Deleted file returned stale successful context.'
        Write-Utf8 $path $original
        Reset-Workspace 'refresh-restored'
        Assert-Live (@(Pipes (Compress 'restored-chains' 'Reactive.ts')).Count -eq 3) 'Restoration failed to rebuild current chains.'
    }
    Invoke-Case 'verbatim-byte-exact' {
        $reply = Compress 'verbatim' 'Reactive.ts' 'verbatim'
        Assert-Live ($reply.result.content[0].text -ceq [IO.File]::ReadAllText((Join-Path $workspace 'Reactive.ts'))) 'Verbatim source was rewritten.'
        Assert-Live ($reply.result.byte_exact -contains 'document') 'Verbatim byte-exact contract missing.'
    }
    Invoke-Case 'minimum-operators-config' {
        Stop-CleanCtxSession $session; $script:session = $null
        Write-Config $true 4
        $script:session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
        Assert-Live (@(Pipes (Compress 'min-four' 'Reactive.ts')).Count -eq 0) 'Minimum-operator filter ignored.'
    }
    Invoke-Case 'disabled-rxjs-config' {
        Stop-CleanCtxSession $session; $script:session = $null
        Write-Config $false 2
        $script:session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
        Assert-Live (@(Pipes (Compress 'rx-disabled' 'Reactive.ts')).Count -eq 0) 'RxJS disabled flag ignored.'
    }
} catch {
    $rows.Add([pscustomobject]@{ name = 'readiness-or-driver'; status = 'FAIL'; evidence = $_.Exception.Message })
    Write-Warning $_.Exception.Message
} finally {
    if ($session) {
        try { Stop-CleanCtxSession $session }
        catch { $rows.Add([pscustomobject]@{ name = 'server-cleanup'; status = 'FAIL'; evidence = $_.Exception.Message }) }
    }
    Save-Capture 'summary' @{ phase = $Phase; planned = $script:planned; selected = $script:selected; cases = @($rows.ToArray()); workspace_retained = [bool]$KeepWorkspace; run_root = $runRoot }
    if ($KeepWorkspace) { Write-Host "Retained workspace: $workspace" }
    else {
        foreach ($attempt in 1..5) {
            try { Remove-Item -LiteralPath $workspace -Recurse -Force; break }
            catch { if ($attempt -eq 5) { Write-Warning "Workspace cleanup failed: $workspace" }; Start-Sleep -Milliseconds 100 }
        }
    }
}
Write-Host "Evidence: $captures"
$failed = @($rows | Where-Object { $_.status -eq 'FAIL' })
Write-Host "Cases: $($rows.Count), failed: $($failed.Count)"
if ($failed.Count -or $script:planned -ne 59 -or $rows.Count -ne $script:selected -or $script:selected -eq 0) { exit 1 }
