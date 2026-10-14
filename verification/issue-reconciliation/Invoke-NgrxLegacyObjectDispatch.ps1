param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-legacy-object-dispatch/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
New-Item -ItemType Directory -Force $runRoot | Out-Null

function Action-Source([string]$Name='SET_AUDIT',[string]$Reducer='auditReducer') {
@"
import { Action } from '@ngrx/store';
export const $Name = '[Audit] Set';
export function $Reducer(state:any, action:Action) { switch(action.type) { case ${Name}: return state; default: return state; } }
"@
}
function Dispatch-Source([string]$Name='SET_AUDIT',[string]$Module='./legacy.actions',[string]$Extra='') {
@"
import {Component} from '@angular/core'; import {Store} from '@ngrx/store'; import {$Name} from '$Module';
@Component({selector:'audit-dispatch',template:''}) export class DispatchProbe {
 constructor(private store:Store<unknown>) {} send() {$Extra this.store.dispatch({type:$Name,payload:1});}
}
"@
}
function Publish([object]$Session,[string]$Root,[string]$File) {$null=Get-BResult (Invoke-BTool $Session 'provide_code_context' @{workspaceRoot=$Root;filePath=$File;fidelity='high'})}
function Targets([object]$Session,[string]$Root) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$Root;type='forward_edges';domain='angular';entity_type='Component';name='DispatchProbe'})
    return @($answer.structuredContent.edges | Where-Object relation -eq 'Dispatches' | ForEach-Object {$_.object.name} | Sort-Object)
}
function Refresh([object]$Session,[string]$Root) {$null=Get-BResult (Invoke-BTool $Session 'refresh_workspace' @{workspaceRoot=$Root})}
function Assert-Targets([object]$Session,[string]$Root,[string[]]$Expected,[string]$Label) {
    $actual=@(Targets $Session $Root)
    Assert-B (($actual -join '|') -eq (@($Expected | Sort-Object) -join '|')) "$Label expected $($Expected -join ',') observed $($actual -join ',')"
}

Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;cbm_enabled=$false;scope='direct-relative unaliased named-value object dispatch refinement'}
try {
    $root=Join-Path $runRoot 'workspace';New-Item -ItemType Directory -Force $root | Out-Null
    Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    $actions=Join-Path $root 'legacy.actions.ts';$dispatch=Join-Path $root 'dispatch.component.ts';$fake=Join-Path $root 'fake.actions.ts'
    Write-BText $actions (Action-Source);Write-BText $fake "export const SET_AUDIT='[Audit] Set';";Write-BText $dispatch (Dispatch-Source)
    $session=Start-BSession 'slice2' $BinaryPath $root
    Invoke-BCase '140-slice2' 'primary-two-file' {Publish $session $root $actions;Publish $session $root $dispatch;Assert-Targets $session $root @('SET_AUDIT') 'primary'}
    Invoke-BCase '140-slice2' 'ordinary-target-retarget-restore' {
        Write-BText $dispatch (Dispatch-Source 'SET_AUDIT' './fake.actions');Refresh $session $root;Publish $session $root $actions;Publish $session $root $dispatch;Assert-Targets $session $root @() 'ordinary target'
        Write-BText $dispatch (Dispatch-Source);Refresh $session $root;Publish $session $root $dispatch;Publish $session $root $actions;Assert-Targets $session $root @('SET_AUDIT') 'restored import'
    }
    foreach($case in @(
        @{id='shadow';source=(Dispatch-Source 'SET_AUDIT' './legacy.actions' "const SET_AUDIT='local';")},
        @{id='missing-module';source=(Dispatch-Source 'SET_AUDIT' './missing')},
        @{id='path-alias';source=(Dispatch-Source 'SET_AUDIT' '@app/actions')},
        @{id='alias';source=((Dispatch-Source 'SET' './legacy.actions') -replace 'import \{SET\}', 'import {SET_AUDIT as SET}')},
        @{id='namespace';source=((Dispatch-Source 'Actions.SET_AUDIT' './legacy.actions') -replace 'import \{Actions.SET_AUDIT\}', 'import * as Actions')}
    )) {
        Invoke-BCase '140-slice2' $case.id {Write-BText $dispatch $case.source;Refresh $session $root;Publish $session $root $actions;Publish $session $root $dispatch;Assert-Targets $session $root @() $case.id}
    }
    Invoke-BCase '140-slice2' 'authority-remove-restore' {
        Write-BText $dispatch (Dispatch-Source);Write-BText $actions ((Action-Source) -replace 'case SET_AUDIT: return state;', '');Refresh $session $root;Publish $session $root $dispatch;Publish $session $root $actions;Assert-Targets $session $root @() 'authority removed'
        Write-BText $actions (Action-Source);Refresh $session $root;Publish $session $root $actions;Publish $session $root $dispatch;Assert-Targets $session $root @('SET_AUDIT') 'authority restored'
    }
    Invoke-BCase '140-slice2' 'multiple-actions' {
        $theme=Join-Path $root 'theme.actions.ts';Write-BText $theme (Action-Source 'SET_THEME' 'themeReducer')
        $multi=(Dispatch-Source) -replace '@Component', "import {SET_THEME} from './theme.actions'; @Component" -replace 'this.store.dispatch\(\{type:SET_AUDIT,payload:1\}\);', 'this.store.dispatch({type:SET_AUDIT,payload:1}); this.store.dispatch({type:SET_THEME});'
        Write-BText $dispatch $multi;Refresh $session $root;Publish $session $root $dispatch;Publish $session $root $theme;Publish $session $root $actions;Assert-Targets $session $root @('SET_AUDIT','SET_THEME') 'multiple actions'
    }
    Invoke-BCase '140-slice2' 'file-order-convergence' {
        Write-BText $dispatch (Dispatch-Source);Refresh $session $root;Publish $session $root $dispatch;Publish $session $root $actions;Assert-Targets $session $root @('SET_AUDIT') 'dispatch first'
        Refresh $session $root;Publish $session $root $actions;Publish $session $root $dispatch;Assert-Targets $session $root @('SET_AUDIT') 'actions first'
    }
    Invoke-BCase '140-slice2' 'delete-destination-and-dispatch' {
        Remove-Item -LiteralPath $actions;Refresh $session $root;Publish $session $root $dispatch;Assert-Targets $session $root @() 'destination deleted'
        Write-BText $actions (Action-Source);Remove-Item -LiteralPath $dispatch;Refresh $session $root;Publish $session $root $actions;Assert-Targets $session $root @() 'dispatch deleted'
    }
} finally {
    foreach($session in $sessions){try{Stop-BSession $session}catch{$verdicts.Add(@{phase='cleanup';case=$session.Name;result='FAIL';evidence=$_.Exception.Message})}}
    Save-B 'verdicts' @($verdicts.ToArray());Write-Host "NgRx Slice 2 evidence: $runRoot"
}
if(@($verdicts | Where-Object result -eq 'FAIL').Count){exit 1}
