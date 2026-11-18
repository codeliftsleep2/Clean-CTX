# Operator-owned focused enforcement and historical evidence replay; not full CI.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/parser-repair-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $source='src/tests/architecture/ci_verify_r1.rs'
    & rustfmt --edition 2024 --config skip_children=true $source
    if($LASTEXITCODE -ne 0){throw 'Focused formatting failed'}
    Expand-Archive -LiteralPath (Join-Path $PSScriptRoot 'consolidated-parser-red.zip') -DestinationPath (Join-Path $out 'historical')
    $runs=@(
        @{name='historical-replay';exe='python';arguments=@('scripts/verify-ci-membership.py','--root',$root,'--listing',(Join-Path $out 'historical/registration.log'),'--execution',(Join-Path $out 'historical/execution.log'),'--suite-exit','0','--output',(Join-Path $out 'membership-result.json'))},
        @{name='tracked-enforcement';exe='cargo';arguments=@('test','--lib','--all-features','ci_verify_r1::ci_verify_r1_required_execution_and_failure_contracts','--','--exact','--test-threads=1')}
    )
    $receipts=[Collections.Generic.List[object]]::new()
    foreach($run in $runs){
        $log=Join-Path $out ($run.name+'.log')
        $argumentsForRun=$run.arguments
        & $run.exe @argumentsForRun *> $log
        $code=$LASTEXITCODE
        $receipts.Add(@{name=$run.name;exe=$run.exe;arguments=$argumentsForRun;exitCode=$code;log_sha256=(Get-FileHash $log).Hash})
        @{head=(git rev-parse HEAD);category='Historical evidence replay and focused tracked enforcement';runs=$receipts.ToArray();sources=@('scripts/verify-ci-membership.py',$source)|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}}}|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 40
        if($code -ne 0){throw "$($run.name) failed; preserve $out"}
        if($run.name -eq 'tracked-enforcement'){
            if(Select-String -LiteralPath $log -Pattern '^warning:' -Quiet){throw 'Compiler warnings'}
            if(-not (Select-String -LiteralPath $log -SimpleMatch -Pattern 'test result: ok. 1 passed; 0 failed; 0 ignored;' -Quiet)){throw 'Unexpected test selection'}
        }
    }
    Write-Output "Return parser repair receipts: $out"
} finally {Pop-Location}
