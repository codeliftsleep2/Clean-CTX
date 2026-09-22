param([string]$BinaryPath = "")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
if (-not $BinaryPath) { $BinaryPath = Join-Path $repositoryRoot "target\debug\clean-ctx.exe" }

Push-Location $repositoryRoot
try {
    & cargo build --all-features
    if ($LASTEXITCODE -ne 0) { throw "Clean-CTX all-feature build failed" }
    & (Join-Path $PSScriptRoot "Reset.ps1")
    & (Join-Path $PSScriptRoot "Prepare-Fixtures.ps1")
    & (Join-Path $PSScriptRoot "Build-MeasureHelper.ps1")
    & (Join-Path $PSScriptRoot "Capture-Baselines.ps1") -BinaryPath $BinaryPath
    & (Join-Path $PSScriptRoot "Measure-CompactA.ps1")
    & (Join-Path $PSScriptRoot "Verify-CompactA1.ps1")
} finally {
    Pop-Location
}

Write-Host "PASS: raw-source economics and lossless file-local A2 verification completed (zero model calls)."
