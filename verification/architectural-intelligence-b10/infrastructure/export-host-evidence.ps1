param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [string]$HostCaptureRoot
)
# Bounded read-only inspection of existing captures; starts no host or executable.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$capture = Join-Path $root 'target/pcg-b10/schema-production-host-01'
$reports = Join-Path $root 'verification/architectural-intelligence-b10/operator'
$baseline = Get-Content (Join-Path $root 'target/pcg-b10/baseline.json') -Raw | ConvertFrom-Json
$acceptedContexts = Import-Csv (Join-Path $root 'docs/architecture/knowledge-normalization/B9_CONTEXT_COMPLETENESS.csv')
$nodes = @{}
foreach ($node in $baseline.nodes) { $nodes[$node.id] = $node }
if (!$HostCaptureRoot) { $HostCaptureRoot = Join-Path $capture 'host' }
$events = Get-Content (Join-Path $HostCaptureRoot 'host-events.jsonl') | ForEach-Object { $_ | ConvertFrom-Json }
$calls = @($events | Where-Object { $_.type -eq 'item.completed' -and $_.item.type -eq 'mcp_tool_call' })
$rows = [Collections.Generic.List[object]]::new()
$search = $null
$get = $null
foreach ($event in $calls) {
    $item = $event.item
    if ($item.status -ne 'completed' -or $item.error) { throw 'Host call failed.' }
    $p = $item.result.content[0].text | ConvertFrom-Json
    if ($p.result_state -ne 'ready' -or !$p.complete -or !$p.catalog_current -or $p.diagnostics.Count -ne 0) {
        throw 'Host payload not ready/current/complete.'
    }
    if ($p.generation -ne $baseline.generation) { throw 'Host generation changed.' }
    foreach ($unit in $p.units) {
        $node = $nodes[$unit.id]
        if (!$node -or $node.revision -ne $unit.revision) { throw 'Host revision differs.' }
        foreach ($component in $unit.components) {
            $original = @($node.components | Where-Object ordinal -eq $component.ordinal)
            if ($original.Count -ne 1) { throw 'Host component ordinal differs.' }
            foreach ($field in @('role','label','path','start','end','source_sha256','content_sha256','text')) {
                if ($component.$field -cne $original[0].$field) { throw "Host exact component differs: $($unit.id)/$field" }
            }
        }
    }
    $referenceCheck = 'not applicable'
    switch ($item.arguments.mode) {
        'search' { $search = $p }
        'get' {
            $get = $p
            $ref = @($search.units | Where-Object id -eq $item.arguments.id)[0].acquire_request
            foreach ($field in @('mode','id','workspaceRoot')) {
                if ($item.arguments.$field -cne $ref.$field) { throw 'Host get did not follow returned reference.' }
            }
            $referenceCheck = 'exact returned acquisition reference'
        }
        'related' {
            $expected = @($baseline.edges | Where-Object { $_.source -eq $item.arguments.id -or $_.target -eq $item.arguments.id })
            if ($p.relationships.Count -ne $expected.Count) { throw 'Related edge membership differs.' }
            foreach ($edge in $p.relationships) {
                if (!@($expected | Where-Object { $_.source -eq $edge.source -and $_.kind -eq $edge.kind -and $_.target -eq $edge.target }).Count) { throw 'Unexpected related edge.' }
                $direction = if ($edge.source -eq $item.arguments.id) { 'outgoing' } else { 'incoming' }
                if ($edge.direction -ne $direction) { throw 'Relationship direction differs.' }
            }
        }
        'context' {
            $ids = @($p.units.id)
            $accepted = @($acceptedContexts | Where-Object id -eq $item.arguments.id)
            if ($accepted.Count -ne 1) { throw 'No accepted B9 closure oracle for the selected context.' }
            $expectedMembers = @($accepted[0].members | ConvertFrom-Json)
            if (Compare-Object $ids $expectedMembers) { throw 'Host context membership differs from accepted closure.' }
            $expected = @($baseline.edges | Where-Object { $_.source -in $ids -and $_.target -in $ids })
            $a = @($p.relationships | ForEach-Object { "$($_.source)|$($_.kind)|$($_.target)" } | Sort-Object)
            $b = @($expected | ForEach-Object { "$($_.source)|$($_.kind)|$($_.target)" } | Sort-Object)
            if (($a -join "`n") -cne ($b -join "`n")) { throw 'Host context edge membership differs.' }
            if ($p.paths.Count -ne $ids.Count -or (Compare-Object @($p.paths.node | Sort-Object -Unique) ($ids | Sort-Object))) { throw 'Host witness membership differs.' }
            foreach ($path in $p.paths) {
                $cursor = $item.arguments.id
                foreach ($step in $path.steps) {
                    if (!@($expected | Where-Object { $_.source -eq $step.source -and $_.kind -eq $step.kind -and $_.target -eq $step.target }).Count) { throw 'Witness edge absent.' }
                    if ($step.direction -eq 'outgoing' -and $cursor -eq $step.source) { $cursor = $step.target }
                    elseif ($step.direction -eq 'incoming' -and $cursor -eq $step.target) { $cursor = $step.source }
                    else { throw 'Witness direction is discontinuous.' }
                }
                if ($cursor -ne $path.node) { throw 'Witness endpoint differs.' }
            }
        }
        'source' {
            $ref = $get.units[0].source_request
            foreach ($field in @('mode','id','workspaceRoot','sourceSha256')) {
                if ($item.arguments.$field -cne $ref.$field) { throw 'Host source did not follow returned reference.' }
            }
            foreach ($source in $p.sources) {
                $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($source.text))).ToLowerInvariant()
                if ($hash -ne $source.source_sha256 -or $hash -ne (Get-FileHash (Join-Path $root $source.path)).Hash.ToLowerInvariant()) { throw 'Host source bytes/hash differ.' }
            }
            $referenceCheck = 'exact returned source reference and current artifact bytes'
        }
    }
    $rows.Add([pscustomobject]@{event=$item.id;mode=$item.arguments.mode;id=$item.arguments.id;state=$p.result_state;current=$p.catalog_current;complete=$p.complete;units=$p.units.Count;edges=$p.relationships.Count;paths=$p.paths.Count;references=$referenceCheck;generation=$p.generation;evidence=(Join-Path $HostCaptureRoot 'host-events.jsonl');category='actual host/operator verification; not independent evaluation'})
}
if (@($rows.mode | Sort-Object -Unique).Count -ne 5) { throw 'Five-mode host coverage incomplete.' }
$rows | Export-Csv (Join-Path $reports 'B10_HOST_INVOCATION_VERIFICATION.csv') -NoTypeInformation -Encoding utf8NoBOM
$parity = foreach ($id in 2..13) {
    $before = (Get-FileHash (Join-Path $root "target/pcg-b10/before/$id.serialized-result.json")).Hash.ToLowerInvariant()
    $after = (Get-FileHash (Join-Path $capture "production/$id.serialized-result.json")).Hash.ToLowerInvariant()
    if ($before -ne $after) { throw "Production result bytes differ: $id" }
    [pscustomobject]@{request=$id;before_sha256=$before;after_sha256=$after;result='byte-exact serialized MCP result';scope='existing representative production operations'}
}
$parity | Export-Csv (Join-Path $reports 'B10_PRODUCTION_RESPONSE_PARITY.csv') -NoTypeInformation -Encoding utf8NoBOM
[pscustomobject]@{host_calls=$calls.Count;covered_modes=@($rows.mode|Sort-Object -Unique).Count;byte_exact_production_results=@($parity).Count} | ConvertTo-Json
