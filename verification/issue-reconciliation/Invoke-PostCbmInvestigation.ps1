param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$ProviderPath,
    [ValidateRange(30,600)][int]$CaseTimeoutSeconds=180,
    [ValidateSet('all','native','provider','controls')][string]$Scope='all'
)
# Investigation only. Existing stdio/session pattern; owned fixtures, no builds.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$ProviderPath=(Resolve-Path $ProviderPath).Path
$runRoot=Join-Path $repo ('target/post-cbm-investigation/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Save-B 'provenance' @{binary=$BinaryPath;binary_hash=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash;provider=$ProviderPath;provider_hash=(Get-FileHash $ProviderPath -Algorithm SHA256).Hash;platform=[Environment]::OSVersion.VersionString}
$counts=@(1,2,4,5,10,15,20,30,55,60)
function Query-I($Session,[string]$Root,[string]$Name,[string]$Scope='') {
    $args=@{workspaceRoot=$Root;type='reverse_edges';domain='builtin';entity_type='TypeRef';name=$Name}
    if ($Scope) {$args.withinPath=$Scope}
    (Get-BResult (Invoke-BTool $Session 'workspace_query' $args)).structuredContent
}
function Assert-IRecall($Answer,$Expected,[string]$Label) {
    $actual=@($Answer.edges | Where-Object relation -eq 'HasConstructorParameterType' | ForEach-Object {$_.subject.name} | Sort-Object -Unique)
    $difference=@(Compare-Object @($Expected) $actual)
    Save-B $Label @{expected=$Expected;observed=$actual;answer=$Answer;exact=($difference.Count -eq 0)}
    Assert-B ($difference.Count -eq 0) "$Label exact constructor identity mismatch ($($actual.Count)/$(@($Expected).Count))."
    Assert-B ($Answer.coverage.source_complete -eq $false -and $Answer.coverage.result_semantics -eq 'lower_bound') 'Coverage falsely claims complete source authority.'
}
try {
    foreach ($enabled in $(if ($Scope -eq 'controls') {@()} elseif ($Scope -eq 'provider') {@($true)} elseif ($Scope -eq 'native') {@($false)} else {@($false,$true)})) {
        foreach ($warm in @($false,$true)) {
            $mode=if ($enabled) {'provider'} else {'native'}
            $mode+='-'+$(if ($warm) {'warm'} else {'cold'})
            $root=Join-Path $runRoot $mode;New-Item -ItemType Directory -Force $root | Out-Null
            & git -C $root init --quiet;Assert-B ($LASTEXITCODE -eq 0) 'Fixture Git root creation failed.'
            Write-BText (Join-Path $root '.clean-ctx.json') (@{cbm=@{enabled=$enabled;binary_path=$ProviderPath}} | ConvertTo-Json -Depth 20)
            $oracle=@{};$files=[Collections.Generic.List[string]]::new()
            for ($t=0;$t -lt 10;$t++) {
                $type='IContract'+$t;$oracle[$type]=@()
                Write-BText (Join-Path $root "$type.cs") "namespace Probe133; public interface $type { int Read(); }"
                for ($i=0;$i -lt $counts[$t];$i++) {
                    $dir='group'+($i%5);New-Item -ItemType Directory -Force (Join-Path $root $dir) | Out-Null
                    $name='Consumer'+$t+'_'+$i.ToString('D3');$oracle[$type]+=$name
                    $file="$dir/$name.cs";$files.Add($file)
                    Write-BText (Join-Path $root $file) "namespace Probe133; public class $name { public $name($type value) {} }"
                }
            }
            # A matching stem/base-list occurrence must not pass as a constructor.
            Write-BText (Join-Path $root 'Contract9.cs') 'namespace Probe133; public class Contract9 : IContract9 { public int Read() => 0; }'
            Save-B "$mode-oracle" @{counts=$counts;identities=$oracle;files=$files.ToArray();scope='group0'}
            $deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
            $session=Start-BSession $mode $BinaryPath $root
            Invoke-BCase '133' "$mode-readiness" {
                if ($enabled) {
                    $status=Get-BResult (Invoke-BReadyTool $session 'get_cbm_status' @{})
                    Save-B "$mode-status" $status
                    Assert-B ($status._meta.cbm_status -eq 'available') 'Provider-enabled control is not available.'
                }
                $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
                if ($warm) {
                    foreach ($file in $files) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='edit'})}
                }
            }
            if ($verdicts[$verdicts.Count-1].result -ne 'PASS') {
                Stop-BSession $session;$null=$sessions.Remove($session)
                continue # Failed admission is never provider-enabled evidence.
            }
            # Large case first, with no unrelated file compilation in cold modes.
            foreach ($t in @(9,0,1,2,3,4,5,6,7,8)) {
                Invoke-BCase '133' "$mode-interface-$t" {
                    $type='IContract'+$t
                    Assert-IRecall (Query-I $session $root $type) $oracle[$type] "$mode-$type"
                }
            }
            Invoke-BCase '133' "$mode-withinPath" {
                $null=Get-BResult (Invoke-BTool $session 'refresh_workspace' @{workspaceRoot=$root})
                if ($warm) {foreach ($file in $files) {$null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='edit'})}}
                $expected=@($oracle['IContract9'] | Where-Object { [int]($_.Split('_')[1])%5 -eq 0 })
                Assert-IRecall (Query-I $session $root 'IContract9' 'group0') $expected "$mode-scoped"
            }
            Stop-BSession $session;$null=$sessions.Remove($session)
        }
    }
    $root=Join-Path $runRoot 'causal-controls';New-Item -ItemType Directory -Force $root | Out-Null
    & git -C $root init --quiet;Assert-B ($LASTEXITCODE -eq 0) 'Control Git root creation failed.'
    Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    $padding=(1..65 | ForEach-Object {"// padding $_"}) -join "`n"
    $sources=@{
        'direct.cs'='using Microsoft.AspNetCore.Mvc; [Route("direct")] public class Direct : ControllerBase { [HttpGet("x")] public int Get() { return 1; } }'
        'indirect.cs'='using Microsoft.AspNetCore.Mvc; [Route("indirect")] public class Indirect : Direct { [HttpGet("x")] public int Get() { return 1; } }'
        'service.ts'="import { Injectable } from '@angular/core';`n@Injectable()`nexport class SoloService { read() { return 1; } }"
        'constructor.component.ts'="import { Component } from '@angular/core';`n@Component({selector:'ctor',template:''})`nexport class ConstructorProbe { constructor(private api: SoloService) {} }"
        'inject.component.ts'="import { Component, inject } from '@angular/core';`n@Component({selector:'inj',template:''})`nexport class FunctionProbe { api = inject(SoloService); }"
        'pipe.ts'="import { Pipe } from '@angular/core';`n@Pipe({name:'p'})`nexport class PipeProbe { constructor(private api: SoloService) {} }"
        'owner.ts'="import { Component } from '@angular/core';`n@Component({selector:'own',template:''})`nclass Owner { read() { return 1; } }"
        'export-owner.ts'="import { Component } from '@angular/core';`n@Component({selector:'ex',template:''})`nexport class ExportOwner { read() { return 1; } }"
        'action.ts'="import { createAction } from '@ngrx/store';`nexport const modernAction = createAction('[Probe] Modern');"
    }
    foreach ($file in $sources.Keys) {Write-BText (Join-Path $root $file) ($padding+"`n"+$sources[$file])}
    $deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds);$session=Start-BSession 'controls' $BinaryPath $root
    foreach ($file in @('direct.cs','indirect.cs','service.ts','constructor.component.ts','inject.component.ts','pipe.ts','owner.ts','export-owner.ts','action.ts')) {
        Invoke-BCase 'controls' "capture-$file" {
            $result=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='edit'})
            Save-B "$file-context" $result
            $answer=Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='entities_in_file';file_path=$file})
            Save-B "$file-entities" $answer
        }
    }
    foreach ($identity in @(@('dotnet','Controller','Direct'),@('dotnet','Controller','Indirect'),@('angular','Component','ConstructorProbe'),@('angular','Component','FunctionProbe'),@('angular','Pipe','PipeProbe'))) {
        Invoke-BCase 'controls' ('edges-'+$identity[2]) {
            Save-B ('edges-'+$identity[2]) (Get-BResult (Invoke-BTool $session 'workspace_query' @{workspaceRoot=$root;type='forward_edges';domain=$identity[0];entity_type=$identity[1];name=$identity[2]}))
        }
    }
    foreach ($owner in @('Owner','ExportOwner')) {
        Invoke-BCase '139' ('focus-'+$owner) {
            $file=if ($owner -eq 'Owner') {'owner.ts'} else {'export-owner.ts'}
            Save-B ('focus-'+$owner) (Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$file;fidelity='edit';focusMethods=@("$owner.read")}))
        }
    }
} finally {
    foreach ($session in $sessions) {try {Stop-BSession $session} catch {$verdicts.Add(@{case='cleanup';result='FAIL';evidence=$_.Exception.Message})}}
    Save-B 'verdicts' $verdicts.ToArray();Write-Host "Investigation evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
