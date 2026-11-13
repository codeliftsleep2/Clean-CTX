param([Parameter(Mandatory)][string]$WorkspaceRoot,[Parameter(Mandatory)][string]$CaptureRoot)
# Static receipt inspection only; no runtime launch.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
$capture=(Resolve-Path -LiteralPath $CaptureRoot).Path
$utf8=[Text.UTF8Encoding]::new($false,$true)
$baseline=Get-Content (Join-Path $root 'target/pcg-b10/baseline.json') -Raw|ConvertFrom-Json
$allowed=@('docs/architecture/PCG_DOCUMENT_AUTHORING.md','docs/agent/pcg-authoring.md')
$graph=Join-Path $capture 'graph'
$requests=@(Get-Content (Join-Path $graph 'queries.json') -Raw|ConvertFrom-Json)
$units=@{}
$changes=[Collections.Generic.List[object]]::new()
$contexts=[Collections.Generic.List[object]]::new()
function Assert-NoContent($value) {
    if($null -eq $value){return}
    if($value -is [Collections.IDictionary]){
        foreach($key in $value.Keys){
            if($key -in @('text','source_text','body','bytes')){throw "Content in graph response: $key"}
            Assert-NoContent $value[$key]
        }
    } elseif($value -is [Array]) {foreach($item in $value){Assert-NoContent $item}}
}
function Payload([string]$directory,[int]$id){
    $r=Get-Content (Join-Path $directory "$id.response.json") -Raw|ConvertFrom-Json
    if($r.error){throw 'Unexpected production protocol error.'}
    return ($r.result.content[0].text|ConvertFrom-Json)
}
$acceptedContexts=Import-Csv (Join-Path $root 'docs/architecture/knowledge-normalization/B9_CONTEXT_COMPLETENESS.csv')
for($i=0;$i -lt $requests.Count;$i++){
    $q=$requests[$i];$p=Payload $graph ($i+2)
    $raw=Get-Content (Join-Path $graph "$($i+2).payload.json") -Raw|ConvertFrom-Json -AsHashtable
    Assert-NoContent $raw
    if($p.version -ne 2 -or $p.content_acquired -ne $false -or !$p.complete -or !$p.catalog_current -or $p.result_state -ne 'ready'){throw 'Graph contract/currentness/completeness mismatch.'}
    if($q.mode -eq 'get'){$units[$q.id]=$p.units[0]}
    if($q.mode -eq 'search' -and $q.text -eq 'PassPipeline'){
        # Compare against the actual accepted B9 witness rather than guessed siblings.
        $before=Get-Content (Join-Path $root 'target/pcg-b10/before/6.payload.json') -Raw|ConvertFrom-Json
        $expected=@($before.units.id)
        if($p.units.Count -ne 4 -or (Compare-Object @($p.units.id) $expected)){throw 'PassPipeline exact four-match witness changed.'}
    }
    if($q.mode -eq 'context'){
        $oracle=@($acceptedContexts|Where-Object id -eq $q.id)[0]
        $expectedMembers=@($oracle.members|ConvertFrom-Json)
        if(Compare-Object @($p.units.id) $expectedMembers){throw "Context membership differs: $($q.id)"}
        $expectedEdges=@($baseline.edges|Where-Object {$_.source -in $expectedMembers -and $_.target -in $expectedMembers}|ForEach-Object {"$($_.source)|$($_.kind)|$($_.target)"}|Sort-Object)
        $actualEdges=@($p.relationships|ForEach-Object {"$($_.source)|$($_.kind)|$($_.target)"}|Sort-Object)
        if(($expectedEdges -join "`n") -cne ($actualEdges -join "`n")){throw 'Context edge identities differ.'}
        if($p.paths.Count -ne $expectedMembers.Count){throw 'Missing context witness.'}
        foreach($path in $p.paths){
            $cursor=$q.id
            foreach($step in $path.steps){
                if("$($step.source)|$($step.kind)|$($step.target)" -notin $actualEdges){throw 'Witness has missing edge.'}
                if($step.direction -eq 'outgoing' -and $cursor -eq $step.source){$cursor=$step.target}
                elseif($step.direction -eq 'incoming' -and $cursor -eq $step.target){$cursor=$step.source}
                else{throw 'Discontinuous witness.'}
            }
            if($cursor -ne $path.node){throw 'Incorrect witness endpoint.'}
        }
        $contexts.Add([pscustomobject]@{id=$q.id;nodes=$p.units.Count;edges=$p.relationships.Count;witnesses=$p.paths.Count;complete=$p.complete;source_content='none';generation=$p.generation})
    }
}
if($units.Count -ne 218){throw 'Not all baseline identities acquired structurally.'}
$edgeKeys=@($units.Values|ForEach-Object {$unit=$_;foreach($edge in $unit.declared_relationships){"$($unit.id)|$($edge.kind)|$($edge.target)"}}|Sort-Object)
$oldEdgeKeys=@($baseline.edges|ForEach-Object {"$($_.source)|$($_.kind)|$($_.target)"}|Sort-Object)
if($edgeKeys.Count -ne 221 -or ($edgeKeys -join "`n") -cne ($oldEdgeKeys -join "`n")){throw 'Baseline edge membership differs.'}
$componentCount=0
foreach($old in $baseline.nodes){
    $unit=$units[$old.id]
    if(!$unit){throw "Missing baseline identity $($old.id)"}
    foreach($field in @('kind','status','title','track')){
        if($unit.$field -cne $old.declaration.$field){throw "Identity metadata changed: $($old.id)/$field"}
    }
    if($unit.revision -ne $old.revision){
        if($old.path -notin $allowed){throw "Unapproved revision change: $($old.id)"}
        $changes.Add([pscustomobject]@{id=$old.id;path=$old.path;old_revision=$old.revision;new_revision=$unit.revision;reason='authorized B9-R1 governing/workflow amendment'})
    }
    if($unit.components.Count -ne $old.components.Count){throw 'Component membership changed.'}
    if($unit.evidence.Count -ne $old.evidence.Count){throw 'Evidence association membership changed.'}
    foreach($association in $unit.evidence){
        $previous=@($old.evidence|Where-Object {$_.kind -ceq $association.kind -and $_.path -ceq $association.path})
        if($previous.Count -ne 1){throw 'Evidence association identity changed.'}
        if($association.path -notin $allowed -and $association.sha256 -cne $previous[0].sha256){throw 'Unapproved evidence source revision change.'}
        if(!$association.source_request){throw 'Missing original evidence acquisition reference.'}
    }
    foreach($component in $unit.components){
        $previous=@($old.components|Where-Object ordinal -eq $component.ordinal)[0]
        foreach($field in @('ordinal','role','label','path')){if($component.$field -cne $previous.$field){throw "Component identity changed: $($old.id)/$field"}}
        if($component.path -notin $allowed){
            foreach($field in @('source_sha256','content_sha256','start','end')){if($component.$field -cne $previous.$field){throw "Unapproved source/boundary change: $($old.id)/$field"}}
        }
        if(!$component.reference -or !$component.acquire_request -or !$component.source_request){throw 'Missing exact source/acquisition reference.'}
        $componentCount++
    }
}
if($componentCount -ne 326){throw 'Baseline component count changed.'}
$acquisition=Join-Path $capture 'acquisition'
$a=Payload $acquisition 2
if(!$a.complete -or $a.acquisitions.Count -ne 326){throw 'Explicit full component verification set failed.'}
$sources=@{};$checked=0
foreach($entry in $a.acquisitions){
    if(!$entry.complete -or $entry.result_state -ne 'ready'){throw 'An explicit reference failed.'}
    $unit=$units[$entry.reference.id]
    $expected=@($unit.components|Where-Object {$_.ordinal -eq $entry.reference.ordinal -or $_.role -eq 'qualification'})
    if($entry.components.Count -ne $expected.Count){throw 'Required qualification membership differs.'}
    foreach($component in $entry.components){
        $descriptor=@($expected|Where-Object ordinal -eq $component.ordinal)[0]
        foreach($field in @('role','label','path','start','end','content_sha256','source_sha256')){if($component.$field -cne $descriptor.$field){throw 'Acquired descriptor differs.'}}
        if(!$sources.ContainsKey($component.path)){$sources[$component.path]=[IO.File]::ReadAllBytes((Join-Path $root $component.path))}
        $bytes=$sources[$component.path]
        $sourceHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
        if($sourceHash -ne $component.source_sha256){throw 'Acquired source artifact hash differs.'}
        $expectedText=$utf8.GetString($bytes,[int]$component.start,[int]($component.end-$component.start))
        if($component.text -cne $expectedText){throw 'Acquired source interval differs.'}
        $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($utf8.GetBytes($component.text))).ToLowerInvariant()
        if($hash -ne $component.content_sha256){throw 'Acquired content hash differs.'}
        $checked++
    }
}
$single=Payload $acquisition 3
if(!$single.complete -or $single.acquisitions.Count -ne 1){throw 'Individual acquisition failed.'}
$mixed=Payload $acquisition 4
if($mixed.complete -or $mixed.result_state -ne 'incomplete' -or $mixed.acquisitions.Count -ne 3 -or $mixed.acquisitions[1].duplicate_of -ne 0 -or $mixed.acquisitions[2].result_state -ne 'refused'){throw 'Duplicate/refusal batch contract failed.'}
$history=Payload $acquisition 5
if(!$history.complete){throw 'Historical pinned component acquisition failed.'}
foreach($entry in $history.acquisitions){
    if($entry.freshness -ne 'historical_snapshot'){throw 'Historical acquisition relabeled current.'}
    $old=@($baseline.nodes|Where-Object id -eq $entry.reference.id)[0]
    foreach($component in $entry.components){$before=@($old.components|Where-Object ordinal -eq $component.ordinal)[0];if($component.text -cne $before.text){throw 'Historical exact component bytes changed.'}}
}
$stale=Payload $acquisition 6
if($stale.complete -or $stale.acquisitions[0].diagnostics[0].code -ne 'source_revision_changed'){throw 'Current reference did not refuse revision drift.'}
$full=Payload $acquisition 7
if($full.sources[0].text -cne $utf8.GetString([IO.File]::ReadAllBytes((Join-Path $root $full.sources[0].path)))){throw 'Full current source differs.'}
$oldFull=Payload $acquisition 8
$oldNode=@($baseline.nodes|Where-Object id -eq 'AUTH.PCG_DOCUMENTS')[0]
$oldHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($utf8.GetBytes($oldFull.sources[0].text))).ToLowerInvariant()
if($oldFull.freshness -ne 'historical_snapshot' -or $oldHash -ne $oldNode.source_sha256){throw 'Historical whole-source hash differs.'}
$summary=[ordered]@{identities=$units.Count;relationships=$edgeKeys.Count;components=$componentCount;authorized_revision_changes=$changes.ToArray();contexts=$contexts.ToArray();explicit_component_references=$a.acquisitions.Count;checked_component_occurrences=$checked;single='exact';duplicate_and_refusal_batch='truthful';historical_components='exact baseline bytes';historical_full_source='exact baseline hash';stale_current_reference='refused';graph_content='none';generation=$a.generation;category='operator production verification; not tracked tests/host adoption/model delivery'}
[IO.File]::WriteAllText((Join-Path $capture 'verification-summary.json'),($summary|ConvertTo-Json -Depth 10),$utf8)
Write-Output "Production capture verified: $capture/verification-summary.json"
