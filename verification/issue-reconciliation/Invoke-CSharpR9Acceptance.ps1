param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [string]$ProviderPath,
    [ValidateRange(60, 900)][int]$CaseTimeoutSeconds = 300
)

# R9 ASP.NET base-resolution acceptance over the production MCP stdio boundary.
# This builds nothing. Fixtures, RPC frames, stderr, and verdicts stay in target/.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
if ($ProviderPath) { $ProviderPath = (Resolve-Path $ProviderPath).Path }
$runRoot = Join-Path $repo ('target/csharp-r9-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

function New-Workspace([string]$Name, [bool]$ProviderEnabled) {
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
    } | ConvertTo-Json -Depth 20)
    return $root
}

function Write-PositiveFixture([string]$Root) {
    Write-BText (Join-Path $Root 'DirectControllers.cs') @'
using Microsoft.AspNetCore.Mvc;
using Mvc = Microsoft.AspNetCore.Mvc;
using ControllerRoot = Microsoft.AspNetCore.Mvc.ControllerBase;
namespace R9.Live;
[Route("imported")] public class ImportedEndpoint : ControllerBase { [HttpGet] public void Imported() {} }
[Route("qualified")] public class QualifiedEndpoint : Microsoft.AspNetCore.Mvc.Controller { [HttpGet] public void Qualified() {} }
[Route("global")] public class GlobalEndpoint : global::Microsoft.AspNetCore.Mvc.ControllerBase { [HttpGet] public void Global() {} }
[Route("namespace-alias")] public class NamespaceAliasEndpoint : Mvc.ControllerBase { [HttpGet] public void NamespaceAlias() {} }
[Route("type-alias")] public class TypeAliasEndpoint : ControllerRoot { [HttpGet] public void TypeAlias() {} }
'@
    Write-BText (Join-Path $Root 'ResolvedBase.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
public class ResolvedBase : ControllerBase {}
'@
    Write-BText (Join-Path $Root 'MiddleBase.cs') @'
namespace R9.Live;
public class MiddleBase : ResolvedBase {}
'@
    Write-BText (Join-Path $Root 'DerivedEndpoint.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
[Route("derived")] public class DerivedEndpoint : MiddleBase { [HttpGet] public void Derived() {} }
'@
    Write-BText (Join-Path $Root 'PartialBase.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
public partial class PartialEndpoint : ControllerBase {}
'@
    Write-BText (Join-Path $Root 'PartialPayload.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
[Route("partial")] public partial class PartialEndpoint { [HttpGet] public void Partial() {} }
'@
}

function Write-NegativeFixture([string]$Root) {
    Write-BText (Join-Path $Root 'LocalShadow.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Shadow;
public class ControllerBase {}
[Route("bad-shadow")] public class ShadowEndpoint : ControllerBase { [HttpGet] public void Shadow() {} }
'@
    Write-BText (Join-Path $Root 'CompetingImport.cs') @'
using Microsoft.AspNetCore.Mvc;
using R9.Other;
namespace R9.Other { public class ControllerBase {} }
namespace R9.Ambiguous {
[Route("bad-ambiguous")] public class AmbiguousEndpoint : ControllerBase { [HttpGet] public void Ambiguous() {} }
}
'@
    Write-BText (Join-Path $Root 'WrongArity.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
[Route("bad-arity")] public class WrongArityEndpoint : Microsoft.AspNetCore.Mvc.ControllerBase<int> { [HttpGet] public void Wrong() {} }
'@
    Write-BText (Join-Path $Root 'CycleA.cs') @'
namespace R9.Cycle;
public class CycleA : CycleB {}
'@
    Write-BText (Join-Path $Root 'CycleB.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Cycle;
[Route("bad-cycle")] public class CycleB : CycleA { [HttpGet] public void Cycle() {} }
'@
    Write-BText (Join-Path $Root 'AttributeBoundary.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
[ApiController] [Route("attribute-boundary")] public class AttributeOnlyEndpoint { [HttpGet] public void AttributeOnly() {} }
'@
    Write-BText (Join-Path $Root 'Neutral.cs') 'namespace R9.Live; public class NeutralType {}'
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

function Controller-Entities($Session, [string]$Root, [string]$Name, [string]$Evidence) {
    Query $Session $Root @{
        type = 'find_entities'; domain = 'dotnet'; entity_type = 'Controller'; name = $Name
    } $Evidence
}

function Controller-Edges($Session, [string]$Root, [string]$Name, [string]$Evidence) {
    Query $Session $Root @{
        type = 'forward_edges'; domain = 'dotnet'; entity_type = 'Controller'; name = $Name
    } $Evidence
}

function Controller-Semantics($Session, [string]$Root, [string]$Name, [string]$Evidence) {
    $entities = Controller-Entities $Session $Root $Name "$Evidence-entities"
    $edges = Controller-Edges $Session $Root $Name "$Evidence-edges"
    return [ordered]@{
        controller_count = @($entities.entities | Where-Object {
            $_.domain -eq 'dotnet' -and $_.entity_type -eq 'Controller' -and $_.name -eq $Name
        }).Count
        routes = @($edges.edges | Where-Object relation -eq 'HasRoute' |
            ForEach-Object { [string]$_.object.name } | Sort-Object)
        actions = @($edges.edges | Where-Object relation -eq 'ControllerAction' |
            ForEach-Object { [string]$_.object.name } | Sort-Object)
    }
}

function Assert-Controller(
    $Actual,
    [string]$Name,
    [string]$Route,
    [string]$Action
) {
    Assert-B ($Actual.controller_count -eq 1) "$Name controller identity count was $($Actual.controller_count)."
    Assert-B ((@($Actual.routes) -join ',') -ceq $Route) "$Name route mismatch: $(@($Actual.routes) -join ',')."
    Assert-B ((@($Actual.actions) -join ',') -ceq $Action) "$Name action mismatch: $(@($Actual.actions) -join ',')."
}

function Assert-NoController($Actual, [string]$Name) {
    Assert-B ($Actual.controller_count -eq 0) "$Name acquired false controller authority."
    Assert-B (@($Actual.routes).Count -eq 0) "$Name acquired false route authority."
    Assert-B (@($Actual.actions).Count -eq 0) "$Name acquired false action authority."
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
        ImportedEndpoint = @('imported', 'Imported')
        QualifiedEndpoint = @('qualified', 'Qualified')
        GlobalEndpoint = @('global', 'Global')
        NamespaceAliasEndpoint = @('namespace-alias', 'NamespaceAlias')
        TypeAliasEndpoint = @('type-alias', 'TypeAlias')
        DerivedEndpoint = @('derived', 'Derived')
        PartialEndpoint = @('partial', 'Partial')
    }
    foreach ($entry in $positive.GetEnumerator()) {
        $actual = Controller-Semantics $Session $Root $entry.Key "$Prefix-$($entry.Key)"
        Assert-Controller $actual $entry.Key $entry.Value[0] $entry.Value[1]
    }
    foreach ($name in @('ShadowEndpoint', 'AmbiguousEndpoint', 'WrongArityEndpoint', 'CycleB')) {
        Assert-NoController (Controller-Semantics $Session $Root $name "$Prefix-$name") $name
    }
}

New-Item -ItemType Directory -Force $runRoot | Out-Null
Save-B 'provenance' @{
    agent = 'Agent MaxHeadRoom'
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash $BinaryPath).Hash
    provider = $ProviderPath
    contract = 'C# semantic resolution R9 ASP.NET base convergence'
}

try {
    $root = New-Workspace 'provider-disabled' $false
    Write-PositiveFixture $root
    Write-NegativeFixture $root
    $session = Start-BSession 'r9-disabled' $BinaryPath $root

    Invoke-BCase 'R9-A' 'registered-production-boundary' {
        $tools = Get-BResult (Invoke-BRpc $session 'tools/list' @{})
        foreach ($name in @('workspace_query', 'refresh_workspace')) {
            Assert-B ($name -in $tools.tools.name) "Registered tool absent: $name."
        }
    }
    Invoke-BCase 'R9-B' 'complete-positive-negative-matrix' {
        Invoke-Matrix $session $root 'disabled-matrix'
    }
    Invoke-BCase 'R9-C' 'history-and-warm-independence' {
        $cold = Controller-Semantics $session $root 'DerivedEndpoint' 'history-cold'
        $null = Query $session $root @{ type = 'find_entities'; name = 'NeutralType' } 'history-unrelated'
        $related = Controller-Semantics $session $root 'DerivedEndpoint' 'history-related'
        $warm = Controller-Semantics $session $root 'DerivedEndpoint' 'history-warm'
        Assert-B ((Stable $cold) -ceq (Stable $related)) 'Unrelated history changed controller truth.'
        Assert-B ((Stable $cold) -ceq (Stable $warm)) 'Warm history changed controller truth.'
    }
    Invoke-BCase 'R9-D' 'neutral-file-control-and-r11-boundary' {
        $neutral = Query $session $root @{
            type = 'entities_in_file'; file_path = 'Neutral.cs'; fidelity = 'high'
        } 'neutral-file'
        Assert-B (@($neutral.entities | Where-Object name -eq 'NeutralType').Count -eq 1) 'Neutral file truth missing.'
        $attribute = Controller-Semantics $session $root 'AttributeOnlyEndpoint' 'r11-attribute-boundary'
        Assert-Controller $attribute 'AttributeOnlyEndpoint' 'attribute-boundary' 'AttributeOnly'
    }
    Invoke-BCase 'R9-E' 'ancestry-removal-restoration-and-owner-deletion' {
        Assert-Controller (Controller-Semantics $session $root 'DerivedEndpoint' 'life-before') 'DerivedEndpoint' 'derived' 'Derived'
        Write-BText (Join-Path $root 'ResolvedBase.cs') 'namespace R9.Live; public class ResolvedBase {}'
        Refresh $session $root 'life-remove-refresh'
        Assert-NoController (Controller-Semantics $session $root 'DerivedEndpoint' 'life-removed') 'DerivedEndpoint'
        Write-BText (Join-Path $root 'ResolvedBase.cs') @'
using Microsoft.AspNetCore.Mvc;
namespace R9.Live;
public class ResolvedBase : ControllerBase {}
'@
        Refresh $session $root 'life-restore-refresh'
        Assert-Controller (Controller-Semantics $session $root 'DerivedEndpoint' 'life-restored') 'DerivedEndpoint' 'derived' 'Derived'
        Remove-Item -LiteralPath (Join-Path $root 'DerivedEndpoint.cs') -Force
        Refresh $session $root 'life-delete-refresh'
        Assert-NoController (Controller-Semantics $session $root 'DerivedEndpoint' 'life-deleted') 'DerivedEndpoint'
    }
    Stop-Owned $session

    if ($ProviderPath) {
        $providerRoot = New-Workspace 'provider-ready-stale' $true
        Write-BText (Join-Path $providerRoot 'ProviderSeed.cs') 'namespace R9.Live; public class ProviderSeed {}'
        $providerSession = Start-BSession 'r9-provider' $BinaryPath $providerRoot
        Invoke-BCase 'R9-F' 'ready-provider-plausible-incomplete-snapshot' {
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
    Write-Host "R9 live evidence: $runRoot"
}

$failed = @($verdicts | Where-Object result -eq 'FAIL')
if ($failed.Count -gt 0) {
    throw "R9 live acceptance failed $($failed.Count) case(s). Evidence: $runRoot"
}
Write-Host 'R9 LIVE MCP ACCEPTANCE: PASS' -ForegroundColor Green
