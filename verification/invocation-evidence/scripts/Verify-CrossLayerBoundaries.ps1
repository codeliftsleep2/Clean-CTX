# Mixed questions share one live server/workspace. General batch protocol is
# already covered elsewhere; these assertions protect the changed payloads.
foreach ($mode in @('mirrored','structured','indexed')) {
    Invoke-Case "phase7-cross-file-layer-context-$mode" {
        $files = @('Reactive.ts','Owners.cs','account.karma.spec.ts','account.vitest.spec.ts')
        $fidelities = @('high','edit','medium','high')
        $items = @(); $expected = @()
        for ($i=0; $i -lt $files.Count; $i++) {
            $items += @{id="item-$i";filePath=(Join-Path $workspace $files[$i]);fidelity=$fidelities[$i]}
            $single = Invoke-Live "cross-$mode-single-$i" 'provide_code_context' @{
                workspaceRoot=$workspace;filePath=(Join-Path $workspace $files[$i]);fidelity=$fidelities[$i];tokenizer='o200k'
            }
            $expected += [string]$single.result.content[0].text
        }
        $batch = Invoke-Live "cross-$mode-batch" 'provide_code_context' @{
            workspaceRoot=$workspace;tokenizer='o200k';responseMode=$mode;files=$items
        }
        $results = @($batch.result.structuredContent.results)
        Assert-Live ($results.Count -eq 4) 'Mixed-layer batch lost an item.'
        for ($i=0; $i -lt $results.Count; $i++) {
            $item=$results[$i]
            Assert-Live ($item.status -eq 'ok' -and $item.id -eq "item-$i") 'Mixed-layer item ownership/order changed.'
            $text = if ($mode -eq 'structured') { [string]$item.content[0].text } else { [string]$batch.result.content[$item.content_index].text }
            Assert-Live ($text -ceq $expected[$i]) 'Mixed-file/layer result differs from its isolated single-tool answer.'
            if ($mode -eq 'mirrored') { Assert-Live ($text -ceq [string]$item.content[0].text) 'Mixed-layer mirrored result changed.' }
        }
    }
}
Invoke-Case 'phase7-interleaved-layer-questions' {
    $before = @(Pipes (Compress 'interleave-rx-before' 'Reactive.ts'))
    $ts = Calls 'interleave-ts' 'Reactive.ts' 'Reactive' 'run'
    Assert-Call @(Flat-Calls $ts) 'Save' 1 2 $true
    foreach ($runner in @('karma','vitest')) {
        $file="account.$runner.spec.ts"
        $null = Compress "interleave-$runner" $file 'medium'
        $edge=Invoke-Live "interleave-$runner-edges" 'workspace_query' @{
            type='forward_edges';workspaceRoot=$workspace;domain='angular';entity_type='TestArtifact';name=$file;relation='Tests'
        }
        $edges=@($edge.result.structuredContent.edges)
        Assert-Live ($edges.Count -eq 1 -and $edges[0].object.name -eq 'AccountComponent') 'Testing layer answer contaminated.'
        Assert-Call @(Flat-Calls (Calls "interleave-cs-$runner" 'Peer.cs' 'Alpha')) 'OnlyPeer' 1 0
    }
    $after = @(Pipes (Compress 'interleave-rx-after' 'Reactive.ts'))
    Assert-Live (($before -join "`n") -ceq ($after -join "`n")) 'Other layer requests changed RxJS facts.'
}
Invoke-Case 'phase7-cross-file-written-reference-truthfulness' {
    Write-Utf8 (Join-Path $workspace 'Target.ts') 'export class Target { Save(item) { Persist(item); } }'
    Write-Utf8 (Join-Path $workspace 'Caller.ts') "import {Target} from './Target'; class Caller { Run() { this.target.Save(item); } }"
    Assert-Sequence (Calls 'cross-caller' 'Caller.ts' 'Caller') @('Save') @(1)
    Assert-Sequence (Calls 'cross-target' 'Target.ts' 'Target' 'Save') @('Persist') @(1)
    Write-Utf8 (Join-Path $workspace 'Target.ts') 'export class Target { Save(item) { Replace(item); } }'
    Assert-Sequence (Calls 'cross-target-edited' 'Target.ts' 'Target' 'Save') @('Replace') @(1)
    Assert-Sequence (Calls 'cross-caller-unchanged' 'Caller.ts' 'Caller') @('Save') @(1)
    # This API exposes written calls; it must not inject another file's body or
    # promote a source reference into a resolved implementation identity.
}
Invoke-Case 'phase7-cross-layer-edit-isolation' {
    $path=Join-Path $workspace 'Reactive.ts'; $original=[IO.File]::ReadAllText($path)
    try {
        Write-Utf8 $path ($original.Replace(' . pipe /* written boundary */ ','.skip').Replace('.pipe(','.skip('))
        Assert-Live (@(Pipes (Compress 'cross-edit-rx' 'Reactive.ts')).Count -eq 0) 'Edited Rx layer remained stale.'
        Assert-Call @(Flat-Calls (Calls 'cross-edit-csharp' 'Peer.cs' 'Alpha')) 'OnlyPeer' 1 0
        foreach ($runner in @('karma','vitest')) {
            $file="account.$runner.spec.ts"
            $reply=Compress "cross-edit-$runner" $file
            Assert-Live (@($reply.result.pretty.ir | Where-Object { $_[0] -eq 'TYPE' -and $_[1] -eq '@testBed' }).Count -gt 0) 'Unrelated testing metadata was removed by Rx edit.'
        }
    } finally { Write-Utf8 $path $original }
    Assert-Live (@(Pipes (Compress 'cross-edit-restored' 'Reactive.ts')).Count -eq 3) 'Restored Rx layer was not fresh.'
}
Invoke-Case 'phase7-mixed-layer-query-batch-and-freshness' {
    $path=Join-Path $workspace 'Mixed.cs'
    Write-Utf8 $path 'class Mixed { void Run() { Alpha(); } }'
    $queries=@(
        @{id='ts';type='calls_in_file';filePath=(Join-Path $workspace 'Reactive.ts');owner=@{kind='class';name='Reactive'};method=@{name='run'}},
        @{id='karma';type='forward_edges';domain='angular';entity_type='TestArtifact';name='account.karma.spec.ts';relation='Tests'},
        @{id='cs';type='calls_in_file';filePath=$path;owner=@{kind='class';name='Mixed'};method=@{name='Run'}},
        @{id='vitest';type='forward_edges';domain='angular';entity_type='TestArtifact';name='account.vitest.spec.ts';relation='Tests'},
        @{id='bad';type='calls_in_file';filePath=(Join-Path $workspace 'missing.cs');owner=@{kind='class';name='Mixed'};method=@{name='Run'}}
    )
    foreach ($callee in @('Alpha','Bravo')) {
        Write-Utf8 $path "class Mixed { void Run() { $callee(); } }"
        $batch=Invoke-Live "cross-query-batch-$callee" 'workspace_query' @{workspaceRoot=$workspace;queries=$queries}
        $items=@($batch.result.structuredContent.results)
        Assert-Live (($items.id -join ',') -eq 'ts,karma,cs,vitest,bad') 'Heterogeneous query order/identity changed.'
        Assert-Live (@($items[0..3] | Where-Object {$_.status -ne 'ok'}).Count -eq 0) 'A valid layer question failed.'
        Assert-Sequence $items[2].result @($callee) @(0)
        Assert-Call @(Flat-Calls $items[0].result) 'Save' 1 2 $true
        foreach ($i in @(1,3)) {
            $edges=@($items[$i].result.edges)
            Assert-Live ($edges.Count -eq 1 -and $edges[0].object.name -eq 'AccountComponent') 'Batch graph layer lost component linkage.'
        }
        Assert-Live ($items[4].status -eq 'error' -and -not $items[4].PSObject.Properties['result']) 'Missing file leaked a successful or cross-item result.'
        $text=[string]$batch.result.content[0].text
        $envelope=$text.Substring($text.IndexOf('{')) | ConvertFrom-Json -Depth 100
        $visible=@($envelope.results)
        Assert-Live (($visible.id -join ',') -eq ($items.id -join ',')) 'Text batch identities differ.'
        foreach ($i in @(0,2)) {
            Assert-Live (($visible[$i].result | ConvertTo-Json -Depth 100 -Compress) -ceq ($items[$i].result | ConvertTo-Json -Depth 100 -Compress)) 'Written-call text/structured answers disagree.'
            Assert-Live ($visible[$i].completeness.authority -eq 'fresh_canonical_file_ir') 'Fresh canonical authority omitted from text.'
        }
        foreach ($i in @(1,3)) {
            # Text adds occurrence indexes and envelope completeness. Compare
            # semantic edge payloads, not intentionally different wire shapes.
            $edge=@($visible[$i].result.edges)[0]; $structured=@($items[$i].result.edges)[0]
            foreach ($key in @('subject','object','relation','layer')) {
                Assert-Live (($edge.$key | ConvertTo-Json -Depth 30 -Compress) -ceq ($structured.$key | ConvertTo-Json -Depth 30 -Compress)) 'Graph text/structured facts disagree.'
            }
            Assert-Live ($visible[$i].completeness.result_semantics -eq 'lower_bound') 'Text falsely claims graph completeness.'
        }
        Assert-Live ($visible[4].status -eq 'error') 'Text hid the isolated missing-file failure.' 
    }
}
Invoke-Case 'phase7-csharp-cross-file-reference-isolation' {
    Write-Utf8 (Join-Path $workspace 'Target.cs') 'class Target { void Save(object item) { Persist(item); } }'
    Write-Utf8 (Join-Path $workspace 'Caller.cs') 'class Caller { void Run() { target.Save(item); } }'
    Assert-Sequence (Calls 'cross-cs-caller' 'Caller.cs' 'Caller') @('Save') @(1)
    Assert-Sequence (Calls 'cross-cs-target' 'Target.cs' 'Target' 'Save') @('Persist') @(1)
    Write-Utf8 (Join-Path $workspace 'Target.cs') 'class Target { void Save(object item) { Replace(item); } }'
    Assert-Sequence (Calls 'cross-cs-target-edited' 'Target.cs' 'Target' 'Save') @('Replace') @(1)
    Assert-Sequence (Calls 'cross-cs-caller-unchanged' 'Caller.cs' 'Caller') @('Save') @(1)
}
Invoke-Case 'phase7-same-source-angular-ngrx-rx-composition' {
    $source=@'
import { Injectable } from '@angular/core';
import { createAction } from '@ngrx/store';
import {map,tap} from 'rxjs/operators';
const loadUsers = createAction('[User] Load Users');
@Injectable({providedIn:'root'})
export class LayeredService {
  value$ = source.pipe(map(x => x),tap(log));
  Run() { Save(item); }
}
'@
    Write-Utf8 (Join-Path $workspace 'Layered.ts') $source
    $reply=Compress 'cross-same-source' 'Layered.ts'
    $aliases=@($reply.result.pretty.ir | Where-Object { $_[0] -eq 'TYPE' })
    foreach ($name in @('@svc','@action','@pipeRx')) {
        Assert-Live (@($aliases | Where-Object { $_[1] -eq $name }).Count -gt 0) "Same-source composition lost $name."
    }
    Assert-Live (@(Pipes $reply).Count -eq 1) 'Same-source composition duplicated reactive chains.'
    Assert-Sequence (Calls 'cross-same-source-calls' 'Layered.ts' 'LayeredService') @('Save') @(1)
    $edge=Invoke-Live 'cross-same-source-entity' 'workspace_query' @{
        type='find_entities';workspaceRoot=$workspace;domain='angular';entity_type='Service';name='LayeredService'
    }
    Assert-Live ($edge.result.structuredContent.count -eq 1) 'Composed compilation lost or duplicated its Angular service identity.'
}
Invoke-Case 'phase6-rx-durable-marker-handoff' {
    try {
        Stop-CleanCtxSession $session; $script:session=$null
        Write-Config
        $configPath=Join-Path $workspace '.clean-ctx.json'
        $config=[IO.File]::ReadAllText($configPath) | ConvertFrom-Json -AsHashtable
        $config.persistence=@{enabled=$true;auto_save=$true;db_path=(Join-Path $workspace 'rx-durable.db')}
        Write-Utf8 $configPath ($config | ConvertTo-Json -Depth 30)
        $script:session=Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
        $before=@(Pipes (Compress 'durable-rx-baseline' 'Reactive.ts'))
        Assert-Live ($before.Count -eq 3) 'Durable baseline omitted reactive facts.'
        Stop-CleanCtxSession $session; $script:session=$null
        $script:session=Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
        $restored=Invoke-Live 'durable-rx-restored' 'restore_context' @{
            workspaceRoot=$workspace;filePath=(Join-Path $workspace 'Reactive.ts')
        }
        $json=$restored | ConvertTo-Json -Depth 100 -Compress
        foreach ($marker in $before) {
            $encoded=ConvertTo-Json -InputObject $marker -Compress
            Assert-Live ($json.Contains($encoded)) 'Durable restore lost a complete multiline reactive marker.'
        }
    } finally {
        if ($session) { Stop-CleanCtxSession $session; $script:session=$null }
        Write-Config
        $script:session=Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
    }
}
