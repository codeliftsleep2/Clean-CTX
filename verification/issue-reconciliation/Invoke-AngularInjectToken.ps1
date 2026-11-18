param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/angular-inject-token/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
function Inject-Source([string]$Consumer,[string]$Fields,[string]$Decorator) {
    return "import { Injectable, Component, Directive, Pipe, inject } from '@angular/core';`n@Injectable()`nexport class AuditService {}`n@Injectable()`nexport class LoggerService {}`n$Decorator`nexport class $Consumer {`n$Fields`n}`n"+('// unchanged economics padding'+"`n")*100
}
function Inject-Edges($Session,[string]$Consumer,[string]$Kind,[string]$Label) {
    $answer=Get-BResult (Invoke-BTool $Session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='angular';entity_type=$Kind;name=$Consumer})
    Save-B "$Label-edges" $answer
    return ,@($answer.structuredContent.edges | Where-Object relation -eq 'Injects')
}
function Assert-InjectEdges($Found,[string[]]$Expected,[string]$Consumer,[string]$Kind,[string]$File) {
    $Found=@($Found | Where-Object {$null -ne $_});$Expected=@($Expected | Where-Object {$null -ne $_})
    Assert-B ($Found.Count -eq $Expected.Count) "Exact Injects count $($Found.Count)/$($Expected.Count)."
    foreach ($target in $Expected) {Assert-B (@($Found | Where-Object {$_.object.name -eq $target}).Count -eq 1) "Token missing/duplicated: $target"}
    foreach ($edge in $Found) {
        Assert-B ($edge.subject.domain -eq 'angular' -and $edge.subject.entity_type -eq $Kind -and $edge.subject.name -eq $Consumer) 'Wrong consumer.'
        Assert-B ($edge.object.domain -eq 'angular' -and $edge.object.entity_type -eq 'Service') 'Wrong symbolic dependency kind.'
        Assert-B ($edge.subject.file -eq (Join-Path $root $File) -and $edge.object.file -eq (Join-Path $root $File)) 'Wrong asserting-source provenance; no resolved target-file claim.'
    }
}
function Assert-InjectMarkers([string]$Text,[string[]]$Expected) {
    $tokens=@($Text.Split("`n") | Where-Object { $_.StartsWith('T @injects = [') } | ForEach-Object {
        $_.Substring(14).TrimEnd(']',"`r").Split(',')
    } | Where-Object {$_})
    $Expected=@($Expected | Where-Object {$null -ne $_})
    Assert-B ($tokens.Count -eq $Expected.Count) "Exact marker token count $($tokens.Count)/$($Expected.Count)."
    foreach ($target in $Expected) {Assert-B (@($tokens | Where-Object {$_ -ceq $target}).Count -eq 1) "Marker token missing/duplicated: $target"}
}
$cases=@(
    @{id='primary';fields='private readonly completelyDifferentName = inject(AuditService);';expected=@('AuditService')},
    @{id='matching';fields='private auditService = inject(AuditService);';expected=@('AuditService')},
    @{id='compact';fields='private completelyDifferentName=inject(AuditService);';expected=@('AuditService')},
    @{id='spacing';fields='private unrelated = inject( AuditService );';expected=@('AuditService')},
    @{id='multiline';fields="private unrelated = inject(`n AuditService`n);";expected=@('AuditService')},
    @{id='public';fields='public unrelated = inject(AuditService);';expected=@('AuditService')},
    @{id='protected';fields='protected unrelated = inject(AuditService);';expected=@('AuditService')},
    @{id='multiple';fields='private strangeOne = inject(AuditService); private unrelatedTwo = inject(LoggerService);';expected=@('AuditService','LoggerService')},
    @{id='same-token';fields='private first = inject(AuditService); private second = inject(AuditService);';expected=@('AuditService')},
    @{id='non-inject';fields='private unrelated = makeService(AuditService);';expected=@()},
    @{id='conditional';fields='private unrelated = condition ? inject(AuditService) : otherValue;';expected=@()},
    @{id='wrapper';fields='private unrelated = wrapper(inject(AuditService));';expected=@()},
    @{id='member-call';fields='private unrelated = obj.inject(AuditService);';expected=@()},
    @{id='parenthesized';fields='private unrelated = (inject(AuditService));';expected=@()},
    @{id='optional';fields='private unrelated = inject?.(AuditService);';expected=@()},
    @{id='qualified';fields='private unrelated = inject(namespace.AuditService);';expected=@()},
    @{id='missing';fields='private unrelated = inject(MissingService);';expected=@('MissingService')},
    @{id='symbol';fields='private unrelated = inject(MY_TOKEN);';expected=@('MY_TOKEN')},
    @{id='options';fields='private unrelated = inject(AuditService, { optional: true });';expected=@('AuditService')},
    @{id='component';decorator="@Component({selector:'x-audit'})";kind='Component';fields='private unrelated = inject(LoggerService); constructor(private different: AuditService) {}';expected=@('AuditService','LoggerService')},
    @{id='directive';decorator="@Directive({selector:'[audit]'})";kind='Directive';fields='private unrelated = inject(LoggerService); constructor(private different: AuditService) {}';expected=@('AuditService','LoggerService')},
    @{id='pipe';decorator="@Pipe({name:'audit'})";kind='Pipe';fields='private unrelated = inject(LoggerService); constructor(private different: AuditService) {}';expected=@('AuditService')},
    @{id='constructor';fields='constructor(private completelyDifferentName: AuditService) {}';expected=@('AuditService')},
    @{id='arrow-descendant';fields='private unrelated = () => inject(AuditService);';expected=@()},
    @{id='method-call';fields='getValue() { return inject(AuditService); }';expected=@()}
)
foreach ($case in $cases) {
    if (-not $case.ContainsKey('kind')) {$case.kind='Service';$case.decorator='@Injectable()'}
    $case.consumer=if ($case.id -eq 'primary') {'AuditConsumer'} else {'Consumer_'+$case.id.Replace('-','_')}
    $case.file=$case.id+'.ts';$case.source=Inject-Source $case.consumer $case.fields $case.decorator
    Write-BText (Join-Path $root $case.file) $case.source
}
Write-BText (Join-Path $root 'lifecycle.ts') (Inject-Source 'LifecycleConsumer' 'private dependency = inject(AuditService);' '@Injectable()')
foreach ($file in @('a.ts','b.ts')) {Write-BText (Join-Path $root $file) "import { Injectable } from '@angular/core';`n@Injectable() export class CollisionService {}"}
Write-BText (Join-Path $root 'collision.ts') "import { Injectable, inject } from '@angular/core';`nimport { CollisionService } from './a';`n@Injectable() export class CollisionConsumer { private unrelated = inject(CollisionService); }"
Save-B 'source-oracles' $cases
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;scope='bounded #138b native MCP acceptance; symbolic token identities, not runtime or import resolution'}
try {
    $session=Start-BSession 'inject-token' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    Save-B 'registered-tools' $registry
    foreach ($tool in @('provide_code_context','workspace_query','refresh_workspace')) {Assert-B ($tool -in $registry.tools.name) "Tool absent: $tool"}
    foreach ($case in $cases) {
        Invoke-BCase '138b' "$($case.id)-declarations" {
            $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            Save-B "$($case.id)-context" $context
            Assert-B ($context.content[0].text.Contains("C $($case.consumer)")) 'Canonical owner missing.'
            $entities=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$case.file})
            Save-B "$($case.id)-entities" $entities
            foreach ($declarationName in @('AuditService','LoggerService')) {
                Assert-B (@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -eq $declarationName -and $_.file -eq (Join-Path $root $case.file)}).Count -eq 1) 'Independent token declaration absent/duplicated.'
            }
            Assert-B (@($entities.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq $case.kind -and $_.name -eq $case.consumer}).Count -eq 1) 'Consumer declaration/recognition absent.'
        }
        Invoke-BCase '138b' "$($case.id)-marker-tokens" {
            $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            Assert-InjectMarkers $context.content[0].text $case.expected
        }
        Invoke-BCase '138b' "$($case.id)-exact-edges" {
            Assert-InjectEdges (Inject-Edges $session $case.consumer $case.kind $case.id) $case.expected $case.consumer $case.kind $case.file
        }
        Invoke-BCase '138b' "$($case.id)-repeat-dedup" {
            $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity='high'})
            Assert-InjectEdges (Inject-Edges $session $case.consumer $case.kind ($case.id+'-repeat')) $case.expected $case.consumer $case.kind $case.file
            Assert-B ([IO.File]::ReadAllText((Join-Path $root $case.file)) -ceq $case.source) 'Authored source changed.'
        }
        Invoke-BCase '138b' "$($case.id)-fidelity-semantics" {
            foreach ($fidelity in @('medium','low')) {
                $context=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity=$fidelity})
                Save-B "$($case.id)-$fidelity-context" $context
                Assert-B (-not $context.content[0].text.Contains('T @injects')) 'DI marker lost its High-only policy.'
                Assert-InjectEdges (Inject-Edges $session $case.consumer $case.kind ($case.id+'-'+$fidelity)) $case.expected $case.consumer $case.kind $case.file
            }
        }
    }
    Invoke-BCase '138b' 'source-distinct-declarations' {
        foreach ($file in @('a.ts','b.ts')) {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$file})
            Save-B "$file-collision-entities" $answer
            Assert-B (@($answer.structuredContent.entities | Where-Object {$_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -eq 'CollisionService'}).Count -eq 1) 'Distinct source declaration occurrence absent.'
        }
    }
    Invoke-BCase '138b' 'cross-file-symbolic-token-limitation' {
        $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='collision.ts';fidelity='high'})
        Assert-InjectEdges (Inject-Edges $session 'CollisionConsumer' 'Service' 'collision') @('CollisionService') 'CollisionConsumer' 'Service' 'collision.ts'
        Save-B 'collision-limitation' @{meaning='symbolic written token; object.file is asserting occurrence, not resolved imported declaration';resolved_target_file='NOT ESTABLISHED'}
    }
    foreach ($stage in @('AuditService','LoggerService','none','delete-file')) {
        Invoke-BCase '138b' "lifecycle-$stage" {
            if ($stage -eq 'delete-file') {Remove-Item -LiteralPath (Join-Path $root 'lifecycle.ts')}
            else {
                $fields=if ($stage -eq 'none') {'private dependency = makeService(AuditService);'} else {"private dependency = inject($stage);"}
                $source=Inject-Source 'LifecycleConsumer' $fields '@Injectable()'
                Write-BText (Join-Path $root 'lifecycle.ts') $source
                Save-B "lifecycle-$stage-source" @{source=$source}
            }
            $refresh=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
            Assert-B ($refresh.structuredContent.refreshed -eq $true) 'Refresh failed.'
            $null=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path='lifecycle.ts'})
            $expected=if ($stage -in @('none','delete-file')) {@()} else {@($stage)}
            Assert-InjectEdges (Inject-Edges $session 'LifecycleConsumer' 'Service' "lifecycle-$stage") $expected 'LifecycleConsumer' 'Service' 'lifecycle.ts'
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Inject token evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
