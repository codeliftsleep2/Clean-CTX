param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
# Acceptance probes for current issues, not fixes or automatic closure decisions.
# Uses the established live MCP session helpers; no CBM or language build runs.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/issue-reconciliation/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
New-Item -ItemType Directory -Force $runRoot | Out-Null
$root=Join-Path $runRoot 'native';New-Item -ItemType Directory -Force $root | Out-Null
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
Write-BText (Join-Path $root 'Contract.cs') 'namespace Audit133; public interface IEvidenceContract { int Read(); }'
for ($i=0;$i -lt 55;$i++) {
    $dir=Join-Path $root ('consumers/group'+[int][Math]::Floor($i/11));New-Item -ItemType Directory -Force $dir | Out-Null
    $name='Consumer'+$i.ToString('D3')
    Write-BText (Join-Path $dir "$name.cs") "namespace Audit133; public class $name { private readonly IEvidenceContract value; public $name(IEvidenceContract value) { this.value = value; } public int Read() { return value.Read(); } }"
}
$padding=(1..65 | ForEach-Object {"// audit padding $_"}) -join "`n"
Write-BText (Join-Path $root 'SharedHttpBase.cs') 'using Microsoft.AspNetCore.Mvc; namespace Audit134; public class SharedHttpBase : ControllerBase {}'
Write-BText (Join-Path $root 'SecondHttpBase.cs') 'namespace Audit134; public class SecondHttpBase : SharedHttpBase {}'
Write-BText (Join-Path $root 'OrdersEndpoint.cs') ($padding+"`n"+'using Microsoft.AspNetCore.Mvc; namespace Audit134; [Route("orders")] public class OrdersEndpoint : SecondHttpBase { [HttpGet("one")] public int GetOne() { return 1; } }')
Write-BText (Join-Path $root 'EditProbe.cs') ($padding+"`npublic class EditProbe {`n public int First() { return 1; }`n public int Second() { return 2; }`n}")
Write-BText (Join-Path $root 'service.ts') @'
import { Injectable } from '@angular/core';
@Injectable()
export class AuditApiService {
 read() { return 1; }
}
'@
Write-BText (Join-Path $root 'probe.pipe.ts') @'
import { Pipe } from '@angular/core';
import { AuditApiService } from './service';
@Pipe({name:'audit'})
export class AuditPipe {
 constructor(private api: AuditApiService) {}
 transform(value: string) { return value; }
}
'@
Write-BText (Join-Path $root 'inject.component.ts') @'
import { Component, inject, input, output } from '@angular/core';
import { AuditApiService } from './service';
@Component({selector:'audit-inject',template:''})
export class InjectProbe {
 api = inject(AuditApiService);
 item = input.required<string>();
 changed = output<string>();
}
'@
Write-BText (Join-Path $root 'owner.component.ts') ($padding+"`nimport { Component } from '@angular/core';`n@Component({`n selector:'audit-owner',`n template:''`n})`nclass OwnerProbe { read(): number { return 7; } }")
Write-BText (Join-Path $root 'legacy.actions.ts') @'
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set';
export function auditReducer(state: any, action: Action) {
 switch(action.type) {
 case SET_AUDIT: return state;
 default: return state;
 }
}
'@
Write-BText (Join-Path $root 'dispatch.component.ts') @'
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { SET_AUDIT } from './legacy.actions';
@Component({selector:'audit-dispatch',template:''})
export class DispatchProbe {
 constructor(private store: Store<any>) {}
 send() { this.store.dispatch({type:SET_AUDIT,payload:1}); }
}
'@
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;fixture_root=$root;ctor_consumers=55;ctor_directories=5}
function Publish-Audit([string]$File) {
    Get-BResult (Invoke-BTool $native 'provide_code_context' @{workspaceRoot=$root;filePath=$File;fidelity='edit'})
}
function Query-Audit([string]$Type,[string]$Domain,[string]$EntityType,[string]$Name) {
    $answer=Get-BResult (Invoke-BTool $native 'workspace_query' @{workspaceRoot=$root;type=$Type;domain=$Domain;entity_type=$EntityType;name=$Name})
    Assert-B ([bool]$answer.PSObject.Properties['structuredContent']) 'Missing native query body.'
    $body=$answer.structuredContent
    if ($Type -eq 'find_entities') {
        # The existing find_entities surface is cross-domain/type. Select the
        # requested declaration identity locally; never count a builtin Class
        # or an injection-site occurrence as a requested framework declaration.
        $selected=@($body.entities | Where-Object { $_.domain -eq $Domain -and $_.entity_type -eq $EntityType })
        $body | Add-Member -NotePropertyName verifier_selected_count -NotePropertyValue $selected.Count
        $body | Add-Member -NotePropertyName verifier_selected_entities -NotePropertyValue $selected
    }
    return $body
}
try {
    $native=Start-BSession 'native' $BinaryPath $root
    Invoke-BCase '133' 'cold-constructor-recall-55-files-five-directories' {
        $null=Get-BResult (Invoke-BTool $native 'refresh_workspace' @{workspaceRoot=$root})
        $answer=Query-Audit 'reverse_edges' 'builtin' 'TypeRef' 'IEvidenceContract'
        Save-B '133-cold' $answer
        $subjects=[Collections.Generic.HashSet[string]]::new()
        foreach ($edge in $answer.edges) {
            if ($edge.relation -ne 'HasConstructorParameterType') {continue}
            $match=[regex]::Match($edge.subject.name,'Consumer\d{3}')
            if ($match.Success) {$null=$subjects.Add($match.Value)}
        }
        Save-B '133-oracle' @{expected=55;observed=$subjects.Count;subjects=@($subjects)}
        Assert-B ($subjects.Count -eq 55) "Cold constructor recall is $($subjects.Count)/55; inspect coverage/discovery rather than infer completeness."
    }
    Invoke-BCase '134' 'controller-two-project-bases-below-framework' {
        foreach ($file in @('SharedHttpBase.cs','SecondHttpBase.cs','OrdersEndpoint.cs')) {$null=Publish-Audit $file}
        $answer=Query-Audit 'find_entities' 'dotnet' 'Controller' 'OrdersEndpoint'
        Assert-B ($answer.verifier_selected_count -gt 0) 'Indirect framework controller not classified after publishing its whole base chain.'
        $edges=Query-Audit 'forward_edges' 'dotnet' 'Controller' 'OrdersEndpoint'
        Save-B '134-controller-ancestry' @{entity=$answer;edges=$edges}
        Assert-B (@($edges.edges | Where-Object { $_.relation -eq 'HasRoute' -and $_.object.name -eq 'orders' }).Count -eq 1) 'Indirect controller route was not projected from the original endpoint.'
        Assert-B (@($edges.edges | Where-Object { $_.relation -eq 'ControllerAction' -and $_.object.name -eq 'GetOne' }).Count -eq 1) 'Indirect controller action set was not projected from the original endpoint.'
    }
    Invoke-BCase '136' 'insert-before-whole-member-declaration' {
        $null=Publish-Audit 'EditProbe.cs'
        $unit=" public int AddedBefore() { return 0; }`n"
        $answer=Invoke-BTool $native 'apply_edit' @{workspaceRoot=$root;filePath='EditProbe.cs';operations=@(@{type='insert_before';anchor='EditProbe.First';unitText=$unit})}
        $null=Get-BResult $answer
        $disk=[IO.File]::ReadAllText((Join-Path $root 'EditProbe.cs'))
        Assert-B ($disk.IndexOf('public int AddedBefore') -ge 0 -and $disk.IndexOf('public int AddedBefore') -lt $disk.IndexOf('public int First')) 'Insertion landed inside the body/signature rather than before the whole declaration.'
    }
    Invoke-BCase '137' 'return-type-only-committed-diff' {
        $diffRoot=Join-Path $root 'diff-fixture';New-Item -ItemType Directory -Force $diffRoot | Out-Null
        & git -C $diffRoot init --quiet;Assert-B ($LASTEXITCODE -eq 0) 'Diff fixture init failed.'
        & git -C $diffRoot config user.name 'Issue acceptance fixture'
        & git -C $diffRoot config user.email 'fixture@example.invalid'
        $hooks=Join-Path $diffRoot '.empty-hooks';New-Item -ItemType Directory -Force $hooks | Out-Null
        & git -C $diffRoot config core.hooksPath $hooks
        Write-BText (Join-Path $diffRoot 'DiffProbe.cs') 'public class DiffProbe { public int Get() { return 1; } }'
        & git -C $diffRoot add -- DiffProbe.cs
        & git -C $diffRoot commit --quiet -m 'fixture baseline'
        Assert-B ($LASTEXITCODE -eq 0) 'Diff baseline commit failed.'
        Write-BText (Join-Path $diffRoot 'DiffProbe.cs') 'public class DiffProbe { public long Get() { return 1; } }'
        & git -C $diffRoot add -- DiffProbe.cs
        & git -C $diffRoot commit --quiet -m 'fixture return type only'
        Assert-B ($LASTEXITCODE -eq 0) 'Diff changed commit failed.'
        $answer=Get-BResult (Invoke-BTool $native 'diff_commits' @{workspaceRoot=$diffRoot;fromRef='HEAD~1';toRef='HEAD';fidelity='high'})
        Save-B '137-return-type-diff' $answer
        Assert-B ($answer.content[0].text -match '~ method Get') 'Return-type-only change not reported as a changed member.'
    }
    Invoke-BCase '138' 'pipe-constructor-injection-published-as-edge' {
        foreach ($file in @('service.ts','probe.pipe.ts')) {$null=Publish-Audit $file}
        $answer=Query-Audit 'forward_edges' 'angular' 'Pipe' 'AuditPipe'
        Assert-B (@($answer.edges | Where-Object { $_.relation -eq 'Injects' -and $_.object.name -eq 'AuditApiService' }).Count -eq 1) 'Pipe constructor dependency missing from semantic edges.'
    }
    Invoke-BCase '138' 'class-inject-function-published-as-edge' {
        $null=Publish-Audit 'inject.component.ts'
        $answer=Query-Audit 'forward_edges' 'angular' 'Component' 'InjectProbe'
        Assert-B (@($answer.edges | Where-Object { $_.relation -eq 'Injects' -and $_.object.name -eq 'AuditApiService' }).Count -eq 1) 'Class inject() dependency missing from semantic edges.'
    }
    Invoke-BCase '139' 'nonexport-decorated-class-qualified-focus' {
        $answer=Get-BResult (Invoke-BTool $native 'provide_code_context' @{workspaceRoot=$root;filePath='owner.component.ts';fidelity='edit';focusMethods=@('OwnerProbe.read')})
        Assert-B (($answer | ConvertTo-Json -Depth 100 -Compress).Contains('OwnerProbe')) 'Decorated nonexport owner is not delivered to qualified focus.'
    }
    Invoke-BCase '139' 'service-find-is-declaration-only' {
        $answer=Query-Audit 'find_entities' 'angular' 'Service' 'AuditApiService'
        Assert-B ($answer.verifier_selected_count -eq 1 -and (Split-Path $answer.verifier_selected_entities[0].file -Leaf) -eq 'service.ts') "Service declaration selection returns $($answer.verifier_selected_count) declared Service occurrences instead of one."
    }
    Invoke-BCase '140' 'legacy-string-action-has-declaration' {
        foreach ($file in @('legacy.actions.ts','dispatch.component.ts')) {$null=Publish-Audit $file}
        $answer=Query-Audit 'find_entities' 'ngrx' 'Action' 'SET_AUDIT'
        Assert-B (@($answer.verifier_selected_entities | Where-Object { (Split-Path $_.file -Leaf) -eq 'legacy.actions.ts' }).Count -gt 0) 'Legacy action exists only as a dispatch-site reference, not a declaration in legacy.actions.ts.'
    }
} finally {
    foreach ($session in $sessions) {
        try {Stop-BSession $session} catch {$verdicts.Add(@{phase='cleanup';case=$session.Name;result='FAIL';evidence=$_.Exception.Message})}
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue acceptance evidence: $runRoot"
    Write-Host 'PASS means this specific controlled probe passed, not that its entire issue may be closed.'
}
# Expected findings remain visible as FAIL acceptance rows; no issue is changed.
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
