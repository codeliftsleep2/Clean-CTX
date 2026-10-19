param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [string]$ProviderPath,
    [ValidateRange(60, 900)][int]$CaseTimeoutSeconds = 300
)

# R10 MSTest attribute-resolution acceptance over production MCP stdio.
# This builds nothing. Generated fixtures and evidence remain under target/.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
if ($ProviderPath) { $ProviderPath = (Resolve-Path $ProviderPath).Path }
$runRoot = Join-Path $repo ('target/csharp-r10-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

function New-Workspace([string]$Name, [bool]$ProviderEnabled, [bool]$TestingEnabled = $true) {
    $root = Join-Path $runRoot $Name
    New-Item -ItemType Directory -Force $root | Out-Null
    & git -C $root init --quiet
    Assert-B ($LASTEXITCODE -eq 0) "Could not initialize fixture root $Name."
    $cbm = @{ enabled = $ProviderEnabled; auto_launch = $ProviderEnabled }
    if ($ProviderEnabled) { $cbm.binary_path = $ProviderPath }
    Write-BText (Join-Path $root '.clean-ctx.json') (@{
        auto_delta = $false
        cbm = $cbm
        persistence = @{ enabled = $false; auto_save = $false }
        meta_layers = @{ dotnet = @{ testing = @{ enabled = $TestingEnabled } } }
    } | ConvertTo-Json -Depth 20)
    return $root
}

function Write-Subjects([string]$Root) {
    Write-BText (Join-Path $Root 'Subjects.cs') @'
namespace R10.Live;
public class ImportedService {}
public class ExplicitService {}
public class QualifiedService {}
public class GlobalService {}
public class NamespaceAliasService {}
public class TypeAliasService {}
public class PartialService {}
public class LifecycleService {}
'@
}

function Write-PositiveFixture([string]$Root) {
    Write-Subjects $Root
    Write-BText (Join-Path $Root 'ResolvedTests.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
using MsTest = Microsoft.VisualStudio.TestTools.UnitTesting;
using TestMarker = Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute;
namespace R10.Live;
[TestClass] public class ImportedServiceTests {}
[TestClassAttribute] public class ExplicitServiceTests {}
[Microsoft.VisualStudio.TestTools.UnitTesting.TestClass] public class QualifiedServiceTests {}
[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute] public class GlobalServiceTests {}
[MsTest.TestClass] public class NamespaceAliasServiceTests {}
[TestMarker] public class TypeAliasServiceTests {}
'@
    Write-BText (Join-Path $Root 'PartialBody.cs') @'
namespace R10.Live;
public partial class PartialServiceTests { public void Works() {} }
'@
    Write-BText (Join-Path $Root 'PartialMarker.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
namespace R10.Live;
[TestClass] public partial class PartialServiceTests {}
'@
    Write-BText (Join-Path $Root 'LifecycleTests.cs') @'
using Marker = Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute;
namespace R10.Live;
[Marker] public class LifecycleServiceTests {}
'@
}

function Write-NegativeFixture([string]$Root) {
    Write-BText (Join-Path $Root 'LocalShadow.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
namespace R10.Shadow;
public class TestClassAttribute : System.Attribute {}
[TestClass] public class ShadowServiceTests {}
'@
    Write-BText (Join-Path $Root 'CompetingImport.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
using R10.Other;
namespace R10.Other { public class TestClassAttribute : System.Attribute {} }
namespace R10.Ambiguous { [TestClass] public class AmbiguousServiceTests {} }
'@
    Write-BText (Join-Path $Root 'NoImport.cs') @'
namespace R10.Unsupported;
[TestClass] public class NoImportServiceTests {}
'@
    Write-BText (Join-Path $Root 'Neutral.cs') 'namespace R10.Live; public class NeutralType {}'
}

function Tool($Session, [string]$Name, [hashtable]$Arguments, [string]$Evidence) {
    $result = Get-BResult (Invoke-BTool $Session $Name $Arguments)
    Save-B $Evidence $result
    return $result
}

function Query($Session, [string]$Root, [hashtable]$Arguments, [string]$Evidence) {
    $Arguments.workspaceRoot = $Root
    $result = Tool $Session 'workspace_query' $Arguments $Evidence
    Assert-B ([bool]$result.PSObject.Properties['structuredContent']) "$Evidence omitted structuredContent."
    return $result.structuredContent
}

function Refresh($Session, [string]$Root, [string]$Evidence) {
    $result = Tool $Session 'refresh_workspace' @{ workspaceRoot = $Root } $Evidence
    Assert-B ($result.structuredContent.refreshed -eq $true) "$Evidence did not refresh."
}

function Test-Semantics($Session, [string]$Root, [string]$Name, [string]$Evidence) {
    $entities = Query $Session $Root @{
        type = 'find_entities'; domain = 'dotnet'; entity_type = 'TestClass'; name = $Name
    } "$Evidence-entities"
    $edges = Query $Session $Root @{
        type = 'forward_edges'; domain = 'dotnet'; entity_type = 'TestClass'; name = $Name
    } "$Evidence-edges"
    return [ordered]@{
        test_class_count = @($entities.entities | Where-Object {
            $_.domain -eq 'dotnet' -and $_.entity_type -eq 'TestClass' -and $_.name -eq $Name
        }).Count
        tests = @($edges.edges | Where-Object relation -eq 'Tests' | ForEach-Object {
            "$($_.object.domain)|$($_.object.entity_type)|$($_.object.name)|$([IO.Path]::GetFileName([string]$_.object.file))"
        } | Sort-Object -Unique)
    }
}

function Assert-TestClass($Actual, [string]$Name, [string]$Subject) {
    Assert-B ($Actual.test_class_count -eq 1) "$Name TestClass identity count was $($Actual.test_class_count)."
    $expected = "builtin|Class|$Subject|Subjects.cs"
    Assert-B ((@($Actual.tests) -join ',') -ceq $expected) "$Name Tests target mismatch: $(@($Actual.tests) -join ',')."
}

function Assert-NoTestClass($Actual, [string]$Name) {
    Assert-B ($Actual.test_class_count -eq 0) "$Name acquired false MSTest authority."
    Assert-B (@($Actual.tests).Count -eq 0) "$Name acquired a false Tests relationship."
}

function Stable($Value) { $Value | ConvertTo-Json -Depth 30 -Compress }

function Wait-Provider($Session, [string]$Evidence) {
    do {
        $status = Tool $Session 'get_cbm_status' @{} "$Evidence-$rpcId"
        $observed = [string]$status._meta.cbm_status
        if ($observed -eq 'available') { return }
        Assert-B ($observed -eq 'initializing') "Provider reached terminal state '$observed'."
        Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Provider readiness deadline exceeded.'
        Start-Sleep -Milliseconds 250
    } while ($true)
}

function Stop-Owned($Session) {
    Stop-BSession $Session
    $null = $sessions.Remove($Session)
}

function Invoke-Matrix($Session, [string]$Root, [string]$Prefix) {
    $positive = [ordered]@{
        ImportedServiceTests = 'ImportedService'
        ExplicitServiceTests = 'ExplicitService'
        QualifiedServiceTests = 'QualifiedService'
        GlobalServiceTests = 'GlobalService'
        NamespaceAliasServiceTests = 'NamespaceAliasService'
        TypeAliasServiceTests = 'TypeAliasService'
        PartialServiceTests = 'PartialService'
    }
    foreach ($entry in $positive.GetEnumerator()) {
        Assert-TestClass (Test-Semantics $Session $Root $entry.Key "$Prefix-$($entry.Key)") $entry.Key $entry.Value
    }
    foreach ($name in @('ShadowServiceTests', 'AmbiguousServiceTests', 'NoImportServiceTests')) {
        Assert-NoTestClass (Test-Semantics $Session $Root $name "$Prefix-$name") $name
    }
}

function New-HistorySnapshot([string]$Mode) {
    $root = New-Workspace "history-$Mode" $false
    Write-PositiveFixture $root
    Write-NegativeFixture $root
    $session = Start-BSession "r10-history-$Mode" $BinaryPath $root
    try {
        switch ($Mode) {
            'unrelated' {
                $null = Query $session $root @{ type = 'find_entities'; name = 'NeutralType' } "$Mode-prelude"
            }
            'declaration' {
                $null = Query $session $root @{ type = 'find_entities'; name = 'ImportedService' } "$Mode-prelude"
            }
            'typeref' {
                $null = Query $session $root @{
                    type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'ImportedService'
                } "$Mode-prelude"
            }
            'warm' {
                $null = Test-Semantics $session $root 'ImportedServiceTests' "$Mode-prelude"
            }
        }
        return Test-Semantics $session $root 'ImportedServiceTests' "$Mode-result"
    } finally {
        Stop-Owned $session
    }
}

New-Item -ItemType Directory -Force $runRoot | Out-Null
Save-B 'provenance' @{
    agent = 'Agent MaxHeadRoom'
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash $BinaryPath).Hash
    provider = $ProviderPath
    contract = 'C# semantic resolution R10 attribute foundation and MSTest migration'
}

try {
    $root = New-Workspace 'provider-disabled' $false
    Write-PositiveFixture $root
    Write-NegativeFixture $root
    $session = Start-BSession 'r10-disabled' $BinaryPath $root

    Invoke-BCase 'R10-A' 'registered-production-boundary' {
        $tools = Get-BResult (Invoke-BRpc $session 'tools/list' @{})
        foreach ($name in @('workspace_query', 'refresh_workspace')) {
            Assert-B ($name -in $tools.tools.name) "Registered tool absent: $name."
        }
    }
    Invoke-BCase 'R10-B' 'complete-positive-negative-matrix' {
        Invoke-Matrix $session $root 'disabled-matrix'
    }
    Invoke-BCase 'R10-C' 'fresh-history-and-warm-independence' {
        $baseline = New-HistorySnapshot 'direct'
        foreach ($mode in @('unrelated', 'declaration', 'typeref', 'warm')) {
            $actual = New-HistorySnapshot $mode
            Assert-B ((Stable $actual) -ceq (Stable $baseline)) "$mode history changed MSTest semantic truth."
        }
    }
    Invoke-BCase 'R10-D' 'alias-removal-restoration-and-deletion' {
        Assert-TestClass (Test-Semantics $session $root 'LifecycleServiceTests' 'life-before') 'LifecycleServiceTests' 'LifecycleService'
        Write-BText (Join-Path $root 'LifecycleTests.cs') @'
using Marker = R10.Other.TestClassAttribute;
namespace R10.Other { public class TestClassAttribute : System.Attribute {} }
namespace R10.Live { [Marker] public class LifecycleServiceTests {} }
'@
        Refresh $session $root 'life-retarget-refresh'
        Assert-NoTestClass (Test-Semantics $session $root 'LifecycleServiceTests' 'life-retargeted') 'LifecycleServiceTests'
        Write-BText (Join-Path $root 'LifecycleTests.cs') @'
using Marker = Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute;
namespace R10.Live;
[Marker] public class LifecycleServiceTests {}
'@
        Refresh $session $root 'life-restore-refresh'
        Assert-TestClass (Test-Semantics $session $root 'LifecycleServiceTests' 'life-restored') 'LifecycleServiceTests' 'LifecycleService'
        Remove-Item -LiteralPath (Join-Path $root 'LifecycleTests.cs') -Force
        Refresh $session $root 'life-delete-refresh'
        Assert-NoTestClass (Test-Semantics $session $root 'LifecycleServiceTests' 'life-deleted') 'LifecycleServiceTests'

        Remove-Item -LiteralPath (Join-Path $root 'PartialMarker.cs') -Force
        Refresh $session $root 'partial-remove-refresh'
        Assert-NoTestClass (Test-Semantics $session $root 'PartialServiceTests' 'partial-removed') 'PartialServiceTests'
        Write-BText (Join-Path $root 'PartialMarker.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
namespace R10.Live;
[TestClass] public partial class PartialServiceTests {}
'@
        Refresh $session $root 'partial-restore-refresh'
        Assert-TestClass (Test-Semantics $session $root 'PartialServiceTests' 'partial-restored') 'PartialServiceTests' 'PartialService'
    }
    Stop-Owned $session

    $disabledRoot = New-Workspace 'testing-disabled' $false $false
    Write-PositiveFixture $disabledRoot
    $disabledSession = Start-BSession 'r10-testing-disabled' $BinaryPath $disabledRoot
    Invoke-BCase 'R10-E' 'testing-config-disables-semantic-authority-only' {
        Assert-NoTestClass (Test-Semantics $disabledSession $disabledRoot 'ImportedServiceTests' 'config-disabled') 'ImportedServiceTests'
        $neutral = Query $disabledSession $disabledRoot @{
            type = 'find_entities'; domain = 'builtin'; entity_type = 'Class'; name = 'ImportedServiceTests'
        } 'config-disabled-neutral'
        Assert-B (@($neutral.entities | Where-Object {
            $_.domain -eq 'builtin' -and $_.entity_type -eq 'Class' -and $_.name -eq 'ImportedServiceTests'
        }).Count -eq 1) 'Testing opt-out removed neutral C# class truth.'
    }
    Stop-Owned $disabledSession

    $crossRoot = New-Workspace 'cross-root-primary' $false
    $additionalRoot = Join-Path $runRoot 'cross-root-additional'
    New-Item -ItemType Directory -Force $additionalRoot | Out-Null
    & git -C $additionalRoot init --quiet
    Assert-B ($LASTEXITCODE -eq 0) 'Could not initialize additional fixture root.'
    Write-BText (Join-Path $crossRoot '.clean-ctx.json') (@{
        auto_delta = $false
        cbm = @{ enabled = $false; auto_launch = $false }
        persistence = @{ enabled = $false; auto_save = $false }
        additional_roots = @($additionalRoot)
    } | ConvertTo-Json -Depth 20)
    Write-BText (Join-Path $crossRoot 'Subjects.cs') 'namespace R10.Cross; public class CrossRootService {}'
    Write-BText (Join-Path $additionalRoot 'CrossRootServiceTests.cs') @'
using Microsoft.VisualStudio.TestTools.UnitTesting;
namespace R10.Cross;
[TestClass] public class CrossRootServiceTests {}
'@
    $crossSession = Start-BSession 'r10-cross-root' $BinaryPath $crossRoot
    Invoke-BCase 'R10-F' 'cross-root-subject-and-test-authority' {
        Assert-TestClass (Test-Semantics $crossSession $crossRoot 'CrossRootServiceTests' 'cross-root') 'CrossRootServiceTests' 'CrossRootService'
    }
    Stop-Owned $crossSession

    if ($ProviderPath) {
        $providerRoot = New-Workspace 'provider-ready-stale' $true
        Write-BText (Join-Path $providerRoot 'ProviderSeed.cs') 'namespace R10.Live; public class ProviderSeed {}'
        $providerSession = Start-BSession 'r10-provider' $BinaryPath $providerRoot
        Invoke-BCase 'R10-G' 'ready-provider-plausible-incomplete-snapshot' {
            Wait-Provider $providerSession 'provider-ready'
            Write-PositiveFixture $providerRoot
            Write-NegativeFixture $providerRoot
            Invoke-Matrix $providerSession $providerRoot 'provider-stale-matrix'
        }
        Stop-Owned $providerSession
    }
} finally {
    foreach ($owned in @($sessions.ToArray())) {
        try { Stop-BSession $owned } catch { Write-Warning $_.Exception.Message }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "R10 live evidence: $runRoot"
}

$failed = @($verdicts | Where-Object result -eq 'FAIL')
if ($failed.Count -gt 0) {
    throw "R10 live acceptance failed $($failed.Count) case(s). Evidence: $runRoot"
}
Write-Host 'R10 LIVE MCP ACCEPTANCE: PASS' -ForegroundColor Green
