param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/framework-declaration-publication/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new()
$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'

function Service-Source([string]$Name) {
    return "import { Injectable } from '@angular/core';`n@Injectable()`nexport class $Name {}`n"
}
function Action-Source([string]$Name) {
    return "import { createAction } from '@ngrx/store';`nexport const $Name = createAction(`n '[Audit] $Name'`n);`n"
}
function Publication-Entities($Session,[string]$File,[string]$Domain,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$File})
    Save-B "$Label-result" $answer
    $kind=if ($Domain -eq 'angular') {'Service'} else {'Action'}
    return ,@($answer.structuredContent.entities | Where-Object {$_.domain -eq $Domain -and $_.entity_type -eq $kind})
}
function Assert-PublicationEntities($Found,[string[]]$Names,[string]$File) {
    $Found=@($Found | Where-Object {$null -ne $_})
    $expectedNames=@($Names | Where-Object {$null -ne $_})
    Assert-B ($Found.Count -eq $expectedNames.Count) "Expected $($expectedNames.Count) occurrences; got $($Found.Count)."
    foreach ($name in $expectedNames) {Assert-B (@($Found | Where-Object name -eq $name).Count -eq 1) "Exact occurrence absent/duplicated: $name"}
    foreach ($entity in $Found) {Assert-B ($entity.file -eq (Join-Path $root $File)) 'Occurrence provenance is not the exact authored source.'}
}
function Find-PublicationName($Session,[string]$Name,[string]$Domain,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='find_entities';name=$Name})
    Save-B "$Label-result" $answer
    return ,@($answer.structuredContent.entities | Where-Object {$_.domain -eq $Domain})
}

$cases=@(
    @{name='isolated-service';domain='angular';names=@('OwnerProbe');marker='T @svc = OwnerProbe';role='declaration';source="import { Injectable } from '@angular/core';`n@Injectable()`nclass OwnerProbe {}`n"},
    @{name='service-consumer';domain='angular';names=@('ConsumedService');marker='T @svc = ConsumedService';role='declaration_and_relationship';source="import { Injectable, Component } from '@angular/core';`n@Injectable() class ConsumedService {}`n@Component({selector:'x-di'}) class ConsumerDI { constructor(private a: ConsumedService, private b: ConsumedService) {} }`n"},
    @{name='two-services';domain='angular';names=@('FirstService','SecondService');marker='T @svc = FirstService';role='declarations';source="import { Injectable } from '@angular/core';`n@Injectable() export class FirstService {}`n@Injectable() export class SecondService {}`n"},
    @{name='service-a';domain='angular';names=@('SharedService');marker='T @svc = SharedService';role='declaration';source=(Service-Source 'SharedService')},
    @{name='service-b';domain='angular';names=@('SharedService');marker='T @svc = SharedService';role='declaration';source=(Service-Source 'SharedService')},
    @{name='isolated-action';domain='ngrx';names=@('setAudit');marker='T @action = setAudit';role='declaration';source="import { createAction } from '@ngrx/store';`nexport const setAudit = createAction(`n '[Audit] Set Audit'`n);`n"},
    @{name='action-dispatch';domain='ngrx';names=@('usedAudit');marker='T @action = usedAudit';role='declaration_and_relationship';source="import { Component } from '@angular/core';`nimport { createAction, Store } from '@ngrx/store';`nexport const usedAudit = createAction('[Audit] Used');`n@Component({selector:'x-store'}) class ConsumerStore { constructor(private store: Store) {} run() { this.store.dispatch(usedAudit()); this.store.dispatch(usedAudit()); } }`n"},
    @{name='two-actions';domain='ngrx';names=@('firstAudit','secondAudit');marker='T @action = firstAudit';role='declarations';source="import { createAction } from '@ngrx/store';`nexport const firstAudit = createAction('[Audit] First');`nexport const secondAudit = createAction('[Audit] Second');`n"},
    @{name='action-a';domain='ngrx';names=@('sharedAudit');marker='T @action = sharedAudit';role='declaration';source=(Action-Source 'sharedAudit')},
    @{name='action-b';domain='ngrx';names=@('sharedAudit');marker='T @action = sharedAudit';role='declaration';source=(Action-Source 'sharedAudit')},
    @{name='service-reference';domain='angular';names=@('SharedService');marker='';role='existing_reference_endpoint';source="import { Component } from '@angular/core';`nimport { SharedService } from './service-a';`n@Component({selector:'x-ref'}) class ReferenceDI { constructor(private owner: SharedService) {} }`n"},
    @{name='action-reference';domain='ngrx';names=@('sharedAudit');marker='';role='existing_reference_endpoint';source="import { Component } from '@angular/core';`nimport { Store } from '@ngrx/store';`nimport { sharedAudit } from './action-a';`n@Component({selector:'x-ref-store'}) class ReferenceStore { constructor(private store: Store) {} run() { this.store.dispatch(sharedAudit()); } }`n"},
    @{name='legacy-negative';domain='ngrx';names=@();marker='';role='unsupported_not_declared';source="import { Action } from '@ngrx/store';`nexport class LegacyAudit implements Action { readonly type = '[Audit] Legacy'; }`n"}
)
foreach ($case in $cases) {
    $case.file=$case.name+'.ts'
    $case.source+=("// unchanged economics padding`n"*100)
    Write-BText (Join-Path $root $case.file) $case.source
}
foreach ($domain in @('angular','ngrx')) {Write-BText (Join-Path $root "lifecycle-$domain.ts") 'export const initial = 0;'}
$retained=@(
    @{domain='angular';name='RetainedService';declaration=(Service-Source 'RetainedService');use="import { Component } from '@angular/core';`nimport { RetainedService } from './retained-angular-declaration';`n@Component({selector:'x-retained'}) class RetainedDI { constructor(private owner: RetainedService) {} }"},
    @{domain='ngrx';name='retainedAudit';declaration=(Action-Source 'retainedAudit');use="import { Component } from '@angular/core';`nimport { Store } from '@ngrx/store';`nimport { retainedAudit } from './retained-ngrx-declaration';`n@Component({selector:'x-retained-store'}) class RetainedStore { constructor(private store: Store) {} run() { this.store.dispatch(retainedAudit()); } }"}
)
foreach ($case in $retained) {
    Write-BText (Join-Path $root "retained-$($case.domain)-declaration.ts") $case.declaration
    Write-BText (Join-Path $root "retained-$($case.domain)-use.ts") $case.use
}
Save-B 'retained-source-manifest' $retained
Save-B 'source-manifest' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;public_occurrence_contract='domain/type/name/file; role and span are not public schema fields; roles in this source manifest are independent fixture oracles'}
try {
    $session=Start-BSession 'publication' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    foreach ($tool in @('provide_code_context','workspace_query','refresh_workspace')) {Assert-B ($tool -in $registry.tools.name) "Registered tool absent: $tool"}
    foreach ($case in $cases) {
        if ($case.marker) {
            Invoke-BCase 'publication' "$($case.name)-recognition" {
                $answer=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
                Save-B "$($case.name)-recognition" $answer
                Assert-B ($answer.content[0].text.Contains($case.marker)) 'Existing producer marker absent.'
                if ($case.name -eq 'isolated-service') {Assert-B ($answer.content[0].text.Contains("C OwnerProbe`n")) 'Frozen core owner absent.'}
            }
        }
        Invoke-BCase 'publication' "$($case.name)-entities" {
            $found=Publication-Entities $session $case.file $case.domain "$($case.name)-entities"
            Assert-PublicationEntities $found $case.names $case.file
        }
        Invoke-BCase 'publication' "$($case.name)-repeat-publication" {
            $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            $found=Publication-Entities $session $case.file $case.domain "$($case.name)-repeat"
            Assert-PublicationEntities $found $case.names $case.file
        }
    }
    foreach ($control in @(
        @{name='service-consumer';subject='ConsumerDI';domain='angular';relation='Injects';object='ConsumedService'},
        @{name='action-dispatch';subject='ConsumerStore';domain='ngrx';relation='Dispatches';object='usedAudit'},
        @{name='service-reference';subject='ReferenceDI';domain='angular';relation='Injects';object='SharedService'},
        @{name='action-reference';subject='ReferenceStore';domain='ngrx';relation='Dispatches';object='sharedAudit'}
    )) {
        Invoke-BCase 'publication' "$($control.name)-relationship" {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='angular';entity_type='Component';name=$control.subject})
            Save-B "$($control.name)-relationships" $answer
            $matching=@($answer.structuredContent.edges | Where-Object {$_.relation -eq $control.relation -and $_.object.name -eq $control.object -and $_.object.domain -eq $control.domain})
            Assert-B ($matching.Count -eq 1) 'Existing relationship lost or duplicated.'
            Assert-B (@($answer.structuredContent.edges | Where-Object relation -eq 'Defines').Count -eq 0) 'Registration carrier leaked into relationship graph.'
            Assert-B ($matching[0].subject.file -eq (Join-Path $root ($control.name+'.ts'))) 'Relationship asserting file is wrong.'
        }
    }
    foreach ($control in @(
        @{domain='angular';kind='Service';name='OwnerProbe'},
        @{domain='ngrx';kind='Action';name='setAudit'}
    )) {
        Invoke-BCase 'publication' "$($control.domain)-edge-free" {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain=$control.domain;entity_type=$control.kind;name=$control.name})
            Save-B "$($control.domain)-edge-free" $answer
            Assert-B ($answer.structuredContent.count -eq 0) 'Isolated declaration requires or created a relationship.'
        }
    }
    foreach ($domain in @('angular','ngrx')) {
        $targetEntityName=if ($domain -eq 'angular') {'SharedService'} else {'sharedAudit'}
        $prefix=if ($domain -eq 'angular') {'service'} else {'action'}
        Invoke-BCase 'publication' "$domain-source-distinct-with-reference" {
            $found=Find-PublicationName $session $targetEntityName $domain "$domain-source-distinct"
            Assert-B ($found.Count -eq 3) 'Expected two declaration-source occurrences plus one existing use-source endpoint occurrence.'
            foreach ($file in @("$prefix-a.ts","$prefix-b.ts","$prefix-reference.ts")) {
                Assert-B (@($found | Where-Object file -eq (Join-Path $root $file)).Count -eq 1) 'Source-distinct occurrence missing/collapsed/duplicated.'
            }
        }
    }
    foreach ($domain in @('angular','ngrx')) {
        $file="lifecycle-$domain.ts"
        foreach ($stage in @('add','rename','remove-declaration','re-add','delete-file')) {
            Invoke-BCase 'publication' "$domain-lifecycle-$stage" {
                $old="LifeOld_$domain";$new="LifeNew_$domain"
                $expectedName=if ($stage -eq 'add') {$old} elseif ($stage -in @('rename','re-add')) {$new} else {''}
                if ($stage -eq 'delete-file') {Remove-Item -LiteralPath (Join-Path $root $file)}
                else {
                    $source=if (-not $expectedName) {'export const noDeclaration = 1;'} elseif ($domain -eq 'angular') {Service-Source $expectedName} else {Action-Source $expectedName}
                    Write-BText (Join-Path $root $file) $source
                    Save-B "$domain-$stage-source" @{file=$file;source=$source}
                }
                $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
                Save-B "$domain-$stage-refresh" $refresh
                Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Registered refresh failed.'
                $found=Publication-Entities $session $file $domain "$domain-$stage"
                $expected=if ($expectedName) {@($expectedName)} else {@()}
                Assert-PublicationEntities $found $expected $file
                foreach ($removed in @($old,$new) | Where-Object {$_ -ne $expectedName}) {
                    $stale=Find-PublicationName $session $removed $domain "$domain-$stage-$removed"
                    Assert-B ($stale.Count -eq 0) 'Old declaration identity survived refresh/removal.'
                }
            }
        }
    }
    foreach ($case in $retained) {
        Invoke-BCase 'publication' "$($case.domain)-removal-keeps-reference" {
            $file="retained-$($case.domain)-declaration.ts"
            $use="retained-$($case.domain)-use.ts"
            $declaration=Publication-Entities $session $file $case.domain "$($case.domain)-retained-declaration"
            Assert-PublicationEntities $declaration @($case.name) $file
            $reference=Publication-Entities $session $use $case.domain "$($case.domain)-retained-use"
            Assert-PublicationEntities $reference @($case.name) $use
            Write-BText (Join-Path $root $file) 'export const noDeclaration = 1;'
            $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            $removed=Publication-Entities $session $file $case.domain "$($case.domain)-retained-removed"
            Assert-PublicationEntities $removed @() $file
            $surviving=Find-PublicationName $session $case.name $case.domain "$($case.domain)-retained-survivor"
            Assert-PublicationEntities $surviving @($case.name) $use
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Framework declaration evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
