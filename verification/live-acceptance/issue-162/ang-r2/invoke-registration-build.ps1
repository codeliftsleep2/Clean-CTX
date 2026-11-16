# Operator-owned scoped lint, production build and repository guards. No runtime.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    $out = Join-Path $root ('target/ang-r2/registration-build-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    function Source-Pins {
        $paths = @(& git ls-files '*.rs' '*.toml' 'Cargo.lock') + @(& git ls-files --others --exclude-standard src)
        @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ } | ForEach-Object {
            [ordered]@{path=$_; sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}
        })
    }
    $before = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest.json'), $before, $utf8)
    $runs = @(
        @{name='scoped-clippy'; exe='cargo'; arguments=@('clippy','--lib','--bin','clean-ctx','--tests','--all-features','--','-D','warnings')},
        @{name='production-build'; exe='cargo'; arguments=@('build','--bin','clean-ctx','--all-features')},
        @{name='utf8'; exe='pwsh'; arguments=@('-NoProfile','-File','scripts/check-utf8.ps1')},
        @{name='file-sizes'; exe='pwsh'; arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')}
    )
    foreach ($run in $runs) {
        $argsForRun = $run.arguments
        $stdout = Join-Path $out ($run.name + '-stdout.txt')
        $stderr = Join-Path $out ($run.name + '-stderr.txt')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & $run.exe @argsForRun 1> $stdout 2> $stderr
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Operator local scoped verification, not CI or production runtime acceptance'
            command=($run.exe + ' ' + ($argsForRun -join ' ')); exit_code=$code
            elapsed_seconds=$watch.Elapsed.TotalSeconds; head=(& git rev-parse HEAD)
            source_manifest_sha256=(Get-FileHash (Join-Path $out 'source-manifest.json')).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant()
            stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out ($run.name + '-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve $out" }
        if ($run.exe -eq 'cargo' -and (Select-String -LiteralPath $stderr -Pattern '^warning:' -Quiet)) { throw "Cargo warnings; preserve $out" }
    }
    $after = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest-after.json'), $after, $utf8)
    if ($before -cne $after) { throw "Sources changed during verification; preserve $out" }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    [ordered]@{path=$binary; sha256=(Get-FileHash $binary).Hash.ToLowerInvariant(); head=(& git rev-parse HEAD)} |
        ConvertTo-Json | Set-Content (Join-Path $out 'binary.json') -Encoding utf8NoBOM
    Write-Host "Return receipts from $out. Executable built but NOT started. CI and stdio acceptance remain separate."
} finally { Pop-Location }
