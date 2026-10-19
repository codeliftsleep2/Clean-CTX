param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$ProviderPath,
    [ValidateRange(60, 900)][int]$CaseTimeoutSeconds = 300,
    [ValidateRange(1, 30)][double]$MaxQuerySeconds = 15
)

# Issue #148 live MCP acceptance. This is operator verification, not a Rust
# test or CI gate. It builds nothing and retains all evidence under target/.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$ProviderPath = (Resolve-Path $ProviderPath).Path
$runRoot = Join-Path $repo ('target/issue-148-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$workspace = Join-Path $runRoot 'workspace'
$additional = Join-Path $runRoot 'additional'
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$timings = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

New-Item -ItemType Directory -Force $workspace, $additional | Out-Null
& git -C $workspace init --quiet
Assert-B ($LASTEXITCODE -eq 0) 'Fixture Git root creation failed.'
& git -C $additional init --quiet
Assert-B ($LASTEXITCODE -eq 0) 'Additional fixture Git root creation failed.'

function Write-Config(
    [bool]$ProviderEnabled,
    [string]$ConfiguredProvider = $ProviderPath,
    [bool]$IncludeAdditional = $true
) {
    [string[]]$roots = @()
    if ($IncludeAdditional) { $roots = [string[]]@($additional) }
    Write-BText (Join-Path $workspace '.clean-ctx.json') (@{
        additional_roots = $roots
        auto_delta = $false
        cbm = @{
            enabled = $ProviderEnabled
            auto_launch = $ProviderEnabled
            binary_path = $ConfiguredProvider
        }
        persistence = @{ enabled = $false; auto_save = $false }
    } | ConvertTo-Json -Depth 20)
}

function Write-Fixture {
    Write-BText (Join-Path $workspace 'service-x.ts') @'
import { Injectable } from '@angular/core';
@Injectable()
export class ServiceX {}
'@
    Write-BText (Join-Path $workspace 'ConsumerA.ts') @'
import { Component } from '@angular/core';
import { ServiceX } from './service-x';
@Component({ selector: 'issue-148-a', template: '' })
export class ConsumerA { constructor(private service: ServiceX) {} }
'@
    Write-BText (Join-Path $additional 'ConsumerB.ts') @'
import { Component } from '@angular/core';
import { ServiceX } from './service-x';
@Component({ selector: 'issue-148-b', template: '' })
export class ConsumerB { constructor(private service: ServiceX) {} }
'@
    Write-BText (Join-Path $workspace 'ContextA.cs') @'
namespace Issue148;
public class ContextA {}
'@
    Write-BText (Join-Path $workspace 'ConsumerOne.cs') @'
namespace Issue148;
public class ConsumerOne { public ConsumerOne(ContextA value) {} }
'@
    Write-BText (Join-Path $additional 'ConsumerTwo.cs') @'
namespace Issue148;
public class ConsumerTwo { public ConsumerTwo(ContextA value) {} }
'@
}

function Clear-FixtureSources {
    foreach ($root in @($workspace, $additional)) {
        foreach ($name in @('placeholder.cs', 'service-x.ts', 'ConsumerA.ts', 'ConsumerB.ts',
            'ContextA.cs', 'ConsumerOne.cs', 'ConsumerTwo.cs')) {
            $path = Join-Path $root $name
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
        }
    }
}

function Write-StagedFixture([ValidateSet('empty', 'partial', 'plausible', 'useful')][string]$Mode) {
    Clear-FixtureSources
    if ($Mode -eq 'empty') {
        Write-BText (Join-Path $workspace 'placeholder.cs') 'public class ProviderPlaceholder {}'
        return
    }
    Write-BText (Join-Path $workspace 'service-x.ts') @'
import { Injectable } from '@angular/core';
@Injectable()
export class ServiceX {}
'@
    Write-BText (Join-Path $workspace 'ContextA.cs') @'
namespace Issue148;
public class ContextA {}
'@
    if ($Mode -in @('partial', 'plausible', 'useful')) {
        Write-BText (Join-Path $workspace 'ConsumerA.ts') @'
import { Component } from '@angular/core';
import { ServiceX } from './service-x';
@Component({ selector: 'issue-148-a', template: '' })
export class ConsumerA { constructor(private service: ServiceX) {} }
'@
    }
    if ($Mode -in @('plausible', 'useful')) {
        Write-BText (Join-Path $workspace 'ConsumerOne.cs') @'
namespace Issue148;
public class ConsumerOne { public ConsumerOne(ContextA value) {} }
'@
    }
    if ($Mode -eq 'useful') { Write-Fixture }
}

function Tool($Session, [string]$Name, [hashtable]$Arguments, [string]$Evidence) {
    $result = Get-BResult (Invoke-BTool $Session $Name $Arguments)
    Save-B $Evidence $result
    return $result
}

function Wait-CbmStatus($Session, [string]$Expected, [string]$Evidence) {
    $attempt = 0
    do {
        $attempt++
        $result = Get-BResult (Invoke-BTool $Session 'get_cbm_status' @{})
        Save-B "$Evidence-$attempt" $result
        $observed = [string]$result._meta.cbm_status
        if ($observed -eq $Expected) { return $result }
        Assert-B ($observed -eq 'initializing') (
            "$Evidence expected $Expected, observed terminal state $observed."
        )
        Assert-B ([DateTime]::UtcNow -lt $script:deadline) (
            "$Evidence did not reach $Expected before the case deadline."
        )
        Start-Sleep -Milliseconds 250
    } while ($true)
}

function Query($Session, [hashtable]$Arguments, [string]$Evidence) {
    $Arguments.workspaceRoot = $workspace
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $result = Tool $Session 'workspace_query' $Arguments $Evidence
    $watch.Stop()
    $timings.Add(@{ evidence = $Evidence; seconds = $watch.Elapsed.TotalSeconds })
    Assert-B ($watch.Elapsed.TotalSeconds -le $MaxQuerySeconds) (
        "$Evidence exceeded ${MaxQuerySeconds}s: $($watch.Elapsed.TotalSeconds.ToString('F3'))s."
    )
    Assert-B ([bool]$result.PSObject.Properties['structuredContent']) (
        "$Evidence omitted structuredContent."
    )
    return $result.structuredContent
}

function Angular-Reverse($Session, [string]$Evidence) {
    Query $Session @{
        type = 'reverse_edges'; domain = 'angular'; entity_type = 'Service'; name = 'ServiceX'
    } $Evidence
}

function Angular-Forward($Session, [string]$Consumer, [string]$Evidence) {
    Query $Session @{
        type = 'forward_edges'; domain = 'angular'; entity_type = 'Component'; name = $Consumer
    } $Evidence
}

function Context-Entities($Session, [string]$Evidence) {
    Query $Session @{ type = 'find_entities'; name = 'ContextA' } $Evidence
}

function Context-TypeRefs($Session, [string]$Evidence) {
    Query $Session @{
        type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'ContextA'
    } $Evidence
}

function Leaf([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path)) { return '' }
    return [IO.Path]::GetFileName($Path)
}

function Angular-Keys($Result) {
    @($Result.edges | Where-Object relation -eq 'Injects' | ForEach-Object {
        "$($_.subject.domain)|$($_.subject.entity_type)|$($_.subject.name)|" +
        "$(Leaf $_.subject.file)|$($_.object.domain)|$($_.object.entity_type)|" +
        "$($_.object.name)"
    } | Sort-Object)
}

function Entity-Keys($Result) {
    @($Result.entities | ForEach-Object {
        "$($_.domain)|$($_.entity_type)|$($_.name)|$(Leaf $_.file)"
    } | Sort-Object)
}

function Assert-Set($Actual, $Expected, [string]$Message) {
    $difference = @(Compare-Object @($Expected | Sort-Object) @($Actual | Sort-Object))
    Assert-B ($difference.Count -eq 0) (
        "$Message Expected=$(@($Expected) -join ',') Actual=$(@($Actual) -join ',')"
    )
}

function Assert-LowerBound($Result, [string]$Label) {
    Assert-B ($Result.coverage.result_semantics -eq 'lower_bound') (
        "$Label unexpectedly claimed exact coverage."
    )
    Assert-B ($Result.coverage.omitted_possible -eq $true) (
        "$Label unexpectedly denied possible omissions."
    )
}

function Assert-NoWarmCompilation($Result, [string]$Label) {
    if ($Result.PSObject.Properties['discovery'] -and
        $Result.discovery.PSObject.Properties['compiled']) {
        Assert-B ([int]$Result.discovery.compiled -eq 0) "$Label repeated semantic compilation."
    }
}

function Capture-Semantics(
    $Session,
    [string]$Prefix,
    [ValidateSet('angular-first', 'context-first', 'unrelated', 'declaration', 'typeref', 'warm')]
    [string]$History
) {
    if ($History -eq 'unrelated') {
        $null = Query $Session @{ type = 'find_entities'; name = 'UnrelatedMissing' } (
            "$Prefix-unrelated-warmup"
        )
    }
    if ($History -eq 'declaration') {
        $null = Query $Session @{ type = 'find_entities'; name = 'ServiceX' } (
            "$Prefix-service-declaration-warmup"
        )
        $null = Context-Entities $Session "$Prefix-context-declaration-warmup"
    }
    if ($History -eq 'typeref') {
        $null = Context-TypeRefs $Session "$Prefix-type-ref-warmup"
    }
    if ($History -eq 'context-first') {
        $contextCold = Context-Entities $Session "$Prefix-context-q"
        $angularCold = Angular-Reverse $Session "$Prefix-angular-after-context"
    } else {
        $angularCold = Angular-Reverse $Session "$Prefix-angular-q"
        $contextCold = Context-Entities $Session "$Prefix-context-after-angular"
    }
    $forwardA = Angular-Forward $Session 'ConsumerA' "$Prefix-forward-a"
    $forwardB = Angular-Forward $Session 'ConsumerB' "$Prefix-forward-b"
    $contextKeys = @(Entity-Keys $contextCold)
    $angularKeys = @(Angular-Keys $angularCold)
    Assert-Set $angularKeys @(
        'angular|Component|ConsumerA|ConsumerA.ts|angular|Service|ServiceX',
        'angular|Component|ConsumerB|ConsumerB.ts|angular|Service|ServiceX'
    ) "$Prefix Angular reverse oracle mismatch."
    Assert-Set (@(Angular-Keys $forwardA) + @(Angular-Keys $forwardB)) $angularKeys (
        "$Prefix forward/reverse generation mismatch."
    )
    Assert-Set $contextKeys @(
        'builtin|Class|ContextA|ContextA.cs',
        'builtin|TypeRef|ContextA|ConsumerOne.cs',
        'builtin|TypeRef|ContextA|ConsumerTwo.cs'
    ) "$Prefix ContextA occurrence oracle mismatch."
    # Relationship queries expose a structured coverage certificate. The
    # declaration/occurrence projection instead exposes an authoritative index
    # snapshot without a `coverage` member. Keep that presentation distinction
    # separate from the semantic occurrence oracle above.
    Assert-LowerBound $angularCold "$Prefix Angular"
    if ($History -eq 'warm') {
        $angularWarm = Angular-Reverse $Session "$Prefix-angular-warm"
        $contextWarm = Context-Entities $Session "$Prefix-context-warm"
        Assert-Set (Angular-Keys $angularWarm) $angularKeys "$Prefix Angular cold/warm mismatch."
        Assert-Set (Entity-Keys $contextWarm) $contextKeys "$Prefix ContextA cold/warm mismatch."
        Assert-NoWarmCompilation $angularWarm "$Prefix Angular warm"
        Assert-NoWarmCompilation $contextWarm "$Prefix ContextA warm"
    }
    return @{ angular = $angularKeys; context = $contextKeys }
}

function Stop-OwnedSession($Session) {
    Stop-BSession $Session
    $null = $sessions.Remove($Session)
}

function Unavailable-Environment {
    $isolatedUserRoot = Join-Path $runRoot 'provider-unavailable-home'
    New-Item -ItemType Directory -Force $isolatedUserRoot | Out-Null
    $names = @('codebase-memory-mcp.exe', 'codebase-memory-mcp.cmd',
        'codebase-memory-mcp.bat', 'codebase-memory-mcp')
    $path = @([Environment]::GetEnvironmentVariable('PATH') -split [IO.Path]::PathSeparator |
        Where-Object {
            $directory = $_
            $directory -and -not @($names | Where-Object {
                Test-Path -LiteralPath (Join-Path $directory $_) -PathType Leaf
            }).Count
        }) -join [IO.Path]::PathSeparator
    return @{
        PATH = $path; HOME = $isolatedUserRoot; USERPROFILE = $isolatedUserRoot
        LOCALAPPDATA = Join-Path $isolatedUserRoot 'AppData/Local'
        APPDATA = Join-Path $isolatedUserRoot 'AppData/Roaming'
    }
}

function Assert-ProviderSnapshot($Session, [string[]]$Projects, [string]$Mode) {
    $targets = [ordered]@{
        ServiceX = 'service-x.ts'; ConsumerA = 'ConsumerA.ts'; ConsumerB = 'ConsumerB.ts'
        ContextA = 'ContextA.cs'; ConsumerOne = 'ConsumerOne.cs'; ConsumerTwo = 'ConsumerTwo.cs'
    }
    $presentFiles = @{
        empty = @()
        partial = @('service-x.ts', 'ConsumerA.ts', 'ContextA.cs')
        plausible = @('service-x.ts', 'ConsumerA.ts', 'ContextA.cs', 'ConsumerOne.cs')
        useful = @($targets.Values)
    }[$Mode]
    $observed = @{}
    foreach ($entry in $targets.GetEnumerator()) {
        $name = $entry.Key
        $expectedFile = $entry.Value
        $matchingFiles = [Collections.Generic.HashSet[string]]::new(
            [StringComparer]::OrdinalIgnoreCase
        )
        foreach ($project in $Projects) {
            $answer = Get-BResult (Invoke-BReadyTool $Session 'graph_search' @{
                project = $project; query = $name
            })
            foreach ($node in @($answer.structuredContent.nodes)) {
                if ((Leaf ([string]$node.file)) -eq $expectedFile) {
                    $null = $matchingFiles.Add($expectedFile)
                }
            }
        }
        $expectedPresent = $expectedFile -in $presentFiles
        $observed[$name] = @($matchingFiles)
        Assert-B (($matchingFiles.Count -eq 1) -eq $expectedPresent) (
            "$Mode provider snapshot for $name/$expectedFile expected present=$expectedPresent, " +
            "observed files=$(@($matchingFiles) -join ',')."
        )
    }
    Save-B "provider-snapshot-$Mode-$($Session.Name)" $observed
}

function Run-HistoryMatrix([string]$Mode, $ExpectedTruth) {
    foreach ($historyName in @(
        'angular-first', 'context-first', 'unrelated', 'declaration', 'typeref', 'warm'
    )) {
        $sessionName = "$Mode-$historyName"
        Write-StagedFixture $Mode
        Write-Config $true
        $session = Start-BSession $sessionName $BinaryPath $workspace
        Invoke-BCase '148-MATRIX' $sessionName {
            $null = Wait-CbmStatus $session 'available' "$sessionName-status"
            $primaryIndex = Invoke-BCleanIndexRepository $session $workspace
            $additionalIndex = Invoke-BCleanIndexRepository $session $additional
            Assert-B ($primaryIndex.status -eq 'indexed') "$sessionName primary index failed."
            Assert-B ($additionalIndex.status -eq 'indexed') "$sessionName additional index failed."
            Assert-ProviderSnapshot $session @(
                $primaryIndex.project, $additionalIndex.project
            ) $Mode
            # Freeze the verified provider view, then make the Clean-owned
            # filesystem truth identical across every matrix cell.
            Write-Fixture
            $truth = Capture-Semantics $session $sessionName $historyName
            Assert-Set $truth.angular $ExpectedTruth.angular (
                "$sessionName changed Angular semantic truth."
            )
            Assert-Set $truth.context $ExpectedTruth.context (
                "$sessionName changed ContextA semantic truth."
            )
        }
        Stop-OwnedSession $session
    }
}

Write-Fixture
Save-B 'fixture-manifest' @{
    angular_reverse = @(
        'angular|Component|ConsumerA|ConsumerA.ts|angular|Service|ServiceX',
        'angular|Component|ConsumerB|ConsumerB.ts|angular|Service|ServiceX'
    )
    context_occurrences = @(
        'builtin|Class|ContextA|ContextA.cs',
        'builtin|TypeRef|ContextA|ConsumerOne.cs',
        'builtin|TypeRef|ContextA|ConsumerTwo.cs'
    )
}
Save-B 'provenance' @{
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    provider = $ProviderPath
    provider_sha256 = (Get-FileHash $ProviderPath -Algorithm SHA256).Hash
    platform = [Environment]::OSVersion.VersionString
    workspace = $workspace
    contract = 'Issue #148 provider-state root-satisfaction parity'
}

$nativeTruth = $null
try {
    Write-Config $false
    $native = Start-BSession 'native' $BinaryPath $workspace
    Invoke-BCase '148-A' 'provider-disabled-production-oracle' {
        $tools = Get-BResult (Invoke-BRpc $native 'tools/list' @{})
        Assert-B (@($tools.tools | Where-Object name -eq 'workspace_query').Count -eq 1) (
            'workspace_query is not registered exactly once.'
        )
        $script:nativeTruth = Capture-Semantics $native 'native' 'warm'
    }
    Stop-OwnedSession $native
    if ($verdicts[$verdicts.Count - 1].result -ne 'PASS') {
        throw 'Provider-disabled oracle failed; later provider comparisons are not meaningful.'
    }

    Write-Config $true (Join-Path $runRoot 'missing-provider.exe')
    $unavailable = Start-BSession 'provider-unavailable' $BinaryPath $workspace (Unavailable-Environment)
    Invoke-BCase '148-B' 'provider-unavailable-parity' {
        $null = Wait-CbmStatus $unavailable 'unavailable' 'provider-unavailable-status'
        $truth = Capture-Semantics $unavailable 'unavailable' 'warm'
        Assert-Set $truth.angular $nativeTruth.angular 'Unavailable Angular truth differs.'
        Assert-Set $truth.context $nativeTruth.context 'Unavailable ContextA truth differs.'
    }
    Stop-OwnedSession $unavailable
    if ($verdicts[$verdicts.Count - 1].result -ne 'PASS') {
        throw 'Provider-unavailable control failed; ready-provider comparisons are not meaningful.'
    }

    foreach ($mode in @('empty', 'partial', 'plausible', 'useful')) {
        Run-HistoryMatrix $mode $nativeTruth
    }

    Invoke-BCase '148-TOPOLOGY' 'single-root-plausible-history-parity' {
        Write-StagedFixture 'plausible'
        Write-Config $true $ProviderPath $false
        $single = Start-BSession 'single-root-plausible' $BinaryPath $workspace
        $null = Wait-CbmStatus $single 'available' 'single-root-status'
        $index = Invoke-BCleanIndexRepository $single $workspace
        Assert-B ($index.status -eq 'indexed') 'Single-root provider index failed.'
        Assert-ProviderSnapshot $single @($index.project) 'plausible'
        Write-Fixture
        Move-Item -LiteralPath (Join-Path $additional 'ConsumerB.ts') `
            -Destination (Join-Path $workspace 'ConsumerB.ts') -Force
        Move-Item -LiteralPath (Join-Path $additional 'ConsumerTwo.cs') `
            -Destination (Join-Path $workspace 'ConsumerTwo.cs') -Force
        $truth = Capture-Semantics $single 'single-root-plausible' 'typeref'
        Assert-Set $truth.angular $nativeTruth.angular 'Single-root Angular truth differs.'
        Assert-Set $truth.context $nativeTruth.context 'Single-root ContextA truth differs.'
        Stop-OwnedSession $single
    }
} finally {
    foreach ($openSession in @($sessions)) {
        try { Stop-BSession $openSession }
        catch {
            $verdicts.Add(@{
                phase = 'cleanup'; case = $openSession.Name; result = 'FAIL'
                evidence = $_.Exception.Message
            })
        }
    }
    Save-B 'timings' @($timings.ToArray())
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #148 live evidence: $runRoot"
}

if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
