$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$measure = Join-Path $repositoryRoot "target\context-compression-verification\scripts\measure.exe"
if (-not (Test-Path -LiteralPath $measure)) { throw "Run Build-MeasureHelper.ps1 first" }

$legend = Join-Path $captures "compact-a3-legend.txt"
& $measure a3-legend $legend
if ($LASTEXITCODE -ne 0) { throw "A3 legend render failed" }
$records = @()
try {
    foreach ($directory in Get-ChildItem -LiteralPath $captures -Directory) {
        $full = Join-Path $directory.FullName "control-full.txt"
        $raw = Join-Path $directory.FullName "raw-source.txt"
        if (-not (Test-Path -LiteralPath $full)) { continue }
        if (-not (Test-Path -LiteralPath $raw)) { throw "$($directory.Name): missing raw source" }
        if (-not (Get-Content -Raw $full).StartsWith("// CONTROL-FULL v2")) { continue }
        $candidate = Join-Path $directory.FullName "compact-a3.txt"
        & $measure a3 $full $candidate
        if ($LASTEXITCODE -ne 0) { throw "$($directory.Name): A3 render failed" }
        $fidelity = ((Get-Content -Raw $candidate) -split "`r?`n", 4)[2].Split('|')[2]
        foreach ($tokenizer in @("cl100k", "o200k")) {
            $rawTokens = [int](& $measure count $tokenizer $raw)
            $a3Tokens = [int](& $measure count $tokenizer $candidate)
            $legendTokens = [int](& $measure count $tokenizer $legend)
            $records += [ordered]@{
                capture = $directory.Name
                lane = if ($directory.Name.StartsWith("economics-")) { "tracked_economics" } else { "correctness_lifecycle" }
                fidelity = @{ L="low"; M="medium"; H="high"; E="edit" }[$fidelity]
                tokenizer = $tokenizer
                raw_source_tokens = $rawTokens
                compact_a3_tokens = $a3Tokens
                legend_tokens = $legendTokens
                saved_tokens = $rawTokens - $a3Tokens
                reduction_percent = if ($rawTokens) { [Math]::Round((($rawTokens-$a3Tokens)*100.0/$rawTokens), 2) } else { 0 }
                economical_vs_raw = $a3Tokens -lt $rawTokens
                candidate_payload = $candidate
            }
        }
    }
} finally {
    Remove-Item -LiteralPath $legend -Force -ErrorAction SilentlyContinue
}
if (-not $records.Count) { throw "No A3 captures were measured" }
$output = Join-Path $captures "compact-a3-token-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
foreach ($tokenizer in @("cl100k", "o200k")) {
    $rows = @($records | Where-Object tokenizer -eq $tokenizer)
    foreach ($lane in @("correctness_lifecycle", "tracked_economics")) {
        $subset = @($rows | Where-Object lane -eq $lane)
        if (-not $subset.Count) { continue }
        $raw = ($subset.raw_source_tokens | Measure-Object -Sum).Sum
        $a3 = ($subset.compact_a3_tokens | Measure-Object -Sum).Sum
        $wins = @($subset | Where-Object economical_vs_raw).Count
        Write-Host "$tokenizer ${lane}: $($subset.Count), raw $raw -> A3 $a3 ($([Math]::Round((($raw-$a3)*100.0/$raw),2))%; $wins/$($subset.Count) economical)"
        foreach ($fidelity in @("low", "medium", "high", "edit")) {
            $mode = @($subset | Where-Object fidelity -eq $fidelity)
            if (-not $mode.Count) { continue }
            $modeRaw = ($mode.raw_source_tokens | Measure-Object -Sum).Sum
            $modeA3 = ($mode.compact_a3_tokens | Measure-Object -Sum).Sum
            Write-Host "  ${fidelity}: $($mode.Count), raw $modeRaw -> A3 $modeA3 ($([Math]::Round((($modeRaw-$modeA3)*100.0/$modeRaw),2))%)"
        }
    }
}
Write-Host "Wrote $output ($($records.Count) records; zero model calls)"
