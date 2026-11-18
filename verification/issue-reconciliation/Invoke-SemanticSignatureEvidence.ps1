param([Parameter(Mandatory)][string]$BinaryPath,[string]$HistoryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$base=Join-Path $repo 'target/semantic-signature-evidence';New-Item -ItemType Directory -Force $base | Out-Null
$runRoot=Join-Path $base ((Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8));New-Item -ItemType Directory -Force $runRoot | Out-Null
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
function Source-S([string]$Return,[string]$Attribute,[string]$Parameter,[string]$Body) {
    "public class DiffProbe {`n  $Attribute`n  public $Return Get($Parameter)`n  {`n    return $Body;`n  }`n  public int Neighbor()`n  {`n    return 9;`n  }`n}`n"
}
# Preserve the supported last-method body contract, separately from the known
# first-method/next-marker attribution issue. Return/attribute neighbor oracles
# still use their original following-neighbor layout.
function Body-S([string]$Value) {
    "public class DiffProbe {`n  public int Neighbor() { return 9; }`n  public int Get()`n  {`n    return $Value;`n  }`n}`n"
}

function Git-S([string[]]$GitArgs) {
    $output=& git -C $HistoryPath @GitArgs
    Assert-B ($LASTEXITCODE -eq 0) "Fixture Git failed: $($GitArgs -join ' ')"
    return ($output -join "`n")
}
$cases=@(
    @{file='Return.cs';before=(Source-S 'int' '' '' '1');after=(Source-S 'long' '' '' '1');changed=$true},
    @{file='Attribute.cs';before=(Source-S 'int' '[Authorize(Roles = "Reader")]' '' '1');after=(Source-S 'int' '[Authorize(Roles = "Admin")]' '' '1');changed=$true},
    @{file='Param.cs';before=(Source-S 'int' '' 'int value' '1');after=(Source-S 'int' '' 'long value' '1');changed=$true},
    @{file='Body.cs';before=(Body-S '1');after=(Body-S '2');changed=$true},
    @{file='Unchanged.cs';before=(Source-S 'int' '' '' '1');after=("// no semantic change`n"+(Source-S 'int' '' '' '1'));changed=$false}
)
if (-not $HistoryPath) {
    $HistoryPath=Join-Path $runRoot 'history';New-Item -ItemType Directory -Force $HistoryPath | Out-Null
    $null=Git-S @('init','--quiet')
    foreach ($setting in @(@('user.name','Signature fixture'),@('user.email','fixture@example.invalid'),@('commit.gpgsign','false'),@('core.autocrlf','false'))) {$null=Git-S @('config',$setting[0],$setting[1])}
    $hooks=Join-Path $HistoryPath 'empty-hooks';New-Item -ItemType Directory -Force $hooks | Out-Null
    $null=Git-S @('config','core.hooksPath',$hooks)
    Write-BText (Join-Path $HistoryPath '.clean-ctx.json') '{"cbm":{"enabled":false}}'
    foreach ($case in $cases) {Write-BText (Join-Path $HistoryPath $case.file) $case.before}
    $null=Git-S @('add','--','*.cs');$null=Git-S @('commit','--quiet','-m','before')
    $from=Git-S @('rev-parse','HEAD')
    foreach ($case in $cases) {Write-BText (Join-Path $HistoryPath $case.file) $case.after}
    $null=Git-S @('add','--','*.cs');$null=Git-S @('commit','--quiet','-m','after')
    $to=Git-S @('rev-parse','HEAD')
    Write-BText (Join-Path $HistoryPath 'signature-history.json') (@{from=$from;to=$to;cases=$cases} | ConvertTo-Json -Depth 20)
} else {
    $HistoryPath=(Resolve-Path $HistoryPath).Path
    Assert-B ($HistoryPath.StartsWith($base+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) 'Only this harness-owned history may be reused.'
    $saved=Get-Content -Raw (Join-Path $HistoryPath 'signature-history.json') | ConvertFrom-Json -Depth 30
    $from=$saved.from;$to=$saved.to
}
foreach ($case in $cases) {
    # Native Git line capture is joined with LF for fixture admission. Exact
    # commit hashes retain the same committed bytes across RED/GREEN runs.
    Assert-B ((Git-S @('show',"${from}:$($case.file)")) -ceq $case.before.TrimEnd("`r","`n")) 'Before source does not match immutable oracle.'
    Assert-B ((Git-S @('show',"${to}:$($case.file)")) -ceq $case.after.TrimEnd("`r","`n")) 'After source does not match immutable oracle.'
}
$changed=Git-S @('diff','--name-status',$from,$to)
Assert-B (($changed.Split("`n")).Count -eq 5) 'Git-level changed-file oracle failed.'
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;history=$HistoryPath;from=$from;to=$to;git_file_evidence=$changed;cases=$cases}
try {
    $session=Start-BSession 'signature' $BinaryPath $HistoryPath
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    Assert-B ('diff_commits' -in $registry.tools.name) 'Registered diff_commits absent.'
    foreach ($fidelity in @('low','medium','high','edit','verbatim')) {
        $deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
        $answer=Get-BResult (Invoke-BTool $session 'diff_commits' @{workspaceRoot=$HistoryPath;fromRef=$from;toRef=$to;fidelity=$fidelity})
        Save-B "$fidelity-result" $answer
        $text=$answer.content[0].text
        foreach ($case in $cases) {
            Invoke-BCase '137' "$fidelity-$($case.file)" {
                $start=$text.IndexOf(": $($case.file) (");Assert-B ($start -ge 0) 'Changed file missing from MCP result.'
                $section=$text.Substring($start).Split('┌ FILE')[0]
                Assert-B ($section.Contains('~ method Get') -eq $case.changed) "Wrong member semantic change: $section"
                Assert-B (-not $section.Contains('~ method Neighbor')) 'Unchanged neighbor falsely changed.'
                if ($case.changed) {Assert-B (-not $section.Contains('class DiffProbe (unchanged)')) 'Changed declaration falsely unchanged.'}
                else {Assert-B ($section.Contains('class DiffProbe (unchanged)')) 'Identical member falsely changed.'}
            }
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray();Write-Host "Signature evidence: $runRoot";Write-Host "Reuse history for GREEN: $HistoryPath"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
