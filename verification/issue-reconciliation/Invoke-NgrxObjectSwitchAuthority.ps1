param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [ValidateRange(30, 600)][int]$CaseTimeoutSeconds = 180
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$runRoot = Join-Path $repo ('target/ngrx-object-switch/' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

function New-Workspace([string]$Name) {
    $root = Join-Path $runRoot $Name
    New-Item -ItemType Directory -Force $root | Out-Null
    & git -C $root init --quiet
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git root initialization failed.' }
    Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    return $root
}

function Write-AuthorityFixture([string]$Root) {
    Write-BText (Join-Path $Root 'actions.ts') @"
export const ACTIONS = {
  SET_X: '[Feature] Same',
  CLEAR_X: '[Feature] Same'
};
"@
    Write-BText (Join-Path $Root 'reducer.ts') @"
import { ACTIONS as FeatureActions } from './actions';
export function reducer(state: unknown, evt: any) {
  switch (evt.type) {
    case FeatureActions.SET_X: return state;
    case FeatureActions.CLEAR_X: return state;
    default: return state;
  }
}
"@
    Write-BText (Join-Path $Root 'dispatch.ts') @"
import { Store } from '@ngrx/store';
import { ACTIONS as FeatureActions } from './actions';
export class FeatureComponent {
  constructor(private state: Store<unknown>) {}
  send() {
    this.state.dispatch({ type: FeatureActions.SET_X });
    this.state.dispatch({ type: FeatureActions.CLEAR_X });
  }
}
"@
    Write-BText (Join-Path $Root 'module.ts') @"
import { StoreModule as FeatureStoreModule } from '@ngrx/store';
import { reducer as featureReducer } from './reducer';
FeatureStoreModule.forFeature('feature', featureReducer);
"@
}

function Query($Session, [string]$Root, [hashtable]$Arguments, [string]$Capture) {
    $Arguments.workspaceRoot = $Root
    $result = Get-BResult (Invoke-BTool $Session 'workspace_query' $Arguments)
    Save-B $Capture $result
    return $result.structuredContent
}

function Find-Role($Session, [string]$Root, [string]$Name, [string]$Type, [string]$Capture) {
    $answer = Query $Session $Root @{type='find_entities';name=$Name} $Capture
    return @($answer.entities | Where-Object {
        $_.domain -eq 'ngrx' -and $_.entity_type -eq $Type -and $_.name -ceq $Name
    })
}

function Action-Forward($Session, [string]$Root, [string]$Name, [string]$Capture) {
    $answer = Query $Session $Root @{
        type='forward_edges';domain='ngrx';entity_type='Action';name=$Name
    } $Capture
    return @($answer.edges)
}

function Action-Reverse($Session, [string]$Root, [string]$Name, [string]$Capture) {
    $answer = Query $Session $Root @{
        type='reverse_edges';domain='ngrx';entity_type='Action';name=$Name
    } $Capture
    return @($answer.edges)
}

function Assert-AuthoritativeGraph($Session, [string]$Root, [string]$Label) {
    foreach ($action in @('ACTIONS.SET_X', 'ACTIONS.CLEAR_X')) {
        $roles = @(Find-Role $Session $Root $action 'Action' "$Label-$($action.Replace('.', '-'))-entity")
        Assert-B ($roles.Count -eq 1) "$action was not published exactly once as an NgRx Action."
        $forward = @(Action-Forward $Session $Root $action "$Label-$($action.Replace('.', '-'))-forward")
        Assert-B (@($forward | Where-Object {
            $_.relation -eq 'TriggersReducer' -and $_.object.domain -eq 'ngrx' -and
            $_.object.entity_type -eq 'Reducer' -and $_.object.name -ceq 'reducer'
        }).Count -eq 1) "$action did not trigger the registered reducer exactly once."
        $reverse = @(Action-Reverse $Session $Root $action "$Label-$($action.Replace('.', '-'))-reverse")
        Assert-B (@($reverse | Where-Object {
            $_.relation -eq 'Dispatches' -and $_.object.name -ceq $action
        }).Count -ge 1) "$action had no authoritative Store dispatch."
    }
    Assert-B (@(Find-Role $Session $Root 'FeatureActions.SET_X' 'Action' "$Label-alias-entity").Count -eq 0) 'Local alias leaked into semantic action identity.'
    Assert-B (@(Find-Role $Session $Root 'reducer' 'Reducer' "$Label-reducer-entity").Count -eq 1) 'Registered reducer authority was absent.'
}

function Assert-NoAction($Session, [string]$Root, [string]$Name, [string]$Label) {
    Assert-B (@(Find-Role $Session $Root $Name 'Action' "$Label-action").Count -eq 0) "$Name retained or fabricated NgRx Action authority."
}

function Assert-NoReducer($Session, [string]$Root, [string]$Label) {
    Assert-B (@(Find-Role $Session $Root 'reducer' 'Reducer' "$Label-reducer").Count -eq 0) 'Reducer authority survived without registration proof.'
}

function Acquire-ReducerContext($Session, [string]$Root, [string]$Label) {
    return Query $Session $Root @{
        type='entities_in_file';file_path=(Join-Path $Root 'reducer.ts');fidelity='high'
    } $Label
}

Save-B 'provenance' @{
    binary = $BinaryPath
    sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    cbm_enabled = $false
    scope = 'NgRx exported object members, exact Store dispatch, forFeature registration, switch binding, reconciliation and retraction'
}

try {
    $root = New-Workspace 'authority'
    Write-AuthorityFixture $root
    $session = Start-BSession 'authority' $BinaryPath $root

    Invoke-BCase '150-ngrx-object-switch' 'cold-query-acquires-complete-authority' {
        $cold = Query $session $root @{
            type='entities_in_file';file_path=(Join-Path $root 'reducer.ts');fidelity='high'
        } 'cold-reducer-file'
        Assert-B (-not $cold.coverage.PSObject.Properties['semantic_omissions']) 'Supported cold reducer retained semantic omissions.'
        Assert-AuthoritativeGraph $session $root 'cold'
    }

    Invoke-BCase '150-ngrx-object-switch' 'warm-query-is-semantically-equivalent' {
        foreach ($file in @('module.ts', 'dispatch.ts', 'reducer.ts', 'actions.ts')) {
            $null = Query $session $root @{
                type='entities_in_file';file_path=(Join-Path $root $file);fidelity='high'
            } ('warm-publish-' + $file.Replace('.', '-'))
        }
        Assert-AuthoritativeGraph $session $root 'warm'
    }

    Invoke-BCase '150-ngrx-object-switch' 'dispatch-removal-retracts-actions' {
        Write-BText (Join-Path $root 'dispatch.ts') 'export const removed = true;'
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        Assert-NoAction $session $root 'ACTIONS.SET_X' 'dispatch-removed'
        Assert-NoAction $session $root 'ACTIONS.CLEAR_X' 'dispatch-removed-clear'
    }

    Invoke-BCase '150-ngrx-object-switch' 'dispatch-restoration-rebuilds-authority' {
        Write-AuthorityFixture $root
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        $null = Acquire-ReducerContext $session $root 'restored-reducer-file'
        Assert-AuthoritativeGraph $session $root 'restored'
    }

    Invoke-BCase '150-ngrx-object-switch' 'registration-removal-retracts-reducer-and-handler' {
        Write-BText (Join-Path $root 'module.ts') 'export const removed = true;'
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        Assert-NoReducer $session $root 'registration-removed'
        $edges = @(Action-Forward $session $root 'ACTIONS.SET_X' 'registration-removed-forward')
        Assert-B (@($edges | Where-Object relation -eq 'TriggersReducer').Count -eq 0) 'Handler survived registration removal.'
    }

    Invoke-BCase '150-ngrx-object-switch' 'member-removal-retracts-only-that-action' {
        Write-AuthorityFixture $root
        Write-BText (Join-Path $root 'actions.ts') "export const ACTIONS = { CLEAR_X: '[Feature] Same' };"
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        $null = Acquire-ReducerContext $session $root 'member-change-reducer-file'
        Assert-NoAction $session $root 'ACTIONS.SET_X' 'member-removed'
        Assert-B (@(Find-Role $session $root 'ACTIONS.CLEAR_X' 'Action' 'member-remaining').Count -eq 1) 'Unchanged member lost action authority.'
    }

    Stop-BSession $session
    $sessions.Remove($session) | Out-Null

    $fakeRoot = New-Workspace 'fail-closed'
    Write-BText (Join-Path $fakeRoot 'actions.ts') "export const ACTIONS = { SET_X: '[Feature] Set X' };"
    Write-BText (Join-Path $fakeRoot 'consumer.ts') @"
import { ACTIONS } from './actions';
const something = { dispatch(value: unknown) {} };
something.dispatch({ type: ACTIONS.SET_X });
export function reducer(state: unknown, action: any) {
  switch (wrong.type) { case ACTIONS.SET_X: return state; default: return state; }
}
"@
    $fakeSession = Start-BSession 'fail-closed' $BinaryPath $fakeRoot
    Invoke-BCase '150-ngrx-object-switch' 'lookalikes-do-not-create-framework-authority' {
        $answer = Query $fakeSession $fakeRoot @{
            type='entities_in_file';file_path=(Join-Path $fakeRoot 'consumer.ts');fidelity='high'
        } 'fail-closed-file'
        Assert-B (@($answer.coverage.semantic_omissions) -contains 'ngrx_switch_action_type_handlers') 'Unsupported switch omission was not retained.'
        Assert-NoAction $fakeSession $fakeRoot 'ACTIONS.SET_X' 'fake-dispatch'
        Assert-NoReducer $fakeSession $fakeRoot 'unregistered'
    }
} finally {
    foreach ($openSession in $sessions) {
        try { Stop-BSession $openSession } catch {
            $verdicts.Add(@{phase='cleanup';case=$openSession.Name;result='FAIL';evidence=$_.Exception.Message})
        }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "NgRx object/switch evidence: $runRoot"
}

if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
Write-Host 'PASS: live MCP NgRx object-action/switch authority verification completed.'
