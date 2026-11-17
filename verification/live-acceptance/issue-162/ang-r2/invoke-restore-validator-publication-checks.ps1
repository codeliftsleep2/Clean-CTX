# Operator-owned scoped checks and actual document admission. No commit or push.
param([switch]$GuardsAndAdmissionOnly)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$out=Join-Path $root ('target/issue-162/restore-publication-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $files=@(git ls-files --modified --others --exclude-standard -- '*.rs')
    $runs=@(
        @{name='scoped-format';exe='rustfmt';arguments=(@('--edition','2024','--config','skip_children=true','--check')+$files)},
        @{name='root-lib-test-clippy';exe='cargo';arguments=@('clippy','--lib','--tests','--all-features','--','-D','warnings')},
        @{name='oracle-prerequisites';exe='python';arguments=@('scripts/verify-ci-membership.py','--root',$root,'--prerequisites-only','--output',(Join-Path $out 'oracle-receipt.json'))},
        @{name='utf8';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-utf8.ps1')},
        @{name='file-sizes';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')},
        @{name='actual-admission';exe='pwsh';arguments=@('-NoProfile','-File',(Join-Path $PSScriptRoot 'invoke-restore-validator-admission.ps1'))}
    )
    if($GuardsAndAdmissionOnly){$runs=@($runs|Where-Object{$_.name -in @('utf8','file-sizes','actual-admission')})}
    $receipts=[Collections.Generic.List[object]]::new()
    foreach($run in $runs){
        $log=Join-Path $out ($run.name+'.log')
        $argumentsForRun=$run.arguments
        & $run.exe @argumentsForRun *> $log
        $code=$LASTEXITCODE
        $receipts.Add([ordered]@{name=$run.name;exe=$run.exe;arguments=$argumentsForRun;exitCode=$code;log_sha256=(Get-FileHash $log).Hash})
        [ordered]@{category='Scoped operator checks; not full CI';head=(git rev-parse HEAD);runs=$receipts.ToArray()}|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 30
        if($code -ne 0){throw "$($run.name) failed; preserve $out"}
    }
    Write-Output "Return scoped checks and actual admission receipts: $out"
} finally {Pop-Location}
