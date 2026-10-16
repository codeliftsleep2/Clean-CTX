param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-service-select/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

$imports="import { Injectable, Component } from '@angular/core';`nimport { Store, createSelector } from '@ngrx/store';`n"
$selectors="const base = (state: unknown) => state;`nexport const selectAudit = createSelector(base, state => state);`nexport const selectOther = createSelector(base, state => state);`nconst ordinary = base;`n"
$service="@Injectable()`nexport class AuditService {`n  constructor(private state: Store<unknown>) {}`n  load() { BODY }`n}`n"
$second="@Injectable()`nexport class OtherService {`n  constructor(private state: Store<unknown>) {}`n  load() { BODY }`n}`n"
function Select-Source($Case) {
    $source=$imports+$selectors+$service.Replace('BODY',$Case.body)
    if ($Case.ContainsKey('from')) {$source=$source.Replace($Case.from,$Case.to)}
    if ($Case.ContainsKey('suffix')) {$source+=$Case.suffix}
    return $source+"`n"+('// unchanged indexing padding'+"`n")*100
}
$cases=@(
    @{id='primary';body='return this.state.select(selectAudit);';expected=@('AuditService|selectAudit')},
    @{id='duplicate';body='this.state.select(selectAudit); return this.state.select(selectAudit);';expected=@('AuditService|selectAudit')},
    @{id='two-selectors';body='this.state.select(selectAudit); return this.state.select(selectOther);';expected=@('AuditService|selectAudit','AuditService|selectOther')},
    @{id='ordinary-class';body='return this.state.select(selectAudit);';from="@Injectable()`nexport class AuditService";to='export class AuditService';services=@();expected=@()},
    @{id='component';body='return this.state.select(selectAudit);';from="@Injectable()`nexport class AuditService";to="@Component({selector:'x-audit'})`nexport class AuditService";services=@();expected=@()},
    @{id='fake-service-name';body='return this.state.select(selectAudit);';from="@Injectable()`nexport class AuditService";to='export class AuditService';services=@();expected=@()},
    @{id='foreign-store';body='return this.state.select(selectAudit);';from="import { Store, createSelector } from '@ngrx/store';";to="import { Store } from './foreign';`nimport { createSelector } from '@ngrx/store';";expected=@()},
    @{id='fake-store';body='return this.state.select(selectAudit);';from='Store<unknown>';to='FakeStore';expected=@()},
    @{id='ordinary-target';body='return this.state.select(ordinary);';expected=@()},
    @{id='nested-function';body='function inner(){return this.state.select(selectAudit);} return inner();';expected=@()},
    @{id='arrow-helper';body='const inner=()=>this.state.select(selectAudit); return inner();';expected=@()},
    @{id='static-method';body='';suffix='@Injectable() export class StaticService {constructor(private state:Store<unknown>){} static load(){return this.state.select(selectAudit);}}';services=@('AuditService','StaticService');expected=@()},
    @{id='neighbor-class';body='return this.state.select(selectAudit);';suffix='export class Helper {constructor(private state:Store<unknown>){} load(){return this.state.select(selectOther);}}';expected=@('AuditService|selectAudit')},
    @{id='two-services';body='return this.state.select(selectAudit);';suffix=$second.Replace('BODY','return this.state.select(selectOther);');services=@('AuditService','OtherService');expected=@('AuditService|selectAudit','OtherService|selectOther')},
    @{id='same-selector-two-services';body='return this.state.select(selectAudit);';suffix=$second.Replace('BODY','return this.state.select(selectAudit);');services=@('AuditService','OtherService');expected=@('AuditService|selectAudit','OtherService|selectAudit')}
)
foreach ($case in $cases+@(@{id='collision'},@{id='lifecycle'})) {
    $case.root=Join-Path $runRoot $case.id
    New-Item -ItemType Directory -Force $case.root | Out-Null
    & git -C $case.root init --quiet
    if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
    Write-BText (Join-Path $case.root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    if ($case.ContainsKey('body')) {$case.source=Select-Source $case;Write-BText (Join-Path $case.root 'service.ts') $case.source}
}
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;cbm_enabled=$false;scope='direct same-file this-property Store select ownership only'}

function Assert-SelectEdges($Session,[string]$File,[string[]]$Expected,[string[]]$Services,[string]$Label) {
    $Expected=@($Expected | Where-Object {$null -ne $_});$Services=@($Services | Where-Object {$null -ne $_})
    $absolute=Join-Path $root $File
    $entities=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B ($Label+'-entities') $entities
    $found=@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -ne 'Store'})
    foreach ($name in $Services) {Assert-B (@($found | Where-Object {$_.name -ceq $name -and $_.file -eq $absolute}).Count -eq 1) "Exact Service declaration absent: $name"}
    foreach ($owner in @('AuditService','OtherService','StaticService','Helper')) {
        $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='angular';entity_type='Service';name=$owner})
        Save-B ($Label+'-'+$owner+'-forward') $answer
        $edges=@($answer.structuredContent.edges | Where-Object {$_.relation -eq 'Selects' -and $_.subject.file -eq $absolute})
        $actual=@($edges | ForEach-Object {$_.subject.name+'|'+$_.object.name} | Sort-Object)
        $wanted=@($Expected | Where-Object {$_.StartsWith($owner+'|')} | Sort-Object)
        Assert-B (($actual -join "`n") -ceq ($wanted -join "`n")) "Exact Service Selects set differs for $owner."
        foreach ($edge in $edges) {Assert-B ($edge.object.domain -eq 'ngrx' -and $edge.object.entity_type -eq 'Selector' -and $edge.object.file -eq $absolute) 'Selector identity/file mismatch.'}
    }
}

try {
    foreach ($case in $cases) {
        $root=$case.root;$session=Start-BSession $case.id $BinaryPath $root
        foreach ($fidelity in @('high','medium','low')) {
            Invoke-BCase '140e-exact-ownership' ($case.id+'-'+$fidelity) {
                $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='service.ts';fidelity=$fidelity})
                $services=if ($case.ContainsKey('services')) {$case.services} else {@('AuditService')}
                Assert-SelectEdges $session 'service.ts' $case.expected $services ($case.id+'-'+$fidelity)
                Assert-B ([IO.File]::ReadAllText((Join-Path $root 'service.ts')) -ceq $case.source) 'Source changed.'
            }
        }
        Stop-BSession $session;$sessions.Remove($session) | Out-Null
    }
    $root=Join-Path $runRoot 'collision';$session=Start-BSession 'collision' $BinaryPath $root
    foreach ($file in @('a.ts','b.ts')) {Write-BText (Join-Path $root $file) (Select-Source @{body='return this.state.select(selectAudit);'})}
    foreach ($file in @('a.ts','b.ts')) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='high'})}
    foreach ($file in @('a.ts','b.ts')) {Assert-SelectEdges $session $file @('AuditService|selectAudit') @('AuditService') ('collision-'+$file)}
    Stop-BSession $session;$sessions.Remove($session) | Out-Null
    $root=Join-Path $runRoot 'lifecycle';$session=Start-BSession 'lifecycle' $BinaryPath $root
    foreach ($stage in @(
        @{id='initial';source=(Select-Source @{body='return this.state.select(selectAudit);'});expected=@('AuditService|selectAudit');services=@('AuditService')},
        @{id='remove-service';source=(Select-Source @{body='return this.state.select(selectAudit);';from="@Injectable()`nexport class AuditService";to='export class AuditService'});expected=@();services=@()},
        @{id='restore-renamed-property';source=(Select-Source @{body='return this.appStore.select(selectAudit);';from='private state:';to='private appStore:'});expected=@('AuditService|selectAudit');services=@('AuditService')},
        @{id='foreign-store';source=(Select-Source @{body='return this.state.select(selectAudit);';from="import { Store, createSelector } from '@ngrx/store';";to="import { Store } from './foreign';`nimport { createSelector } from '@ngrx/store';"});expected=@();services=@('AuditService')},
        @{id='invalid-selector';source=(Select-Source @{body='return this.state.select(ordinary);'});expected=@();services=@('AuditService')},
        @{id='restore';source=(Select-Source @{body='return this.state.select(selectAudit);'});expected=@('AuditService|selectAudit');services=@('AuditService')},
        @{id='delete-file';source=$null;expected=@();services=@()}
    )) {
        Invoke-BCase '140e-refresh-lifecycle' $stage.id {
            $path=Join-Path $root 'service.ts'
            if ($null -eq $stage.source) {Remove-Item -LiteralPath $path} else {Write-BText $path $stage.source}
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            if ($null -ne $stage.source) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='service.ts';fidelity='high'})}
            Assert-SelectEdges $session 'service.ts' $stage.expected $stage.services ('lifecycle-'+$stage.id)
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Service select evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
