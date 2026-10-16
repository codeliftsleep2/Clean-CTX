# Dot-sourced by the existing live driver. Every scenario uses registered MCP
# tools against isolated source, with exact positive and negative expectations.
# Private evidence fields and parse counts remain tracked-test contracts.
function Assert-Sequence($Answer, [string[]]$Names, [int[]]$Arities) {
    $calls = @(Flat-Calls $Answer)
    Assert-Live (($calls.callee_written -join ',') -ceq ($Names -join ',')) 'Written call ordering/selection changed.'
    Assert-Live (($calls.explicit_argument_count -join ',') -eq ($Arities -join ',')) 'Whole-argument boundaries changed.'
    for ($i = 0; $i -lt $calls.Count; $i++) {
        Assert-Live ($calls[$i].occurrence -eq $i) 'Occurrence numbering changed.'
        Assert-Live (-not $calls[$i].PSObject.Properties['resolved_target']) 'Written facts fabricated resolution.'
    }
}
$tsCases = @(
    @{ name='generics'; body='service.Save<User, Pair<number,string>>(item, ...items); Load<User>();'; names=@('Save','Load'); arities=@(2,0) },
    @{ name='rich-arguments'; body='Send({text:"a,b"}, [1,2], `c,${value}`, item);'; names=@('Send'); arities=@(4) },
    @{ name='nested-receiver'; body='Factory.Create().Save(Load());'; names=@('Create','Save','Load'); arities=@(0,1,0) },
    @{ name='await-return'; body='await service.Save(item); return service.Load();'; names=@('Save','Load'); arities=@(1,0) },
    @{ name='conditional-repetition'; body='if (a) Foo(); if (b) Foo();'; names=@('Foo','Foo'); arities=@(0,0) },
    @{ name='dynamic-decoys'; body='const café = "Fake()"; /* Nope() */ handlers[name](item); Real();'; names=@('Real'); arities=@(0) }
)
foreach ($case in $tsCases) {
    Invoke-Case "phase2-ts-$($case.name)" {
        Write-Utf8 (Join-Path $workspace 'Boundary.ts') ("class Boundary { field = FieldOnly(); async Run() { " + $case.body + ' } }')
        $answer = Calls "phase2-$($case.name)" 'Boundary.ts' 'Boundary'
        Assert-Sequence $answer $case.names $case.arities
        if ($case.name -eq 'generics') { Assert-Call @(Flat-Calls $answer) 'Save' 1 2 $true }
    }
}
$csCases = @(
    @{ name='generics-named'; body='service.Save<User, Pair<int,string>>(user: dto, count: 2); Load<User>();'; names=@('Save','Load'); arities=@(2,0) },
    @{ name='rich-arguments'; body='Send(new Pair(1,2), new[] {1,2}, ref item, out other, in input);'; names=@('Send'); arities=@(5) },
    @{ name='nested-receiver'; body='Factory.Create().Save(value: Load());'; names=@('Create','Save','Load'); arities=@(0,1,0) },
    @{ name='local-initializer'; body='var item = Load(); Save(item);'; names=@('Load','Save'); arities=@(0,1) },
    @{ name='conditional-repetition'; body='if (a) Foo(); if (b) Foo();'; names=@('Foo','Foo'); arities=@(0,0) },
    @{ name='dynamic-decoys'; body='var café = "Fake()"; /* Nope() */ handlers[name](item); Real();'; names=@('Real'); arities=@(0) }
)
foreach ($case in $csCases) {
    Invoke-Case "phase3-cs-$($case.name)" {
        Write-Utf8 (Join-Path $workspace 'Boundary.cs') ('class Boundary { object field = FieldOnly(); object Value { get; } = PropertyOnly(); void Run() { ' + $case.body + ' } }')
        Assert-Sequence (Calls "phase3-$($case.name)" 'Boundary.cs' 'Boundary') $case.names $case.arities
    }
}
foreach ($extension in @('ts','cs')) {
    Invoke-Case "phase1-$extension-source-isolation" {
        $declaration = if ($extension -eq 'ts') { 'class Twin { Run() { TOKEN(); } }' } else { 'class Twin { void Run() { TOKEN(); } }' }
        foreach ($entry in @(@('TwinA','Alpha'),@('TwinB','Bravo'))) {
            Write-Utf8 (Join-Path $workspace "$($entry[0]).$extension") ($declaration.Replace('TOKEN',$entry[1]))
        }
        Assert-Sequence (Calls "phase1-$extension-a" "TwinA.$extension" 'Twin') @('Alpha') @(0)
        Assert-Sequence (Calls "phase1-$extension-b" "TwinB.$extension" 'Twin') @('Bravo') @(0)
        Assert-Sequence (Calls "phase1-$extension-a-repeat" "TwinA.$extension" 'Twin') @('Alpha') @(0)
    }
    Invoke-Case "phase6-$extension-same-length-edit-and-recovery" {
        $file = "Fresh.$extension"
        $declaration = if ($extension -eq 'ts') { 'class Fresh { Run() { Alpha(); } }' } else { 'class Fresh { void Run() { Alpha(); } }' }
        $path = Join-Path $workspace $file
        Write-Utf8 $path $declaration
        Assert-Sequence (Calls "phase6-$extension-before" $file 'Fresh') @('Alpha') @(0)
        Write-Utf8 $path ($declaration.Replace('Alpha','Bravo'))
        Assert-Sequence (Calls "phase6-$extension-edited" $file 'Fresh') @('Bravo') @(0)
        Write-Utf8 $path ($declaration.Replace('Alpha();',''))
        Assert-Live ((Calls "phase6-$extension-empty" $file 'Fresh').count -eq 0) 'Empty replacement retained prior calls.'
        Remove-Item -LiteralPath $path
        $missing = Invoke-Live "phase6-$extension-missing" 'workspace_query' @{
            type='calls_in_file'; workspaceRoot=$workspace; filePath=$path
            owner=@{kind='class';name='Fresh'}; method=@{name='Run'}
        } -AllowError
        Assert-Live ($null -ne $missing.PSObject.Properties['error']) 'Missing source fabricated successful calls.'
        Write-Utf8 $path $declaration
        Assert-Sequence (Calls "phase6-$extension-restored" $file 'Fresh') @('Alpha') @(0)
    }
}
$rxCases = @(
    @{ name='commented-terminal'; body='value$ = source . pipe /* boundary */ (map(x => x), tap(log));'; expected=1; pattern='(?s)^value\$.*map:x => x.*tap:log' },
    @{ name='rich-boundaries'; body='value$ = source.pipe(map(x => ({text:"a,b",list:[1,2]})), switchMap(x => load(x,{retry:[1,2]})), tap(log));'; expected=1; pattern='(?s)map:.*switchMap:.*tap:log' },
    @{ name='nested-callback-policy'; body='value$ = source.pipe(switchMap(x => inner.pipe(map(y => y),tap(log))),tap(log));'; expected=1; pattern='(?s)^value\$.*switchMap:.*tap:' },
    @{ name='receiver-chain'; body='value$ = source.pipe(map(x => x),tap(log)).pipe(map(y => y),tap(log));'; expected=2; pattern='(?s)^value\$.*map:.*tap:' },
    @{ name='lexical-decoys'; body='text = "source.pipe(map(fake),tap(fake))"; /* source.pipe(map(fake),tap(fake)); */ value$ = source.pipe(map(x => x),tap(log));'; expected=1; pattern='(?s)^value\$.*map:.*tap:' },
    @{ name='nonterminal-dynamic'; body='value$ = source.pipeLike(map(x => x),tap(log)); other$ = source["pipe"](map(x => x),tap(log));'; expected=0; pattern='' },
    @{ name='below-threshold'; body='value$ = source.pipe(map(x => x));'; expected=0; pattern='' },
    @{ name='method-owner'; body='Run() { return source.pipe(map(x => x),tap(log)); }'; expected=1; pattern='(?s)^Run.*map:.*tap:' }
)
foreach ($case in $rxCases) {
    Invoke-Case "phase5-rx-$($case.name)" {
        Write-Utf8 (Join-Path $workspace 'RxBoundary.ts') ("import {map,switchMap,tap} from 'rxjs/operators'; class RxBoundary { " + $case.body + ' }')
        $pipes = @(Pipes (Compress "phase5-$($case.name)" 'RxBoundary.ts'))
        Assert-Live ($pipes.Count -eq $case.expected) 'RxJS chain selection/count changed.'
        if ($case.expected) { Assert-Live ($pipes[0] -match $case.pattern) 'RxJS owner/operator arguments lost at MCP boundary.' }
    }
}
Invoke-Case 'phase5-rx-import-gate' {
    Write-Utf8 (Join-Path $workspace 'RxBoundary.ts') 'class RxBoundary { value$ = source.pipe(map(x => x),tap(log)); }'
    Assert-Live (@(Pipes (Compress 'phase5-no-import' 'RxBoundary.ts')).Count -eq 0) 'Missing RxJS import fabricated domain markers.'
}
Invoke-Case 'phase4-cross-language-consumer-parity' {
    Write-Utf8 (Join-Path $workspace 'Parity.ts') 'class Parity { Run() { Save(item); Save(item); Load(); } }'
    Write-Utf8 (Join-Path $workspace 'Parity.cs') 'class Parity { void Run() { Save(item); Save(item); Load(); } }'
    $ts = Calls 'phase4-parity-ts' 'Parity.ts' 'Parity'
    $cs = Calls 'phase4-parity-cs' 'Parity.cs' 'Parity'
    Assert-Sequence $ts @('Save','Save','Load') @(1,1,0)
    Assert-Sequence $cs @('Save','Save','Load') @(1,1,0)
    Assert-Live (((Flat-Calls $ts) | ConvertTo-Json -Depth 20 -Compress) -ceq ((Flat-Calls $cs) | ConvertTo-Json -Depth 20 -Compress)) 'Shared written-call surface diverged by language.'
}
foreach ($fidelity in @('medium','high','edit')) {
    Invoke-Case "phase7-rx-delivery-$fidelity" {
        $canonical = @(Pipes (Compress "phase7-compress-$fidelity" 'Reactive.ts' $fidelity))
        $provided = Invoke-Live "phase7-provide-$fidelity" 'provide_code_context' @{
            workspaceRoot=$workspace; filePath=(Join-Path $workspace 'Reactive.ts'); fidelity=$fidelity; tokenizer='o200k'
        }
        $text = [string]$provided.result.content[0].text
        if ($provided.result._meta.content_kind -eq 'raw_passthrough') {
            Assert-Live ($text -ceq [IO.File]::ReadAllText((Join-Path $workspace 'Reactive.ts'))) 'Raw fallback was not current exact source.'
        } else {
            foreach ($pipe in $canonical) {
                foreach ($line in ($pipe -split "`n")) {
                    Assert-Live ($text.Contains($line.Trim())) 'Provided model-facing text omitted a canonical RxJS detail.'
                }
            }
        }
        Assert-Live ($canonical.Count -eq 3) 'Composed projection changed chain discovery.'
    }
}
foreach ($mode in @('mirrored','structured','indexed')) {
    Invoke-Case "phase7-rx-batch-$mode" {
        $single = Invoke-Live "phase7-single-$mode" 'provide_code_context' @{
            workspaceRoot=$workspace; filePath=(Join-Path $workspace 'Reactive.ts'); fidelity='high'; tokenizer='o200k'
        }
        $batch = Invoke-Live "phase7-batch-$mode" 'provide_code_context' @{
            workspaceRoot=$workspace; tokenizer='o200k'; responseMode=$mode
            files=@(@{id='rx';filePath=(Join-Path $workspace 'Reactive.ts');fidelity='high'})
        }
        $item = @($batch.result.structuredContent.results)[0]
        Assert-Live ($item.status -eq 'ok' -and $item.id -eq 'rx') 'Reactive batch item failed or changed identity.'
        $text = if ($mode -eq 'structured') { [string]$item.content[0].text } else { [string]$batch.result.content[$item.content_index].text }
        Assert-Live ($text -ceq [string]$single.result.content[0].text) 'Batch delivery altered the reactive payload.'
        if ($mode -eq 'mirrored') { Assert-Live ($text -ceq [string]$item.content[0].text) 'Reactive mirrored channels disagree.' }
    }
}
foreach ($extension in @('ts','cs')) {
    Invoke-Case "phase4-$extension-deferred-callback-calls" {
        $source = if ($extension -eq 'ts') { 'class Deferred { Run() { Send(x => Nested(x)); } }' } else { 'class Deferred { void Run() { Send(x => Nested(x)); } }' }
        Write-Utf8 (Join-Path $workspace "Deferred.$extension") $source
        Assert-Sequence (Calls "phase4-$extension-deferred" "Deferred.$extension" 'Deferred') @('Send','Nested') @(1,1)
    }
}
Invoke-Case 'phase6-rx-model-text-freshness' {
    $path = Join-Path $workspace 'FreshRx.ts'
    $source = "import {map,tap} from 'rxjs/operators'; class FreshRx { value`$ = source.pipe(map(x => x),tap(log)); }"
    foreach ($operator in @('map','tap')) {
        Write-Utf8 $path ($source.Replace('map(x', "$operator(x"))
        $canonical = @(Pipes (Compress "phase6-rx-$operator" 'FreshRx.ts'))
        Assert-Live ($canonical.Count -eq 1 -and $canonical[0].Contains("$($operator):x => x")) 'Same-length operator edit returned stale canonical arguments.'
        $provided = Invoke-Live "phase6-rx-text-$operator" 'provide_code_context' @{
            workspaceRoot=$workspace;filePath=$path;fidelity='high';tokenizer='o200k'
        }
        $text = [string]$provided.result.content[0].text
        if ($provided.result._meta.content_kind -eq 'raw_passthrough') {
            Assert-Live ($text -ceq [IO.File]::ReadAllText($path)) 'Cached raw response retained old operator.'
        } else { Assert-Live ($text.Contains("$($operator):x => x")) 'Cached model text retained old operator.' }
    }
}
Invoke-Case 'phase2-ts-bound-arrow-owner-isolation' {
    Write-Utf8 (Join-Path $workspace 'Arrows.ts') 'class Arrows { Run = () => Save(item,...items); Other = () => Load(); method() { Send(x => Nested(x)); } }'
    $run=Calls 'phase2-arrow-run' 'Arrows.ts' 'Arrows'
    Assert-Sequence $run @('Save') @(2)
    Assert-Call @(Flat-Calls $run) 'Save' 1 2 $true
    Assert-Sequence (Calls 'phase2-arrow-other' 'Arrows.ts' 'Arrows' 'Other') @('Load') @(0)
    Assert-Sequence (Calls 'phase2-arrow-method' 'Arrows.ts' 'Arrows' 'method') @('Send','Nested') @(1,1)
}
Invoke-Case 'phase2-ts-framework-neutral-written-call' {
    Write-Utf8 (Join-Path $workspace 'Neutral.ts') 'class Neutral { Run() { FakeStore.dispatch(item); } }'
    Assert-Sequence (Calls 'phase2-neutral-calls' 'Neutral.ts' 'Neutral') @('dispatch') @(1)
    $reply=Compress 'phase2-neutral-context' 'Neutral.ts'
    Assert-Live (@($reply.result.pretty.ir | Where-Object { $_[0] -eq 'TYPE' -and $_[1] -in @('@dispatch','@store','@effect') }).Count -eq 0) 'Structural call fabricated NgRx interpretation without domain evidence.'
}
