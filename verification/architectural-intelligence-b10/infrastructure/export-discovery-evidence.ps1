param([Parameter(Mandatory)][string]$WorkspaceRoot)
# Bounded inspection/export of existing ten-session captures; no runtime launches.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$capture = Join-Path $root 'target/pcg-b10/discovery-01'
$reports = Join-Path $root 'verification/architectural-intelligence-b10/operator'
$questionsPath = Join-Path $PSScriptRoot 'discovery-questions.csv'
$questions = Import-Csv $questionsPath
$manifest = Get-Content (Join-Path $capture 'manifest.json') -Raw | ConvertFrom-Json
if ($manifest.questions_sha256 -ne (Get-FileHash $questionsPath).Hash.ToLowerInvariant()) { throw 'Question manifest changed.' }
$rows = [Collections.Generic.List[object]]::new()
$sessions = [Collections.Generic.List[object]]::new()
$sourceRows = [Collections.Generic.List[object]]::new()
foreach ($question in $questions) {
    $probe = $question.probe
    $directory = Join-Path $capture $probe
    $eventsPath = Join-Path $directory 'host-events.jsonl'
    $events = @(Get-Content $eventsPath | ForEach-Object { $_ | ConvertFrom-Json })
    $calls = @($events | Where-Object { $_.type -eq 'item.completed' -and $_.item.type -eq 'mcp_tool_call' })
    $startOrder = @{}
    $sequence = 0
    foreach ($event in $events) {
        if ($event.type -eq 'item.started' -and $event.item.type -eq 'mcp_tool_call') {
            $sequence++
            $startOrder[$event.item.id] = $sequence
        }
    }
    foreach ($call in $calls) {
        $item = $call.item
        if ($item.tool -eq 'pcg_knowledge_v1') {
            if ($item.status -ne 'completed' -or $item.error) { throw "Knowledge invocation failed: $probe" }
            $p = $item.result.content[0].text | ConvertFrom-Json
            $rows.Add([pscustomobject]@{
                probe=$probe;event=$item.id;start_sequence=$startOrder[$item.id]
                mode=$item.arguments.mode;id=$item.arguments.id
                arguments=($item.arguments|ConvertTo-Json -Compress)
                state=$p.result_state;current=$p.catalog_current;complete=$p.complete
                matched_or_acquired_identities=(@($p.units|ForEach-Object {$_.id}) -join ';')
                unit_count=@($p.units).Count;edges=@($p.relationships).Count;witnesses=@($p.paths).Count
                match_origins=(@($p.units|ForEach-Object {$_.matches}|ForEach-Object {$_.origin}|Sort-Object -Unique) -join ';')
                typed_edges=(@($p.relationships|ForEach-Object {"$($_.source)|$($_.kind)|$($_.target)"}) -join ';')
                requested_component_roles=(@($p.units|ForEach-Object {$_.components}|ForEach-Object {$_.role}|Sort-Object -Unique) -join ';')
                diagnostics=@($p.diagnostics).Count;generation=$p.generation
                evidence="target/pcg-b10/discovery-01/$probe/host-events.jsonl"
            })
        }
        if ($item.tool -eq 'provide_code_context') {
            $files = @($item.arguments.files)
            if ($item.arguments.filePath) { $files += [pscustomobject]@{filePath=$item.arguments.filePath} }
            foreach ($file in $files) {
                if ($file.filePath) {
                    $sourceRows.Add([pscustomobject]@{probe=$probe;event=$item.id;requested_path=$file.filePath;tool=$item.tool;transport_status=$item.status;limit='requested path is not proof of complete agent inspection'})
                }
            }
        }
    }
    $knowledge = @($rows | Where-Object probe -eq $probe)
    $sessions.Add([pscustomobject]@{
        probe=$probe;question=$question.question
        exit_code=(Get-Content (Join-Path $directory 'exit-code.txt'))
        completed_mcp_calls=$calls.Count;knowledge_calls=$knowledge.Count
        chosen_searches=(@($knowledge|Where-Object mode -eq search|ForEach-Object {$_.arguments}) -join ' | ')
        empty_search_requests=(@($knowledge|Where-Object {$_.mode -eq 'search' -and $_.unit_count -eq 0}|ForEach-Object {$_.arguments}) -join ' | ')
        acquired_identities=(@($knowledge|Where-Object {$_.mode -in @('get','context')}|ForEach-Object {$_.matched_or_acquired_identities -split ';'}|Sort-Object -Unique) -join ';')
        contexts=(@($knowledge|Where-Object mode -eq context|ForEach-Object {$_.id}|Sort-Object -Unique) -join ';')
        match_origins=(@($knowledge|ForEach-Object {$_.match_origins -split ';'}|Where-Object {$_}|Sort-Object -Unique) -join ';')
        tool_transport_errors=@($calls|Where-Object {$_.item.error}).Count
        result='completed operational session; semantic disposition reviewed separately'
        intervention='no coaching or retry recorded during execution'
        independent_evaluation='ineligible/unestablished; not a frozen B7 run'
        prompt_sha256=(Get-FileHash (Join-Path $directory 'prompt.txt')).Hash.ToLowerInvariant()
        events_sha256=(Get-FileHash $eventsPath).Hash.ToLowerInvariant()
        response_sha256=(Get-FileHash (Join-Path $directory 'agent-response.md')).Hash.ToLowerInvariant()
    })
}
$rows | Export-Csv (Join-Path $reports 'B10_DISCOVERY_CALLS.csv') -NoTypeInformation -Encoding utf8NoBOM
$sessions | Export-Csv (Join-Path $reports 'B10_DISCOVERY_PROBES.csv') -NoTypeInformation -Encoding utf8NoBOM
$sourceRows | Export-Csv (Join-Path $reports 'B10_DISCOVERY_SOURCE_REQUESTS.csv') -NoTypeInformation -Encoding utf8NoBOM
[pscustomobject]@{
    completed_sessions=$sessions.Count;knowledge_calls=$rows.Count
    empty_search_requests=@($rows|Where-Object {$_.mode -eq 'search' -and $_.unit_count -eq 0}).Count
    knowledge_not_ready=@($rows|Where-Object state -ne ready).Count
    generation_changes=@($rows|Where-Object generation -ne '80c1195a7a9df253e4221635bd3feb8201d15ac4e0fbc328425e59cbe04b03e5').Count
} | ConvertTo-Json
