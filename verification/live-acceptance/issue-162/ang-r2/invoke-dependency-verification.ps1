# Operator-owned dependency integration compilation/lint; not a full local CI gate.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$previousRustFlags = $env:RUSTFLAGS
Push-Location $root
try {
    $lockHash = (Get-FileHash Cargo.lock).Hash.ToLowerInvariant()
    if ($lockHash -cne 'c1d18af45660ebb7dd3898ff567a856a18f85a2ba9344d15996c49b8a4e74a59') { throw 'Audited lockfile changed.' }
    $out = Join-Path $root ('target/ang-r2/dependency-verification-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $env:RUSTFLAGS = (($previousRustFlags+' -D warnings').Trim())
    $runs = @(
        @{name='default-workspace-check';exe='cargo';arguments=@('check','--workspace','--all-targets','--locked')},
        @{name='all-feature-workspace-clippy';exe='cargo';arguments=@('clippy','--workspace','--all-targets','--all-features','--locked','--','-D','warnings')},
        @{name='utf8';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-utf8.ps1')},
        @{name='file-sizes';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')}
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
            category='Focused operator dependency integration; not runtime tests, full CI or field acceptance'
            command=($run.exe+' '+($argumentsForRun -join ' '));rustflags=$env:RUSTFLAGS
            exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds;head=(& git rev-parse HEAD)
            lockfile_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant();stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out ($run.name+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve $out" }
        if ((Get-FileHash Cargo.lock).Hash.ToLowerInvariant() -cne $lockHash) { throw 'Lockfile changed during locked verification.' }
    }
    Write-Host "Return receipts from $out. Root and proxy compiled; runtime, CI, commit and branch-only push remain separate."
} finally {
    $env:RUSTFLAGS = $previousRustFlags
    Pop-Location
}
