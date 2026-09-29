param(
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path,
    [string]$MeasureHelper,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($MeasureHelper)) {
    $MeasureHelper = Join-Path $RepositoryRoot "target\context-compression-verification\scripts\measure.exe"
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepositoryRoot "target\document-intelligence\baselines\full-document-tokens.json"
}

$manifestPath = Join-Path $RepositoryRoot "verification\document-intelligence\benchmark-manifest.json"
if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "Benchmark manifest not found: $manifestPath"
}
if (-not (Test-Path -LiteralPath $MeasureHelper)) {
    throw @"
Measurement helper not found: $MeasureHelper
Build it first with:
  pwsh -File verification/context-compression/scripts/Build-MeasureHelper.ps1
"@
}

$manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json -Depth 100
$records = [System.Collections.Generic.List[object]]::new()
$totalBytes = [long]0
$totalCl100k = [long]0
$totalO200k = [long]0

foreach ($document in $manifest.documents) {
    $absolutePath = Join-Path $RepositoryRoot ([string]$document.path)
    if (-not (Test-Path -LiteralPath $absolutePath -PathType Leaf)) {
        throw "Benchmark document not found: $($document.path)"
    }

    $actualHash = (Get-FileHash -LiteralPath $absolutePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($null -ne $document.sha256 -and $actualHash -ne [string]$document.sha256) {
        throw "Pinned hash mismatch for $($document.path): expected $($document.sha256), actual $actualHash"
    }

    $counts = [ordered]@{}
    foreach ($tokenizer in @("cl100k", "o200k")) {
        $rawCount = & $MeasureHelper count $tokenizer $absolutePath
        if ($LASTEXITCODE -ne 0) {
            throw "Tokenizer helper failed for $($document.path) with $tokenizer"
        }
        $parsedCount = 0
        if (-not [int]::TryParse(([string]$rawCount).Trim(), [ref]$parsedCount)) {
            throw "Tokenizer helper returned a non-integer for $($document.path) with ${tokenizer}: $rawCount"
        }
        $counts[$tokenizer] = $parsedCount
    }

    $records.Add([ordered]@{
        id = [string]$document.id
        path = ([string]$document.path).Replace("\", "/")
        sha256 = $actualHash
        bytes = (Get-Item -LiteralPath $absolutePath).Length
        tokens = $counts
    })
    $totalBytes += (Get-Item -LiteralPath $absolutePath).Length
    $totalCl100k += $counts.cl100k
    $totalO200k += $counts.o200k
}

$payload = [ordered]@{
    schema = "clean-ctx/document-intelligence-token-baseline"
    version = 1
    manifestVersion = [int]$manifest.version
    capturedAt = [DateTimeOffset]::UtcNow.ToString("o")
    tokenizers = @("cl100k", "o200k")
    documents = $records
    totals = [ordered]@{
        documents = $records.Count
        bytes = $totalBytes
        cl100k = $totalCl100k
        o200k = $totalO200k
    }
}

$outputDirectory = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
$json = $payload | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText($OutputPath, "$json`n", [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote exact document token baseline: $OutputPath"
