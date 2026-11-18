param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/angular-pipe-di/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'

function Pipe-Edges($Session,[string]$Consumer,[string]$Kind,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='angular';entity_type=$Kind;name=$Consumer})
    Save-B "$Label-result" $answer
    return ,@($answer.structuredContent.edges | Where-Object relation -eq 'Injects')
}
function Assert-PipeEdges($Found,[string[]]$Expected,[string]$Consumer,[string]$Kind,[string]$File) {
    $Found=@($Found | Where-Object {$null -ne $_});$Expected=@($Expected | Where-Object {$null -ne $_})
    Assert-B ($Found.Count -eq $Expected.Count) "Exact Injects count $($Found.Count)/$($Expected.Count)."
    foreach ($target in $Expected) {Assert-B (@($Found | Where-Object {$_.object.name -eq $target}).Count -eq 1) "Dependency absent/duplicated: $target"}
    foreach ($edge in $Found) {
        Assert-B ($edge.subject.domain -eq 'angular' -and $edge.subject.entity_type -eq $Kind -and $edge.subject.name -eq $Consumer) 'Incorrect consumer identity.'
        Assert-B ($edge.object.domain -eq 'angular' -and $edge.object.entity_type -eq 'Service') 'Incorrect dependency identity kind.'
        Assert-B ($edge.subject.file -eq (Join-Path $root $File) -and $edge.object.file -eq (Join-Path $root $File)) 'Incorrect asserting-source provenance; no resolved target-file claim.'
    }
}

$cases=@(
    @{id='service';kind='Service';decorator='@Injectable()';ctor='private completelyDifferentName: AuditService';fields='';expected=@('AuditService')},
    @{id='component';kind='Component';decorator="@Component({selector:'x-audit'})";ctor='private completelyDifferentName: AuditService';fields='';expected=@('AuditService')},
    @{id='directive';kind='Directive';decorator="@Directive({selector:'[audit]'})";ctor='private completelyDifferentName: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-private';kind='Pipe';decorator="@Pipe({name:'audit'})";ctor='private readonly completelyDifferentName: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-two';kind='Pipe';decorator="@Pipe({name:'two'})";ctor='audit: AuditService, private completelyDifferentName: LoggerService';fields='';expected=@('AuditService','LoggerService')},
    @{id='pipe-repeat';kind='Pipe';decorator="@Pipe({name:'repeat'})";ctor='private first: AuditService, private second: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-bare';kind='Pipe';decorator="@Pipe({name:'bare'})";ctor='audit: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-public';kind='Pipe';decorator="@Pipe({name:'public'})";ctor='public audit: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-readonly';kind='Pipe';decorator="@Pipe({name:'readonly'})";ctor='readonly audit: AuditService';fields='';expected=@('AuditService')},
    @{id='pipe-multiline';kind='Pipe';decorator="@Pipe({`n name:'multiline'`n})";ctor="private`n readonly different:`n AuditService";fields='';expected=@('AuditService')},
    @{id='pipe-empty';kind='Pipe';decorator="@Pipe({name:'empty'})";ctor='';fields='';expected=@()},
    @{id='pipe-no-constructor';kind='Pipe';decorator="@Pipe({name:'none'})";ctor=$null;fields='';expected=@()},
    @{id='pipe-functional-only';kind='Pipe';decorator="@Pipe({name:'functional'})";ctor='';fields='private receiver = inject(LoggerService);';expected=@()},
    @{id='pipe-mixed';kind='Pipe';decorator="@Pipe({name:'mixed'})";ctor='private audit: AuditService';fields='private unrelatedReceiver = inject(LoggerService);';expected=@('AuditService')},
    @{id='module-negative';kind='Module';decorator='@NgModule({})';ctor='private audit: AuditService';fields='';expected=@()},
    @{id='ordinary-negative';kind='Class';decorator='';ctor='private audit: AuditService';fields='';expected=@()},
    @{id='pipe-unresolved';kind='Pipe';decorator="@Pipe({name:'unresolved'})";ctor='private receiver: MissingService';fields='';expected=@('MissingService')}
)
foreach ($case in $cases) {
    $case.consumer='Consumer_'+$case.id.Replace('-','_');$case.file=$case.id+'.ts'
    $constructor=if ($null -eq $case.ctor) {''} else {"constructor($($case.ctor)) {}"}
    $case.source="import { Injectable, Component, Directive, Pipe, NgModule, inject } from '@angular/core';`n@Injectable() export class AuditService {}`n@Injectable() export class LoggerService {}`n$($case.decorator)`nexport class $($case.consumer) { $($case.fields) $constructor transform(value: string): string { return value; } }`n"+("// unchanged economics padding`n"*100)
    Write-BText (Join-Path $root $case.file) $case.source
}
Write-BText (Join-Path $root 'lifecycle.ts') 'export const initial = 0;'
Write-BText (Join-Path $root 'a.ts') "import { Injectable } from '@angular/core';`n@Injectable() export class CollisionService {}"
Write-BText (Join-Path $root 'b.ts') "import { Injectable } from '@angular/core';`n@Injectable() export class CollisionService {}"
Write-BText (Join-Path $root 'collision.ts') "import { Pipe } from '@angular/core';`nimport { CollisionService } from './a';`n@Pipe({name:'collision'}) export class CollisionPipe {constructor(private different: CollisionService) {}}"
$deferred=@(
    @{id='spaced';member='completelyDifferentName';token='AuditService';expression='private completelyDifferentName = inject(AuditService);'},
    @{id='compact';member='completelyDifferentName';token='AuditService';expression='private completelyDifferentName=inject(AuditService);'},
    @{id='optional';member='completelyDifferentName';token='AuditService';expression='private completelyDifferentName = inject?.(AuditService);'},
    @{id='namespace';member='receiver';token='SomeNamespace.Token';expression='private receiver = inject(SomeNamespace.Token);'}
)
foreach ($observation in $deferred) {
    Write-BText (Join-Path $root ('deferred-'+$observation.id+'.ts')) "import { Injectable, inject } from '@angular/core';`n@Injectable() export class Deferred_$($observation.id) { $($observation.expression) }"
}
Save-B 'source-manifest' $cases
Save-B 'deferred-manifest' $deferred
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;scope='Pipe constructor admission only; functional DI observations are NOT acceptance'}
try {
    $session=Start-BSession 'pipe-di' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    foreach ($tool in @('provide_code_context','workspace_query','refresh_workspace')) {Assert-B ($tool -in $registry.tools.name) "Registered tool absent: $tool"}
    foreach ($case in $cases) {
        Invoke-BCase '138-pipe' "$($case.id)-declarations-evidence" {
            $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            Save-B "$($case.id)-context" $context
            Assert-B ($context.content[0].text.Contains("C $($case.consumer)")) 'Canonical consumer owner missing.'
            if ($case.expected.Count) {Assert-B ($context.content[0].text.Contains('T @injects')) 'Existing constructor evidence marker absent.'}
            $entities=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$case.file})
            Save-B "$($case.id)-entities" $entities
            foreach ($dependency in @('AuditService','LoggerService')) {
                Assert-B (@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -eq $dependency -and $_.file -eq (Join-Path $root $case.file)}).Count -eq 1) 'Independent dependency declaration absent/duplicated.'
            }
            if ($case.kind -in @('Service','Component','Directive','Pipe')) {
                Assert-B (@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq $case.kind -and $_.name -eq $case.consumer}).Count -eq 1) 'Consumer occurrence/recognition missing.'
            }
        }
        Invoke-BCase '138-pipe' "$($case.id)-exact-injects" {
            $edges=Pipe-Edges $session $case.consumer $case.kind "$($case.id)-edges"
            Assert-PipeEdges $edges $case.expected $case.consumer $case.kind $case.file
        }
        Invoke-BCase '138-pipe' "$($case.id)-repeat-dedup" {
            $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            $edges=Pipe-Edges $session $case.consumer $case.kind "$($case.id)-repeat"
            Assert-PipeEdges $edges $case.expected $case.consumer $case.kind $case.file
            Assert-B ([IO.File]::ReadAllText((Join-Path $root $case.file)) -ceq $case.source) 'Verification changed authored source.'
        }
    }
    Invoke-BCase '138-pipe' 'source-distinct-symbolic-limitation' {
        foreach ($file in @('a.ts','b.ts')) {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$file})
            Save-B "$file-collision-entities" $answer
            Assert-B (@($answer.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -eq 'CollisionService'}).Count -eq 1) 'Source occurrence missing.'
        }
        $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path='collision.ts'})
        Save-B 'collision-consumer-entities' $answer
        Assert-B (@($answer.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Pipe' -and $_.name -eq 'CollisionPipe'}).Count -eq 1) 'Pipe owner/recognition missing.'
        $edges=Pipe-Edges $session 'CollisionPipe' 'Pipe' 'collision-edges'
        Assert-PipeEdges $edges @('CollisionService') 'CollisionPipe' 'Pipe' 'collision.ts'
        Save-B 'collision-limitation' @{target='CollisionService';meaning='written symbolic constructor dependency; object.file is asserting occurrence, not resolved imported declaration';resolved_target_file='NOT ESTABLISHED'}
    }
    foreach ($stage in @('AuditService','LoggerService','none','delete-file')) {
        Invoke-BCase '138-pipe' "lifecycle-$stage" {
            if ($stage -eq 'delete-file') {Remove-Item -LiteralPath (Join-Path $root 'lifecycle.ts')}
            else {
                $parameters=if ($stage -eq 'none') {''} else {"private different: $stage"}
                $source="import { Pipe, Injectable } from '@angular/core';`n@Injectable() export class AuditService {}`n@Injectable() export class LoggerService {}`n@Pipe({name:'lifecycle'}) export class LifecyclePipe {constructor($parameters) {}}"
                Write-BText (Join-Path $root 'lifecycle.ts') $source
                Save-B "lifecycle-$stage-source" @{source=$source}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Registered refresh failed.'
            $null=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path='lifecycle.ts'})
            $edges=Pipe-Edges $session 'LifecyclePipe' 'Pipe' "lifecycle-$stage"
            $expected=if ($stage -in @('none','delete-file')) {@()} else {@($stage)}
            Assert-PipeEdges $edges $expected 'LifecyclePipe' 'Pipe' 'lifecycle.ts'
        }
    }
    # Deferred functional observations have no PASS/FAIL target oracle here.
    # Capture actual behavior without blessing or fixing the known defect.
    foreach ($observation in $deferred) {
        $script:deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
        $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=('deferred-'+$observation.id+'.ts');fidelity='high'})
        $edges=Pipe-Edges $session ('Deferred_'+$observation.id) 'Service' ('deferred-'+$observation.id)
        Save-B ('deferred-'+$observation.id+'-observation') @{member=$observation.member;token=$observation.token;observed_targets=@($edges | ForEach-Object {$_.object.name});acceptance='NOT REPAIRED; separate structured token authority slice'}
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Pipe DI evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
