param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
# Consolidated Issue #138 live MCP acceptance. Builds nothing and starts no CBM provider.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/issue-138-live-verification/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'workspace'
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
New-Item -ItemType Directory -Force $root | Out-Null
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
$service=@"
import { Injectable } from '@angular/core';
@Injectable() export class AuditService { allowed() { return true; } }
@Injectable() export class AlternateService {}
"@
$source=@"
import { Component, EventEmitter, Input, NgModule, Output, inject, input, model, output } from '@angular/core';
import { CanActivateFn } from '@angular/router';
import { AuditService, AlternateService } from './service';
export const API_TOKEN = 'API_TOKEN';
export const FACTORY_TOKEN = 'FACTORY_TOKEN';
export const VALUE_TOKEN = 'VALUE_TOKEN';
export function createAuditService() { return new AuditService(); }
@Component({ selector: 'signal-probe', template: '' })
export class SignalProbe {
  title = input<string>();
  requiredTitle = input.required<string>();
  changed = output<string>();
  selected = model(false);
  receiverNegative = api.input();
  nestedNegative = wrap(input());
}
@Component({ selector: 'classic-probe', template: '' })
export class ClassicProbe {
  @Input('publicValue') set value(next: string) {}
  @Input({ required: true }) item!: string;
  @Output() changedClassic!: EventEmitter<void>;
}
export const auditGuard: CanActivateFn = () => inject(AuditService).allowed();
export const helper = () => inject(AlternateService);
@NgModule({ providers: [
  AuditService,
  { provide: API_TOKEN, useClass: AlternateService },
  { provide: FACTORY_TOKEN, useFactory: createAuditService },
  { provide: VALUE_TOKEN, useValue: 7 },
  { provide: 'UNRESOLVED', useClass: MissingService }
] })
export class AuditModule {}
"@
Write-BText (Join-Path $root 'service.ts') $service
Write-BText (Join-Path $root 'issue138.ts') $source
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;fixture_root=$root}
Save-B 'source-oracle' @{service=$service;issue138=$source}
function Query-Edges([object]$Session,[string]$Type,[string]$EntityType,[string]$Name,[string]$Evidence) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type=$Type;domain='angular';entity_type=$EntityType;name=$Name})
    Save-B $Evidence $answer
    Assert-B ([bool]$answer.PSObject.Properties['structuredContent']) 'workspace_query omitted structuredContent.'
    return @($answer.structuredContent.edges)
}
function Edge-Names([string]$Relation,[object[]]$Edges) {return @($Edges | Where-Object {$_.relation -eq $Relation} | ForEach-Object {$_.object.name} | Sort-Object -Unique)}
function Assert-Names([object[]]$Actual,[string[]]$Expected,[string]$Message) {
    Assert-B ((@($Actual) -join ',') -ceq (@($Expected | Sort-Object -Unique) -join ',')) "$Message Observed=[$(@($Actual)-join ',')] Expected=[$(@($Expected)-join ',')]"
}
try {
    $session=Start-BSession 'issue-138' $BinaryPath $root
    $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
    foreach($file in @('service.ts','issue138.ts')) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='high'})}
    Invoke-BCase '138' 'structured-signals-and-negative-twins' {
        $edges=Query-Edges $session 'forward_edges' 'Component' 'SignalProbe' 'signals-forward'
        Assert-Names (Edge-Names 'HasInput' $edges) @('requiredTitle','selected','title') 'Signal HasInput identities differ.'
        Assert-Names (Edge-Names 'HasOutput' $edges) @('changed','selected') 'Signal HasOutput identities differ.'
        Assert-B (@($edges | Where-Object {$_.object.name -in @('receiverNegative','nestedNegative','?')}).Count -eq 0) 'Receiver/nested/junk signal identity leaked.'
    }
    Invoke-BCase '138' 'classic-setter-definite-assignment-and-alias-identity' {
        $edges=Query-Edges $session 'forward_edges' 'Component' 'ClassicProbe' 'classic-forward'
        Assert-Names (Edge-Names 'HasInput' $edges) @('item','value') 'Classic HasInput identities differ.'
        Assert-Names (Edge-Names 'HasOutput' $edges) @('changedClassic') 'Classic HasOutput identities differ.'
        Assert-B (@($edges | Where-Object {$_.object.name -in @('set','item!','publicValue','?')}).Count -eq 0) 'Accessor punctuation or alias replaced canonical identity.'
    }
    Invoke-BCase '138' 'functional-guard-owned-inject' {
        $guard=Query-Edges $session 'forward_edges' 'Guard' 'auditGuard' 'guard-forward'
        Assert-Names (Edge-Names 'Injects' $guard) @('AuditService') 'Functional guard Injects target differs.'
        $helper=Query-Edges $session 'forward_edges' 'Guard' 'helper' 'helper-negative'
        Assert-B (@($helper | Where-Object {$_.relation -eq 'Injects'}).Count -eq 0) 'Untyped helper acquired Guard injection semantics.'
    }
    Invoke-BCase '138' 'provider-scope-token-and-strategy-separation' {
        $module=Query-Edges $session 'forward_edges' 'Module' 'AuditModule' 'module-providers'
        Assert-Names (Edge-Names 'RegistersProvider' $module) @('API_TOKEN','AuditService','FACTORY_TOKEN','VALUE_TOKEN') 'Registered provider tokens differ.'
        Assert-B (@($module | Where-Object {$_.relation -eq 'Injects'}).Count -eq 0) 'Provider registration was mislabeled as dependency consumption.'
        $classProvider=Query-Edges $session 'forward_edges' 'ProviderToken' 'API_TOKEN' 'provider-class'
        Assert-Names (Edge-Names 'ProviderUsesClass' $classProvider) @('AlternateService') 'useClass endpoint differs.'
        $factoryProvider=Query-Edges $session 'forward_edges' 'ProviderToken' 'FACTORY_TOKEN' 'provider-factory'
        Assert-Names (Edge-Names 'ProviderUsesFactory' $factoryProvider) @('createAuditService') 'useFactory endpoint differs.'
        $valueProvider=Query-Edges $session 'forward_edges' 'ProviderToken' 'VALUE_TOKEN' 'provider-value'
        Assert-Names (Edge-Names 'ProviderUsesValue' $valueProvider) @('7') 'useValue endpoint differs.'
        Assert-B (@($module | Where-Object {$_.object.name -eq 'UNRESOLVED'}).Count -eq 0) 'Unresolved useClass manufactured provider identity.'
    }
    Invoke-BCase '138' 'refresh-removes-replaced-semantics' {
        $replacement=@"
import { Component, NgModule, input } from '@angular/core';
@Component({ selector: 'signal-probe', template: '' })
export class SignalProbe { replacement = input<string>(); }
@NgModule({ providers: [] }) export class AuditModule {}
"@
        Write-BText (Join-Path $root 'issue138.ts') $replacement
        $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Registered refresh failed.'
        $signals=Query-Edges $session 'forward_edges' 'Component' 'SignalProbe' 'lifecycle-signals'
        Assert-Names (Edge-Names 'HasInput' $signals) @('replacement') 'Refresh retained stale signal inputs.'
        Assert-Names (Edge-Names 'HasOutput' $signals) @() 'Refresh retained stale signal outputs.'
        $module=Query-Edges $session 'forward_edges' 'Module' 'AuditModule' 'lifecycle-providers'
        Assert-B (@($module).Count -eq 0) 'Refresh retained stale provider edges.'
    }
} finally {
    foreach($session in $sessions) {try {Stop-BSession $session} catch {$verdicts.Add(@{phase='cleanup';case=$session.Name;result='FAIL';evidence=$_.Exception.Message})}}
    Save-B 'verdicts' @($verdicts.ToArray());Write-Host "Issue #138 live evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}