param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/ngrx-action-type-binding/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
$cases=@(
    @{id='direct';expression='SET_AUDIT'},
    @{id='other';expression='OTHER_AUDIT'},
    @{id='equal-literal';expression="'[Audit] Same'"},
    @{id='wrapper';expression='wrapper(SET_AUDIT)'},
    @{id='conditional';expression='condition ? SET_AUDIT : OTHER_AUDIT'},
    @{id='logical';expression='SET_AUDIT || OTHER_AUDIT'},
    @{id='template';expression='`${SET_AUDIT}`'},
    @{id='parenthesized';expression='(SET_AUDIT)'},
    @{id='qualified';expression='Actions.SET_AUDIT'}
)
foreach ($case in $cases) {
    $case.file=$case.id+'.ts'
    $case.source="import { Action } from '@ngrx/store'; export const SET_AUDIT = '[Audit] Same'; export const OTHER_AUDIT = '[Audit] Same'; export const SET_THEME = 'dark'; export class AuditAction implements Action {readonly type = $($case.expression);}"+"`n"+('// padding'+"`n")*100
    Write-BText (Join-Path $root $case.file) $case.source
}
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;scope='public preservation only; private association requires registered producer tests';cbm_enabled=$false}
function Assert-BindingPublic($Session,$Case,[string]$Fidelity) {
    $context=Get-BResult (Invoke-BTool $Session 'provide_code_context' @{workspaceRoot=$root;filePath=$Case.file;fidelity=$Fidelity})
    Save-B ($Case.id+'-'+$Fidelity+'-context') $context
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$Case.file})
    Save-B ($Case.id+'-'+$Fidelity+'-entities') $answer
    $actions=@($answer.structuredContent.entities | Where-Object {$_.domain -eq 'ngrx' -and $_.entity_type -eq 'Action'})
    Assert-B ($actions.Count -eq 1) 'Constant became an Action or class publication was lost.'
    Assert-B ($actions[0].name -ceq 'AuditAction' -and $actions[0].file -eq (Join-Path $root $Case.file)) 'Exact class identity/provenance changed.'
    $edges=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='ngrx';entity_type='Action';name='AuditAction'})
    Save-B ($Case.id+'-'+$Fidelity+'-edges') $edges
    Assert-B (@($edges.structuredContent.edges).Count -eq 0) 'Private binding leaked a public relation.'
    Assert-B ([IO.File]::ReadAllText((Join-Path $root $Case.file)) -ceq $Case.source) 'Source changed.'
}
try {
    $session=Start-BSession 'type-binding-preservation' $BinaryPath $root
    foreach ($case in $cases) {
        foreach ($fidelity in @('high','medium','low')) {
            Invoke-BCase '140b-prereq2-public-preservation' ($case.id+'-'+$fidelity) {
                Assert-BindingPublic $session $case $fidelity
            }
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Action type binding preservation evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
