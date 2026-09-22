$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not (Test-Path $measure)) { throw "Run Build-MeasureHelper.ps1 first" }
$anatomyScratch = Join-Path $runtimeRoot "token-anatomy-$PID.tmp"

$schema = "S A2 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution]. Workspace graph edges are retrieved with workspace_query. N.D and N.V index existing local facts. B[method_id,start,end,utf8_bytes]"

function Count-TextTokens([string]$tokenizer, [string]$text) {
    try {
        [IO.File]::WriteAllText($anatomyScratch, $text, [Text.UTF8Encoding]::new($false))
        return [int](& $measure count $tokenizer $anatomyScratch)
    } finally {
        Remove-Item -LiteralPath $anatomyScratch -Force -ErrorAction SilentlyContinue
    }
}

function Method-Row($method, [System.Collections.Generic.List[object]]$bodyFrames) {
    if ($null -ne $method.body) {
        if ($null -eq $method.body_start -or $null -eq $method.body_end) {
            throw "Body $($method.id) is missing an exact span"
        }
        $bodyFrames.Add([ordered]@{
            id = [string]$method.id
            start = [uint64]$method.body_start
            end = [uint64]$method.body_end
            body = [string]$method.body
        })
    }
    return ,@(
        $method.id, $method.name, @($method.parameters), $method.return_type,
        @($method.modifier_occurrences), @($method.control_summary_occurrences),
        @($method.pattern_fact_occurrences), @($method.legacy_flag_occurrences),
        @($method.patterns), @($method.control_flow), @($method.data_flow),
        @($method.side_effects), @($method.execution_contexts)
    )
}

function Class-Row($owner, [System.Collections.Generic.List[object]]$bodyFrames) {
    $methods = @($owner.methods | ForEach-Object { Method-Row $_ $bodyFrames })
    return ,@(
        $owner.id, $owner.name, $owner.synthetic, $methods, @($owner.fields),
        @($owner.modifier_occurrences), @($owner.class_flag_occurrences),
        $owner.extends, @($owner.implements), @($owner.injection_occurrences),
        @($owner.patterns)
    )
}

function Interface-Row($owner, [System.Collections.Generic.List[object]]$bodyFrames) {
    $methods = @($owner.methods | ForEach-Object { Method-Row $_ $bodyFrames })
    return ,@(
        $owner.id, $owner.name, $methods, @($owner.fields),
        @($owner.modifier_occurrences), @($owner.extends)
    )
}

function Call-Row($call) {
    return ,@(
        $call.occurrence, $call.caller_method_id, $call.callee_written_name,
        $call.explicit_argument_count, $call.has_spread, $call.callee_resolution
    )
}

function Encode-A2($semantic) {
    $bodyFrames = [System.Collections.Generic.List[object]]::new()
    $classes = @($semantic.classes | ForEach-Object { Class-Row $_ $bodyFrames })
    $interfaces = @($semantic.interfaces | ForEach-Object { Interface-Row $_ $bodyFrames })
    $calls = @($semantic.calls | ForEach-Object { Call-Row $_ })
    $di = @($semantic.classes | Where-Object { @($_.injection_occurrences).Count } |
        ForEach-Object { ,@($_.id, 'inj') })
    $behavior = [Collections.Generic.List[object]]::new()
    @(
        @($semantic.classes) + @($semantic.interfaces) |
            ForEach-Object { $_.methods } |
            ForEach-Object {
                $method = $_
                foreach ($entry in @(
                    @('mod', @($method.modifier_occurrences)), @('cs', @($method.control_summary_occurrences)),
                    @('pf', @($method.pattern_fact_occurrences)), @('lf', @($method.legacy_flag_occurrences)),
                    @('pt', @($method.patterns)), @('cf', @($method.control_flow)),
                    @('df', @($method.data_flow)), @('se', @($method.side_effects)),
                    @('ec', @($method.execution_contexts))
                )) {
                    if (@($entry[1]).Count) { $behavior.Add(@($method.id, $entry[0])) }
                }
            }
    ) | Out-Null
    $envelope = [ordered]@{
        A = 2
        h = @('clean-ctx/file-context', 1, $semantic.file, $semantic.mode)
        d = [ordered]@{ A = 1; c = $classes; i = $interfaces }
        g = [ordered]@{ K = $calls }
        n = [ordered]@{ D = $di; V = @($behavior) }
        i = @($semantic.imports)
        t = @($semantic.type_aliases)
    }
    return [ordered]@{ envelope = $envelope; bodies = $bodyFrames }
}

function Body-Wire($frames) {
    $builder = [Text.StringBuilder]::new()
    foreach ($frame in $frames) {
        $length = [Text.Encoding]::UTF8.GetByteCount($frame.body)
        [void]$builder.Append("B $($frame.id) $($frame.start) $($frame.end) $length`n")
        [void]$builder.Append($frame.body)
        [void]$builder.Append("`n")
    }
    return $builder.ToString()
}

$records = @()
foreach ($directory in Get-ChildItem -LiteralPath $captures -Directory) {
    $fullPath = Join-Path $directory.FullName "control-full.txt"
    $rawPath = Join-Path $directory.FullName "raw-source.txt"
    if (-not (Test-Path $fullPath)) { continue }
    if (-not (Test-Path $rawPath)) { throw "$($directory.Name): missing capture-time raw-source.txt" }
    $fullText = Get-Content -Raw $fullPath
    if (-not $fullText.StartsWith("// CONTROL-FULL v2")) { continue }
    $remainder = $fullText -replace "^[^`r`n]*`r?`n", ""
    $pathmap = $remainder.LastIndexOf("`n§PATHMAP", [StringComparison]::Ordinal)
    if ($pathmap -lt 0) { throw "$($directory.Name): missing PATHMAP footer" }
    $semantic = $remainder.Substring(0, $pathmap) | ConvertFrom-Json -Depth 100
    $footer = $remainder.Substring($pathmap)
    $encoded = Encode-A2 $semantic
    $legendText = "// COMPACT-A A2; file-local; workspace graph via workspace_query`n$schema`n"
    $bodyText = Body-Wire $encoded.bodies
    $candidate = $legendText +
        ($encoded.envelope | ConvertTo-Json -Depth 100 -Compress) +
        "`n§BODIES`n" + $bodyText + $footer
    $candidatePath = Join-Path $directory.FullName "compact-a2.txt"
    [IO.File]::WriteAllText($candidatePath, $candidate, [Text.UTF8Encoding]::new($false))
    foreach ($tokenizer in @("cl100k", "o200k")) {
        $fullTokens = [int](& $measure count $tokenizer $fullPath)
        $rawTokens = [int](& $measure count $tokenizer $rawPath)
        $candidateTokens = [int](& $measure count $tokenizer $candidatePath)
        $records += [ordered]@{
            capture = $directory.Name
            lane = if ($directory.Name.StartsWith("economics-")) { "tracked_economics" } else { "correctness_lifecycle" }
            tokenizer = $tokenizer
            raw_source_tokens = $rawTokens
            control_full_v2_tokens = $fullTokens
            compact_a2_tokens = $candidateTokens
            production_saved_tokens = $rawTokens - $candidateTokens
            production_reduction_percent = if ($rawTokens) { [Math]::Round((($rawTokens - $candidateTokens) * 100.0 / $rawTokens), 2) } else { 0 }
            economical_vs_raw = $candidateTokens -lt $rawTokens
            oracle_saved_tokens = $fullTokens - $candidateTokens
            oracle_reduction_percent = [Math]::Round((($fullTokens - $candidateTokens) * 100.0 / $fullTokens), 2)
            legend_tokens = Count-TextTokens $tokenizer $legendText
            file_mode_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.h -Depth 100 -Compress)
            declaration_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.d -Depth 100 -Compress)
            local_call_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.g -Depth 100 -Compress)
            navigation_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.n -Depth 100 -Compress)
            import_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.i -Depth 100 -Compress)
            type_alias_tokens = Count-TextTokens $tokenizer (ConvertTo-Json -InputObject $encoded.envelope.t -Depth 100 -Compress)
            body_frame_tokens = Count-TextTokens $tokenizer $bodyText
            raw_source_payload = $rawPath
            candidate_payload = $candidatePath
        }
    }
}
if (-not $records.Count) { throw "No CONTROL-FULL v2 captures were measured" }
$output = Join-Path $captures "compact-a2-token-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
foreach ($tokenizer in @("cl100k", "o200k")) {
    $lane = @($records | Where-Object tokenizer -eq $tokenizer)
    $raw = ($lane.raw_source_tokens | Measure-Object -Sum).Sum
    $compact = ($lane.compact_a2_tokens | Measure-Object -Sum).Sum
    $wins = @($lane | Where-Object economical_vs_raw).Count
    Write-Host "${tokenizer}: $($lane.Count) captures, raw $raw -> A2 $compact tokens ($([Math]::Round((($raw-$compact)*100.0/$raw),2))% production reduction; $wins/$($lane.Count) economical)"
    foreach ($laneName in @("correctness_lifecycle", "tracked_economics")) {
        $subset = @($lane | Where-Object lane -eq $laneName)
        if (-not $subset.Count) { continue }
        $subsetRaw = ($subset.raw_source_tokens | Measure-Object -Sum).Sum
        $subsetCompact = ($subset.compact_a2_tokens | Measure-Object -Sum).Sum
        $subsetWins = @($subset | Where-Object economical_vs_raw).Count
        Write-Host "  ${laneName}: $($subset.Count), raw $subsetRaw -> A2 $subsetCompact ($([Math]::Round((($subsetRaw-$subsetCompact)*100.0/$subsetRaw),2))%; $subsetWins/$($subset.Count) economical)"
    }
}
Write-Host "Wrote $output ($($records.Count) records; zero model calls)"
