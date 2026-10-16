param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-legacy-declarations/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
function Legacy-Source([string]$Code) {return $Code+"`n"+('// unchanged economics padding'+"`n")*100}
function Legacy-Entities($Session,[string]$File,[string]$Label) {
    $result=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B "$Label-entities" $result
    return ,@($result.structuredContent.entities | Where-Object {$_.domain -eq 'ngrx' -and $_.entity_type -eq 'Action'})
}
function Assert-LegacyEntities($Found,[string[]]$Expected,[string]$File) {
    $Found=@($Found | Where-Object {$null -ne $_});$Expected=@($Expected | Where-Object {$null -ne $_})
    Assert-B ($Found.Count -eq $Expected.Count) "Exact Action occurrence count $($Found.Count)/$($Expected.Count)."
    foreach ($target in $Expected) {Assert-B (@($Found | Where-Object {$_.name -ceq $target -and $_.file -eq (Join-Path $root $File)}).Count -eq 1) "Exact declaration source occurrence absent/duplicated: $target"}
}
$cases=@(
    @{id='primary';symbol='NoSuffixRecord';expected=@('NoSuffixRecord');source="import { Action } from '@ngrx/store';`nexport class NoSuffixRecord implements Action { readonly type = '[Audit] Set Audit'; constructor(public payload: string) {} }"},
    @{id='non-export';symbol='PlainRecord';expected=@('PlainRecord');source="import { Action } from '@ngrx/store';`nclass PlainRecord implements Action { readonly type = '[Audit] Plain'; }"},
    @{id='abstract';symbol='BaseRecord';expected=@('BaseRecord');source="import { Action } from '@ngrx/store';`nexport abstract class BaseRecord implements Action { abstract type: string; }"},
    @{id='multiple-interfaces';symbol='MultiRecord';expected=@('MultiRecord');source="import { Action } from '@ngrx/store';`nexport class MultiRecord implements OtherInterface, Action { readonly type = '[Audit] Multi'; }"},
    @{id='extends';symbol='DerivedRecord';expected=@('DerivedRecord');source="import { Action } from '@ngrx/store';`nexport class DerivedRecord extends Base implements Action { readonly type = '[Audit] Derived'; }"},
    @{id='multiline';symbol='LayoutRecord';expected=@('LayoutRecord');source="import { Action } from '@ngrx/store';`nexport class LayoutRecord`n implements`n Action { readonly type = '[Audit] Layout'; }"},
    @{id='double-quotes';symbol='QuotedRecord';expected=@('QuotedRecord');source='import { Action } from "@ngrx/store"; export class QuotedRecord implements Action { readonly type = "[Audit] Quoted"; }'},
    @{id='type-import';symbol='TypedRecord';expected=@('TypedRecord');source="import type { Action } from '@ngrx/store';`nexport class TypedRecord implements Action { readonly type = '[Audit] Typed'; }"},
    @{id='ordinary-action-name';symbol='PretendAction';expected=@();source="import { Action } from '@ngrx/store';`nexport class PretendAction { readonly type = '[Audit] Pretend'; }"},
    @{id='local-interface';symbol='LocalRecord';expected=@();source="import { Store } from '@ngrx/store';`ninterface Action { type: string; } export class LocalRecord implements Action { readonly type = '[Audit] Local'; }"},
    @{id='foreign-action-import';symbol='ForeignRecord';expected=@();source="import { Store } from '@ngrx/store'; import { Action } from './other';`nexport class ForeignRecord implements Action { readonly type = '[Audit] Foreign'; }"},
    @{id='aliased-interface';symbol='AliasedRecord';expected=@();source="import { Action as StoreAction } from '@ngrx/store';`nexport class AliasedRecord implements StoreAction { readonly type = '[Audit] Alias'; }"},
    @{id='qualified-interface';symbol='QualifiedRecord';expected=@();source="import * as Store from '@ngrx/store';`nexport class QualifiedRecord implements Store.Action { readonly type = '[Audit] Qualified'; }"},
    @{id='string-constant';symbol='SET_AUDIT';expected=@();source="import { Store } from '@ngrx/store';`nexport const SET_AUDIT = '[Audit] Set Audit';"},
    @{id='generic-shadow';symbol='ShadowRecord';expected=@();source="import { Action } from '@ngrx/store';`nexport class ShadowRecord<Action> implements Action { readonly type = '[Audit] Shadow'; }"},
    @{id='nested-class';symbol='NestedRecord';expected=@();source="import { Action } from '@ngrx/store';`nfunction scope() { class NestedRecord implements Action { readonly type = '[Audit] Nested'; } }"},
    @{id='text-only-class';symbol='TextOnlyRecord';expected=@();source="import { Action } from '@ngrx/store';`nexport const text = 'class TextOnlyRecord implements Action {}';"},
    @{id='modern-guard';symbol='modernAudit';expected=@('modernAudit');modern=$true;source="import { createAction } from '@ngrx/store';`nexport const modernAudit = createAction('[Audit] Modern');"}
)
foreach ($case in $cases) {$case.file=$case.id+'.ts';$case.source=Legacy-Source $case.source;Write-BText (Join-Path $root $case.file) $case.source}
$observations=@(
    @{id='legacy-switch';source="import { Action } from '@ngrx/store'; import { SET_AUDIT } from './string-constant'; export function legacyReducer(state = {}, action: Action) { switch (action.type) { case SET_AUDIT: return {...state}; default: return state; } }";domain='ngrx';kind='Reducer';symbol='legacyReducer'},
    @{id='modern-direct-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; import { modernAudit } from './modern-guard'; @Component({selector:'x-direct'}) export class DirectConsumer { constructor(private store: Store) {} run() { this.store.dispatch(modernAudit()); } }";domain='angular';kind='Component';symbol='DirectConsumer'},
    @{id='qualified-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; import * as Actions from './modern-guard'; @Component({selector:'x-qualified'}) export class QualifiedConsumer { constructor(private store: Store) {} run() { this.store.dispatch(Actions.modernAudit()); } }";domain='angular';kind='Component';symbol='QualifiedConsumer'},
    @{id='alias-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; import { modernAudit as triggerAudit } from './modern-guard'; @Component({selector:'x-alias'}) export class AliasConsumer { constructor(private store: Store) {} run() { this.store.dispatch(triggerAudit()); } }";domain='angular';kind='Component';symbol='AliasConsumer'},
    @{id='class-construction-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; import { NoSuffixRecord } from './primary'; @Component({selector:'x-class'}) export class ClassConsumer { constructor(private store: Store) {} run() { this.store.dispatch(new NoSuffixRecord('payload')); } }";domain='angular';kind='Component';symbol='ClassConsumer'},
    @{id='undeclared-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; @Component({selector:'x-missing'}) export class MissingConsumer { constructor(private store: Store) {} run() { this.store.dispatch(unrecognizedAction()); } }";domain='angular';kind='Component';symbol='MissingConsumer'},
    @{id='collision-use';source="import { Component } from '@angular/core'; import { Store } from '@ngrx/store'; import { sameAudit } from './modern-a'; @Component({selector:'x-collision'}) export class CollisionConsumer { constructor(private store: Store) {} run() { this.store.dispatch(sameAudit()); } }";domain='angular';kind='Component';symbol='CollisionConsumer'}
)
foreach ($case in $observations) {Write-BText (Join-Path $root ($case.id+'.ts')) (Legacy-Source $case.source)}
foreach ($file in @('modern-a.ts','modern-b.ts')) {Write-BText (Join-Path $root $file) (Legacy-Source "import { createAction } from '@ngrx/store'; export const sameAudit = createAction('[Audit] Same');")}
foreach ($file in @('legacy-a.ts','legacy-b.ts')) {Write-BText (Join-Path $root $file) (Legacy-Source "import { Action } from '@ngrx/store'; export class SharedRecord implements Action {readonly type = '[Audit] Shared';}")}
Write-BText (Join-Path $root 'lifecycle.ts') (Legacy-Source "import { Action } from '@ngrx/store'; export class LifecycleRecord implements Action {readonly type = '[Audit] Lifecycle';}")
Save-B 'source-oracles' $cases;Save-B 'partition-oracles' $observations
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;scope='class declaration acceptance only; reference and switch probes are observations of separate defects'}
try {
    $session=Start-BSession 'ngrx-legacy' $BinaryPath $root
    Save-B 'registered-tools' (Get-BResult (Invoke-BRpc $session 'tools/list' @{}))
    foreach ($case in $cases) {
        Invoke-BCase '140-declarations' "$($case.id)-exact-source-identity" {
            $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            Save-B "$($case.id)-context" $context
            Assert-LegacyEntities (Legacy-Entities $session $case.file $case.id) $case.expected $case.file
        }
        Invoke-BCase '140-declarations' "$($case.id)-truthful-marker" {
            $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            if ($case.expected.Count) {
                $prefix=if ($case.ContainsKey('modern')) {'T @action = '} else {'T @action-class = '}
                Assert-B ($context.content[0].text.Contains($prefix+$case.symbol)) 'Recognized Action kind/name missing from marker surface.'
            } else {Assert-B (-not $context.content[0].text.Contains('T @action-class')) 'Unproven class gained Action marker.'}
        }
        Invoke-BCase '140-declarations' "$($case.id)-no-fabricated-links" {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='ngrx';entity_type='Action';name=$case.symbol})
            Save-B "$($case.id)-forward" $answer
            Assert-B (@($answer.structuredContent.edges).Count -eq 0) 'Declaration generated a handler/use relationship.'
        }
        Invoke-BCase '140-declarations' "$($case.id)-repeat-fidelity" {
            foreach ($fidelity in @('low','medium','high')) {
                $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity=$fidelity})
                Save-B "$($case.id)-$fidelity-context" $context
                Assert-LegacyEntities (Legacy-Entities $session $case.file ($case.id+'-'+$fidelity)) $case.expected $case.file
            }
            Assert-B ([IO.File]::ReadAllText((Join-Path $root $case.file)) -ceq $case.source) 'Source changed.'
        }
    }
    foreach ($file in @('legacy-a.ts','legacy-b.ts')) {
        Invoke-BCase '140-declarations' "source-distinct-$file" {
            Assert-LegacyEntities (Legacy-Entities $session $file $file) @('SharedRecord') $file
        }
    }
    foreach ($stage in @('LifecycleRecord','RenamedRecord','remove-declaration','delete-file')) {
        Invoke-BCase '140-declarations' "lifecycle-$stage" {
            if ($stage -eq 'delete-file') {Remove-Item -LiteralPath (Join-Path $root 'lifecycle.ts')}
            else {
                $code=if ($stage -eq 'remove-declaration') {"import { Action } from '@ngrx/store'; export const ordinary = '[Audit] Ordinary';"} else {"import { Action } from '@ngrx/store'; export class $stage implements Action {readonly type = '[Audit] Lifecycle';}"}
                Write-BText (Join-Path $root 'lifecycle.ts') (Legacy-Source $code)
                Save-B "lifecycle-$stage-source" @{source=$code}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            $expected=if ($stage -in @('remove-declaration','delete-file')) {@()} else {@($stage)}
            Assert-LegacyEntities (Legacy-Entities $session 'lifecycle.ts' "lifecycle-$stage") $expected 'lifecycle.ts'
        }
    }
    # Evidence only: these independent linkage/handler families are NOT accepted
    # by declaration extraction and must not be hidden behind a PASS verdict.
    foreach ($observation in $observations) {
        $script:deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
        $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=($observation.id+'.ts');fidelity='high'})
        Save-B ($observation.id+'-observation-context') $context
        $entities=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=($observation.id+'.ts')})
        $edges=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain=$observation.domain;entity_type=$observation.kind;name=$observation.symbol})
        Save-B ($observation.id+'-observation') @{entities=$entities;edges=$edges;acceptance='NOT INCLUDED; written endpoint is not resolved declaration identity'}
        $verdicts.Add(@{phase='140-partition';case=$observation.id;result='OBSERVED'})
    }
    foreach ($file in @('modern-a.ts','modern-b.ts')) {
        $script:deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
        $null=Legacy-Entities $session $file $file
    }
    Save-B 'collision-identity-limitation' @{meaning='sameAudit declarations have source-distinct occurrences; existing graph identity is domain/type/name. No proof selecting imported target file, and no resolver repair.'}
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "NgRx legacy evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
