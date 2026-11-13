param([Parameter(Mandatory)][string]$WorkspaceRoot,[Parameter(Mandatory)][string]$CaptureRoot)
# Static capture inspection, not a host runtime or regression test.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
$capture=(Resolve-Path -LiteralPath $CaptureRoot).Path
$utf8=[Text.UTF8Encoding]::new($false,$true)
$calls=@(Get-Content (Join-Path $capture 'host-events.jsonl')|ForEach-Object {$_|ConvertFrom-Json}|Where-Object {$_.type -eq 'item.completed' -and $_.item.type -eq 'mcp_tool_call'})
$references=@{};$rows=[Collections.Generic.List[object]]::new()
foreach($event in $calls){
    $item=$event.item
    if($item.tool -ne 'pcg_knowledge_v1'){continue}
    if($item.error -or $item.status -ne 'completed'){throw 'Host invocation failed.'}
    $p=$item.result.content[0].text|ConvertFrom-Json
    if(!$p.complete -or !$p.catalog_current -or $p.result_state -ne 'ready' -or $p.diagnostics.Count -ne 0){throw 'Host readiness/currentness/completeness mismatch.'}
    if($p.generation -ne '684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70'){throw 'Host generation changed.'}
    $checked=0
    if($item.arguments.mode -in @('search','get','related','context')){
        if($p.content_acquired -ne $false -or $p.sources.Count -ne 0){throw 'Graph operation acquired source.'}
        foreach($unit in $p.units){
            if($unit.source_text){throw 'Graph source text present.'}
            foreach($component in $unit.components){
                if($component.PSObject.Properties.Name -contains 'text'){throw 'Graph component text present.'}
                $key="$($unit.id)|$($unit.revision)|$($component.ordinal)"
                $references[$key]=[pscustomobject]@{reference=$component.reference;qualifications=@($unit.components|Where-Object role -eq qualification)}
            }
        }
    }
    if($item.arguments.mode -eq 'context'){
        if($p.units.Count -ne 8 -or $p.relationships.Count -ne 7 -or $p.paths.Count -ne 8){throw 'Canonical host graph shape differs.'}
        $members=@('AUTH.CANONICAL_IR','AUTH.CORE_OPS','ARCH-001','ARCH-002','ARCH-003','AUTH-001','IRPAT-001','PIPELINE-001')
        if(Compare-Object $members @($p.units.id)){throw 'Canonical host membership differs.'}
    }
    if($item.arguments.mode -eq 'acquire'){
        if($p.acquisitions.Count -ne $item.arguments.references.Count){throw 'Missing per-reference host acquisition result.'}
        for($index=0;$index -lt $item.arguments.references.Count;$index++){
            $r=$item.arguments.references[$index];$result=$p.acquisitions[$index]
            $key="$($r.id)|$($r.revision)|$($r.ordinal)"
            if(!$references.ContainsKey($key)){throw 'Host did not acquire an actually returned reference.'}
            $original=$references[$key]
            foreach($field in @('id','revision','ordinal','role','label','path','sourceSha256','start','end','contentSha256','selector','historical')){
                if($r.$field -cne $original.reference.$field -or $r.$field -cne $result.reference.$field){throw "Host altered reference $field"}
            }
            if(!$result.complete -or $result.result_state -ne 'ready'){throw 'Selected host acquisition failed.'}
            foreach($qualification in $original.qualifications){
                if(!@($result.components|Where-Object {$_.ordinal -eq $qualification.ordinal -and $_.content_sha256 -eq $qualification.content_sha256}).Count){throw 'Host acquisition lacks required qualification.'}
            }
            foreach($component in $result.components){
                $bytes=[IO.File]::ReadAllBytes((Join-Path $root $component.path))
                $text=$utf8.GetString($bytes,[int]$component.start,[int]($component.end-$component.start))
                if($text -cne $component.text){throw 'Captured host acquisition bytes differ from exact source interval.'}
                $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($utf8.GetBytes($text))).ToLowerInvariant()
                if($hash -ne $component.content_sha256){throw 'Captured host acquisition content hash differs.'}
                $checked++
            }
        }
    }
    if($item.arguments.mode -eq 'source'){
        foreach($source in $p.sources){
            $bytes=[IO.File]::ReadAllBytes((Join-Path $root $source.path))
            if($source.text -cne $utf8.GetString($bytes)){throw 'Captured full source differs.'}
            if((Get-FileHash (Join-Path $root $source.path)).Hash.ToLowerInvariant() -ne $source.source_sha256){throw 'Captured full-source hash differs.'}
        }
    }
    $rows.Add([pscustomobject]@{event=$item.id;mode=$item.arguments.mode;id=$item.arguments.id;references=if($item.arguments.references){$item.arguments.references.Count}else{0};captured_components_checked=$checked;state=$p.result_state;complete=$p.complete;content_acquired=$p.content_acquired;generation=$p.generation})
}
foreach($mode in @('search','get','related','context','acquire','source')){if($mode -notin $rows.mode){throw "Missing actual host mode $mode"}}
if(@($rows|Where-Object {$_.mode -eq 'acquire' -and $_.references -eq 1}).Count -eq 0 -or @($rows|Where-Object {$_.mode -eq 'acquire' -and $_.references -gt 1}).Count -eq 0){throw 'Missing individual/batch host acquisition.'}
$summary=[ordered]@{events=$rows.ToArray();capture='raw host events';boundary='captured payload exactness and actual invocations; not a model-visible delivery/comprehension guarantee';status='verified'}
[IO.File]::WriteAllText((Join-Path $capture 'verified-summary.json'),($summary|ConvertTo-Json -Depth 8),$utf8)
Write-Output "Host capture verified: $capture/verified-summary.json"
