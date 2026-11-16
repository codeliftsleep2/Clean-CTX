param([switch]$MutationOnly)
# Operator-owned focused remaining ANG-R2 lifecycle verification. No PR or push.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/remaining-lifecycle-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $file = 'src/tests/mcp/angular_resolver_registration_persistence_issue_162.rs'
    $filter = if ($MutationOnly) {'issue_162_resolver_registration_remaining_edit_and_delta_transport'} else {'issue_162_resolver_registration_remaining_'}
    $expected = if ($MutationOnly) {1} else {2}
    $runs = @(
        @{name='format';exe='rustfmt';arguments=@('--edition','2024','--config','skip_children=true',$file)},
        @{name='size';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')},
        @{name='focused-tests';exe='cargo';arguments=@('test','--lib','--all-features','--quiet',$filter,'--','--test-threads=1','--nocapture')}
    )
    foreach ($run in $runs) {
        $argumentsForRun = $run.arguments
        $stdout = Join-Path $out ($run.name+'-stdout.txt')
        $stderr = Join-Path $out ($run.name+'-stderr.txt')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & $run.exe @argumentsForRun 1> $stdout 2> $stderr
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Focused operator lifecycle evidence; not CI, field or final acceptance'
            command=($run.exe+' '+($argumentsForRun -join ' '));exit_code=$code
            elapsed_seconds=$watch.Elapsed.TotalSeconds;head=(& git rev-parse HEAD)
            test_sha256=(Get-FileHash $file).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant()
            stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out ($run.name+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve $out" }
        if ($run.name -eq 'focused-tests') {
            if (Select-String -LiteralPath $stderr -Pattern '^warning:' -Quiet) { throw 'Compiler warnings.' }
            if (-not (Select-String -LiteralPath $stdout -SimpleMatch -Pattern "test result: ok. $expected passed; 0 failed;" -Quiet)) { throw 'Unexpected selected tests.' }
        }
    }
    Write-Host "Return receipts from $out. No PR, push, build or broad test suite was started by this wrapper."
} finally { Pop-Location }
