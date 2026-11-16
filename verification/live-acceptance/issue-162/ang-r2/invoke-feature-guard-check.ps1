# Operator-owned compilation-only verification of three observed CI feature guards.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$previousRustFlags = $env:RUSTFLAGS
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/feature-guard-check-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $env:RUSTFLAGS = (($previousRustFlags + ' -D warnings').Trim())
    # Default features deliberately reproduce CI's observed failure. This first
    # command is NOT all-feature verification; the second explicitly supplies it.
    foreach ($mode in @('default','all-features')) {
        $argumentsForRun = @('check','--all-targets')
        if ($mode -eq 'all-features') { $argumentsForRun += '--all-features' }
        $stdout = Join-Path $out ($mode+'-stdout.txt')
        $stderr = Join-Path $out ($mode+'-stderr.txt')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & cargo @argumentsForRun 1> $stdout 2> $stderr
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Focused operator default/all-feature compilation; not full CI or runtime tests'
            command=('cargo '+($argumentsForRun -join ' '));rustflags=$env:RUSTFLAGS
            feature_mode=$mode;exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds
            head=(& git rev-parse HEAD)
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant()
            stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out ($mode+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$mode compilation failed; preserve $out" }
    }
    Write-Host "Return both compilation receipts from $out. No PR, CI dispatch, field pilot or test suite was started."
} finally {
    $env:RUSTFLAGS = $previousRustFlags
    Pop-Location
}
