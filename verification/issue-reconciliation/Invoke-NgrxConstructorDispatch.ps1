param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-constructor-dispatch/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
$base="import {Component} from '@angular/core'; import {Action, Store} from '@ngrx/store'; export class AuditAction implements Action {readonly type='audit';} export class OtherAction implements Action {readonly type='audit';} class NonAction {} @Component({selector:'audit-dispatch', template:''}) export class Dispatcher {constructor(private store: Store<unknown>) {} run() {BODY}}"
function Dispatch-Source($Case) {
    $source=$base.Replace('BODY',$Case.body)
    if ($Case.ContainsKey('from')) {$source=$source.Replace($Case.from,$Case.to)}
    return $source+"`n"+('// unchanged economics padding'+"`n")*100
}
$cases=@(
    @{id='primary';body='this.store.dispatch(new AuditAction());';expected=@('AuditAction')},
    @{id='neighbor';body='this.store.dispatch(new OtherAction());';expected=@('OtherAction')},
    @{id='payload';body='this.store.dispatch(new AuditAction(payload()));';expected=@('AuditAction')},
    @{id='receiver-name';body='this.events.dispatch(new AuditAction());';from='private store';to='private events';expected=@('AuditAction')},
    @{id='non-action';body='this.store.dispatch(new NonAction());';expected=@()},
    @{id='qualified';body='this.store.dispatch(new Actions.AuditAction());';expected=@()},
    @{id='logger';body='this.logger.dispatch(new AuditAction());';expected=@()},
    @{id='wrong-store-type';body='this.store.dispatch(new AuditAction());';from='Store<unknown>';to='Logger';expected=@()},
    @{id='foreign-store';body='this.store.dispatch(new AuditAction());';from="import {Action, Store} from '@ngrx/store';";to="import {Action} from '@ngrx/store'; import {Store} from './fake';";expected=@()},
    @{id='static-owner';body='this.store.dispatch(new AuditAction());';from='run()';to='static run()';expected=@()},
    @{id='disabled';body='this.store.dispatch(new AuditAction());';expected=@();actions=@();disabled=$true},
    @{id='parameter-shadow';body='this.store.dispatch(new AuditAction());';from='run()';to='run(AuditAction: any)';expected=@()},
    @{id='local-shadow';body='const AuditAction=OtherAction; this.store.dispatch(new AuditAction());';expected=@()},
    @{id='loop-shadow';body='for(var AuditAction of values) {} this.store.dispatch(new AuditAction());';expected=@()},
    @{id='optional-call';body='this.store.dispatch?.(new AuditAction());';expected=@()},
    @{id='wrapper';body='this.store.dispatch(wrapper(new AuditAction()));';expected=@('wrapper')},
    @{id='conditional';body='this.store.dispatch(flag ? new AuditAction() : other);';expected=@('flag')},
    @{id='array';body='this.store.dispatch([new AuditAction()]);';expected=@('[new')},
    @{id='nested-wrapper';body='this.store.dispatch(makeAction(new AuditAction()));';expected=@('makeAction')},
    @{id='no-constructor-symbolic';body='this.store.dispatch(AuditAction);';expected=@('AuditAction')},
    @{id='factory-modern';body='this.store.dispatch(AuditAction());';expected=@('AuditAction')},
    @{id='duplicate';body='this.store.dispatch(new AuditAction()); this.store.dispatch(new AuditAction());';expected=@('AuditAction')},
    @{id='multiple';body='this.store.dispatch(new AuditAction()); this.store.dispatch(new OtherAction());';expected=@('AuditAction','OtherAction')},
    @{id='independent';body='';expected=@()},
    @{id='import-boundary';body='this.store.dispatch(new AuditAction());';from="export class AuditAction implements Action {readonly type='audit';}";to="import {AuditAction} from './actions';";expected=@();actions=@('OtherAction')},
    @{id='fallback-owner';body='this.store.dispatch(new AuditAction());';from="@Component({selector:'audit-dispatch', template:''})";to='';expected=@('AuditAction');ownerType='DispatchSite';owner='store'}
)
foreach ($case in $cases+@(@{id='collision'},@{id='lifecycle'})) {
    $case.root=Join-Path $runRoot $case.id
    New-Item -ItemType Directory -Force $case.root | Out-Null
    & git -C $case.root init --quiet
    if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
    $config=if ($case.ContainsKey('disabled')) {'{"cbm":{"enabled":false},"meta_layers":{"angular":{"ngrx":{"enabled":false}}}}'} else {'{"cbm":{"enabled":false}}'}
    Write-BText (Join-Path $case.root '.clean-ctx.json') $config
    if ($case.ContainsKey('body')) {$case.source=Dispatch-Source $case;Write-BText (Join-Path $case.root 'dispatch.ts') $case.source}
}
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;cbm_enabled=$false;scope='same-file direct constructor; imported/qualified forms fail closed; symbolic modern behavior preserved'}
function Read-Dispatch($Session,[string]$Query,[string]$Domain,[string]$Type,[string]$Symbol,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type=$Query;domain=$Domain;entity_type=$Type;name=$Symbol})
    Save-B $Label $answer
    return @($answer.structuredContent.edges | Where-Object relation -eq 'Dispatches')
}
function Assert-Dispatch($Session,[string]$File,[string[]]$Expected,[string[]]$Actions,[string]$Label,[string]$OwnerType='Component',[string]$Owner='Dispatcher') {
    $Expected=@($Expected | Where-Object {$null -ne $_});$Actions=@($Actions | Where-Object {$null -ne $_})
    $absolute=Join-Path $root $File
    $entities=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B ($Label+'-entities') $entities
    $found=@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'ngrx' -and $_.entity_type -eq 'Action'})
    $publicNames=@(@($Actions)+@($Expected) | Sort-Object -Unique)
    Assert-B ($found.Count -eq $publicNames.Count) 'Public Action set differs (independent declarations plus preserved symbolic endpoints).'
    foreach ($targetEntity in $Actions) {Assert-B (@($found | Where-Object {$_.name -ceq $targetEntity -and $_.file -eq $absolute}).Count -eq 1) 'Action declaration identity/file missing.'}
    $forward=@(Read-Dispatch $Session 'forward_edges' 'angular' $OwnerType $Owner ($Label+'-forward') | Where-Object {$_.subject.file -eq $absolute})
    Assert-B ($forward.Count -eq $Expected.Count) 'Wrong exact Dispatches set.'
    foreach ($targetEntity in $Expected) {
        $exact=@($forward | Where-Object {$_.subject.domain -eq 'angular' -and $_.subject.entity_type -eq $OwnerType -and $_.subject.name -ceq $Owner -and $_.object.domain -eq 'ngrx' -and $_.object.entity_type -eq 'Action' -and $_.object.name -ceq $targetEntity -and $_.object.file -eq $absolute})
        Assert-B ($exact.Count -eq 1) 'Endpoint identity/direction/file differs.'
    }
    foreach ($targetEntity in @(@('AuditAction','OtherAction','new','NonAction','wrapper')+@($Expected) | Sort-Object -Unique)) {
        $reverse=@(Read-Dispatch $Session 'reverse_edges' 'ngrx' 'Action' $targetEntity ($Label+'-'+$targetEntity+'-reverse') | Where-Object {$_.subject.file -eq $absolute})
        $wanted=if ($Expected -ccontains $targetEntity) {1} else {0}
        Assert-B ($reverse.Count -eq $wanted) 'Reverse edge is stale or differs from forward set.'
        foreach ($edge in $reverse) {Assert-B ($edge.subject.entity_type -eq $OwnerType -and $edge.subject.name -ceq $Owner -and $edge.object.file -eq $absolute) 'Reverse owner/provenance mismatch.'}
    }
}
try {
    foreach ($case in $cases) {
        $root=$case.root;$session=Start-BSession $case.id $BinaryPath $root
        foreach ($fidelity in @('high','medium','low')) {
            Invoke-BCase '140c-exact-dispatch' ($case.id+'-'+$fidelity) {
                $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='dispatch.ts';fidelity=$fidelity})
                Save-B ($case.id+'-'+$fidelity+'-context') $context
                $actions=if ($case.ContainsKey('actions')) {$case.actions} else {@('AuditAction','OtherAction')}
                $ownerType=if ($case.ContainsKey('ownerType')) {$case.ownerType} else {'Component'}
                $ownerSymbol=if ($case.ContainsKey('owner')) {$case.owner} else {'Dispatcher'}
                Assert-Dispatch $session 'dispatch.ts' $case.expected $actions ($case.id+'-'+$fidelity) $ownerType $ownerSymbol
                Assert-B ([IO.File]::ReadAllText((Join-Path $root 'dispatch.ts')) -ceq $case.source) 'Source changed.'
            }
        }
        if ($case.id -eq 'primary') {
            Invoke-BCase '140c-product-reuse' 'same-source-same-fidelity' {
                foreach ($repeat in @(1,2)) {
                    $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='dispatch.ts';fidelity='high'})
                    Save-B ('reuse-'+$repeat+'-context') $context
                    Assert-Dispatch $session 'dispatch.ts' @('AuditAction') @('AuditAction','OtherAction') ('reuse-'+$repeat)
                }
            }
        }
        Stop-BSession $session;$sessions.Remove($session) | Out-Null
    }
    $root=Join-Path $runRoot 'collision';$session=Start-BSession 'collision' $BinaryPath $root
    foreach ($file in @('a.ts','b.ts')) {Write-BText (Join-Path $root $file) (Dispatch-Source @{body='this.store.dispatch(new AuditAction());'})}
    foreach ($fidelity in @('high','medium','low')) {
        Invoke-BCase '140c-file-occurrences' ('same-names-'+$fidelity) {
            foreach ($file in @('a.ts','b.ts')) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity=$fidelity})}
            foreach ($file in @('a.ts','b.ts')) {Assert-Dispatch $session $file @('AuditAction') @('AuditAction','OtherAction') ('collision-'+$file+'-'+$fidelity)}
        }
    }
    Stop-BSession $session;$sessions.Remove($session) | Out-Null
    $root=Join-Path $runRoot 'lifecycle';$session=Start-BSession 'lifecycle' $BinaryPath $root
    foreach ($stage in @(
        @{id='initial';body='this.store.dispatch(new AuditAction());';expected=@('AuditAction')},
        @{id='neighbor';body='this.store.dispatch(new OtherAction());';expected=@('OtherAction')},
        @{id='non-action';body='this.store.dispatch(new NonAction());';expected=@()},
        @{id='remove';body='';expected=@()},
        @{id='delete-owner';body='';expected=@()},
        @{id='delete-file';body='';expected=@()}
    )) {
        Invoke-BCase '140c-refresh-lifecycle' $stage.id {
            if ($stage.id -eq 'delete-file') {Remove-Item (Join-Path $root 'dispatch.ts')}
            else {
                $source=Dispatch-Source $stage
                if ($stage.id -eq 'delete-owner') {$source=$source.Substring(0,$source.IndexOf('@Component'))}
                Write-BText (Join-Path $root 'dispatch.ts') $source
                Save-B ('lifecycle-'+$stage.id+'-source') @{source=$source}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Save-B ('lifecycle-'+$stage.id+'-refresh') $refresh
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            if ($stage.id -ne 'delete-file') {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='dispatch.ts';fidelity='high'})}
            $actions=if ($stage.id -eq 'delete-file') {@()} else {@('AuditAction','OtherAction')}
            Assert-Dispatch $session 'dispatch.ts' $stage.expected $actions ('lifecycle-'+$stage.id)
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Constructor dispatch evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
