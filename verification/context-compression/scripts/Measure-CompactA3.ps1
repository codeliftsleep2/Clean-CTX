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
        $metaPath = Join-Path $directory.FullName "capture-meta.json"
        $meta = if (Test-Path -LiteralPath $metaPath) { Get-Content -Raw $metaPath | ConvertFrom-Json } else { $null }
        $language = if ($meta -and $meta.language) { $meta.language } else { "" }
        $focusMode = if ($meta -and $meta.focus_mode) { $meta.focus_mode } else { "none" }
        foreach ($tokenizer in @("cl100k", "o200k")) {
            $rawTokens = [int](& $measure count $tokenizer $raw)
            $a3Tokens = [int](& $measure count $tokenizer $candidate)
            $legendTokens = [int](& $measure count $tokenizer $legend)
            $anatomy = (& $measure a3-anatomy $full $tokenizer) | ConvertFrom-Json
            $economical = $a3Tokens -lt $rawTokens
            $selectedTokens = if ($economical) { $a3Tokens } else { $rawTokens }
            $records += [ordered]@{
                capture = $directory.Name
                lane = if ($directory.Name.StartsWith("economics-")) { "tracked_economics" } else { "correctness_lifecycle" }
                language = $language
                fidelity = @{ L="low"; M="medium"; H="high"; E="edit" }[$fidelity]
                focus_mode = $focusMode
                tokenizer = $tokenizer
                raw_source_tokens = $rawTokens
                compact_a3_tokens = $a3Tokens
                legend_tokens = $legendTokens
                anatomy_legend_tokens = $anatomy.legend
                anatomy_declarations_tokens = $anatomy.declarations
                anatomy_facts_tokens = $anatomy.facts
                anatomy_bodies_tokens = $anatomy.bodies
                saved_tokens = $rawTokens - $selectedTokens
                reduction_percent = if ($rawTokens) { [Math]::Round((($rawTokens-$selectedTokens)*100.0/$rawTokens), 2) } else { 0 }
                economical_vs_raw = $economical
                selected_tokens = $selectedTokens
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
        $sel = ($subset.selected_tokens | Measure-Object -Sum).Sum
        $wins = @($subset | Where-Object economical_vs_raw).Count
        Write-Host "$tokenizer ${lane}: $($subset.Count), raw $raw -> selected $sel ($([Math]::Round((($raw-$sel)*100.0/$raw),2))%; $wins/$($subset.Count) economical)"
        foreach ($fidelity in @("low", "medium", "high", "edit")) {
            $mode = @($subset | Where-Object fidelity -eq $fidelity)
            if (-not $mode.Count) { continue }
            $modeRaw = ($mode.raw_source_tokens | Measure-Object -Sum).Sum
            $modeSel = ($mode.selected_tokens | Measure-Object -Sum).Sum
            Write-Host "  ${fidelity}: $($mode.Count), raw $modeRaw -> selected $modeSel ($([Math]::Round((($modeRaw-$modeSel)*100.0/$modeRaw),2))%)"
        }
    }
    $econ = @($rows | Where-Object lane -eq "tracked_economics")
    if ($econ.Count) {
        $rawSum = ($econ.raw_source_tokens | Measure-Object -Sum).Sum
        $selSum = ($econ.selected_tokens | Measure-Object -Sum).Sum
        $winCount = @($econ | Where-Object economical_vs_raw).Count
        $agg = if ($rawSum) { [Math]::Round((($rawSum - $selSum) * 100.0 / $rawSum), 2) } else { 0 }
        Write-Host "$tokenizer PRODUCTION-SELECTED economics aggregate: $($econ.Count) rows, raw $rawSum -> selected $selSum ($agg%; $winCount/$($econ.Count) economical)"
        foreach ($lang in @($econ.language | Sort-Object -Unique)) {
            foreach ($fidelity in @("low", "medium", "high", "edit")) {
                foreach ($focusMode in @("none", "all-bodies", "focused")) {
                    $mode = @($econ | Where-Object { $_.language -eq $lang -and $_.fidelity -eq $fidelity -and $_.focus_mode -eq $focusMode })
                    if (-not $mode.Count) { continue }
                    $modeRaw = ($mode.raw_source_tokens | Measure-Object -Sum).Sum
                    $modeSel = ($mode.selected_tokens | Measure-Object -Sum).Sum
                    $modeRed = if ($modeRaw) { [Math]::Round((($modeRaw - $modeSel) * 100.0 / $modeRaw), 2) } else { 0 }
                    Write-Host ("    {0,-10} {1,-6} {2,-11}: {3} row(s), raw {4} -> selected {5} ({6}%)" -f $lang, $fidelity, $focusMode, $mode.Count, $modeRaw, $modeSel, $modeRed)
                }
            }
        }
    }
}
Write-Host "Wrote $output ($($records.Count) records; zero model calls)"
