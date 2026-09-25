$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"

function Canonical($value) {
    if ($null -eq $value) { return $null }
    if ($value -is [Management.Automation.PSCustomObject]) {
        $result = [ordered]@{}
        foreach ($property in $value.PSObject.Properties | Sort-Object Name -CaseSensitive) {
            $result[$property.Name] = Canonical $property.Value
        }
        return $result
    }
    if ($value -is [Collections.IDictionary]) {
        $result = [ordered]@{}
        foreach ($key in $value.Keys | Sort-Object -CaseSensitive) {
            $result[$key] = Canonical $value[$key]
        }
        return $result
    }
    if ($value -is [Array]) { return ,@($value | ForEach-Object { Canonical $_ }) }
    return $value
}

function Method-Object($row, $bodies) {
    $id = [string]$row[0]
    $bodyValue = $null
    $bodyStart = $null
    $bodyEnd = $null
    if ($bodies.ContainsKey($id)) {
        $sourceMethod = $bodies[$id]
        $bodyValue = $sourceMethod.body
        $bodyStart = $sourceMethod.body_start
        $bodyEnd = $sourceMethod.body_end
    }
    return [ordered]@{
        id=$row[0]; name=$row[1]; parameters=@($row[2]); return_type=$row[3]
        modifier_occurrences=@($row[4]); control_summary_occurrences=@($row[5])
        pattern_fact_occurrences=@($row[6]); legacy_flag_occurrences=@($row[7])
        patterns=@($row[8]); body=$bodyValue; body_start=$bodyStart; body_end=$bodyEnd
        control_flow=@($row[9]); data_flow=@($row[10]); side_effects=@($row[11])
        execution_contexts=@($row[12])
    }
}

function Entity-Object($row) {
    return [ordered]@{ domain=$row[0]; entity_type=$row[1]; name=$row[2]; file=$row[3] }
}

function Expected-BodyWire($semantic, $bodies) {
    $builder = [Text.StringBuilder]::new()
    foreach ($family in @("classes", "interfaces")) {
        foreach ($owner in $semantic.$family) {
            foreach ($method in $owner.methods) {
                if ($null -eq $method.body) { continue }
                $bytes = [Text.Encoding]::UTF8.GetByteCount([string]$method.body)
                [void]$builder.Append("B $($method.id) $($method.body_start) $($method.body_end) $bytes`n")
                [void]$builder.Append([string]$method.body)
                [void]$builder.Append("`n")
                $bodies[[string]$method.id] = $method
            }
        }
    }
    return $builder.ToString()
}

function Decode-A2($encoded, $bodies) {
    $classes = foreach ($row in $encoded.d.c) {
        [ordered]@{
            kind="class"; id=$row[0]; name=$row[1]; synthetic=$row[2]
            methods=@($row[3] | ForEach-Object { Method-Object $_ $bodies })
            fields=@($row[4]); modifier_occurrences=@($row[5]); class_flag_occurrences=@($row[6])
            extends=$row[7]; implements=@($row[8]); injection_occurrences=@($row[9]); patterns=@($row[10])
        }
    }
    $interfaces = foreach ($row in $encoded.d.i) {
        [ordered]@{
            kind="interface"; id=$row[0]; name=$row[1]
            methods=@($row[2] | ForEach-Object { Method-Object $_ $bodies })
            fields=@($row[3]); modifier_occurrences=@($row[4]); extends=@($row[5])
        }
    }
    $calls = foreach ($row in $encoded.g.K) {
        [ordered]@{
            occurrence=$row[0]; caller_method_id=$row[1]; callee_written_name=$row[2]
            explicit_argument_count=$row[3]; has_spread=$row[4]; callee_resolution=$row[5]
        }
    }
    return [ordered]@{
        schema=$encoded.h[0]; schema_version=$encoded.h[1]; file=$encoded.h[2]; mode=$encoded.h[3]
        classes=@($classes); interfaces=@($interfaces); imports=@($encoded.i)
        type_aliases=@($encoded.t); calls=@($calls); semantic_edges=@()
    }
}

function Expected-NavigationIndex($semantic) {
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
    return [ordered]@{ D=$di; V=@($behavior) }
}

$verified = 0
foreach ($directory in Get-ChildItem -LiteralPath $captures -Directory) {
    $fullPath = Join-Path $directory.FullName "control-full.txt"
    $candidatePath = Join-Path $directory.FullName "compact-a2.txt"
    $rawPath = Join-Path $directory.FullName "raw-source.txt"
    if (-not (Test-Path $fullPath) -or -not (Test-Path $candidatePath)) { continue }
    if (-not (Test-Path $rawPath)) { throw "$($directory.Name): missing capture-time raw source" }
    $full = Get-Content -Raw $fullPath
    if (-not $full.StartsWith("// CONTROL-FULL v2")) { continue }
    $fullRemainder = $full -replace "^[^`r`n]*`r?`n", ""
    $fullPathmap = $fullRemainder.LastIndexOf("`n§PATHMAP", [StringComparison]::Ordinal)
    $semantic = $fullRemainder.Substring(0, $fullPathmap) | ConvertFrom-Json -Depth 100

    $candidate = Get-Content -Raw $candidatePath
    $parts = $candidate.Split(@("`n§BODIES`n"), 2, [StringSplitOptions]::None)
    if ($parts.Count -ne 2) { throw "$($directory.Name): missing BODIES boundary" }
    $header = $parts[0].Split(@("`n"), 3, [StringSplitOptions]::None)
    if ($header.Count -ne 3 -or $header[0] -ne "// COMPACT-A A2; file-local; workspace graph via workspace_query") {
        throw "$($directory.Name): invalid A2 header"
    }
    $encoded = $header[2] | ConvertFrom-Json -Depth 100
    $candidatePathmap = $parts[1].LastIndexOf("`n§PATHMAP", [StringComparison]::Ordinal)
    if ($candidatePathmap -lt 0) { throw "$($directory.Name): missing candidate PATHMAP" }
    $actualBodyWire = $parts[1].Substring(0, $candidatePathmap)
    $bodies = @{}
    $expectedBodyWire = Expected-BodyWire $semantic $bodies
    if ($actualBodyWire -cne $expectedBodyWire) { throw "$($directory.Name): body frame bytes differ" }

    $expected = $semantic | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $expected.PSObject.Properties.Remove("navigation")
    $expected.schema = 'clean-ctx/file-context'
    $expected.schema_version = 1
    $expected.semantic_edges = @()
    $decoded = Decode-A2 $encoded $bodies
    $expectedJson = Canonical $expected | ConvertTo-Json -Depth 100 -Compress
    $decodedJson = Canonical $decoded | ConvertTo-Json -Depth 100 -Compress
    if ($decodedJson -cne $expectedJson) {
        $diagnostics = Join-Path $captures "compact-a2-validation-diagnostics"
        New-Item -ItemType Directory -Force $diagnostics | Out-Null
        [IO.File]::WriteAllText(
            (Join-Path $diagnostics "$($directory.Name)-expected.json"),
            $expectedJson,
            [Text.UTF8Encoding]::new($false)
        )
        [IO.File]::WriteAllText(
            (Join-Path $diagnostics "$($directory.Name)-decoded.json"),
            $decodedJson,
            [Text.UTF8Encoding]::new($false)
        )
        throw "$($directory.Name): decoded semantic object differs; diagnostics written to $diagnostics"
    }
    $expectedIndex = Canonical (Expected-NavigationIndex $semantic) | ConvertTo-Json -Depth 100 -Compress
    $actualIndex = Canonical $encoded.n | ConvertTo-Json -Depth 100 -Compress
    if ($actualIndex -cne $expectedIndex) {
        throw "$($directory.Name): navigation index differs from authoritative facts"
    }
    $verified++
}
if (-not $verified) { throw "No COMPACT-A2 captures were verified" }
Write-Host "PASS: $verified COMPACT-A2 captures decode to the file-local CONTROL-FULL projection; exact body frames match."
