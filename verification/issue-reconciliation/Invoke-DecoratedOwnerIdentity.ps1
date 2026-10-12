param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/decorated-owner-identity/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'fixture'
New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new()
$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
$forms=@(
    @{name='plain';head='class OwnerProbe'},
    @{name='exported-plain';head='export class OwnerProbe'},
    @{name='decorated';head="@Injectable()`nclass OwnerProbe"},
    @{name='exported-decorated';head="@Injectable()`nexport class OwnerProbe"},
    @{name='multiline-custom';head="@Audit({`n label: 'class Fake',`n nested: { name: 'Other' }`n})`nclass OwnerProbe"},
    @{name='multiple-custom';head="@First()`n@Second('Alpha')`nclass OwnerProbe"},
    @{name='same-line-custom';head="@Audit() class OwnerProbe"},
    @{name='generic';head="@Audit()`nclass OwnerProbe<T>"}
)
foreach ($case in $forms) {
    $case.file=$case.name+'.ts'
    $case.source="import { Injectable } from '@angular/core';`nclass Alpha { read() { return 'ALPHA_BODY'; } }`n$($case.head) { value: number = 1; read() { return 'OWNER_BODY'; } }`n"+("// unchanged economics padding`n"*100)
    Write-BText (Join-Path $root $case.file) $case.source
}
Save-B 'source-manifest' $forms
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false}
try {
    $session=Start-BSession 'owner' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    Assert-B ('provide_code_context' -in $registry.tools.name) 'Registered context tool absent.'
    foreach ($case in $forms) {
        foreach ($owner in @('OwnerProbe')) {
            Invoke-BCase '139' "$($case.name)-$owner" {
                $answer=Get-BResult (Invoke-BTool $session 'provide_code_context' @{
                    workspaceRoot=$root;filePath=$case.file;fidelity='edit';focusMethods=@("$owner.read")
                })
                Save-B "$($case.name)-$owner-result" $answer
                $included=if ($owner -eq 'OwnerProbe') {'OWNER_BODY'} else {'ALPHA_BODY'}
                $excluded=if ($owner -eq 'OwnerProbe') {'ALPHA_BODY'} else {'OWNER_BODY'}
                Assert-B ($answer._meta.content_kind -eq 'skeleton_with_focused_verbatim_bodies') 'Focused content contract absent.'
                Assert-B ($answer.content[0].text.Contains($included)) 'Intended owner body absent.'
                Assert-B (-not $answer.content[0].text.Contains($excluded)) 'Wrong owner body delivered.'
                Assert-B ($answer.content[0].text.Contains('OwnerProbe')) 'Declaration owner absent.'
                Assert-B ([IO.File]::ReadAllText((Join-Path $root $case.file)) -ceq $case.source) 'Read-only focus changed source.'
            }
        }
        Invoke-BCase '139' "$($case.name)-entities" {
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{
                workspaceRoot=$root;type='entities_in_file';file_path=$case.file
            })
            Save-B "$($case.name)-entities-result" $answer
            $classes=@($answer.structuredContent.entities | Where-Object {
                $_.domain -eq 'builtin' -and $_.entity_type -eq 'Class'
            })
            Assert-B (@($classes | Where-Object name -eq 'OwnerProbe').Count -eq 1) 'Exact builtin class occurrence absent.'
            Assert-B (@($classes | Where-Object name -eq 'Alpha').Count -eq 1) 'Other class occurrence lost.'
            foreach ($entity in $classes) {
                Assert-B ($entity.file -eq (Join-Path $root $case.file)) 'Entity provenance is not the exact fixture.'
            }
            if ($case.name -in @('decorated','exported-decorated')) {
                $services=@($answer.structuredContent.entities | Where-Object {
                    $_.domain -eq 'angular' -and $_.entity_type -eq 'Service' -and $_.name -eq 'OwnerProbe'
                })
                Assert-B ($services.Count -eq 0) 'Framework publication changed outside the owner repair.'
            }
        }
        Invoke-BCase '139' "$($case.name)-qualified-edit" {
            # Prepare all bodies using the existing Edit contract before a
            # controlled write. The oracle is the full independently authored
            # source, including the same-named member in the other class.
            $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{
                workspaceRoot=$root;filePath=$case.file;fidelity='edit'
            })
            $answer=Get-BResult (Invoke-BTool $session 'apply_edit' @{
                workspaceRoot=$root;filePath=$case.file;operations=@(@{
                    type='replace_body';target='OwnerProbe.read'
                    expectedOldText="{ return 'OWNER_BODY'; }"
                    newText="{ return 'OWNER_CHANGED'; }"
                })
            })
            Save-B "$($case.name)-edit-result" $answer
            $expected=$case.source.Replace("{ return 'OWNER_BODY'; }","{ return 'OWNER_CHANGED'; }")
            Assert-B ([IO.File]::ReadAllText((Join-Path $root $case.file)) -ceq $expected) 'Edit did not change exactly the intended owner body.'
        }
        Invoke-BCase '139' "$($case.name)-fresh-focus" {
            $answer=Get-BResult (Invoke-BTool $session 'provide_code_context' @{
                workspaceRoot=$root;filePath=$case.file;fidelity='edit';focusMethods=@('OwnerProbe.read')
            })
            Save-B "$($case.name)-fresh-result" $answer
            Assert-B ($answer.content[0].text.Contains('OWNER_CHANGED')) 'Updated owner body absent after controlled edit.'
            Assert-B (-not $answer.content[0].text.Contains('OWNER_BODY')) 'Stale owner body delivered.'
            Assert-B (-not $answer.content[0].text.Contains('ALPHA_BODY')) 'Other owner body delivered after edit.'
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray()
    Write-Host "Decorated owner evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
