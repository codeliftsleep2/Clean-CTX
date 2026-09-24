param(
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path,
    [string]$Tokenizer = "o200k"
)

$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not (Test-Path -LiteralPath $measure)) { throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1" }

$oracles = Get-Content -Raw (Join-Path $definitionRoot "reasoning\oracles.json") | ConvertFrom-Json
$targets = @($oracles | Where-Object { $_.lane -eq "file" } | ForEach-Object { $_.captures } | Sort-Object -Unique)
if ($targets.Count -eq 0) { throw "No file-lane captures found in oracles.json" }

function Count-Tokens([string]$path) {
    $text = & $measure count $Tokenizer $path
    if ($LASTEXITCODE -ne 0) { throw "token count failed for $path" }
    return [int](($text | Out-String).Trim())
}

$selected = 0; $passthrough = 0
foreach ($capture in $targets) {
    $dir = Join-Path $captures $capture
    $oraclePath = Join-Path $dir "control-full.txt"
    $rawPath = Join-Path $dir "raw-source.txt"
    if (-not (Test-Path -LiteralPath $oraclePath)) { Write-Warning "missing oracle: $capture"; continue }
    if (-not (Test-Path -LiteralPath $rawPath)) { Write-Warning "missing raw source: $capture"; continue }
    $a3Path = Join-Path $dir "context-a3.txt"
    & $measure a3 $oraclePath $a3Path
    if ($LASTEXITCODE -ne 0) { throw "A3 render failed for $capture" }
    # Production gate (ExactLocal): candidate must be strictly cheaper than raw,
    # otherwise ship byte-exact raw source.
    $rawTokens = Count-Tokens $rawPath
    $a3Tokens = Count-Tokens $a3Path
    if ($a3Tokens -lt $rawTokens) {
        Write-Host ("{0}: A3 {1} < raw {2} -> A3" -f $capture, $a3Tokens, $rawTokens)
        $selected++
    } else {
        Copy-Item -LiteralPath $rawPath -Destination $a3Path -Force
        Write-Host ("{0}: A3 {1} >= raw {2} -> raw passthrough" -f $capture, $a3Tokens, $rawTokens)
        $passthrough++
    }
}
Write-Host "Compressed-context selection ($Tokenizer): $selected A3, $passthrough raw passthrough"
