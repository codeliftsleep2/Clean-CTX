param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [string]$ProviderPath,
    [ValidateRange(60, 900)][int]$CaseTimeoutSeconds = 300
)

# R12 cross-consumer applicability/convergence acceptance over production MCP stdio.
# This builds nothing. Generated fixtures and evidence remain under target/.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
if ($ProviderPath) { $ProviderPath = (Resolve-Path $ProviderPath).Path }
$runRoot = Join-Path $repo ('target/csharp-r12-live-verification/' +
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

function New-Workspace([string]$Name, [bool]$ProviderEnabled, [string[]]$AdditionalRoots = @()) {
    $root = Join-Path $runRoot $Name
    New-Item -ItemType Directory -Force $root | Out-Null
    & git -C $root init --quiet
    Assert-B ($LASTEXITCODE -eq 0) "Could not initialize fixture root $Name."
    $cbm = @{ enabled = $ProviderEnabled; auto_launch = $ProviderEnabled }
    if ($ProviderEnabled) { $cbm.binary_path = $ProviderPath }
    $config = @{
        auto_delta = $false
        cbm = $cbm
        persistence = @{ enabled = $false; auto_save = $false }
    }
    if ($AdditionalRoots.Count -gt 0) { $config.additional_roots = $AdditionalRoots }
    Write-BText (Join-Path $root '.clean-ctx.json') ($config | ConvertTo-Json -Depth 20)
    return $root
}

function Write-Fixture([string]$Root) {
    Write-BText (Join-Path $Root 'AppContext.cs') 'public class AppContext : global::Microsoft.EntityFrameworkCore.DbContext {}'
    Write-BText (Join-Path $Root 'Maps.cs') @'
public class Maps : global::AutoMapper.Profile {
    public Maps() { CreateMap<Source, Destination>(); }
}
'@
    Write-BText (Join-Path $Root 'Events.cs') @'
public class Events : global::Microsoft.AspNetCore.SignalR.Hub {
    public void Broadcast() {}
}
'@
    Write-BText (Join-Path $Root 'OrdersEndpoint.cs') @'
[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute]
[global::Microsoft.AspNetCore.Mvc.RouteAttribute("orders")]
public class OrdersEndpoint {
    [global::Microsoft.AspNetCore.Mvc.HttpHeadAttribute]
    public void Head() {}
}
'@
    Write-BText (Join-Path $Root 'OrdersTests.cs') @'
[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute]
public class OrdersTests {}
'@
    Write-BText (Join-Path $Root 'Neutral.cs') 'public class NeutralType {}'
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

function Find-Role($Session, [string]$Root, [string]$Type, [string]$Name, [string]$Evidence) {
    $result = Query $Session $Root @{
        type = 'find_entities'; domain = 'dotnet'; entity_type = $Type; name = $Name
    } $Evidence
    return @($result.entities | Where-Object {
        $_.domain -eq 'dotnet' -and $_.entity_type -eq $Type -and $_.name -eq $Name
    }).Count
}

function Forward-Facts($Session, [string]$Root, [string]$Type, [string]$Name, [string]$Evidence) {
    $result = Query $Session $Root @{
        type = 'forward_edges'; domain = 'dotnet'; entity_type = $Type; name = $Name
    } $Evidence
    return @($result.edges | ForEach-Object {
        "$Type|$Name|$($_.relation)|$($_.object.entity_type)|$($_.object.name)|$([IO.Path]::GetFileName([string]$_.object.file))"
    } | Sort-Object -Unique)
}

function Snapshot($Session, [string]$Root, [string]$Prefix) {
    $roles = [ordered]@{
        DbContext = Find-Role $Session $Root 'DbContext' 'AppContext' "$Prefix-ef-role"
        Controller = Find-Role $Session $Root 'Controller' 'OrdersEndpoint' "$Prefix-controller-role"
        TestClass = Find-Role $Session $Root 'TestClass' 'OrdersTests' "$Prefix-test-role"
    }
    $facts = @()
    $facts += Forward-Facts $Session $Root 'MapperProfile' 'Maps' "$Prefix-maps"
    $facts += Forward-Facts $Session $Root 'Hub' 'Events' "$Prefix-hub"
    $facts += Forward-Facts $Session $Root 'Controller' 'OrdersEndpoint' "$Prefix-controller"
    return [ordered]@{ roles = $roles; facts = @($facts | Sort-Object -Unique) }
}

function Assert-Complete($Actual, [string]$Label) {
    foreach ($entry in $Actual.roles.GetEnumerator()) {
        Assert-B ($entry.Value -eq 1) "$Label $($entry.Key) role count was $($entry.Value)."
    }
    foreach ($required in @(
        'MapperProfile|Maps|MapsFrom|Entity|Source|Maps.cs',
        'MapperProfile|Maps|MapsTo|Entity|Destination|Maps.cs',
        'Hub|Events|HubMethodTargets|HubMethod|Broadcast|Events.cs',
        'Controller|OrdersEndpoint|HasRoute|Route|orders|OrdersEndpoint.cs',
        'Controller|OrdersEndpoint|ControllerAction|Action|Head|OrdersEndpoint.cs'
    )) {
        Assert-B ($required -in $Actual.facts) "$Label omitted $required."
    }
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

function New-HistorySnapshot([string]$Mode) {
    $root = New-Workspace "history-$Mode" $false
    Write-Fixture $root
    $session = Start-BSession "r12-history-$Mode" $BinaryPath $root
    try {
        switch ($Mode) {
            'unrelated' { $null = Query $session $root @{ type = 'find_entities'; name = 'NeutralType' } "$Mode-prelude" }
            'declaration' { $null = Query $session $root @{ type = 'find_entities'; name = 'Source' } "$Mode-prelude" }
            'typeref' {
                $null = Query $session $root @{
                    type = 'reverse_edges'; domain = 'builtin'; entity_type = 'TypeRef'; name = 'Source'
                } "$Mode-prelude"
            }
            'warm' { $null = Snapshot $session $root "$Mode-prelude" }
        }
        return Snapshot $session $root "$Mode-result"
    } finally { Stop-Owned $session }
}

New-Item -ItemType Directory -Force $runRoot | Out-Null
Save-B 'provenance' @{
    agent = 'Agent MaxHeadRoom'
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash $BinaryPath).Hash
    provider = $ProviderPath
    contract = 'C# semantic resolution R12 applicability and cross-consumer convergence'
}

try {
    $root = New-Workspace 'provider-disabled' $false
    Write-Fixture $root
    $session = Start-BSession 'r12-disabled' $BinaryPath $root

    Invoke-BCase 'R12-A' 'registered-production-boundary' {
        $tools = Get-BResult (Invoke-BRpc $session 'tools/list' @{})
        foreach ($name in @('workspace_query', 'refresh_workspace')) {
            Assert-B ($name -in $tools.tools.name) "Registered tool absent: $name."
        }
    }
    Invoke-BCase 'R12-B' 'cross-consumer-qualified-identity-matrix' {
        $snapshot = Snapshot $session $root 'disabled'
        Assert-Complete $snapshot 'provider-disabled'
    }
    Invoke-BCase 'R12-C' 'fresh-history-and-warm-independence' {
        $baseline = New-HistorySnapshot 'direct'
        Assert-Complete $baseline 'history baseline'
        foreach ($mode in @('unrelated', 'declaration', 'typeref', 'warm')) {
            $actual = New-HistorySnapshot $mode
            Assert-B ((Stable $actual) -ceq (Stable $baseline)) "$mode history changed semantic truth."
        }
    }
    Invoke-BCase 'R12-D' 'consumer-retraction-isolation' {
        $before = Snapshot $session $root 'life-before'
        Write-BText (Join-Path $root 'Maps.cs') @'
namespace Local { public class Profile {} }
public class Maps : Local.Profile { public Maps() { CreateMap<Source, Destination>(); } }
'@
        Refresh $session $root 'life-profile-refresh'
        $withoutProfile = Snapshot $session $root 'life-without-profile'
        Assert-B (@($withoutProfile.facts | Where-Object { $_ -like 'MapperProfile|Maps|*' }).Count -eq 0) 'Profile authority did not retract.'
        Assert-B ($withoutProfile.roles.DbContext -eq 1 -and $withoutProfile.roles.Controller -eq 1 -and $withoutProfile.roles.TestClass -eq 1) 'Profile retraction disturbed another role.'
        Remove-Item -LiteralPath (Join-Path $root 'Events.cs') -Force
        Refresh $session $root 'life-hub-delete-refresh'
        $withoutHub = Snapshot $session $root 'life-without-hub'
        Assert-B (@($withoutHub.facts | Where-Object { $_ -like 'Hub|Events|*' }).Count -eq 0) 'Hub deletion did not retract.'
        Assert-B ($withoutHub.roles.DbContext -eq 1 -and $withoutHub.roles.Controller -eq 1 -and $withoutHub.roles.TestClass -eq 1) 'Hub deletion disturbed another role.'
        Assert-B ($before.roles.DbContext -eq 1) 'Lifecycle baseline was not authoritative.'
    }
    Stop-Owned $session

    $additional = Join-Path $runRoot 'cross-root-additional'
    New-Item -ItemType Directory -Force $additional | Out-Null
    & git -C $additional init --quiet
    Assert-B ($LASTEXITCODE -eq 0) 'Could not initialize additional root.'
    $crossRoot = New-Workspace 'cross-root-primary' $false @($additional)
    Write-BText (Join-Path $crossRoot 'OrdersEndpoint.cs') '[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute] public class OrdersEndpoint {}'
    Write-BText (Join-Path $additional 'Events.cs') 'public class Events : global::Microsoft.AspNetCore.SignalR.Hub { public void Broadcast() {} }'
    $crossSession = Start-BSession 'r12-cross-root' $BinaryPath $crossRoot
    Invoke-BCase 'R12-E' 'cross-root-and-query-order-convergence' {
        $null = Find-Role $crossSession $crossRoot 'Controller' 'OrdersEndpoint' 'cross-controller'
        $hub = Forward-Facts $crossSession $crossRoot 'Hub' 'Events' 'cross-hub'
        Assert-B ('Hub|Events|HubMethodTargets|HubMethod|Broadcast|Events.cs' -in $hub) 'Additional-root Hub truth was omitted.'
        Assert-B ((Find-Role $crossSession $crossRoot 'Controller' 'OrdersEndpoint' 'cross-controller-warm') -eq 1) 'Primary-root controller truth changed after additional-root hydration.'
    }
    Stop-Owned $crossSession

    if ($ProviderPath) {
        $providerRoot = New-Workspace 'provider-ready-stale' $true
        Write-BText (Join-Path $providerRoot 'ProviderSeed.cs') 'public class ProviderSeed {}'
        $providerSession = Start-BSession 'r12-provider' $BinaryPath $providerRoot
        Invoke-BCase 'R12-F' 'ready-provider-plausible-incomplete-snapshot' {
            Wait-Provider $providerSession 'provider-ready'
            Write-Fixture $providerRoot
            $snapshot = Snapshot $providerSession $providerRoot 'provider-stale'
            Assert-Complete $snapshot 'ready-provider stale snapshot'
        }
        Stop-Owned $providerSession
    }
} finally {
    foreach ($owned in @($sessions.ToArray())) {
        try { Stop-BSession $owned } catch { Write-Warning $_.Exception.Message }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "R12 live evidence: $runRoot"
}

$failed = @($verdicts | Where-Object result -eq 'FAIL')
if ($failed.Count -gt 0) {
    throw "R12 live acceptance failed $($failed.Count) case(s). Evidence: $runRoot"
}
Write-Host 'R12 LIVE MCP ACCEPTANCE: PASS' -ForegroundColor Green
