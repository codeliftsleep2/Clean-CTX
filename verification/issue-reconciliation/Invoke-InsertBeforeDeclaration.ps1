param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
# Permanent public MCP exact-source regression, using the existing session pattern.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/insert-before-declaration/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$root=Join-Path $runRoot 'fixture';New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;pwsh=$PSVersionTable.PSVersion.ToString();provider_enabled=$false}
# Allocate every independent fixture before any atomic edit can release its
# original inode. This tests placement, not the separate file-identity reuse
# lifecycle. Requests, source/oracles and assertions are unchanged.
foreach ($extension in @('cs','ts')) {
    foreach ($eolName in @('LF','CRLF')) {
        for ($i=0;$i -lt 4;$i++) {Write-BText (Join-Path $root "$extension-$eolName-$i.$extension") ''}
    }
    Write-BText (Join-Path $root "docs.$extension") ''
}
foreach ($file in @('neighbors.ts','atomic.ts','syntax.ts','stale.ts','unread.ts')) {Write-BText (Join-Path $root $file) ''}
function Assert-Source([string]$File,[string]$Expected) {
    $actual=[IO.File]::ReadAllBytes((Join-Path $root $File))
    $wanted=[Text.Encoding]::UTF8.GetBytes($Expected)
    Save-B ($File+'-exact') @{expected_base64=[Convert]::ToBase64String($wanted);actual_base64=[Convert]::ToBase64String($actual)}
    Assert-B ([Convert]::ToBase64String($actual) -ceq [Convert]::ToBase64String($wanted)) "Exact-source mismatch: $File"
}
function Publish-Edit([string]$File) {
    $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$File;fidelity='edit'})
}
function Edit-Case([string]$File,$Operations) {
    Invoke-BTool $session 'apply_edit' @{workspaceRoot=$root;filePath=$File;operations=$Operations}
}
try {
    $session=Start-BSession 'edit' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    Assert-B ('apply_edit' -in $registry.tools.name) 'Registered apply_edit absent.'
    foreach ($extension in @('cs','ts')) {
        $source=if ($extension -eq 'cs') {
            "// π unchanged`nusing System;`npublic class Placement {`n  public int First() { return 1; }`n  public int Middle() { return 2; }`n  [Obsolete]`n  public int Last() { return 3; }`n}`n"
        } else {
            "// π unchanged`nexport class Placement {`n  public first(): number { return 1; }`n  public middle(): number { return 2; }`n  @audit`n  public last(): number { return 3; }`n}`n"
        }
        $cases=if ($extension -eq 'cs') {
            @(@('First','public int First'),@('Middle','public int Middle'),@('Last','[Obsolete]'),@('Last','[Obsolete]'))
        } else {@(@('first','public first'),@('middle','public middle'),@('last','@audit'),@('last','@audit'))}
        foreach ($eolName in @('LF','CRLF')) {
            $eol=if ($eolName -eq 'LF') {"`n"} else {"`r`n"}
            for ($i=0;$i -lt $cases.Count;$i++) {
                Invoke-BCase '136' "$extension-$eolName-$i" {
                    $file="$extension-$eolName-$i.$extension";$before=$source.Replace("`n",$eol)
                    Write-BText (Join-Path $root $file) $before
                    Publish-Edit $file
                    $unit=if ($i -eq 3) {'/*PLACEMENT_PROBE*/'} elseif ($extension -eq 'cs') {"public int Added() { return 9; }`n  "} else {"public added(): number { return 9; }`n  "}
                    $offset=$before.IndexOf($cases[$i][1]);Assert-B ($offset -ge 0) 'Oracle boundary missing.'
                    $expected=$before.Substring(0,$offset)+$unit.Replace("`n",$eol)+$before.Substring($offset)
                    Save-B ($file+'-oracle') @{source_before=$before;unit=$unit;expected=$expected;target=$cases[$i][0];utf8_start=[Text.Encoding]::UTF8.GetByteCount($before.Substring(0,$offset))}
                    $answer=Get-BResult (Edit-Case $file @(@{type='insert_before';anchor="Placement.$($cases[$i][0])";unitText=$unit}))
                    Assert-Source $file $expected
                    Assert-B ($answer._meta.syntaxGated -eq $true) 'Syntax gate not established.'
                    Assert-B ($answer.structuredContent.operations[0].startByte -eq [Text.Encoding]::UTF8.GetByteCount($before.Substring(0,$offset))) 'Receipt does not identify declaration start.'
                }
            }
        }
    }
    foreach ($extension in @('cs','ts')) {
        Invoke-BCase '136' "$extension-unowned-documentation-refusal" {
            $source=if ($extension -eq 'cs') {"public class C {`n /// <summary>Unproven ownership</summary>`n public void Run() {} }"} else {"export class C {`n /** Unproven ownership */`n run() {} }"}
            $file="docs.$extension";Write-BText (Join-Path $root $file) $source;Publish-Edit $file
            $anchor=if ($extension -eq 'cs') {'C.Run'} else {'C.run'}
            $response=Edit-Case $file @(@{type='insert_before';anchor=$anchor;unitText='/*probe*/'})
            Assert-B ([bool]$response.PSObject.Properties['error']) 'Unproven comment ownership was guessed.'
            Assert-B ($response.error.message -match 'declaration insertion start unavailable') 'Refusal is not about declaration authority.'
            Assert-Source $file $source
        }
    }
    Invoke-BCase 'preservation' 'replace-after-delete' {
        $file='neighbors.ts';$source="export class N {`n first() { return 1; }`n second() { return 2; }`n}`n"
        Write-BText (Join-Path $root $file) $source;Publish-Edit $file
        $null=Get-BResult (Edit-Case $file @(@{type='replace_body';target='N.first';expectedOldText='{ return 1; }';newText='{ return 9; }'}))
        $expected=$source.Replace('return 1','return 9');Assert-Source $file $expected
        $null=Get-BResult (Edit-Case $file @(@{type='insert_after';anchor='N.second';unitText="`n added() {}"}))
        $expected=$expected.Replace('second() { return 2; }',"second() { return 2; }`n added() {}");Assert-Source $file $expected
        $null=Get-BResult (Edit-Case $file @(@{type='delete';target='N.added';expectedOldText='{}'}))
        $expected=$expected.Replace('added() {}','');Assert-Source $file $expected
    }
    foreach ($kind in @('atomic','syntax','stale','unread')) {
        Invoke-BCase 'preservation' "$kind-refusal-no-write" {
            $file="$kind.ts";$source='export class C { run() { return 1; } }'
            Write-BText (Join-Path $root $file) $source
            if ($kind -ne 'unread') {Publish-Edit $file}
            $ops=@(@{type='insert_before';anchor='C.run';unitText='added() {} '})
            if ($kind -eq 'atomic') {$ops+=@{type='insert_before';anchor='C.missing';unitText='missing() {} '}}
            if ($kind -eq 'syntax') {$ops[0].unitText='broken( {'}
            if ($kind -eq 'stale') {$source+=' // external change';Write-BText (Join-Path $root $file) $source}
            $response=Edit-Case $file $ops
            Assert-B ([bool]$response.PSObject.Properties['error']) "$kind must reject."
            Assert-Source $file $source
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray();Write-Host "Insertion evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
