# Operator-owned exact replay adoption test and read-only scoped format check.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/prepublication-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $paths = @(@(& git diff HEAD --name-only -- src) + @(& git ls-files --others --exclude-standard src) |
        Sort-Object -Unique | Where-Object { $_ -like '*.rs' -and (Test-Path -LiteralPath $_) })
    $before = @($paths | ForEach-Object { [ordered]@{path=$_;sha256=(Get-FileHash $_).Hash.ToLowerInvariant()} })
    $before | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out 'source-manifest.json') -Encoding utf8NoBOM
    $filter = 'issue_162_resolver_registration_replay_adopts_exact_keys_before_query'
    $runs = @(
        @{name='exact-replay';exe='cargo';arguments=@('test','--lib','--all-features','--quiet',$filter,'--','--test-threads=1','--nocapture')},
        @{name='scoped-format';exe='rustfmt';arguments=(@('--edition','2024','--config','skip_children=true','--check') + $paths)}
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
            category='Focused operator verification; scoped format check is read-only, not full CI'
            command=($run.exe+' '+($argumentsForRun -join ' '));exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds
            head=(& git rev-parse HEAD);stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant();stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out ($run.name+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve $out" }
        if ($run.name -eq 'exact-replay') {
            if (Select-String -LiteralPath $stderr -Pattern '^warning:' -Quiet) { throw 'Replay test has compiler warnings.' }
            if (-not (Select-String -LiteralPath $stdout -SimpleMatch -Pattern 'test result: ok. 1 passed; 0 failed;' -Quiet)) { throw 'Unexpected replay test selection.' }
        }
    }
    $after = @($paths | ForEach-Object { [ordered]@{path=$_;sha256=(Get-FileHash $_).Hash.ToLowerInvariant()} })
    $after | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out 'source-manifest-after.json') -Encoding utf8NoBOM
    if ((ConvertTo-Json -InputObject $before -Depth 10) -cne (ConvertTo-Json -InputObject $after -Depth 10)) { throw 'Sources changed during read-only verification.' }
    Write-Host "Return receipts from $out. No formatter mutation, build, commit or push performed."
} finally { Pop-Location }
