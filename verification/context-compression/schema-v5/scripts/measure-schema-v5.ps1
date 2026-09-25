# Measure SCHEMA-v5 token savings for the large economics fixtures.
#
# The economics captures (produced by Capture-Baselines.ps1) store the production
# response as oracle-source-response.json; its result.content[0].text IS the
# SCHEMA-v5 presentation (for the large fixtures where it is emitted). This
# script counts that content against raw-source.txt — no re-capture required.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$measure = Join-Path $repositoryRoot "target\context-compression-verification\scripts\measure.exe"
if (-not (Test-Path -LiteralPath $measure)) { throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1" }

function Count-Tokens([string]$tokenizer, [string]$text) {
    $temp = Join-Path $env:TEMP ("schema-v5-count-$([guid]::NewGuid().ToString('N')).txt")
    try {
        [IO.File]::WriteAllText($temp, $text, [Text.UTF8Encoding]::new($false))
        return [int]((& $measure count $tokenizer $temp | Out-String).Trim())
    } finally { Remove-Item -LiteralPath $temp -Force -ErrorAction SilentlyContinue }
}

$records = @()
foreach ($dir in Get-ChildItem -LiteralPath $captures -Directory | Where-Object { $_.Name -like 'economics-*' }) {
    $responsePath = Join-Path $dir.FullName "oracle-source-response.json"
    $rawPath = Join-Path $dir.FullName "raw-source.txt"
    $metaPath = Join-Path $dir.FullName "capture-meta.json"
    if (-not (Test-Path -LiteralPath $responsePath) -or -not (Test-Path -LiteralPath $rawPath)) { continue }
    $response = Get-Content -Raw -LiteralPath $responsePath | ConvertFrom-Json -Depth 100
    $schema = [string]$response.result.content[0].text
    if (-not $schema.StartsWith("// SCHEMA v5")) { continue }
    $raw = Get-Content -Raw -LiteralPath $rawPath
    $meta = if (Test-Path -LiteralPath $metaPath) { Get-Content -Raw -LiteralPath $metaPath | ConvertFrom-Json } else { $null }
    $language = if ($meta -and $meta.language) { $meta.language } else { "?" }
    $fidelity = if ($meta -and $meta.fidelity) { $meta.fidelity } else { "?" }
    foreach ($tokenizer in @("cl100k", "o200k")) {
        $rawTokens = Count-Tokens $tokenizer $raw
        $schemaTokens = Count-Tokens $tokenizer $schema
        $records += [ordered]@{
            capture = $dir.Name
            language = $language
            fidelity = $fidelity
            tokenizer = $tokenizer
            raw_tokens = $rawTokens
            schema_v5_tokens = $schemaTokens
            saved_tokens = $rawTokens - $schemaTokens
            reduction_percent = if ($rawTokens) { [Math]::Round(($rawTokens - $schemaTokens) * 100.0 / $rawTokens, 2) } else { 0 }
        }
    }
}

if (-not $records.Count) { throw "No economics captures with SCHEMA-v5 content found. Run Capture-Baselines.ps1 first." }
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM (Join-Path $captures "schema-v5-token-records.json")
Write-Host "Wrote schema-v5-token-records.json ($($records.Count) records)"
Write-Host ""
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host "=== $tokenizer ==="
    foreach ($lang in @($records.language | Sort-Object -Unique)) {
        foreach ($fid in @("low", "medium", "high")) {
            $rows = @($records | Where-Object { $_.tokenizer -eq $tokenizer -and $_.language -eq $lang -and $_.fidelity -eq $fid })
            if (-not $rows.Count) { continue }
            $raw = ($rows.raw_tokens | Measure-Object -Sum).Sum
            $schema = ($rows.schema_v5_tokens | Measure-Object -Sum).Sum
            $pct = if ($raw) { [Math]::Round(($raw - $schema) * 100.0 / $raw, 2) } else { 0 }
            Write-Host ("  {0,-12} {1,-8} raw {2,6} -> SCHEMA-v5 {3,6} ({4}%)" -f $lang, $fid, $raw, $schema, $pct)
        }
    }
}
