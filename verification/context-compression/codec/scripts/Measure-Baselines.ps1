$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not (Test-Path $measure)) { throw "Run Build-MeasureHelper.ps1 first" }
$scenarios = Get-Content -Raw (Join-Path $definitionRoot "expected\scenarios.json") | ConvertFrom-Json

function Count-Text([string]$tokenizer, [string]$text, [string]$name) {
    $temp = Join-Path $env:TEMP ("clean-ctx-$name-$([guid]::NewGuid().ToString('N')).txt")
    try {
        [IO.File]::WriteAllText($temp, $text, [Text.UTF8Encoding]::new($false))
        return [int](& $measure count $tokenizer $temp)
    } finally { Remove-Item -LiteralPath $temp -Force -ErrorAction SilentlyContinue }
}

$records = @()
foreach ($dir in Get-ChildItem -LiteralPath $captures -Directory) {
    $responsePath = Join-Path $dir.FullName "response.json"
    if (-not (Test-Path $responsePath)) { continue }
    $response = Get-Content -Raw $responsePath | ConvertFrom-Json -Depth 100
    $fullPath = Join-Path $dir.FullName "control-full.txt"
    if (-not (Test-Path $fullPath)) { continue }
    $fullText = Get-Content -Raw $fullPath
    $semantic = $null
    if ($fullText.StartsWith("// CONTROL-FULL v2")) {
        $withoutHeader = $fullText -replace "^[^`r`n]*`r?`n", ""
        $pathmap = $withoutHeader.LastIndexOf("PATHMAP", [StringComparison]::Ordinal)
        if ($pathmap -lt 0) { throw "$($dir.Name): CONTROL-FULL PATHMAP footer is absent" }
        $footerLine = $withoutHeader.LastIndexOf("`n", $pathmap)
        if ($footerLine -lt 0) { throw "$($dir.Name): malformed CONTROL-FULL PATHMAP footer" }
        $jsonText = $withoutHeader.Substring(0, $footerLine).TrimEnd("`r", "`n")
        $semantic = $jsonText | ConvertFrom-Json -Depth 100
    }
    $fixedPath = Join-Path $dir.FullName "control-full-fixed.txt"
    if ($semantic) {
        & $measure fixed $fullPath $fixedPath
        if ($LASTEXITCODE -ne 0) { throw "Fixed-envelope extraction failed for $($dir.Name)" }
    }
    $scenario = $scenarios | Where-Object { $dir.Name -eq $_.id } | Select-Object -First 1
    if (-not $scenario) {
        $scenario = $scenarios |
            Where-Object { $dir.Name.StartsWith("$($_.id)-") } |
            Sort-Object { $_.id.Length } -Descending |
            Select-Object -First 1
    }
    $footer = if ($fullText.Contains("PATHMAP")) {
        $pathmap = $fullText.LastIndexOf("PATHMAP", [StringComparison]::Ordinal)
        $footerLine = $fullText.LastIndexOf("`n", $pathmap)
        $fullText.Substring($footerLine + 1)
    } else { "" }
    $bodies = if ($semantic) { @(@($semantic.classes) + @($semantic.interfaces) | ForEach-Object { $_.methods } | ForEach-Object { $_ } | Where-Object { $null -ne $_.body } | ForEach-Object { $_.body }) -join "`n" } else { "" }
    $edges = if ($semantic) { $semantic.semantic_edges | ConvertTo-Json -Depth 100 -Compress } else { "" }
    $calls = if ($semantic) { $semantic.calls | ConvertTo-Json -Depth 100 -Compress } else { "" }
    $injections = if ($semantic) { $semantic.classes.injection_occurrences | ConvertTo-Json -Depth 100 -Compress } else { "" }
    foreach ($tokenizer in @("cl100k", "o200k")) {
        $prodPath = Join-Path $dir.FullName "control-prod-$tokenizer.txt"
        $fullTokens = [int](& $measure count $tokenizer $fullPath)
        $prodTokens = if (Test-Path $prodPath) { [int](& $measure count $tokenizer $prodPath) } else { $null }
        $records += [ordered]@{
            capture = $dir.Name
            fixture = if ($scenario) { $scenario.fixture } else { $dir.Name }
            source_path = if ($semantic) { $semantic.file.source_path } else { if ($scenario) { $scenario.fixture } else { $dir.Name } }
            language = if ($scenario) { $scenario.language } else { "typescript" }
            fidelity = if ($scenario) { $scenario.fidelity } else { if ($dir.Name -like "marginal-exact-body*") { "edit" } else { "high" } }
            intent = if ($scenario) { $scenario.intent } else { $null }
            focus_mode = if ($scenario) { $scenario.focusMode } else { "marginal pair" }
            production_operation = if ($scenario) { $scenario.operation } else { "provide_code_context" }
            body_policy = if ($scenario) { $scenario.bodyPolicy } else { if ($dir.Name -like "marginal-exact-body*") { "known exact body" } else { "none" } }
            matrix_row = if ($scenario) { $scenario.matrixRow } else { "Marginal-cost pair" }
            tokenizer = $tokenizer
            control_prod_tokens = $prodTokens
            control_full_tokens = $fullTokens
            payload_bytes = [Text.Encoding]::UTF8.GetByteCount($fullText)
            payload_chars = $fullText.Length
            class_count = if ($semantic) { @($semantic.classes).Count } else { $null }
            interface_count = if ($semantic) { @($semantic.interfaces).Count } else { $null }
            call_count = if ($semantic) { @($semantic.calls).Count } else { $null }
            semantic_edge_count = if ($semantic) { @($semantic.semantic_edges).Count } else { $null }
            injection_occurrence_count = if ($semantic) { ($semantic.classes | ForEach-Object { @($_.injection_occurrences).Count } | Measure-Object -Sum).Sum } else { $null }
            exact_body_count = if ($semantic) { @($semantic.mode.exact_body_method_ids).Count } else { $null }
            semantic_family_counts = if ($semantic) { [ordered]@{
                classes = @($semantic.classes).Count
                interfaces = @($semantic.interfaces).Count
                calls = @($semantic.calls).Count
                injections = ($semantic.classes | ForEach-Object { @($_.injection_occurrences).Count } | Measure-Object -Sum).Sum
                semantic_edges = @($semantic.semantic_edges).Count
                exact_bodies = @($semantic.mode.exact_body_method_ids).Count
            } } else { $null }
            header_tokens = Count-Text $tokenizer (($fullText -split "`n", 2)[0]) "header"
            fixed_overhead_tokens = if (Test-Path $fixedPath) { [int](& $measure count $tokenizer $fixedPath) } else { $null }
            path_footer_tokens = Count-Text $tokenizer $footer "footer"
            exact_body_value_tokens = Count-Text $tokenizer $bodies "bodies"
            semantic_edge_json_tokens = Count-Text $tokenizer $edges "edges"
            call_json_tokens = Count-Text $tokenizer $calls "calls"
            injection_json_tokens = Count-Text $tokenizer $injections "injections"
            control_prod_payload = if (Test-Path $prodPath) { $prodPath } else { $null }
            control_full_payload = $fullPath
            reasoning_oracle = (Join-Path $definitionRoot "codec\oracles.json")
            candidate_tokens = $null
        }
    }
}
$output = Join-Path $captures "baseline-records.json"
$records | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
$marginals = foreach ($tokenizer in @("cl100k", "o200k")) {
    foreach ($definition in Get-Content -Raw (Join-Path $definitionRoot "expected\marginal-fixtures.json") | ConvertFrom-Json) {
        $base = $records | Where-Object { $_.capture -eq "marginal-$($definition.id)-base" -and $_.tokenizer -eq $tokenizer } | Select-Object -First 1
        $plus = $records | Where-Object { $_.capture -eq "marginal-$($definition.id)-plus" -and $_.tokenizer -eq $tokenizer } | Select-Object -First 1
        if ($base -and $plus) {
            [ordered]@{
                semantic_family = $definition.id
                tokenizer = $tokenizer
                control_prod_marginal_tokens = if ($null -ne $base.control_prod_tokens -and $null -ne $plus.control_prod_tokens) { $plus.control_prod_tokens - $base.control_prod_tokens } else { $null }
                control_full_marginal_tokens = $plus.control_full_tokens - $base.control_full_tokens
                base_capture = $base.capture
                plus_capture = $plus.capture
            }
        }
    }
}
$marginalOutput = Join-Path $captures "marginal-costs.json"
$marginals | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $marginalOutput
Write-Host "Wrote $output ($($records.Count) tokenizer records)"
Write-Host "Wrote $marginalOutput ($($marginals.Count) marginal records)"
