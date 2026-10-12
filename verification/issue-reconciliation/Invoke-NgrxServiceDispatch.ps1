param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-service-dispatch/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
$head="import {Injectable, Component} from '@angular/core'; import {Action,Store} from '@ngrx/store'; export class AuditAction implements Action {readonly type='audit';} export class OtherAction implements Action {readonly type='audit';} class NonAction {} "
$owner="@Injectable() export class AuditService {constructor(private store:Store<unknown>){} run(){BODY}}"
$other="@Injectable() export class OtherService {constructor(private store:Store<unknown>){} run(){BODY}}"
$helper="export class Helper {constructor(private store:Store<unknown>){} run(){this.store.dispatch(new AuditAction());}}"
function Service-Source($Case) {
    $source=$head+$owner.Replace('BODY',$Case.body)
    if ($Case.ContainsKey('from')) {$source=$source.Replace($Case.from,$Case.to)}
    if ($Case.ContainsKey('prefix')) {$source=$Case.prefix+$source}
    if ($Case.ContainsKey('suffix')) {$source+=$Case.suffix}
    return $source+"`n"+('// unchanged economics padding'+"`n")*100
}
$cases=@(
    @{id='primary';body='this.store.dispatch(new AuditAction());';expected=@('Service|AuditService|AuditAction')},
    @{id='second-method';body='this.store.dispatch(new AuditAction());';from='run(){';to='save(){';expected=@('Service|AuditService|AuditAction')},
    @{id='duplicate';body='this.store.dispatch(new AuditAction()); this.store.dispatch(new AuditAction());';expected=@('Service|AuditService|AuditAction')},
    @{id='two-methods';body='this.store.dispatch(new AuditAction());';from='run(){this.store.dispatch(new AuditAction());}';to='run(){this.store.dispatch(new AuditAction());} second(){this.store.dispatch(new AuditAction());}';expected=@('Service|AuditService|AuditAction')},
    @{id='neighbor-action';body='this.store.dispatch(new OtherAction());';expected=@('Service|AuditService|OtherAction')},
    @{id='component';body='this.store.dispatch(new AuditAction());';from='@Injectable()';to="@Component({selector:'x'})";services=@();expected=@('Component|AuditService|AuditAction')},
    @{id='helper';body='this.store.dispatch(new AuditAction());';from='@Injectable()';to='';services=@();expected=@('DispatchSite|store|AuditAction')},
    @{id='non-ngrx';body='this.logger.dispatch(new AuditAction());';expected=@()},
    @{id='non-action';body='this.store.dispatch(new NonAction());';expected=@()},
    @{id='unknown';body='this.store.dispatch(new UnknownAction());';expected=@()},
    @{id='qualified';body='this.store.dispatch(new Actions.AuditAction());';expected=@()},
    @{id='plain-store-separate';body='this.store.dispatch(new AuditAction());';from='Store<unknown>';to='Store';expected=@()},
    @{id='top-before';body='';prefix='this.store.dispatch(new AuditAction());';expected=@()},
    @{id='top-after';body='';suffix='this.store.dispatch(new AuditAction());';expected=@()},
    @{id='nested-function';body='function nested(){store.dispatch(new AuditAction());}';expected=@()},
    @{id='callback';body='const callback=()=>{this.store.dispatch(new AuditAction());};';expected=@()},
    @{id='independent';body='';expected=@()},
    @{id='other-service';body='';suffix=$other.Replace('BODY','this.store.dispatch(new AuditAction());');services=@('AuditService','OtherService');expected=@('Service|OtherService|AuditAction')},
    @{id='two-services';body='this.store.dispatch(new AuditAction());';suffix=$other.Replace('BODY','this.store.dispatch(new OtherAction());');services=@('AuditService','OtherService');expected=@('Service|AuditService|AuditAction','Service|OtherService|OtherAction')},
    @{id='same-action-fallback';body='this.store.dispatch(new AuditAction());';suffix=$helper;expected=@('Service|AuditService|AuditAction','DispatchSite|store|AuditAction')},
    @{id='same-action-component';body='this.store.dispatch(new AuditAction());';suffix="@Component({selector:'x'}) export class ComponentOwner {constructor(private store:Store<unknown>){} run(){this.store.dispatch(new AuditAction());}}";expected=@('Service|AuditService|AuditAction','Component|ComponentOwner|AuditAction')},
    @{id='symbolic-class-call';body='this.store.dispatch(AuditAction());';expected=@('DispatchSite|store|AuditAction')},
    @{id='modern-creator';body='this.store.dispatch(modernAction());';from="import {Action,Store} from '@ngrx/store';";to="import {Action,Store,createAction} from '@ngrx/store';";suffix="export const modernAction = createAction('[Audit] Modern');";actions=@('AuditAction','OtherAction','modernAction');expected=@('DispatchSite|store|modernAction')},
    @{id='disabled';body='this.store.dispatch(new AuditAction());';disabled=$true;services=@('AuditService');actions=@();expected=@()}
)
foreach ($case in $cases+@(@{id='collision'},@{id='lifecycle'})) {
    $case.root=Join-Path $runRoot $case.id
    New-Item -ItemType Directory -Force $case.root | Out-Null
    & git -C $case.root init --quiet
    if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
    $config=if ($case.ContainsKey('disabled')) {'{"cbm":{"enabled":false},"meta_layers":{"angular":{"ngrx":{"enabled":false}}}}'} else {'{"cbm":{"enabled":false}}'}
    Write-BText (Join-Path $case.root '.clean-ctx.json') $config
    if ($case.ContainsKey('body')) {$case.source=Service-Source $case;Write-BText (Join-Path $case.root 'service.ts') $case.source}
}
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;cbm_enabled=$false;scope='direct same-file constructor dispatch ownership; no plain Store/module/select/callback ownership expansion'}
function Read-ServiceEdges($Session,[string]$Query,[string]$Type,[string]$Symbol,[string]$Label) {
    $domain=if ($Query -eq 'reverse_edges') {'ngrx'} else {'angular'}
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type=$Query;domain=$domain;entity_type=$Type;name=$Symbol})
    Save-B $Label $answer
    return @($answer.structuredContent.edges | Where-Object relation -eq 'Dispatches')
}
function Assert-ServiceEdges($Session,[string]$File,[string[]]$Expected,[string[]]$Services,[string[]]$Actions,[string]$Label) {
    $Expected=@($Expected | Where-Object {$null -ne $_});$Services=@($Services | Where-Object {$null -ne $_});$Actions=@($Actions | Where-Object {$null -ne $_})
    $absolute=Join-Path $root $File
    $entities=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B ($Label+'-entities') $entities
    foreach ($family in @(@{kind='Service';domain='angular';names=$Services},@{kind='Action';domain='ngrx';names=$Actions})) {
        $found=@($entities.structuredContent.entities | Where-Object {$_.entity_type -eq $family.kind -and $_.domain -eq $family.domain -and ($family.kind -ne 'Service' -or $_.name -ne 'Store')})
        Assert-B ($found.Count -eq $family.names.Count) 'Independent declaration set differs.'
        foreach ($symbol in $family.names) {Assert-B (@($found | Where-Object {$_.name -ceq $symbol -and $_.file -eq $absolute}).Count -eq 1) 'Exact independent declaration/file absent.'}
    }
    $incoming=@()
    foreach ($targetEntity in @(@('AuditAction','OtherAction','new','NonAction','UnknownAction')+@($Actions) | Sort-Object -Unique)) {
        $reverse=@(Read-ServiceEdges $Session 'reverse_edges' 'Action' $targetEntity ($Label+'-'+$targetEntity+'-reverse') | Where-Object {$_.subject.file -eq $absolute})
        foreach ($edge in $reverse) {
            Assert-B ($edge.subject.domain -eq 'angular' -and $edge.object.domain -eq 'ngrx' -and $edge.object.entity_type -eq 'Action' -and $edge.object.file -eq $absolute) 'Endpoint kind/file mismatch.'
            $incoming+=($edge.subject.entity_type+'|'+$edge.subject.name+'|'+$edge.object.name)
        }
    }
    $actualSet=($incoming | Sort-Object) -join "`n"
    $expectedSet=($Expected | Sort-Object) -join "`n"
    Assert-B ($actualSet -ceq $expectedSet) 'Exact reverse owner/target set differs.'
    foreach ($ownerIdentity in @('Service|AuditService','Service|OtherService','Component|AuditService','Component|ComponentOwner','DispatchSite|store','Service|Helper')) {
        $parts=$ownerIdentity.Split('|')
        $forward=@(Read-ServiceEdges $Session 'forward_edges' $parts[0] $parts[1] ($Label+'-'+$parts[0]+'-'+$parts[1]+'-forward') | Where-Object {$_.subject.file -eq $absolute})
        $wanted=@($Expected | Where-Object {$_.StartsWith($ownerIdentity+'|')})
        Assert-B ($forward.Count -eq $wanted.Count) 'Forward owner count differs.'
        foreach ($edge in $forward) {Assert-B ($edge.object.file -eq $absolute -and $edge.object.domain -eq 'ngrx' -and $edge.object.entity_type -eq 'Action' -and $Expected -ccontains ($ownerIdentity+'|'+$edge.object.name)) 'Forward identity differs from exact reverse oracle.'}
    }
}
try {
    foreach ($case in $cases) {
        $root=$case.root;$session=Start-BSession $case.id $BinaryPath $root
        foreach ($fidelity in @('high','medium','low')) {
            Invoke-BCase '140d-exact-ownership' ($case.id+'-'+$fidelity) {
                $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='service.ts';fidelity=$fidelity})
                Save-B ($case.id+'-'+$fidelity+'-context') $context
                $services=if ($case.ContainsKey('services')) {$case.services} else {@('AuditService')}
                $actions=if ($case.ContainsKey('actions')) {$case.actions} else {@('AuditAction','OtherAction')}
                Assert-ServiceEdges $session 'service.ts' $case.expected $services $actions ($case.id+'-'+$fidelity)
                Assert-B ([IO.File]::ReadAllText((Join-Path $root 'service.ts')) -ceq $case.source) 'Source changed.'
            }
        }
        if ($case.id -eq 'primary') {
            Invoke-BCase '140d-product-reuse' 'same-source-same-fidelity' {
                foreach ($repeat in @(1,2)) {
                    $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='service.ts';fidelity='high'})
                    Save-B ('reuse-'+$repeat+'-context') $context
                    Assert-ServiceEdges $session 'service.ts' $case.expected @('AuditService') @('AuditAction','OtherAction') ('reuse-'+$repeat)
                }
            }
        }
        Stop-BSession $session;$sessions.Remove($session) | Out-Null
    }
    $root=Join-Path $runRoot 'collision';$session=Start-BSession 'collision' $BinaryPath $root
    foreach ($file in @('a.ts','b.ts')) {Write-BText (Join-Path $root $file) (Service-Source @{body='this.store.dispatch(new AuditAction());'})}
    foreach ($fidelity in @('high','medium','low')) {
        Invoke-BCase '140d-file-occurrences' ('same-names-'+$fidelity) {
            foreach ($file in @('a.ts','b.ts')) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity=$fidelity})}
            foreach ($file in @('a.ts','b.ts')) {Assert-ServiceEdges $session $file @('Service|AuditService|AuditAction') @('AuditService') @('AuditAction','OtherAction') ('collision-'+$file+'-'+$fidelity)}
        }
    }
    Stop-BSession $session;$sessions.Remove($session) | Out-Null
    $root=Join-Path $runRoot 'lifecycle';$session=Start-BSession 'lifecycle' $BinaryPath $root
    foreach ($stage in @(
        @{id='initial';a='this.store.dispatch(new AuditAction());';b='';expected=@('Service|AuditService|AuditAction')},
        @{id='move-owner';a='';b='this.store.dispatch(new AuditAction());';expected=@('Service|OtherService|AuditAction')},
        @{id='remove-qualification';a='this.store.dispatch(new AuditAction());';b='';expected=@('DispatchSite|store|AuditAction')},
        @{id='restore';a='this.store.dispatch(new AuditAction());';b='';expected=@('Service|AuditService|AuditAction')},
        @{id='change-target';a='this.store.dispatch(new OtherAction());';b='';expected=@('Service|AuditService|OtherAction')},
        @{id='remove-dispatch';a='';b='';expected=@()},
        @{id='delete-owner';a='';b='';expected=@()},
        @{id='delete-file';a='';b='';expected=@()}
    )) {
        Invoke-BCase '140d-refresh-lifecycle' $stage.id {
            $services=@('AuditService','OtherService');$actions=@('AuditAction','OtherAction')
            if ($stage.id -eq 'delete-file') {Remove-Item (Join-Path $root 'service.ts');$services=@();$actions=@()}
            else {
                $first=$owner.Replace('BODY',$stage.a)
                if ($stage.id -eq 'remove-qualification') {$first=$first.Replace('@Injectable()','');$services=@('OtherService')}
                if ($stage.id -eq 'delete-owner') {$first='';$services=@('OtherService')}
                $source=$head+$first+$other.Replace('BODY',$stage.b)+"`n"+('// padding'+"`n")*100
                Write-BText (Join-Path $root 'service.ts') $source
                Save-B ('lifecycle-'+$stage.id+'-source') @{source=$source}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Save-B ('lifecycle-'+$stage.id+'-refresh') $refresh
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            if ($stage.id -ne 'delete-file') {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='service.ts';fidelity='high'})}
            Assert-ServiceEdges $session 'service.ts' $stage.expected $services $actions ('lifecycle-'+$stage.id)
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Service dispatch evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
