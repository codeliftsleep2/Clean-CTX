param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=240)
# Consolidated Issue #139 live MCP acceptance. Builds nothing and starts no CBM provider.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/issue-139-live-verification/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'workspace'
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'

function Pad([string]$Source) {return $Source+(([Environment]::NewLine+'// unchanged economics padding')*100)+[Environment]::NewLine}
function Publish($Session,[string[]]$Files) {
    foreach($file in $Files) {
        $answer=Get-BResult (Invoke-BTool $Session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='high'})
        Save-B ('publish-'+$file.Replace('.','-')) $answer
    }
}
function Query-Edges($Session,[string]$EntityType,[string]$Name,[string]$Evidence) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='angular';entity_type=$EntityType;name=$Name})
    Save-B $Evidence $answer
    Assert-B ([bool]$answer.PSObject.Properties['structuredContent']) 'workspace_query omitted structuredContent.'
    return @($answer.structuredContent.edges)
}
function Relation-Targets([object[]]$Edges,[string]$Relation) {
    return @($Edges | Where-Object relation -eq $Relation | ForEach-Object {"$($_.object.entity_type)/$($_.object.name)"} | Sort-Object -Unique)
}
function Assert-Targets([object[]]$Edges,[string]$Relation,[string[]]$Expected,[string]$Message) {
    $actual=@(Relation-Targets $Edges $Relation);$wanted=@($Expected | Sort-Object -Unique)
    Assert-B (($actual -join ',') -ceq ($wanted -join ',')) "$Message Observed=[$($actual-join ',')] Expected=[$($wanted-join ',')]"
}
function Find-Entities($Session,[string]$Name,[bool]$DeclarationsOnly,[string]$Evidence,[string]$Domain='',[string]$EntityType='') {
    $args=@{workspaceRoot=$root;type='find_entities';name=$Name}
    if ($DeclarationsOnly) {$args.declarations_only=$true}
    if ($Domain) {$args.domain=$Domain}
    if ($EntityType) {$args.entity_type=$EntityType}
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' $args)
    Save-B $Evidence $answer
    return @($answer.structuredContent.entities)
}

$files=[ordered]@{
'shared.meta.ts'=Pad @'
import { Directive, Pipe } from '@angular/core';
@Directive({ selector: '[shared]' }) export class SharedDirective {}
@Pipe({ name: 'shared' }) export class SharedPipe {}
export const SHARED = [SharedDirective, SharedPipe];
'@
'feature.component.ts'=Pad @'
import { Component } from '@angular/core';
@Component({ standalone: true, selector: 'feature-probe', template: '' })
export class FeatureComponent {}
'@
'app.module.ts'=Pad @'
import { Component, NgModule } from '@angular/core';
import { RouterModule } from '@angular/router';
import { SHARED } from './shared.meta';
import { FeatureComponent } from './feature.component';
@Component({ selector: 'local-probe', template: '' }) class LocalComponent {}
const LOCAL = [LocalComponent];
@NgModule({
  declarations: [...LOCAL, ...SHARED, ...missingDeclarations],
  imports: [FeatureComponent, RouterModule.forChild([])],
  exports: [LocalComponent]
})
export class AppModule {}
'@
'lazy.component.ts'=Pad @'
import { Component } from '@angular/core';
@Component({ selector: 'lazy-probe', template: '' }) export class LazyComponent {}
'@
'admin.module.ts'=Pad @'
import { NgModule } from '@angular/core';
@NgModule({}) export class AdminModule {}
'@
'child.routes.ts'=Pad @'
import { Routes } from '@angular/router';
export const childRoutes: Routes = [];
'@
'app.routes.ts'=Pad @'
import { Routes } from '@angular/router';
export const routes: Routes = [
  { path: 'admin', component: AdminComponent, canActivate: [AdminGuard], children: [
      { path: 'users', component: UsersComponent, resolve: { users: UsersResolver } },
      { path: '', component: AdminHomeComponent },
      { path: 'settings', component: SettingsComponent }
  ]},
  { path: 'lazy', loadComponent: () => import('./lazy.component').then(m => m.LazyComponent) },
  { path: 'admin-lazy', loadChildren: () => import('./admin.module').then(m => m.AdminModule) },
  { path: 'child', loadChildren: () => import('./child.routes').then(m => m.childRoutes) },
  { path: 'dynamic', loadChildren: chooseLoader() }
];
'@
'audit.service.ts'=Pad @'
import { Injectable } from '@angular/core';
@Injectable() export class AuditService {}
'@
'consumer-a.ts'=Pad @'
import { Component } from '@angular/core';
import { AuditService } from './audit.service';
@Component({ selector: 'consumer-a', template: '' })
export class ConsumerA { constructor(private audit: AuditService) {} }
'@
'consumer-b.ts'=Pad @'
import { Directive } from '@angular/core';
import { AuditService } from './audit.service';
@Directive({ selector: '[consumerB]' })
export class ConsumerB { constructor(private audit: AuditService) {} }
'@
'real-class.ts'=Pad @'
export class Class { value = 1; }
'@
}
foreach($entry in $files.GetEnumerator()) {Write-BText (Join-Path $root $entry.Key) $entry.Value}
Save-B 'source-oracle' $files
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;fixture_root=$root;contract='Issue 139 public acceptance'}

try {
    $session=Start-BSession 'issue-139' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    foreach($tool in @('provide_code_context','workspace_query','refresh_workspace')) {Assert-B ($tool -in $registry.tools.name) "Registered tool absent: $tool"}
    $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
    Publish $session @($files.Keys)

    Invoke-BCase '139' 'ngmodule-bounded-composition-and-independent-kinds' {
        $edges=Query-Edges $session 'Module' 'AppModule' 'module-forward'
        Assert-Targets $edges 'DeclaresInModule' @('Component/LocalComponent','Directive/SharedDirective','Pipe/SharedPipe') 'NgModule declaration targets differ.'
        Assert-Targets $edges 'ImportsModule' @('Component/FeatureComponent','Module/RouterModule') 'NgModule import targets differ.'
        Assert-Targets $edges 'ExportsFromModule' @('Component/LocalComponent') 'NgModule export targets differ.'
        Assert-B (@($edges | Where-Object {$_.object.name -match 'missing|^\.\.\.'}).Count -eq 0) 'Unresolved spread became an entity endpoint.'
    }
    Invoke-BCase '139' 'unresolved-spread-is-public-non-entity-evidence' {
        $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='app.module.ts';fidelity='high'})
        Save-B 'unresolved-spread-context' $context
        Assert-B ($context.content[0].text.Contains('T @modspread = missingDeclarations')) 'Unresolved spread evidence is absent from public context.'
        Assert-B (@(Find-Entities $session 'missingDeclarations' $false 'unresolved-spread-entities').Count -eq 0) 'Unresolved spread was published as an entity occurrence.'
    }
    Invoke-BCase '139' 'structured-route-tree-exact-node-ownership' {
        $admin=Query-Edges $session 'Route' 'admin' 'route-admin'
        Assert-B (@($admin | Where-Object {$_.relation -eq 'RouteMapsTo' -and $_.object.name -eq 'AdminComponent'}).Count -eq 1) 'Parent route component is absent.'
        Assert-B (@($admin | Where-Object {$_.relation -eq 'GuardedBy' -and $_.object.name -eq 'AdminGuard'}).Count -eq 1) 'Parent route guard is absent.'
        Assert-B (@($admin | Where-Object {$_.object.name -in @('UsersComponent','UsersResolver','SettingsComponent')}).Count -eq 0) 'Child evidence leaked onto the parent route.'
        Assert-Targets (Query-Edges $session 'Route' 'admin/users' 'route-users') 'RouteMapsTo' @('Component/UsersComponent') 'Nested route component differs.'
        Assert-Targets (Query-Edges $session 'Route' 'admin/users' 'route-users-resolver') 'ResolvedBy' @('Resolver/UsersResolver') 'Nested route resolver differs.'
        Assert-Targets (Query-Edges $session 'Route' 'admin/settings' 'route-settings') 'RouteMapsTo' @('Component/SettingsComponent') 'Sibling route component differs.'
        $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='app.routes.ts';fidelity='high'})
        Save-B 'route-ownership-context' $context
        $adminLines=@($context.content[0].text -split '\r?\n' | Where-Object {$_ -match '^T @route = admin(?: |$)'})
        Assert-B ($adminLines.Count -eq 2) 'Parent and empty-child route nodes were not rendered independently.'
        Assert-B (@($adminLines | Where-Object {$_ -match 'component=AdminComponent' -and $_.Contains('guards=[AdminGuard]')}).Count -eq 1) 'Parent route evidence is not owned by one exact node.'
        Assert-B (@($adminLines | Where-Object {$_ -match 'component=AdminHomeComponent' -and $_ -notmatch 'guards='}).Count -eq 1) 'Empty-child evidence is not owned by one exact node.'
    }
    Invoke-BCase '139' 'lazy-routes-use-separated-exact-relations' {
        $lazy=Query-Edges $session 'Route' 'lazy' 'route-lazy-component'
        Assert-Targets $lazy 'RouteMapsTo' @('Component/LazyComponent') 'Lazy Component target differs.'
        Assert-B (@($lazy | Where-Object relation -eq 'LoadsChildren').Count -eq 0) 'Lazy Component used LoadsChildren.'
        $module=Query-Edges $session 'Route' 'admin-lazy' 'route-lazy-module'
        Assert-Targets $module 'LoadsChildren' @('Module/AdminModule') 'Lazy Module target differs.'
        Assert-B (@($module | Where-Object relation -eq 'RouteMapsTo').Count -eq 0) 'Lazy Module made RouteMapsTo heterogeneous.'
        Assert-Targets (Query-Edges $session 'Route' 'child' 'route-lazy-config') 'LoadsChildren' @('RouteConfig/childRoutes') 'Lazy route-config target differs.'
        Assert-B (@(Query-Edges $session 'Route' 'dynamic' 'route-dynamic-negative').Count -eq 0) 'Dynamic lazy target did not fail closed.'
    }
    Invoke-BCase '139' 'declaration-only-query-and-lgr003' {
        Assert-B (@(Find-Entities $session 'AuditService' $false 'service-occurrences').Count -ge 3) 'Default occurrence query lost declaration/reference occurrences.'
        $declarations=@(Find-Entities $session 'AuditService' $true 'service-declarations' 'angular' 'Service')
        Assert-B ($declarations.Count -eq 1) 'Declaration-only Service query did not return exactly one declaration.'
        Assert-B ($declarations[0].file -eq (Join-Path $root 'audit.service.ts')) 'Declaration-only Service provenance is wrong.'
        Assert-B (@($declarations | Where-Object {$_.domain -eq 'builtin' -or $_.entity_type -eq 'Class'}).Count -eq 0) 'Builtin Class occurrence leaked into filtered declaration results.'
        $real=@(Find-Entities $session 'Class' $true 'real-class-declaration' 'builtin' 'Class')
        Assert-B ($real.Count -eq 1) 'Genuine declaration named Class was hidden.'
        Assert-B ($real[0].file -eq (Join-Path $root 'real-class.ts')) 'Genuine Class declaration provenance is wrong.'
    }
    Invoke-BCase '139' 'refresh-removes-authority-and-retains-occurrence-contract' {
        Write-BText (Join-Path $root 'audit.service.ts') (Pad 'export const noService = 1;')
        Remove-Item -LiteralPath (Join-Path $root 'lazy.component.ts')
        $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        Save-B 'lifecycle-refresh' $refresh
        Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Registered refresh failed.'
        Publish $session @('consumer-a.ts','consumer-b.ts','app.routes.ts')
        Assert-B (@(Find-Entities $session 'AuditService' $true 'lifecycle-service-declarations' 'angular' 'Service').Count -eq 0) 'Removed Service declaration survived refresh.'
        Assert-B (@(Find-Entities $session 'AuditService' $false 'lifecycle-service-occurrences').Count -ge 2) 'Occurrence-oriented references were lost with declaration removal.'
        Assert-B (@(Query-Edges $session 'Route' 'lazy' 'lifecycle-lazy-route' | Where-Object relation -eq 'RouteMapsTo').Count -eq 0) 'Removed lazy target authority survived refresh.'
    }
} finally {
    foreach($session in $sessions) {try {Stop-BSession $session} catch {$verdicts.Add(@{phase='cleanup';case=$session.Name;result='FAIL';evidence=$_.Exception.Message})}}
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #139 live evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
