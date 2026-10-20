param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$ProviderPath,
    [ValidateRange(60, 900)][int]$CaseTimeoutSeconds = 300,
    [ValidateRange(1, 10)][double]$MaxReverseSeconds = 10
)

# Consolidated Issue #144 field acceptance. Builds nothing. Every fixture,
# request, response, stderr stream, oracle and verdict is retained under target/.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$ProviderPath = (Resolve-Path $ProviderPath).Path
$runRoot = Join-Path $repo ('target/issue-144-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$primary = Join-Path $runRoot 'workspace'
$additional = Join-Path $runRoot 'additional'
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$timings = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

New-Item -ItemType Directory -Force $primary, $additional,
    (Join-Path $primary 'group-a'), (Join-Path $primary 'group-b'),
    (Join-Path $primary 'excluded') | Out-Null
& git -C $primary init --quiet
Assert-B ($LASTEXITCODE -eq 0) 'Primary fixture Git root creation failed.'
& git -C $additional init --quiet
Assert-B ($LASTEXITCODE -eq 0) 'Additional fixture Git root creation failed.'

function Write-Config([bool]$ProviderEnabled, [string]$ConfiguredProvider = $ProviderPath) {
    $config = @{
        additional_roots = @($additional)
        exclude_patterns = @('excluded')
        auto_delta = $false
        cbm = @{
            enabled = $ProviderEnabled
            auto_launch = $ProviderEnabled
            binary_path = $ConfiguredProvider
        }
        persistence = @{ enabled = $false; auto_save = $false }
    } | ConvertTo-Json -Depth 20
    Write-BText (Join-Path $primary '.clean-ctx.json') $config
}

function Wait-ProviderStatus($Session, [string]$EvidenceName) {
    do {
        $status = Get-BResult (Invoke-BTool $Session 'get_cbm_status' @{})
        $state = [string]$status._meta.cbm_status
        if ($state -eq 'initializing') {
            Assert-B ([DateTime]::UtcNow -lt $script:deadline) (
                "CBM provider remained initializing through the case deadline: $EvidenceName"
            )
            Start-Sleep -Milliseconds 250
        }
    } while ($state -eq 'initializing')
    Save-B $EvidenceName $status
    return $status
}

function Write-Fixture {
    New-Item -ItemType Directory -Force (Join-Path $primary 'bulk') | Out-Null
    Write-BText (Join-Path $primary 'Target.cs') @'
namespace Issue144;
public class ContextA { }
public class NeverReferenced { }
'@
    Write-BText (Join-Path $primary 'group-a/PlainConsumer.cs') @'
namespace Issue144;
public class PlainConsumer { public PlainConsumer(ContextA value) { } }
'@
    Write-BText (Join-Path $primary 'group-a/AttributedConsumer.cs') @'
namespace Issue144;
public class AttributedConsumer { public AttributedConsumer([FromServices] ContextA value) { } }
'@
    Write-BText (Join-Path $primary 'group-a/DuplicateParameterConsumer.cs') @'
namespace Issue144;
public class DuplicateParameterConsumer {
    public DuplicateParameterConsumer(ContextA first, ContextA second) { }
}
'@
    Write-BText (Join-Path $primary 'group-b/OtherConsumer.cs') @'
namespace Issue144;
public class OtherConsumer { public OtherConsumer(ContextA value) { } }
'@
    Write-BText (Join-Path $primary 'group-b/DerivedConsumer.cs') @'
namespace Issue144;
public class DerivedConsumer : ContextA { }
'@
    Write-BText (Join-Path $primary 'WrittenForms.cs') @'
using System.Collections.Generic;
using AliasA = Issue144.ContextA;
namespace Issue144;
public class NullableConsumer { public NullableConsumer(ContextA? value) { } }
public class ArrayConsumer { public ArrayConsumer(ContextA[] value) { } }
public class GenericConsumer { public GenericConsumer(List<ContextA> value) { } }
public class QualifiedConsumer { public QualifiedConsumer(Issue144.ContextA value) { } }
public class AliasConsumer { public AliasConsumer(AliasA value) { } }
'@
    Write-BText (Join-Path $primary 'NegativeControls.cs') @'
namespace Issue144;
public class MethodOnly { public void Handle(ContextA value) { } }
public class TextOnly {
    // GhostTarget and ContextA are not semantic relationships here.
    public const string Value = "GhostTarget ContextA";
}
'@
    Write-BText (Join-Path $primary 'DuplicateOwnerOne.cs') @'
namespace Issue144.One;
public class RepeatedConsumer { public RepeatedConsumer(ContextA value) { } }
'@
    Write-BText (Join-Path $primary 'DuplicateOwnerTwo.cs') @'
namespace Issue144.Two;
public class RepeatedConsumer { public RepeatedConsumer(ContextA value) { } }
'@
    Write-BText (Join-Path $primary 'excluded/ExcludedConsumer.cs') @'
namespace Issue144;
public class ExcludedConsumer { public ExcludedConsumer(ContextA value) { } }
'@
    Write-BText (Join-Path $additional 'AdditionalConsumer.cs') @'
namespace Issue144;
public class AdditionalConsumer { public AdditionalConsumer(ContextA value) { } }
'@
    foreach ($index in 0..36) {
        $name = 'BulkConsumer' + $index.ToString('D2')
        Write-BText (Join-Path $primary "bulk/$name.cs") "namespace Issue144; public class $name { public $name(ContextA value) { } }"
    }
}

function Tool($Session, [string]$Name, [hashtable]$Arguments, [string]$Evidence) {
    $result = Get-BResult (Invoke-BTool $Session $Name $Arguments)
    Save-B $Evidence $result
    return $result
}

function Query($Session, [hashtable]$Arguments, [string]$Evidence) {
    $containsReverse = $Arguments.ContainsKey('type') -and
        $Arguments['type'] -eq 'reverse_edges'
    if (-not $containsReverse -and $Arguments.ContainsKey('queries')) {
        $containsReverse = @(
            $Arguments['queries'] | Where-Object { $_.type -eq 'reverse_edges' }
        ).Count -gt 0
    }
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    $result = Tool $Session 'workspace_query' $Arguments $Evidence
    $stopwatch.Stop()
    if ($containsReverse) {
        $timings.Add(@{
            evidence = $Evidence
            seconds = $stopwatch.Elapsed.TotalSeconds
            ceiling_seconds = $MaxReverseSeconds
        })
    }
    Assert-B ([bool]$result.PSObject.Properties['structuredContent']) 'workspace_query omitted structuredContent.'
    if ($containsReverse) {
        Assert-B ($stopwatch.Elapsed.TotalSeconds -le $MaxReverseSeconds) (
            "$Evidence exceeded the reverse-edge latency ceiling: " +
            "$($stopwatch.Elapsed.TotalSeconds.ToString('F3'))s > $($MaxReverseSeconds.ToString('F3'))s."
        )
    }
    return $result.structuredContent
}

function Reverse($Session, [string]$Target, [string]$Evidence, [string]$WithinPath = '') {
    $args = @{
        workspaceRoot = $primary
        type = 'reverse_edges'
        domain = 'builtin'
        entity_type = 'TypeRef'
        name = $Target
    }
    if ($WithinPath) { $args.withinPath = $WithinPath }
    return Query $Session $args $Evidence
}

function Edge-Key($Edge) {
    @(
        $Edge.relation,
        $Edge.subject.domain, $Edge.subject.entity_type, $Edge.subject.name, $Edge.subject.file,
        $Edge.object.domain, $Edge.object.entity_type, $Edge.object.name, $Edge.object.file,
        $Edge.layer
    ) -join '|'
}

function Edge-Keys($Result) {
    @($Result.edges | ForEach-Object { Edge-Key $_ } | Sort-Object)
}

function Constructor-Subjects($Result, [string]$Target) {
    @($Result.edges | Where-Object {
        $_.relation -eq 'HasConstructorParameterType' -and
        $_.object.domain -eq 'builtin' -and
        $_.object.entity_type -eq 'TypeRef' -and
        $_.object.name -eq $Target
    } | ForEach-Object { $_.subject.name } | Sort-Object)
}

function Assert-Set($Actual, $Expected, [string]$Message) {
    $difference = @(Compare-Object @($Expected | Sort-Object) @($Actual | Sort-Object))
    Assert-B ($difference.Count -eq 0) "$Message Expected=$(@($Expected) -join ',') Actual=$(@($Actual) -join ',')"
}

function Assert-Exact($Result, [string]$Label) {
    $coverage = $Result.coverage
    Assert-B ($coverage.status -eq 'authoritative_complete') "$Label status is not authoritative_complete."
    Assert-B ($coverage.source_complete -eq $true) "$Label source_complete is not true."
    Assert-B ($coverage.result_semantics -eq 'exact') "$Label result semantics are not exact."
    Assert-B ($coverage.omitted_possible -eq $false) "$Label still permits omissions."
    Assert-B (@($coverage.evidence_requirements).Count -gt 0) "$Label omitted evidence requirements."
    Assert-B (-not [string]::IsNullOrWhiteSpace([string]$coverage.source_generation)) "$Label omitted proof generation."
}

function Assert-LowerBound($Result, [string]$Label) {
    $coverage = $Result.coverage
    Assert-B ($coverage.source_complete -eq $false) "$Label falsely claims source completeness."
    Assert-B ($coverage.result_semantics -eq 'lower_bound') "$Label falsely claims exact semantics."
    Assert-B ($coverage.omitted_possible -eq $true) "$Label denies possible omissions."
}

function Assert-NoWarmCompilation($Result, [string]$Label) {
    if ($Result.PSObject.Properties['discovery'] -and
        $Result.discovery.PSObject.Properties['compiled']) {
        Assert-B ([int]$Result.discovery.compiled -eq 0) "$Label repeated semantic compilation."
    }
}

function Stop-OwnedSession($Session) {
    Stop-BSession $Session
    $null = $sessions.Remove($Session)
}

Write-Fixture
Write-Config $false
Save-B 'provenance' @{
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    provider = $ProviderPath
    provider_sha256 = (Get-FileHash $ProviderPath -Algorithm SHA256).Hash
    platform = [Environment]::OSVersion.VersionString
    primary_root = $primary
    additional_root = $additional
    contract = 'Issue #144 provider-neutral reverse-edge completeness'
    max_reverse_seconds = $MaxReverseSeconds
}

$bulkConsumers = @(0..36 | ForEach-Object { 'BulkConsumer' + $_.ToString('D2') })
$expectedContextConsumers = @(
    'AdditionalConsumer', 'AttributedConsumer', 'DuplicateParameterConsumer',
    'OtherConsumer', 'PlainConsumer', 'RepeatedConsumer', 'RepeatedConsumer'
) + $bulkConsumers
$nativeContextKeys = $null

try {
    $native = Start-BSession 'native' $BinaryPath $primary
    Invoke-BCase '144-A' 'registered-production-surface' {
        $tools = Get-BResult (Invoke-BRpc $native 'tools/list' @{})
        Assert-B (@($tools.tools | Where-Object name -eq 'workspace_query').Count -eq 1) 'workspace_query is not registered exactly once.'
        Assert-B (@($tools.tools | Where-Object name -eq 'refresh_workspace').Count -eq 1) 'refresh_workspace is not registered exactly once.'
    }
    Invoke-BCase '144-A' 'cold-authoritative-completeness-and-source-oracle' {
        $cold = Reverse $native 'ContextA' 'native-cold-context-a'
        $script:nativeContextKeys = @(Edge-Keys $cold)
        Assert-Exact $cold 'native cold ContextA'
        Assert-Set (Constructor-Subjects $cold 'ContextA') $expectedContextConsumers 'Cold constructor consumer set differs.'
        Assert-B (@($cold.edges | Where-Object {
            $_.relation -eq 'HasBaseType' -and $_.subject.name -eq 'DerivedConsumer'
        }).Count -eq 1) 'DerivedConsumer base-TypeRef edge is missing.'
        Assert-B (@($cold.edges | Where-Object { $_.subject.name -eq 'MethodOnly' }).Count -eq 0) 'Ordinary method parameter became an incoming TypeRef edge.'
        Assert-B (@($cold.edges | Where-Object { $_.subject.name -eq 'ExcludedConsumer' }).Count -eq 0) 'Excluded source entered the admitted universe.'
    }
    Invoke-BCase '144-A' 'warm-equivalence-and-no-recompilation' {
        $warm = Reverse $native 'ContextA' 'native-warm-context-a'
        Assert-Exact $warm 'native warm ContextA'
        Assert-Set (Edge-Keys $warm) $nativeContextKeys 'Warm edge truth differs from cold truth.'
        Assert-NoWarmCompilation $warm 'native warm ContextA'
    }
    Invoke-BCase '144-A' 'within-path-is-an-exact-narrowing' {
        $scoped = Reverse $native 'ContextA' 'native-within-group-a' 'group-a'
        Assert-Exact $scoped 'withinPath ContextA'
        Assert-Set (Constructor-Subjects $scoped 'ContextA') @(
            'AttributedConsumer', 'DuplicateParameterConsumer', 'PlainConsumer'
        ) 'withinPath returned an occurrence outside group-a.'
        $absolute = Reverse $native 'ContextA' 'native-within-group-a-absolute' (Join-Path $primary 'group-a')
        Assert-Set (Edge-Keys $absolute) (Edge-Keys $scoped) 'Relative and absolute withinPath spellings disagree.'
        $outside = Invoke-BTool $native 'workspace_query' @{
            workspaceRoot = $primary; withinPath = $runRoot;
            type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'ContextA'
        }
        Save-B 'native-within-unauthorized' $outside
        $outsideWasRefused = $null -ne $outside.PSObject.Properties['error']
        if (-not $outsideWasRefused -and $null -ne $outside.PSObject.Properties['result']) {
            $outsideWasRefused = (
                $null -ne $outside.result.PSObject.Properties['isError'] -and
                $outside.result.isError -eq $true
            )
        }
        Assert-B $outsideWasRefused 'An outside withinPath was not refused.'
    }
    Invoke-BCase '144-A' 'exact-zero-case-and-spelling-controls' {
        $zero = Reverse $native 'NeverReferenced' 'native-exact-zero'
        Assert-Exact $zero 'NeverReferenced exact zero'
        Assert-B ([int]$zero.count -eq 0) 'NeverReferenced returned an unexpected edge.'
        $caseControl = Reverse $native 'contexta' 'native-case-sensitive-zero'
        Assert-Exact $caseControl 'case-sensitive exact zero'
        Assert-B ([int]$caseControl.count -eq 0) 'TypeRef identity unexpectedly ignored case.'
        $textOnly = Reverse $native 'GhostTarget' 'native-text-only-zero'
        Assert-Exact $textOnly 'text-only exact zero'
        Assert-B ([int]$textOnly.count -eq 0) 'Comment/string occurrence became a semantic edge.'
    }
    Invoke-BCase '144-A' 'written-type-form-matrix' {
        $matrix = [ordered]@{
            'ContextA?' = 'NullableConsumer'
            'ContextA[]' = 'ArrayConsumer'
            'List<ContextA>' = 'GenericConsumer'
            'Issue144.ContextA' = 'QualifiedConsumer'
            'AliasA' = 'AliasConsumer'
        }
        foreach ($entry in $matrix.GetEnumerator()) {
            $answer = Reverse $native $entry.Key ('native-written-' + $entry.Value)
            Assert-Exact $answer ("written TypeRef " + $entry.Key)
            Assert-Set (Constructor-Subjects $answer $entry.Key) @($entry.Value) ("Written TypeRef mismatch for " + $entry.Key)
        }
    }
    Invoke-BCase '144-A' 'batch-and-single-query-equivalence' {
        $batch = Query $native @{
            workspaceRoot = $primary
            queries = @(
                @{ id = 'context'; type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'ContextA' },
                @{ id = 'zero'; type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'NeverReferenced' }
            )
        } 'native-batch'
        $context = @($batch.results | Where-Object id -eq 'context')[0].result
        $zero = @($batch.results | Where-Object id -eq 'zero')[0].result
        Assert-Exact $context 'batched ContextA'
        Assert-Exact $zero 'batched NeverReferenced'
        Assert-Set (Edge-Keys $context) $nativeContextKeys 'Batch ContextA differs from the single-query oracle.'
        Assert-B ([int]$zero.count -eq 0) 'Batched exact-zero result is not empty.'
    }
    Invoke-BCase '144-A' 'unscoped-and-non-TypeRef-families-stay-lower-bound' {
        $unscoped = Query $native @{
            type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'ContextA'
        } 'native-unscoped'
        Assert-LowerBound $unscoped 'unscoped session view'
        $declaration = Query $native @{
            workspaceRoot = $primary; type = 'reverse_edges'; domain = 'builtin';
            entity_type = 'Class'; name = 'ContextA'
        } 'native-class-control'
        Assert-LowerBound $declaration 'non-TypeRef reverse family'
    }
    Invoke-BCase '144-B' 'edit-refresh-invalidates-proof-and-removes-stale-edge' {
        Write-BText (Join-Path $primary 'group-b/OtherConsumer.cs') @'
namespace Issue144;
public class OtherConsumer { public OtherConsumer() { } }
'@
        $null = Tool $native 'refresh_workspace' @{ workspaceRoot = $primary } 'native-refresh-after-removal'
        $after = Reverse $native 'ContextA' 'native-after-removal'
        Assert-Exact $after 'post-removal ContextA'
        Assert-B ('OtherConsumer' -notin @(Constructor-Subjects $after 'ContextA')) 'Removed constructor dependency survived refresh.'
    }
    Invoke-BCase '144-B' 'delete-restore-and-rename-lifecycle' {
        $additionalFile = Join-Path $additional 'AdditionalConsumer.cs'
        Remove-Item -LiteralPath $additionalFile
        $null = Tool $native 'refresh_workspace' @{ workspaceRoot = $primary } 'native-refresh-after-delete'
        $deleted = Reverse $native 'ContextA' 'native-after-delete'
        Assert-Exact $deleted 'post-delete ContextA'
        Assert-B ('AdditionalConsumer' -notin @(Constructor-Subjects $deleted 'ContextA')) 'Deleted additional-root consumer survived.'
        Write-BText $additionalFile @'
namespace Issue144;
public class RestoredAdditionalConsumer { public RestoredAdditionalConsumer(ContextA value) { } }
'@
        Move-Item -LiteralPath $additionalFile -Destination (Join-Path $additional 'RenamedAdditionalConsumer.cs')
        $null = Tool $native 'refresh_workspace' @{ workspaceRoot = $primary } 'native-refresh-after-restore-rename'
        $restored = Reverse $native 'ContextA' 'native-after-restore-rename'
        Assert-Exact $restored 'post-restore/rename ContextA'
        Assert-B ('RestoredAdditionalConsumer' -in @(Constructor-Subjects $restored 'ContextA')) 'Restored renamed consumer was not acquired.'
        Assert-B ('AdditionalConsumer' -notin @(Constructor-Subjects $restored 'ContextA')) 'Pre-delete consumer identity reappeared.'
    }
    Stop-OwnedSession $native

    Invoke-BCase '147-B' 'mixed-producer-capability-and-full-unit-proof' {
        $mixed = Join-Path $runRoot 'mixed-language'
        New-Item -ItemType Directory -Force $mixed | Out-Null
        & git -C $mixed init --quiet
        Assert-B ($LASTEXITCODE -eq 0) 'Mixed-language Git root creation failed.'
        Write-BText (Join-Path $mixed '.clean-ctx.json') '{"cbm":{"enabled":false}}'
        Write-BText (Join-Path $mixed 'ContextA.cs') 'public class ContextA { }'
        Write-BText (Join-Path $mixed 'Consumer.cs') 'public class Consumer { public Consumer(ContextA value) { } }'
        Write-BText (Join-Path $mixed 'Unproved.ts') 'export class Unproved { }'
        Write-BText (Join-Path $mixed 'Unproved.java') 'public class UnprovedJava { }'
        Write-BText (Join-Path $mixed 'incapable.rs') 'pub struct Incapable;'
        $mixedSession = Start-BSession 'mixed' $BinaryPath $mixed
        $answer = Query $mixedSession @{
            workspaceRoot = $mixed; type = 'reverse_edges'; domain = 'builtin';
            entity_type = 'TypeRef'; name = 'ContextA'
        } 'mixed-language-full-unit-proof'
        Assert-Exact $answer 'mixed C#/TypeScript/Java/Rust unit set'
        Assert-Set (Constructor-Subjects $answer 'ContextA') @('Consumer') `
            'Mixed-language full-unit acquisition changed semantic truth.'
        Stop-OwnedSession $mixedSession
    }

    $renamedAdditional = Join-Path $additional 'RenamedAdditionalConsumer.cs'
    if (Test-Path -LiteralPath $renamedAdditional) {
        Remove-Item -LiteralPath $renamedAdditional
    }
    Write-Fixture

    Write-Config $true (Join-Path $runRoot 'missing-provider.exe')
    $unavailableHome = Join-Path $runRoot 'provider-unavailable-home'
    New-Item -ItemType Directory -Force $unavailableHome | Out-Null
    $providerNames = @(
        'codebase-memory-mcp.exe',
        'codebase-memory-mcp.cmd',
        'codebase-memory-mcp.bat',
        'codebase-memory-mcp'
    )
    $isolatedPath = @(
        [Environment]::GetEnvironmentVariable('PATH') -split [IO.Path]::PathSeparator |
            Where-Object {
                $directory = $_
                $directory -and -not @(
                    $providerNames | Where-Object {
                        Test-Path -LiteralPath (Join-Path $directory $_) -PathType Leaf
                    }
                ).Count
            }
    ) -join [IO.Path]::PathSeparator
    $unavailable = Start-BSession 'provider-unavailable' $BinaryPath $primary @{
        PATH = $isolatedPath
        HOME = $unavailableHome
        USERPROFILE = $unavailableHome
        HOMEDRIVE = [IO.Path]::GetPathRoot($unavailableHome).TrimEnd('\')
        HOMEPATH = $unavailableHome.Substring([IO.Path]::GetPathRoot($unavailableHome).Length - 1)
        LOCALAPPDATA = Join-Path $unavailableHome 'AppData/Local'
        APPDATA = Join-Path $unavailableHome 'AppData/Roaming'
    }
    Invoke-BCase '144-C' 'provider-unavailable-retains-Clean-authority' {
        $status = Wait-ProviderStatus $unavailable 'provider-unavailable-status'
        Assert-B ($status._meta.cbm_status -eq 'unavailable') (
            "Provider-unavailable fixture unexpectedly resolved CBM: $($status._meta.cbm_status)"
        )
        $answer = Reverse $unavailable 'ContextA' 'provider-unavailable-context-a'
        Assert-Exact $answer 'provider-unavailable ContextA'
        Assert-Set (Edge-Keys $answer) $nativeContextKeys 'Provider-unavailable truth differs from the native oracle.'
    }
    Stop-OwnedSession $unavailable

    Write-Config $true
    $provider = Start-BSession 'provider' $BinaryPath $primary
    Invoke-BCase '144-D' 'provider-admission' {
        $status = Wait-ProviderStatus $provider 'provider-status'
        Assert-B ($status._meta.cbm_status -eq 'available') 'Provider-enabled session did not establish CBM availability.'
        $null = Tool $provider 'refresh_workspace' @{ workspaceRoot = $primary } 'provider-refresh'
    }
    if ($verdicts[$verdicts.Count - 1].result -eq 'PASS') {
        Invoke-BCase '144-D' 'provider-and-Clean-semantic-parity' {
            $cold = Reverse $provider 'ContextA' 'provider-cold-context-a'
            Assert-Exact $cold 'provider cold ContextA'
            Assert-Set (Edge-Keys $cold) $nativeContextKeys 'Provider-enabled truth differs from the native oracle.'
            $warm = Reverse $provider 'ContextA' 'provider-warm-context-a'
            Assert-Exact $warm 'provider warm ContextA'
            Assert-Set (Edge-Keys $warm) $nativeContextKeys 'Provider warm truth differs from the native oracle.'
            Assert-NoWarmCompilation $warm 'provider warm ContextA'
        }
        Invoke-BCase '144-D' 'provider-empty-and-zero-have-no-unearned-authority' {
            $zero = Reverse $provider 'NeverReferenced' 'provider-exact-zero'
            Assert-Exact $zero 'provider exact zero backed by Clean proof'
            Assert-B ([int]$zero.count -eq 0) 'Provider exact-zero control returned an edge.'
        }
    }
    Stop-OwnedSession $provider
} finally {
    foreach ($openSession in @($sessions)) {
        try { Stop-BSession $openSession }
        catch { $verdicts.Add(@{ phase = 'cleanup'; case = $openSession.Name; result = 'FAIL'; evidence = $_.Exception.Message }) }
    }
    Save-B 'timings' @($timings.ToArray())
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #144 live evidence: $runRoot"
}

if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }

