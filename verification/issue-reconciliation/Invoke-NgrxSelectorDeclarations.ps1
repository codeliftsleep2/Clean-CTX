param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [ValidateRange(30, 600)][int]$CaseTimeoutSeconds = 180
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$runRoot = Join-Path $repo ('target/ngrx-selector-declarations/' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$root = Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) { throw 'Fixture Git root failed.' }
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'

$sources = @{
    'declared.ts' = "import { createSelector } from '@ngrx/store'; export const selectAudit = createSelector(selectState, state => state.audit);"
    'use-only.ts' = "import { Store } from '@ngrx/store'; declare const store: Store<unknown>; store.select(selectAudit);"
    'quoted-use.ts' = "import { Store } from '@ngrx/store'; declare const store: Store<unknown>; store.select('selectAudit');"
    'declared-used.ts' = "import { createSelector, Store } from '@ngrx/store'; declare const store: Store<unknown>; export const selectAudit = createSelector(selectState, state => state.audit); store.select(selectAudit);"
    'ordinary.ts' = "export const selectAudit = 'ordinary';"
    'action.ts' = "import { createAction } from '@ngrx/store'; export const setAudit = createAction('[Audit] Set');"
}
foreach ($entry in $sources.GetEnumerator()) {
    Write-BText (Join-Path $root $entry.Key) $entry.Value
}
Save-B 'provenance' @{
    binary = $BinaryPath
    sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    cbm = 'disabled'
    contract = 'workspace_query/declarations_in_file'
}

function Get-FileEntities($Session, [string]$Type, [string]$File) {
    $result = Get-BResult (Invoke-BTool $Session 'workspace_query' @{
        workspaceRoot = $root
        type = $Type
        file_path = $File
        fidelity = 'high'
    })
    Save-B "$File-$Type" $result
    return @($result.structuredContent.entities)
}

function Assert-Declaration($Entities, [string]$File, [string]$Kind, [string]$Name) {
    $absolute = Join-Path $root $File
    $exact = @($Entities | Where-Object {
        $_.domain -ceq 'ngrx' -and $_.entity_type -ceq $Kind -and
        $_.name -ceq $Name -and $_.file -ceq $absolute
    })
    Assert-B ($exact.Count -eq 1) "Expected one exact $Kind declaration in $File."
    Assert-B (@($Entities).Count -eq 1) "Unexpected additional declarations in $File."
}

try {
    $session = Start-BSession 'selector-declarations' $BinaryPath $root
    Save-B 'registered-tools' (Get-BResult (Invoke-BRpc $session 'tools/list' @{}))
    Invoke-BCase '140e-prereq1b' 'declaration-only' {
        Assert-Declaration (Get-FileEntities $session 'declarations_in_file' 'declared.ts') 'declared.ts' 'Selector' 'selectAudit'
    }
    Invoke-BCase '140e-prereq1b' 'use-only-remains-occurrence-not-declaration' {
        $occurrences = Get-FileEntities $session 'entities_in_file' 'use-only.ts'
        Assert-B (@($occurrences | Where-Object {$_.domain -ceq 'ngrx' -and $_.entity_type -ceq 'Selector'}).Count -eq 1) 'Existing use-created Selector occurrence disappeared.'
        Assert-B (@(Get-FileEntities $session 'declarations_in_file' 'use-only.ts').Count -eq 0) 'Use endpoint became declaration authority.'
    }
    Invoke-BCase '140e-prereq1b' 'quoted-use-is-not-declaration' {
        Assert-B (@(Get-FileEntities $session 'declarations_in_file' 'quoted-use.ts').Count -eq 0) 'Quoted use became declaration authority.'
    }
    Invoke-BCase '140e-prereq1b' 'declaration-plus-use-is-one-declaration' {
        Assert-Declaration (Get-FileEntities $session 'declarations_in_file' 'declared-used.ts') 'declared-used.ts' 'Selector' 'selectAudit'
    }
    Invoke-BCase '140e-prereq1b' 'ordinary-name-is-not-selector' {
        Assert-B (@(Get-FileEntities $session 'declarations_in_file' 'ordinary.ts').Count -eq 0) 'Ordinary const became Selector authority.'
    }
    Invoke-BCase '140e-prereq1b' 'modern-action-uses-generic-query' {
        Assert-Declaration (Get-FileEntities $session 'declarations_in_file' 'action.ts') 'action.ts' 'Action' 'setAudit'
    }
} finally {
    foreach ($session in $sessions) { Stop-BSession $session }
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "NgRx Selector declaration evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
