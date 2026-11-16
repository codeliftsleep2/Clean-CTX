# Operator-owned local tool install and audit-only retry. No permission changes.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$previousBuildDirectory = $env:CARGO_BUILD_BUILD_DIR
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/audit-tool-install-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $buildDirectory = Join-Path $out 'build'
    $installDirectory = Join-Path $out 'installed'
    $env:CARGO_BUILD_BUILD_DIR = $buildDirectory
    $stdout = Join-Path $out 'install-stdout.txt'
    $stderr = Join-Path $out 'install-stderr.txt'
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & cargo install cargo-audit --version 0.23.0 --locked --target-dir $buildDirectory --root $installDirectory 1> $stdout 2> $stderr
    $code = $LASTEXITCODE
    $watch.Stop()
    [ordered]@{
        category='Operator repository-local tool installation; not successful audit or project compilation'
        version='0.23.0';exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds
        build_directory=$buildDirectory;install_directory=$installDirectory
        stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant();stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    if ($code -ne 0) {
        Get-Content -LiteralPath $stderr -Tail 20
        throw "Local tool installation failed; preserve $out. Do not change security settings; return logs for diagnosis."
    }
    $audit = Join-Path $installDirectory 'bin/cargo-audit.exe'
    [ordered]@{path=$audit;sha256=(Get-FileHash $audit).Hash.ToLowerInvariant()} | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $out 'audit-binary.json') -Encoding utf8NoBOM
    & (Join-Path $PSScriptRoot 'invoke-audit-dependency-update.ps1') -AuditOnly -AuditExecutable $audit
} finally {
    $env:CARGO_BUILD_BUILD_DIR = $previousBuildDirectory
    Pop-Location
}
