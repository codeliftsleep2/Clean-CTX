param(
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
)

$ErrorActionPreference = "Stop"
$manifest = Join-Path $PSScriptRoot "..\measure-helper\Cargo.toml"
$targetDirectory = Join-Path $RepositoryRoot "target"
$runtimeScripts = Join-Path $RepositoryRoot "target\context-compression-verification\scripts"
New-Item -ItemType Directory -Force $runtimeScripts | Out-Null
$output = Join-Path $runtimeScripts "measure.exe"
Write-Host "Building measurement helper with the cached repository dependency graph..."
& cargo build --offline --manifest-path $manifest --target-dir $targetDirectory
if ($LASTEXITCODE -ne 0) { throw "measure helper compilation failed" }
$built = Join-Path $targetDirectory "debug\clean-ctx-measure-helper.exe"
if (-not (Test-Path -LiteralPath $built)) { throw "Cargo did not produce $built" }
Copy-Item -LiteralPath $built -Destination $output -Force
Write-Host "Built $output"
