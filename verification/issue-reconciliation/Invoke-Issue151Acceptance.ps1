param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$ProviderPath,
    [ValidateRange(30, 900)][int]$CaseTimeoutSeconds = 300
)

# Agent MaxHeadRoom. Operator-run MCP acceptance; builds nothing.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$ProviderPath = (Resolve-Path -LiteralPath $ProviderPath).Path
$runRoot = Join-Path $repo ('target/issue-151-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$primary = Join-Path $runRoot 'primary'
$additional = Join-Path $runRoot 'additional'
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
New-Item -ItemType Directory -Force $primary, $additional | Out-Null
foreach ($root in @($primary, $additional)) {
    & git -C $root init --quiet
    Assert-B ($LASTEXITCODE -eq 0) "Cannot initialize fixture root: $root"
}

function Write-Config([string]$Mode, [string[]]$Roots = @($additional)) {
    $provider = if ($Mode -eq 'unavailable') { Join-Path $runRoot 'missing-provider.exe' } else { $ProviderPath }
    Write-BText (Join-Path $primary '.clean-ctx.json') (@{
        additional_roots = $Roots
        auto_delta = $false
        cbm = @{ enabled = $Mode -ne 'disabled'; auto_launch = $Mode -ne 'disabled'; binary_path = $provider }
        persistence = @{ enabled = $false; auto_save = $false }
    } | ConvertTo-Json -Depth 20)
}

function Write-Fixture {
    Write-BText (Join-Path $primary 'Probe.rs') 'pub struct Probe; impl Probe { pub fn Shared151(&self) {} }'
    Write-BText (Join-Path $additional 'Probe.rs') 'pub struct Probe; impl Probe { pub fn Shared151(&self) {} pub fn OnlyAdditional151(&self) {} }'
    Write-BText (Join-Path $primary 'Native.ts') 'export class Native151 { public TsMethod151() {} }'
    Write-BText (Join-Path $primary 'Native.cs') 'public class Native151Cs { public void CsMethod151() {} }'
    Write-BText (Join-Path $additional 'Native.java') 'public class Native151Java { public void JavaMethod151() {} }'
}

function Tool($Session, [string]$Name, [hashtable]$Arguments, [string]$Evidence) {
    $result = Get-BResult (Invoke-BTool $Session $Name $Arguments)
    Save-B $Evidence $result
    return $result
}

function Query($Session, [hashtable]$Arguments, [string]$Evidence) {
    $result = Tool $Session 'workspace_query' $Arguments $Evidence
    Assert-B ([bool]$result.PSObject.Properties['structuredContent']) 'Missing structured result.'
    $structured = $result.structuredContent
    if (-not $Arguments.ContainsKey('queries')) {
        Assert-B ([bool]$structured.PSObject.Properties['coverage']) (
            'Binary omitted Issue #151 coverage. Rebuild the production executable with cargo build --all-features --bin clean-ctx; inspect provenance.json and this saved response.'
        )
    }
    $text = [string]$result.content[0].text
    $newline = $text.IndexOf("`n")
    Assert-B ($newline -ge 0) 'Missing model-facing answer envelope.'
    $envelope = $text.Substring($newline + 1) | ConvertFrom-Json
    if ($Arguments.ContainsKey('queries')) {
        foreach ($item in $structured.results) {
            Assert-B ($item.status -eq 'ok') "Failed batch item: $($item.id)"
            $model = @($envelope.results | Where-Object id -eq $item.id)
            Assert-B ($model.Count -eq 1) 'Model envelope lost a batch item.'
            Assert-Tuple $item.result.coverage $model[0].completeness 'batch channel parity'
        }
    } else {
        Assert-Tuple $structured.coverage $envelope.completeness 'single channel parity'
    }
    return $structured
}

function Assert-Tuple($Left, $Right, [string]$Label) {
    foreach ($field in @('source_complete', 'result_semantics', 'omitted_possible')) {
        Assert-B ($Left.$field -ceq $Right.$field) "$Label disagrees on $field"
    }
}

function Find($Session, [string]$Name, [bool]$Declarations, [string]$Evidence, [string]$Within = '') {
    $arguments = @{ type = 'find_entities'; name = $Name; workspaceRoot = $primary; declarations_only = $Declarations }
    if ($Within) { $arguments.withinPath = $Within }
    return Query $Session $arguments $Evidence
}

function Assert-Exact($Result) {
    Assert-B ($Result.coverage.status -eq 'authoritative_complete') 'Declaration result lacks certificate.'
    Assert-B ($Result.coverage.source_complete -eq $true) 'Declaration source completeness was not earned.'
    Assert-B ($Result.coverage.result_semantics -eq 'exact') 'Declaration result is not exact.'
    Assert-B ($Result.coverage.omitted_possible -eq $false) 'Certified result permits omissions.'
    Assert-B ($Result.coverage.fact_semantics -eq 'source_proven_declarations') 'Wrong declaration fact semantics.'
    Assert-B (-not [string]::IsNullOrWhiteSpace($Result.coverage.source_generation)) 'Missing proof generation.'
    Assert-B ('source_declaration_units_proven' -in $Result.coverage.evidence_requirements) 'Missing declaration obligation.'
}

function Assert-Lower($Result) {
    Assert-B ($Result.coverage.source_complete -eq $false) 'Uncertified result claims completeness.'
    Assert-B ($Result.coverage.result_semantics -eq 'lower_bound') 'Uncertified result claims exactness.'
    Assert-B ($Result.coverage.omitted_possible -eq $true) 'Uncertified result denies omissions.'
}

function Signature($Result) {
    return (@($Result.entities | ForEach-Object {
        "$($_.domain)|$($_.entity_type)|$($_.name)|$($_.file)"
    } | Sort-Object) -join "`n")
}

function Assert-Files($Result, [string[]]$Expected) {
    $actual = @($Result.entities | ForEach-Object { [IO.Path]::GetFullPath($_.file).Replace('\', '/') } | Sort-Object -Unique)
    $wanted = @($Expected | ForEach-Object { [IO.Path]::GetFullPath($_).Replace('\', '/') } | Sort-Object -Unique)
    Assert-B (($actual -join '|') -ceq ($wanted -join '|')) "Unexpected occurrence files: $($actual -join ',')"
}

function Wait-Provider($Session, [string]$Expected) {
    do {
        $result = Tool $Session 'get_cbm_status' @{} "$($Session.Name)-provider-status-$rpcId"
        $status = [string]$result._meta.cbm_status
        if ($status -eq $Expected) { return }
        Assert-B ($status -eq 'initializing') "Expected provider $Expected, observed $status"
        Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Provider readiness deadline exceeded.'
        Start-Sleep -Milliseconds 250
    } while ($true)
}

function Unavailable-Environment {
    $isolated = Join-Path $runRoot 'isolated-provider-home'
    New-Item -ItemType Directory -Force $isolated | Out-Null
    $names = @('codebase-memory-mcp.exe', 'codebase-memory-mcp.cmd', 'codebase-memory-mcp.bat', 'codebase-memory-mcp')
    $filteredPath = @($env:PATH -split [IO.Path]::PathSeparator | Where-Object {
        $directory = $_
        $directory -and -not @($names | Where-Object {
            Test-Path -LiteralPath (Join-Path $directory $_) -PathType Leaf
        }).Count
    }) -join [IO.Path]::PathSeparator
    return @{ PATH = $filteredPath; HOME = $isolated; USERPROFILE = $isolated;
        LOCALAPPDATA = Join-Path $isolated 'AppData/Local'; APPDATA = Join-Path $isolated 'AppData/Roaming' }
}

function Close-Session($Session) {
    Stop-BSession $Session
    $null = $sessions.Remove($Session)
}

function Start-Live([string]$Name, [hashtable]$Environment = @{}) {
    $script:deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
    $started = Start-BSession $Name $BinaryPath $primary $Environment
    $registry = Get-BResult (Invoke-BRpc $started 'tools/list' @{})
    Save-B "$Name-tool-registry" $registry
    $tool = @($registry.tools | Where-Object name -eq 'workspace_query')
    Assert-B ($tool.Count -eq 1) 'workspace_query is absent or ambiguous in the public registry.'
    $fields = $tool[0].outputSchema.properties.coverage.properties
    Assert-B ([bool]$fields.PSObject.Properties['fact_semantics']) (
        'Binary lacks the Issue #151 public schema. Rebuild with cargo build --all-features --bin clean-ctx and rerun using that executable. See provenance.json for the binary path and hash.'
    )
    return $started
}

Save-B 'provenance' @{
    agent = 'MaxHeadRoom'; binary = $BinaryPath; binary_sha256 = (Get-FileHash -LiteralPath $BinaryPath).Hash
    provider = $ProviderPath; provider_sha256 = (Get-FileHash -LiteralPath $ProviderPath).Hash
    git_revision = (& git -C $repo rev-parse HEAD); working_tree = @(& git -C $repo status --short)
    pwsh = $PSVersionTable.PSVersion.ToString(); contract = 'Issue 151 operator acceptance'
    limits = @{ case_timeout_seconds = $CaseTimeoutSeconds; publication_injection = 'Rust tests only' }
}
$baseline = $null
try {
    foreach ($mode in @('disabled', 'unavailable', 'ready')) {
        Write-Fixture
        Write-Config $mode
        $environment = if ($mode -eq 'unavailable') { Unavailable-Environment } else { @{} }
        $session = Start-Live "151-$mode" $environment
        if ($mode -ne 'disabled') {
            $script:deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
            Wait-Provider $session $(if ($mode -eq 'ready') { 'available' } else { 'unavailable' })
        }
        Invoke-BCase $mode 'fresh-session-declaration-first' {
            $script:coldFirst = Find $session 'Shared151' $true "$mode-declaration-first"
            Assert-Exact $script:coldFirst
            Assert-Files $script:coldFirst @((Join-Path $primary 'Probe.rs'), (Join-Path $additional 'Probe.rs'))
        }
        Close-Session $session
        $session = Start-Live "151-$mode-ordinary-first" $environment
        if ($mode -ne 'disabled') {
            $script:deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
            Wait-Provider $session $(if ($mode -eq 'ready') { 'available' } else { 'unavailable' })
        }
        Invoke-BCase $mode 'ordinary-cache-then-certified-declarations-and-warm-reuse' {
            $ordinary = Find $session 'Shared151' $false "$mode-ordinary"
            Assert-Lower $ordinary
            Assert-B ($ordinary.coverage.fact_semantics -eq 'authoritative_indexed_occurrences') 'Wrong ordinary fact semantics.'
            $cold = Find $session 'Shared151' $true "$mode-declaration"
            Assert-Exact $cold
            Assert-B ((Signature $script:coldFirst) -ceq (Signature $cold)) 'Query history changed declaration truth.'
            Assert-B ($script:coldFirst.coverage.source_generation -ceq $cold.coverage.source_generation) 'Query history changed certificate generation.'
            Assert-Files $cold @((Join-Path $primary 'Probe.rs'), (Join-Path $additional 'Probe.rs'))
            $warm = Find $session 'Shared151' $true "$mode-warm"
            Assert-Exact $warm
            Assert-B ((Signature $cold) -ceq (Signature $warm)) 'Warm state changed declaration truth.'
            Assert-B ($cold.coverage.source_generation -ceq $warm.coverage.source_generation) 'Warm state changed certificate generation.'
            if ($null -eq $script:baseline) { $script:baseline = $cold }
            else {
                Assert-B ((Signature $script:baseline) -ceq (Signature $cold)) 'Provider state changed declaration truth.'
                Assert-Tuple $script:baseline.coverage $cold.coverage 'provider parity'
                Assert-B ($script:baseline.coverage.source_generation -ceq $cold.coverage.source_generation) 'Provider state changed proof generation.'
            }
        }
        Invoke-BCase $mode 'cold-declaration-and-answer-projection' {
            $scoped = Find $session 'OnlyAdditional151' $true "$mode-excluded" $primary
            Assert-Exact $scoped
            Assert-B ($scoped.count -eq 0) 'Primary projection retained additional-root answer.'
            $included = Find $session 'OnlyAdditional151' $true "$mode-included" $additional
            Assert-Exact $included
            Assert-Files $included @((Join-Path $additional 'Probe.rs'))
            Assert-B ($scoped.coverage.source_generation -ceq $included.coverage.source_generation) 'Answer narrowing changed acquisition proof.'
            $absent = Find $session 'DefinitelyAbsent151' $true "$mode-absent"
            Assert-Exact $absent
            Assert-B ($absent.count -eq 0) 'Absent declaration was fabricated.'
        }
        Invoke-BCase $mode 'mixed-language-and-batch-channel-parity' {
            foreach ($name in @('TsMethod151', 'CsMethod151', 'JavaMethod151')) {
                $result = Find $session $name $true "$mode-$name"
                Assert-Exact $result
                Assert-B ($result.count -gt 0) "Mixed-language declaration missing: $name"
            }
            foreach ($reverse in @($false, $true)) {
                $items = @(
                    @{ id = 'ordinary'; type = 'find_entities'; name = 'Shared151' },
                    @{ id = 'declaration'; type = 'find_entities'; name = 'Shared151'; declarations_only = $true }
                )
                if ($reverse) { [array]::Reverse($items) }
                $batch = Query $session @{ workspaceRoot = $primary; queries = $items } "$mode-batch-$reverse"
                $decl = @($batch.results | Where-Object id -eq 'declaration')[0].result
                Assert-Exact $decl
                Assert-B ((Signature $decl) -ceq (Signature $script:baseline)) 'Batch order changed declaration truth.'
                Assert-Lower (@($batch.results | Where-Object id -eq 'ordinary')[0].result)
            }
        }
        Close-Session $session
    }
    Write-Fixture
    Write-Config 'disabled'
    $session = Start-Live '151-refresh'
    Invoke-BCase 'lifecycle' 'edit-delete-refresh-invalidates-proof' {
        $before = Find $session 'OnlyAdditional151' $true 'lifecycle-before'
        Assert-Exact $before
        Write-BText (Join-Path $additional 'Probe.rs') 'pub struct Probe; impl Probe { pub fn Replacement151(&self) {} }'
        $null = Tool $session 'refresh_workspace' @{ workspaceRoot = $primary } 'lifecycle-edit-refresh'
        $after = Find $session 'OnlyAdditional151' $true 'lifecycle-after-edit'
        Assert-Exact $after
        Assert-B ($after.count -eq 0) 'Edit retained stale declarations.'
        Assert-B ($before.coverage.source_generation -cne $after.coverage.source_generation) 'Edit retained stale proof generation.'
        $replacement = Find $session 'Replacement151' $true 'lifecycle-replacement'
        Assert-Exact $replacement
        Assert-B ($replacement.count -gt 0) 'Replacement declaration absent.'
        # Exact generated leaf under this run directory; retain all other evidence.
        Remove-Item -LiteralPath (Join-Path $additional 'Probe.rs')
        $null = Tool $session 'refresh_workspace' @{ workspaceRoot = $primary } 'lifecycle-delete-refresh'
        $deleted = Find $session 'Replacement151' $true 'lifecycle-after-delete'
        Assert-Exact $deleted
        Assert-B ($deleted.count -eq 0) 'Deletion retained stale declarations.'
    }
    Close-Session $session
    $missing = Join-Path $runRoot 'initially-missing'
    Write-Config 'disabled' @($additional, $missing)
    $session = Start-Live '151-missing-root'
    Invoke-BCase 'failure' 'missing-root-prevents-proof-and-recovery-retries' {
        Assert-Lower (Find $session 'Shared151' $true 'missing-root-before')
        New-Item -ItemType Directory $missing | Out-Null
        $null = Tool $session 'refresh_workspace' @{ workspaceRoot = $primary } 'missing-root-refresh'
        Assert-Exact (Find $session 'Shared151' $true 'missing-root-recovered')
    }
    Close-Session $session
    Write-Config 'disabled'
    $session = Start-Live '151-unreadable'
    Invoke-BCase 'failure' 'unreadable-source-prevents-proof-and-recovery-retries' {
        Assert-B $IsWindows 'Exclusive-read fault scenario requires Windows; no PASS is inferred on other hosts.'
        $lockedFile = Join-Path $additional 'locked.rs'
        Write-BText $lockedFile 'pub struct Locked; impl Locked { pub fn Locked151(&self) {} }'
        $handle = [IO.File]::Open($lockedFile, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
        try { Assert-Lower (Find $session 'Locked151' $true 'unreadable-before') }
        finally { $handle.Dispose() }
        $null = Tool $session 'refresh_workspace' @{ workspaceRoot = $primary } 'unreadable-refresh'
        $recovered = Find $session 'Locked151' $true 'unreadable-recovered'
        Assert-Exact $recovered
        Assert-B ($recovered.count -gt 0) 'Recovered readable declaration is missing.'
    }
} catch {
    $verdicts.Add(@{ phase = 'orchestration'; case = 'startup-or-execution'; result = 'FAIL'; evidence = $_.Exception.Message })
} finally {
    foreach ($open in @($sessions.ToArray())) {
        try { Close-Session $open }
        catch { $verdicts.Add(@{ phase = 'cleanup'; case = $open.Name; result = 'FAIL'; evidence = $_.Exception.Message }) }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #151 live evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
Assert-B ($verdicts.Count -eq 15) 'Acceptance matrix did not execute every required case.'
