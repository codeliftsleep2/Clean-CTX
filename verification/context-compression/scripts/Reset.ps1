param([switch]$KeepHelper)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$root = Join-Path $repositoryRoot "target\context-compression-verification"
foreach ($relative in @("captures", "runtime")) {
    $path = Join-Path $root $relative
    if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Recurse -Force }
}
$helper = Join-Path $root "scripts\measure.exe"
if (-not $KeepHelper -and (Test-Path -LiteralPath $helper)) { Remove-Item -LiteralPath $helper -Force }
$helperMessage = if ($KeepHelper) { "Measurement helper was preserved." } else { "Measurement helper was removed." }
Write-Host "Removed generated captures and runtime state. $helperMessage Fixtures/oracles remain."
