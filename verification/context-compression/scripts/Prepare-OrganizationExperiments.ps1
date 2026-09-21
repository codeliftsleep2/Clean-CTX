$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$outputRoot = Join-Path $repositoryRoot "target\context-compression-verification\organization-experiments"
$manifest = Get-Content -Raw (Join-Path $definitionRoot "investigation\organization-experiments-v1.json") | ConvertFrom-Json -Depth 100
New-Item -ItemType Directory -Force $outputRoot | Out-Null

function Split-ControlFull([string]$text) {
    $jsonStart = $text.IndexOf("{")
    $pathStart = $text.IndexOf("`n§PATHMAP")
    if ($jsonStart -lt 0 -or $pathStart -lt 0) { throw "CONTROL-FULL framing not recognized" }
    return @{
        Header = $text.Substring(0, $jsonStart)
        Json = $text.Substring($jsonStart, $pathStart - $jsonStart).TrimEnd()
        Trailer = $text.Substring($pathStart)
    }
}

foreach ($experiment in $manifest.experiments) {
    $source = Join-Path $captures "$($experiment.capture)\control-full.txt"
    if (-not (Test-Path $source)) { throw "Missing capture: $source" }
    $parts = Split-ControlFull (Get-Content -Raw $source)
    $payload = $parts.Json | ConvertFrom-Json -Depth 100
    $index = [ordered]@{
        experiment = $experiment.id
        instruction = "Navigation paths only; the referenced records remain authoritative."
        paths = @()
    }
    if ($experiment.id -eq "di-provenance-index") {
        for ($i = 0; $i -lt @($payload.semantic_edges).Count; $i++) {
            $edge = $payload.semantic_edges[$i]
            if ($edge.relation -eq "Injects") {
                $index.paths += [ordered]@{
                    locator = [ordered]@{
                        collection = "semantic_edges"
                        occurrence = $edge.occurrence
                        relation = $edge.relation
                        layer = $edge.layer
                        subject = [ordered]@{
                            domain = $edge.subject.domain
                            entity_type = $edge.subject.entity_type
                            name = $edge.subject.name
                        }
                        object = [ordered]@{
                            domain = $edge.object.domain
                            entity_type = $edge.object.entity_type
                            name = $edge.object.name
                        }
                    }
                    endpoint_fields = [ordered]@{
                        subject_file = [ordered]@{ endpoint = "subject"; field = "file" }
                        object_file = [ordered]@{ endpoint = "object"; field = "file" }
                    }
                }
            }
        }
    } elseif ($experiment.id -eq "occurrence-group-index") {
        for ($ci = 0; $ci -lt @($payload.classes).Count; $ci++) {
            for ($mi = 0; $mi -lt @($payload.classes[$ci].methods).Count; $mi++) {
                $method = $payload.classes[$ci].methods[$mi]
                $fields = @("modifier_occurrences", "control_summary_occurrences", "pattern_fact_occurrences", "legacy_flag_occurrences") |
                    Where-Object { @($method.$_).Count -gt 0 }
                if ($fields.Count) {
                    foreach ($field in $fields) {
                        $index.paths += [ordered]@{
                            locator = [ordered]@{
                                owner = [ordered]@{ kind = "class"; id = $payload.classes[$ci].id }
                                member = [ordered]@{ kind = "method"; id = $method.id }
                                field = [ordered]@{ kind = "occurrence_group_array"; name = $field }
                            }
                            semantics = [ordered]@{
                                outer_array = "ordered_occurrences"
                                inner_array = "one_occurrence_group"
                                duplicates_significant = $true
                                empty_groups_significant = $true
                            }
                        }
                    }
                }
            }
        }
    } else { throw "Unknown experiment $($experiment.id)" }
    $payload | Add-Member -NotePropertyName _organization -NotePropertyValue $index
    $variant = $parts.Header + ($payload | ConvertTo-Json -Depth 100) + $parts.Trailer
    $destination = Join-Path $outputRoot "$($experiment.id).txt"
    [IO.File]::WriteAllText($destination, $variant, [Text.UTF8Encoding]::new($false))
    Write-Host "Prepared $destination"
}
