param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-legacy-transitions/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
function Transition-Source($Case) {
    $other=if ($Case.ContainsKey('noOther')) {''} else {'export class OtherAction implements Action {readonly type=OTHER_AUDIT;}'}
    $audit=if ($Case.ContainsKey('noClasses')) {''} else {'export class AuditAction implements Action {readonly type=SET_AUDIT;}'}
    if ($Case.ContainsKey('noClasses')) {$other=''}
    $value=if ($Case.ContainsKey('differentValue')) {'different'} else {'same'}
    $source="import {Action} from '@ngrx/store'; export const SET_AUDIT='same'; export const OTHER_AUDIT='$value'; export const UNUSED_AUDIT='unused'; $audit $other export function auditReducer(state: any, action: Action) {switch(action.type) {$($Case.body) default: return state;}}"
    if ($Case.ContainsKey('from')) {$source=$source.Replace($Case.from,$Case.to)}
    if ($Case.ContainsKey('suffix')) {$source+=$Case.suffix}
    return $source+"`n"+('// unchanged economics padding'+"`n")*100
}
$cases=@(
    @{id='standalone-primary';body='case SET_AUDIT: return state;';noClasses=$true;expected=@('SET_AUDIT')},
    @{id='primary';body='case SET_AUDIT: return state;';expected=@('AuditAction','SET_AUDIT')},
    @{id='wrong-identifier';body='case OTHER_AUDIT: return state;';noOther=$true;differentValue=$true;expected=@('OTHER_AUDIT')},
    @{id='equal-value-wrong-identifier';body='case OTHER_AUDIT: return state;';noOther=$true;expected=@('OTHER_AUDIT')},
    @{id='neighbor';body='case OTHER_AUDIT: return state;';expected=@('OtherAction','OTHER_AUDIT')},
    @{id='literal';body="case 'same': return state;";expected=@()},
    @{id='unbound-class';body='case UNUSED_AUDIT: return state;';expected=@('UNUSED_AUDIT')},
    @{id='qualified';body='case Actions.SET_AUDIT: return state;';expected=@()},
    @{id='wrapper';body='case wrapper(SET_AUDIT): return state;';expected=@()},
    @{id='conditional';body='case flag ? SET_AUDIT : OTHER_AUDIT: return state;';expected=@()},
    @{id='no-case';body='';expected=@()},
    @{id='unrelated-switch';body='case SET_AUDIT: return state;';from='switch(action.type)';to='switch(state.mode)';expected=@()},
    @{id='untyped-parameter';body='case SET_AUDIT: return state;';from='action: Action';to='action: any';expected=@()},
    @{id='optional-receiver';body='case SET_AUDIT: return state;';from='switch(action.type)';to='switch(action?.type)';expected=@()},
    @{id='receiver-shadow';body='case SET_AUDIT: return state;';from='{switch(action.type)';to="{let action = {type:'other'}; switch(action.type)";expected=@()},
    @{id='hoisted-loop-shadow';body='case SET_AUDIT: return state;';from='{switch(action.type)';to='{for(var SET_AUDIT of values) {} switch(action.type)';expected=@()},
    @{id='constant-shadow';body='case SET_AUDIT: return state;';from='{switch(action.type)';to="{const SET_AUDIT='same'; switch(action.type)";expected=@()},
    @{id='shared-discriminant';body='case SET_AUDIT: return state;';from='readonly type=OTHER_AUDIT';to='readonly type=SET_AUDIT';expected=@('SET_AUDIT')},
    @{id='foreign-action';body='case SET_AUDIT: return state;';noClasses=$true;from="from '@ngrx/store'";to="from './fake-action'";expected=@()},
    @{id='local-action';body='case SET_AUDIT: return state;';noClasses=$true;from="import {Action} from '@ngrx/store';";to='interface Action {type:string;}' ;expected=@()},
    @{id='nested-owner';body='case SET_AUDIT: return state;';from='export function auditReducer';to='function make() { function auditReducer';suffix='}';expected=@()},
    @{id='duplicate';body='case SET_AUDIT: return state; case SET_AUDIT: return state;';expected=@('AuditAction','SET_AUDIT')},
    @{id='multiple';body='case SET_AUDIT: return state; case OTHER_AUDIT: return state;';expected=@('AuditAction','OtherAction','SET_AUDIT','OTHER_AUDIT')}
)
# Preallocate every fixture before destructive lifecycle edits; each has its own
# admitted Git root/config and source namespace. No repository files are edited.
foreach ($case in $cases+@(@{id='collision'},@{id='lifecycle'})) {
    $case.root=Join-Path $runRoot $case.id
    New-Item -ItemType Directory -Force $case.root | Out-Null
    & git -C $case.root init --quiet
    if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
    Write-BText (Join-Path $case.root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    if ($case.ContainsKey('body')) {$case.source=Transition-Source $case;Write-BText (Join-Path $case.root 'actions.ts') $case.source}
}
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;cbm_enabled=$false;scope='same-file source-bound Action to TriggersReducer to Reducer; no cross-file resolver claim'}
function Read-TransitionEdges($Session,[string]$Kind,[string]$Type,[string]$Symbol,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type=$Kind;domain='ngrx';entity_type=$Type;name=$Symbol})
    Save-B $Label $answer
    return @($answer.structuredContent.edges | Where-Object relation -eq 'TriggersReducer')
}
function Assert-TransitionFile($Session,[string]$File,[string[]]$Expected,[string[]]$Actions,[string]$Label) {
    $Expected=@($Expected | Where-Object {$null -ne $_});$Actions=@($Actions | Where-Object {$null -ne $_})
    $absolute=Join-Path $root $File
    $entities=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B ($Label+'-entities') $entities
    $found=@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'ngrx' -and $_.entity_type -eq 'Action'})
    Assert-B ($found.Count -eq $Actions.Count) 'Action independence lost or constant fabricated as Action.'
    foreach ($target in $Actions) {Assert-B (@($found | Where-Object {$_.name -ceq $target -and $_.file -eq $absolute}).Count -eq 1) 'Exact Action declaration/file missing.'}
    $reverse=@(Read-TransitionEdges $Session 'reverse_edges' 'Reducer' 'auditReducer' ($Label+'-reverse') | Where-Object {$_.object.file -eq $absolute})
    Assert-B ($reverse.Count -eq $Expected.Count) 'Wrong exact incoming transition set.'
    foreach ($target in $Expected) {
        Assert-B (@($reverse | Where-Object {$_.subject.domain -eq 'ngrx' -and $_.subject.entity_type -eq 'Action' -and $_.subject.name -ceq $target -and $_.subject.file -eq $absolute -and $_.object.domain -eq 'ngrx' -and $_.object.entity_type -eq 'Reducer' -and $_.object.name -ceq 'auditReducer' -and $_.object.file -eq $absolute}).Count -eq 1) 'Exact edge direction/identity/provenance missing.'
    }
    foreach ($target in $Actions) {
        $forward=@(Read-TransitionEdges $Session 'forward_edges' 'Action' $target ($Label+'-'+$target+'-forward') | Where-Object {$_.subject.file -eq $absolute})
        $wanted=if ($Expected -ccontains $target) {1} else {0}
        Assert-B ($forward.Count -eq $wanted) 'Stale/wrong Action forward edge.'
        foreach ($edge in $forward) {Assert-B ($edge.object.domain -eq 'ngrx' -and $edge.object.entity_type -eq 'Reducer' -and $edge.object.name -ceq 'auditReducer' -and $edge.object.file -eq $absolute) 'Forward edge target/provenance differs from reverse oracle.'}
    }
}
try {
    foreach ($case in $cases) {
        $root=$case.root;$session=Start-BSession $case.id $BinaryPath $root
        foreach ($fidelity in @('high','medium','low')) {
            Invoke-BCase '140b-exact-transitions' ($case.id+'-'+$fidelity) {
                $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='actions.ts';fidelity=$fidelity})
                Save-B ($case.id+'-'+$fidelity+'-context') $context
                $actions=@()
                if ($case.ContainsKey('noClasses')) {}
                elseif ($case.ContainsKey('noOther')) {$actions+='AuditAction'}
                else {$actions+=@('AuditAction','OtherAction')}
                $actions+=@($case.expected | Where-Object {$_ -cmatch '^[A-Z0-9_]+$'})
                $actions=@($actions | Sort-Object -Unique)
                Assert-TransitionFile -Session $session -File 'actions.ts' -Expected @($case.expected) -Actions @($actions) -Label ($case.id+'-'+$fidelity)
                Assert-B ([IO.File]::ReadAllText((Join-Path $root 'actions.ts')) -ceq $case.source) 'Source changed.'
            }
        }
        Stop-BSession $session;$sessions.Remove($session) | Out-Null
    }
    $root=Join-Path $runRoot 'collision';$session=Start-BSession 'collision' $BinaryPath $root
    $collision=Transition-Source @{body='case SET_AUDIT: return state;'}
    foreach ($file in @('a.ts','b.ts')) {Write-BText (Join-Path $root $file) $collision}
    foreach ($fidelity in @('high','medium','low')) {
        Invoke-BCase '140b-source-occurrences' ('same-names-'+$fidelity) {
            foreach ($file in @('a.ts','b.ts')) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity=$fidelity})}
            foreach ($file in @('a.ts','b.ts')) {Assert-TransitionFile -Session $session -File $file -Expected @('AuditAction','SET_AUDIT') -Actions @('AuditAction','OtherAction','SET_AUDIT') -Label ('collision-'+$file+'-'+$fidelity)}
        }
    }
    Stop-BSession $session;$sessions.Remove($session) | Out-Null
    $root=Join-Path $runRoot 'lifecycle';$session=Start-BSession 'lifecycle' $BinaryPath $root
    $stages=@(
        @{id='initial';body='case SET_AUDIT: return state;';expected=@('AuditAction','SET_AUDIT')},
        @{id='change-to-other';body='case OTHER_AUDIT: return state;';expected=@('OtherAction','OTHER_AUDIT')},
        @{id='literal';body="case 'same': return state;";expected=@()},
        @{id='remove-case';body='';expected=@()},
        @{id='foreign-action';body='case SET_AUDIT: return state;';foreign=$true;expected=@()},
        @{id='restore';body='case SET_AUDIT: return state;';expected=@('AuditAction','SET_AUDIT')},
        @{id='delete-reducer';body='';expected=@()},
        @{id='delete-file';body='';expected=@()}
    )
    foreach ($stage in $stages) {
        Invoke-BCase '140b-refresh-lifecycle' $stage.id {
            if ($stage.id -eq 'delete-file') {Remove-Item (Join-Path $root 'actions.ts')}
            else {
                $source=Transition-Source $stage
                if ($stage.ContainsKey('foreign')) {$source=$source.Replace("from '@ngrx/store'","from './fake-action'")}
                if ($stage.id -eq 'delete-reducer') {$source=$source.Substring(0,$source.IndexOf('export function auditReducer'))}
                Write-BText (Join-Path $root 'actions.ts') $source
                Save-B ('lifecycle-'+$stage.id+'-source') @{source=$source}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Save-B ('lifecycle-'+$stage.id+'-refresh') $refresh
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            if ($stage.id -ne 'delete-file') {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='actions.ts';fidelity='high'})}
            $actions=@()
            if ($stage.id -ne 'delete-file') {$actions+=@('AuditAction','OtherAction')}
            if ($stage.ContainsKey('foreign')) {$actions=@()}
            $actions+=@($stage.expected | Where-Object {$_ -cmatch '^[A-Z0-9_]+$'})
            $actions=@($actions | Sort-Object -Unique)
            Assert-TransitionFile -Session $session -File 'actions.ts' -Expected @($stage.expected) -Actions @($actions) -Label ('lifecycle-'+$stage.id)
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Legacy transition evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
