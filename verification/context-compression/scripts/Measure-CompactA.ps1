$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not (Test-Path $measure)) { throw "Run Build-MeasureHelper.ps1 first" }

$schema = "S A1 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution] E[occurrence,relation,S(domain,type,name,file),O(domain,type,name,file),layer,call_evidence]. N.D[owner_id,ordered_core_injection_groups]; NO_CORE_INJECTION_OCCURRENCES is authoritative; constructor parameters are signatures and NEVER injection evidence; duplicates significant. N.V tagged rows: mod,cs,pf,lf,pt,cf,df,se,ec. N.E framework relation direction is subject -> object; subject_file and object_file are independent. B[method_id,start,end,utf8_bytes]"

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

function Entity-Row($entity) {
    return ,@($entity.domain, $entity.entity_type, $entity.name, $entity.file)
}

function Call-Row($call) {
    return ,@(
        $call.occurrence, $call.caller_method_id, $call.callee_written_name,
        $call.explicit_argument_count, $call.has_spread, $call.callee_resolution
    )
}

function Edge-Row($edge) {
    return ,@(
        $edge.occurrence, $edge.relation, (Entity-Row $edge.subject),
        (Entity-Row $edge.object), $edge.layer, $edge.call_evidence
    )
}

function Encode-A1($semantic) {
    $bodyFrames = [System.Collections.Generic.List[object]]::new()
    $classes = @($semantic.classes | ForEach-Object { Class-Row $_ $bodyFrames })
    $interfaces = @($semantic.interfaces | ForEach-Object { Interface-Row $_ $bodyFrames })
    $calls = @($semantic.calls | ForEach-Object { Call-Row $_ })
    $edges = @($semantic.semantic_edges | ForEach-Object { Edge-Row $_ })
    $di = @($semantic.classes | ForEach-Object {
        ,@($_.id, $(if (@($_.injection_occurrences).Count) { @($_.injection_occurrences) } else { 'NO_CORE_INJECTION_OCCURRENCES' }))
    })
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
                    if (@($entry[1]).Count) { $behavior.Add(@($method.id, $entry[0], $entry[1])) }
                }
            }
    ) | Out-Null
    $edgeNavigation = @($semantic.semantic_edges | Where-Object { $_.layer -ne 'builtin' } | ForEach-Object {
        [ordered]@{ occurrence=$_.occurrence; relation=$_.relation; subject_name=$_.subject.name; subject_file=$_.subject.file; object_name=$_.object.name; object_file=$_.object.file; layer=$_.layer }
    })
    $envelope = [ordered]@{
        A = 1
        h = @($semantic.schema, $semantic.schema_version, $semantic.file, $semantic.mode)
        d = [ordered]@{ A = 1; c = $classes; i = $interfaces }
        g = [ordered]@{ K = $calls; E = $edges }
        n = [ordered]@{ D = $di; V = @($behavior); E = $edgeNavigation }
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
    if (-not (Test-Path $fullPath)) { continue }
    $fullText = Get-Content -Raw $fullPath
    if (-not $fullText.StartsWith("// CONTROL-FULL v2")) { continue }
    $remainder = $fullText -replace "^[^`r`n]*`r?`n", ""
    $pathmap = $remainder.LastIndexOf("`n§PATHMAP", [StringComparison]::Ordinal)
    if ($pathmap -lt 0) { throw "$($directory.Name): missing PATHMAP footer" }
    $semantic = $remainder.Substring(0, $pathmap) | ConvertFrom-Json -Depth 100
    $footer = $remainder.Substring($pathmap)
    $encoded = Encode-A1 $semantic
    $candidate = "// COMPACT-A A1; decodes to normalized CONTROL-FULL v2`n$schema`n" +
        ($encoded.envelope | ConvertTo-Json -Depth 100 -Compress) +
        "`n§BODIES`n" + (Body-Wire $encoded.bodies) + $footer
    $candidatePath = Join-Path $directory.FullName "compact-a1.txt"
    [IO.File]::WriteAllText($candidatePath, $candidate, [Text.UTF8Encoding]::new($false))
    foreach ($tokenizer in @("cl100k", "o200k")) {
        $fullTokens = [int](& $measure count $tokenizer $fullPath)
        $candidateTokens = [int](& $measure count $tokenizer $candidatePath)
        $records += [ordered]@{
            capture = $directory.Name
            tokenizer = $tokenizer
            control_full_v2_tokens = $fullTokens
            compact_a1_tokens = $candidateTokens
            saved_tokens = $fullTokens - $candidateTokens
            reduction_percent = [Math]::Round((($fullTokens - $candidateTokens) * 100.0 / $fullTokens), 2)
            candidate_payload = $candidatePath
        }
    }
}
if (-not $records.Count) { throw "No CONTROL-FULL v2 captures were measured" }
$output = Join-Path $captures "compact-a1-token-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
foreach ($tokenizer in @("cl100k", "o200k")) {
    $lane = @($records | Where-Object tokenizer -eq $tokenizer)
    $full = ($lane.control_full_v2_tokens | Measure-Object -Sum).Sum
    $compact = ($lane.compact_a1_tokens | Measure-Object -Sum).Sum
    Write-Host "${tokenizer}: $($lane.Count) captures, $full -> $compact tokens ($([Math]::Round((($full-$compact)*100.0/$full),2))% reduction)"
}
Write-Host "Wrote $output ($($records.Count) records; zero model calls)"
