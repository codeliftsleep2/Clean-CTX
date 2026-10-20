param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [ValidateRange(30, 600)][int]$CaseTimeoutSeconds = 180
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$runRoot = Join-Path $repo ('target/issue-152-ngrx/' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
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

function Write-Fixture([string]$Root) {
    Write-BText (Join-Path $Root 'actions.ts') @"
export const ACTION_1 = 'v1';
export const ActionObjectA = { ACTION_1 };
"@
    Write-BText (Join-Path $Root 'dispatch.ts') @"
import { Store } from '@ngrx/store';
import { ActionObjectA as AliasA } from './actions';
export class ComponentA {
  constructor(private store: Store<unknown>) {}
  send() { this.store.dispatch({ type: AliasA.ACTION_1 }); }
}
"@
    Write-BText (Join-Path $Root 'reducer.ts') @"
import { Action } from '@ngrx/store';
import { ACTION_1 } from './actions';
export function ReducerA(state: unknown, action: Action) {
  switch (action.type) { case ACTION_1: return state; default: return state; }
}
"@
    Write-BText (Join-Path $Root 'reducers.ts') @"
import { ReducerA } from './reducer';
export const reducers = { feature: ReducerA };
"@
    Write-BText (Join-Path $Root 'module.ts') @"
import { StoreModule } from '@ngrx/store';
import { reducers } from './reducers';
StoreModule.forRoot(reducers);
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

function Forward($Session, [string]$Root, [string]$Name, [string]$Capture) {
    $answer = Query $Session $Root @{
        type='forward_edges';domain='ngrx';entity_type='Action';name=$Name
    } $Capture
    return @($answer.edges)
}

function Assert-Graph($Session, [string]$Root, [string]$Label) {
    Assert-B (@(Find-Role $Session $Root 'ActionObjectA.ACTION_1' 'Action' "$Label-action").Count -eq 1) 'Qualified shorthand Action authority was absent.'
    Assert-B (@(Find-Role $Session $Root 'ACTION_1' 'Action' "$Label-bare-action").Count -eq 0) 'Bare constant replaced qualified Action identity.'
    Assert-B (@(Find-Role $Session $Root 'ReducerA' 'Reducer' "$Label-reducer").Count -eq 1) 'Reducer-map registration did not authorize ReducerA.'
    $edges = @(Forward $Session $Root 'ActionObjectA.ACTION_1' "$Label-forward")
    Assert-B (@($edges | Where-Object {
        $_.relation -eq 'TriggersReducer' -and $_.object.name -ceq 'ReducerA'
    }).Count -eq 1) 'Bare case did not join the qualified Action to ReducerA.'
}

Save-B 'provenance' @{
    binary = $BinaryPath
    sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    cbm_enabled = $false
    scope = 'Issue 152 shorthand/reference actions, bare cases, reducer maps, dependency closure and lifecycle'
}

try {
    $root = New-Workspace 'authority'
    Write-Fixture $root
    $session = Start-BSession 'authority' $BinaryPath $root

    Invoke-BCase '152-ngrx-semantic-envelope' 'cold-module-query-acquires-complete-authority' {
        $cold = Query $session $root @{
            type='entities_in_file';file_path=(Join-Path $root 'module.ts');fidelity='high'
        } 'cold-module'
        Assert-B ($cold.coverage.status -eq 'authoritative_complete') 'Cold module result was not authoritative.'
        Assert-B (-not $cold.coverage.PSObject.Properties['semantic_omissions']) 'Cold module retained semantic omissions.'
        Assert-B (@($cold.entities | Where-Object entity_type -eq 'StoreRegistration').Count -eq 1) 'Cold module omitted StoreRegistration.'
        $script:coldModuleSignature = @{
            entities = @($cold.entities)
            coverage = $cold.coverage
        } | ConvertTo-Json -Depth 20 -Compress
    }

    Invoke-BCase '152-ngrx-semantic-envelope' 'warm-result-is-semantically-equivalent' {
        foreach ($file in @('dispatch.ts', 'actions.ts', 'reducers.ts', 'reducer.ts', 'module.ts')) {
            $null = Query $session $root @{
                type='entities_in_file';file_path=(Join-Path $root $file);fidelity='high'
            } ('warm-' + $file.Replace('.', '-'))
        }
        $warmModule = Query $session $root @{
            type='entities_in_file';file_path=(Join-Path $root 'module.ts');fidelity='high'
        } 'warm-module-comparison'
        $warmModuleSignature = @{
            entities = @($warmModule.entities)
            coverage = $warmModule.coverage
        } | ConvertTo-Json -Depth 20 -Compress
        Assert-B ($warmModuleSignature -ceq $script:coldModuleSignature) 'Cold and warm module truth diverged.'
        Assert-Graph $session $root 'warm'
    }

    Invoke-BCase '152-ngrx-semantic-envelope' 'map-member-removal-retracts-reducer-and-handler' {
        Write-BText (Join-Path $root 'reducers.ts') 'export const reducers = {};'
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        Assert-B (@(Find-Role $session $root 'ReducerA' 'Reducer' 'map-removed-reducer').Count -eq 0) 'Reducer authority survived map-member removal.'
        $removedEdges = @(Forward $session $root 'ActionObjectA.ACTION_1' 'map-removed-forward')
        Assert-B (@($removedEdges | Where-Object relation -eq 'TriggersReducer').Count -eq 0) 'Handler survived map-member removal.'
    }

    Invoke-BCase '152-ngrx-semantic-envelope' 'restoration-rebuilds-authority' {
        Write-Fixture $root
        $null = Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        foreach ($file in @('module.ts', 'reducers.ts', 'reducer.ts', 'dispatch.ts', 'actions.ts')) {
            $null = Query $session $root @{
                type='entities_in_file';file_path=(Join-Path $root $file);fidelity='high'
            } ('restored-' + $file.Replace('.', '-'))
        }
        Assert-Graph $session $root 'restored'
    }

    Stop-BSession $session
    $sessions.Remove($session) | Out-Null

    $ambiguousRoot = New-Workspace 'ambiguous'
    Write-BText (Join-Path $ambiguousRoot 'actions.ts') "export const X='same'; export const A={X}; export const B={X};"
    Write-BText (Join-Path $ambiguousRoot 'dispatch.ts') @"
import { Store } from '@ngrx/store'; import { A, B } from './actions';
export class D { constructor(private store: Store<unknown>) {}
send(){ this.store.dispatch({type:A.X}); this.store.dispatch({type:B.X}); } }
"@
    Write-BText (Join-Path $ambiguousRoot 'reducer.ts') @"
import { Action } from '@ngrx/store'; import { X } from './actions';
export function R(s:unknown,a:Action){switch(a.type){case X:return s;default:return s;}}
"@
    Write-BText (Join-Path $ambiguousRoot 'module.ts') "import {StoreModule} from '@ngrx/store'; import {R} from './reducer'; StoreModule.forFeature('x',R);"
    $ambiguousSession = Start-BSession 'ambiguous' $BinaryPath $ambiguousRoot
    Invoke-BCase '152-ngrx-semantic-envelope' 'shared-constant-ambiguity-fails-closed' {
        foreach ($file in @('module.ts', 'dispatch.ts', 'actions.ts', 'reducer.ts')) {
            $null = Query $ambiguousSession $ambiguousRoot @{
                type='entities_in_file';file_path=(Join-Path $ambiguousRoot $file);fidelity='high'
            } ('ambiguous-' + $file.Replace('.', '-'))
        }
        foreach ($actionIdentity in @('A.X', 'B.X')) {
            $slug = $actionIdentity.Replace('.', '-')
            $roles = @(Find-Role $ambiguousSession $ambiguousRoot $actionIdentity 'Action' "ambiguous-$slug")
            Assert-B ($roles.Count -eq 1) "$actionIdentity lost its distinct qualified identity."
            $ambiguousEdges = @(Forward $ambiguousSession $ambiguousRoot $actionIdentity "ambiguous-forward-$slug")
            Assert-B (@($ambiguousEdges | Where-Object relation -eq 'TriggersReducer').Count -eq 0) 'Ambiguous bare case fabricated a handler.'
        }
    }
} finally {
    foreach ($openSession in $sessions) {
        try { Stop-BSession $openSession } catch {
            $verdicts.Add(@{phase='cleanup';case=$openSession.Name;result='FAIL';evidence=$_.Exception.Message})
        }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #152 NgRx evidence: $runRoot"
}

if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
Write-Host 'PASS: live MCP Issue #152 NgRx semantic-envelope verification completed.'
