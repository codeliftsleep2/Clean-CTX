param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/native-causal-controls/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
$root=Join-Path $runRoot 'fixture';New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet;Assert-B ($LASTEXITCODE -eq 0) 'Fixture Git root creation failed.'
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
$padding=(1..65 | ForEach-Object {"// padding $_"}) -join "`n"
Write-BText (Join-Path $root 'Contract.cs') 'public interface IPartial {} public class GoodConsumer { public GoodConsumer(IPartial x) {} }'
# A source-read failure is real but deliberately confined to an owned fixture.
[IO.File]::WriteAllBytes((Join-Path $root 'UnreadableEncoding.cs'), [byte[]](255,254,255))
$registrations=(0..39 | ForEach-Object {"services.AddScoped<IService$_, Service$_>();"}) -join "`n"
Write-BText (Join-Path $root 'Registrations.cs') ($padding+"`nusing Microsoft.Extensions.DependencyInjection; public class RegistrationProbe { public void Register() {`n$registrations`n} }")
Write-BText (Join-Path $root 'Edit.cs') ($padding+"`npublic class EditProbe { public int First() { return 1; } }")
Write-BText (Join-Path $root 'GhostTests.cs') ($padding+"`nusing Microsoft.VisualStudio.TestTools.UnitTesting; [TestClass] public class GhostTests { [TestMethod] public void Verify() {} }")
Save-B 'provenance' @{binary=$BinaryPath;hash=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;registrations=40;broken_read='UnreadableEncoding.cs'}
try {
    $session=Start-BSession 'causal' $BinaryPath $root
    Invoke-BCase '133' 'read-failure-partial-disclosure' {
        $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
        $answer=(Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='reverse_edges';domain='builtin';entity_type='TypeRef';name='IPartial'})).structuredContent
        Save-B 'partial-disclosure' $answer
        Assert-B ($answer.coverage.source_complete -eq $false -and $answer.coverage.result_semantics -eq 'lower_bound' -and $answer.discovery.status -eq 'partial') 'Read failure is not disclosed as partial.'
    }
    Invoke-BCase '135' 'forty-registration-visible-truth' {
        $answer=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='Registrations.cs';fidelity='high'})
        Save-B 'di40' $answer
        $text=$answer.content[0].text
        Assert-B ($text.Contains('IService39') -or ($text -match '40' -and $text -match '(?i)omit|truncat|shown')) 'Forty DI facts are not delivered or disclosed.'
    }
    Invoke-BCase '136' 'parseable-comment-must-precede-declaration' {
        $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='Edit.cs';fidelity='edit'})
        $receipt=Get-BResult (Invoke-BTool $session 'apply_edit' @{workspaceRoot=$root;filePath='Edit.cs';operations=@(@{type='insert_before';anchor='EditProbe.First';unitText='/*INSERTION_PROBE*/'})})
        $text=[IO.File]::ReadAllText((Join-Path $root 'Edit.cs'))
        Save-B 'parseable-insert' @{receipt=$receipt;source=$text;marker=$text.IndexOf('/*INSERTION_PROBE*/');declaration=$text.IndexOf('public int First')}
        Assert-B ($text.IndexOf('/*INSERTION_PROBE*/') -lt $text.IndexOf('public int First')) 'Syntax gate accepted body-offset insertion after the declaration header.'
    }
    Invoke-BCase '134' 'test-target-must-exist' {
        $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='GhostTests.cs';fidelity='edit'})
        $answer=(Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain='dotnet';entity_type='TestClass';name='GhostTests'})).structuredContent
        Save-B 'ghost-test-target' $answer
        Assert-B (@($answer.edges | Where-Object { $_.relation -eq 'Tests' -and $_.object.name -eq 'Ghost' }).Count -eq 0) 'Tests edge targets a non-existent guessed type.'
    }
    Invoke-BCase '137' 'diff-signature-attribute-body-controls' {
        $diffRoot=Join-Path $root 'history';New-Item -ItemType Directory -Force $diffRoot | Out-Null
        & git -C $diffRoot init --quiet
        & git -C $diffRoot config user.name 'Causal fixture'
        & git -C $diffRoot config user.email 'fixture@example.invalid'
        $hooks=Join-Path $diffRoot '.hooks';New-Item -ItemType Directory -Force $hooks | Out-Null
        & git -C $diffRoot config core.hooksPath $hooks
        $before=@{
            'Return.cs'='public class ReturnProbe { public int Get() { return 1; } }'
            'Attribute.cs'='public class AttributeProbe { [Authorize("old")] public int Get() { return 1; } }'
            'Body.cs'='public class BodyProbe { public int First() { return 1; } public int Last() { return 2; } }'
            'Param.cs'='public class ParamProbe { public int Get(int x) { return 1; } }'
        }
        foreach ($file in $before.Keys) {Write-BText (Join-Path $diffRoot $file) $before[$file]}
        & git -C $diffRoot add -- '*.cs'; & git -C $diffRoot commit --quiet -m 'before'
        Assert-B ($LASTEXITCODE -eq 0) 'Before commit failed.'
        Write-BText (Join-Path $diffRoot 'Return.cs') ($before['Return.cs'].Replace('int Get','long Get'))
        Write-BText (Join-Path $diffRoot 'Attribute.cs') ($before['Attribute.cs'].Replace('old','new'))
        Write-BText (Join-Path $diffRoot 'Body.cs') ($before['Body.cs'].Replace('return 2','return 3'))
        Write-BText (Join-Path $diffRoot 'Param.cs') ($before['Param.cs'].Replace('int x','string x'))
        & git -C $diffRoot add -- '*.cs'; & git -C $diffRoot commit --quiet -m 'after'
        Assert-B ($LASTEXITCODE -eq 0) 'After commit failed.'
        Save-B 'diff-controls' (Get-BResult (Invoke-BTool $session 'diff_commits' @{workspaceRoot=$diffRoot;fromRef='HEAD~1';toRef='HEAD';fidelity='high'}))
    }
    Invoke-BCase '141' 'same-content-filename-and-nested-condition' {
        $template="@if (items.length > 3 && !loading) { <outer-row>Visible {{ title | uppercase }} @if (selected) { <inner-row i18n>Inner</inner-row> } </outer-row> }`n@if (empty) { <empty-row>Empty</empty-row> }"
        Write-BText (Join-Path $root 'same.component.html') $template
        Write-BText (Join-Path $root 'arbitrary-view.html') $template
        Write-BText (Join-Path $root 'template-owner.ts') "import { Component } from '@angular/core';`n@Component({selector:'template-control',templateUrl:'./arbitrary-view.html'})`nexport class TemplateControl {}"
        foreach ($fidelity in @('medium','high')) {
            $named=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='same.component.html';fidelity=$fidelity})
            $owned=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath='arbitrary-view.html';fidelity=$fidelity})
            Save-B ("141-$fidelity") @{source=$template;named=$named;owned=$owned}
            Assert-B ($named.content[0].text -ceq $owned.content[0].text) 'Identical template content differs by ownership filename.'
            $text=$owned.content[0].text
            foreach ($token in @('items.length > 3 && !loading','selected','empty','Visible','{{ title | uppercase }}','i18n','Inner','Empty')) {
                Assert-B ($text.Contains($token)) "Nested template lost $token at $fidelity."
            }
            Assert-B ($text.IndexOf('<outer-row') -lt $text.IndexOf('<inner-row') -and $text.IndexOf('<inner-row') -lt $text.IndexOf('</outer-row>') -and $text.IndexOf('</outer-row>') -lt $text.IndexOf('<empty-row')) 'Nested/sequential element ownership changed.'
        }
    }
    Invoke-BCase '141' 'linked-html-entities-in-file' {
        $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path='arbitrary-view.html'})
        Save-B '141-linked-entities' $answer
        Assert-B (@($answer.structuredContent.entities | Where-Object { $_.domain -eq 'angular' -and $_.entity_type -eq 'Template' }).Count -gt 0) 'Linked HTML does not deliver a Template entity through entities_in_file.'
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray();Write-Host "Causal evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
