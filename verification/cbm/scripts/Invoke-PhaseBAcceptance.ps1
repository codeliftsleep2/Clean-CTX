param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$ProviderPath,
    [Parameter(Mandatory)][ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ExpectedProviderHash,
    [ValidateRange(30,600)][int]$CaseTimeoutSeconds=180
)
# Live B1–B4 acceptance. Builds nothing. Starts only owned frontend sessions,
# indexes/mutates only generated fixtures, and never kills the shared daemon.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path;$ProviderPath=(Resolve-Path $ProviderPath).Path
if ((Get-FileHash $ProviderPath -Algorithm SHA256).Hash -ne $ExpectedProviderHash) { throw 'Pinned provider hash mismatch.' }
if ([Environment]::GetEnvironmentVariable('CBM_CACHE_DIR')) { throw 'Use the agreed normal account cache, without CBM_CACHE_DIR override.' }
$runRoot=Join-Path $repo ('target/cbm-phase-b/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$primary=Join-Path $runRoot 'primary';$additional=Join-Path $runRoot 'additional'
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot 'PhaseBSession.ps1')
New-Item -ItemType Directory -Force $primary,$additional | Out-Null
foreach ($root in @($primary,$additional)) { & git -C $root init --quiet; Assert-B ($LASTEXITCODE -eq 0) 'Fixture Git init failed.' }
$manifest=@{}
$imports=[Collections.Generic.List[string]]::new();$calls=[Collections.Generic.List[string]]::new()
for ($chunk=0;$chunk -lt 9;$chunk++) {
    $names=@();$lines=@()
    for ($i=0;$i -lt 25;$i++) {
        $name='bProbe'+($chunk*25+$i).ToString('D3');$names+=$name
        $lines+="export function ${name}(): number { return $($chunk*25+$i); }"
        $calls.Add("  ${name}();")
    }
    $file="part$chunk.ts";Write-BText (Join-Path $primary $file) ($lines -join "`n")
    $imports.Add("import { $($names -join ', ') } from './part$chunk';")
    foreach ($name in $names) {$manifest[$name]=$file}
}
Write-BText (Join-Path $primary 'hub.ts') (($imports -join "`n")+"`nexport function bHub(): void {`n"+($calls -join "`n")+"`n}")
Write-BText (Join-Path $primary 'Contract.cs') 'namespace PhaseB; public interface IProbe { int Read(); }'
Write-BText (Join-Path $primary 'Probe.cs') 'namespace PhaseB; public class Probe : IProbe { public int Read() => 7; }'
Write-BText (Join-Path $primary 'Consumer.cs') 'namespace PhaseB; public class Consumer { private readonly IProbe probe; public Consumer(IProbe probe) { this.probe = probe; } public int Run() => probe.Read(); }'
Write-BText (Join-Path $additional 'Probe.cs') 'namespace OtherPhaseB; public interface IProbe { string Read(); } public class Probe : IProbe { public string Read() => "other"; }'
Write-BText (Join-Path $additional 'probe.ts') 'export function bAdditionalOnly(): string { return "additional"; }'
Write-BText (Join-Path $primary '.clean-ctx.json') (@{cbm=@{binary_path=$ProviderPath};additional_roots=@($additional)} | ConvertTo-Json -Depth 20)
Save-B 'fixture-manifest' @{primary=$primary;additional=$additional;function_files=$manifest;expected_probe_count=225}
Save-B 'provenance' @{pinned_revision='82b1ec908f975354ceb4bac8a8bfc594942a9bc4';provider=$ProviderPath;provider_sha256=$ExpectedProviderHash;clean_binary=$BinaryPath;clean_sha256=(Get-FileHash $BinaryPath).Hash;pwsh=$PSVersionTable.PSVersion.ToString()}
$provider=$null;$clean=$null;$project=$null;$other=$null;$oracle=$null;$query='MATCH (f:Function) RETURN f.name, f.file_path ORDER BY f.name'
try {
    $provider=Start-BSession 'provider' $ProviderPath $primary
    Invoke-BCase 'B1' 'provider-registry-and-indexed-identity' {
        $registry=Get-BResult (Invoke-BRpc $provider 'tools/list' @{})
        foreach ($name in @('index_repository','search_graph','query_graph','trace_path','get_architecture','list_projects','index_status','check_index_coverage')) { Assert-B ($name -in $registry.tools.name) "Missing provider tool: $name" }
        $indexed=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_repository' @{repo_path=$primary;mode='fast'}))
        Assert-B ($indexed.status -eq 'indexed') 'Primary index is not successfully terminal.';$script:project=$indexed.project
        $indexed=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_repository' @{repo_path=$additional;mode='fast'}))
        Assert-B ($indexed.status -eq 'indexed') 'Additional index is not successfully terminal.';$script:other=$indexed.project
        Assert-B ($project -ne $other) 'Different roots share a provider identity.'
    }
    Assert-B ($project -and $other) 'Index admission failed; remaining cases cannot run.'
    $script:deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
    $clean=Start-BSession 'clean' $BinaryPath $primary
    Invoke-BCase 'B1' 'clean-tools-and-readiness' {
        $registry=Get-BResult (Invoke-BRpc $clean 'tools/list' @{})
        foreach ($name in @('graph_search','graph_query','graph_trace','get_architecture','cbm_proxy','provide_code_context')) { Assert-B ($name -in $registry.tools.name) "Missing Clean tool: $name" }
        $status=Get-BResult (Invoke-BReadyTool $clean 'get_cbm_status' @{})
        Assert-B ($status._meta.cbm_status -eq 'available') 'Provider admission unavailable; inspect stderr.'
    }
    Invoke-BCase 'B1' 'grouped-search-qn-file-and-empty-query' {
        $raw=Get-BBody (Get-BResult (Invoke-BTool $provider 'search_graph' @{project=$project;name_pattern='bProbe000';format='json'}))
        $rows=Get-BGroupedRows $raw;Assert-B ($rows.Count -eq 1 -and $rows[0].file_path -eq 'part0.ts') 'Provider fixture identity mismatch.'
        $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='bProbe000'})
        Assert-B ([bool]$answer.PSObject.Properties['structuredContent']) 'Typed response is not ready/structured.'
        $nodes=@($answer.structuredContent.nodes);Assert-B ($nodes.Count -eq 1) 'Typed search lost the grouped row.'
        Assert-B ($nodes[0].id -eq $rows[0].qualified_name -and $nodes[0].file -eq $rows[0].file_path) 'QN/file changed at typed boundary.'
        $empty=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='NoSuchPhaseBName'})
        Assert-B (@($empty.structuredContent.nodes).Count -eq 0) 'Valid zero-result search failed.'
    }
    Invoke-BCase 'B2' 'serving-job-coverage-are-independent' {
        $job=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_repository' @{project=$project;status=$true}))
        $serving=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_status' @{project=$project;format='json'}))
        Assert-B ($job.state -eq 'succeeded' -and $serving.status -eq 'ready') 'Job success and serving readiness not both established.'
        Assert-B ((Resolve-Path $serving.root_path).Path -eq $primary) 'Serving root differs from admitted fixture.'
        $coverage=Get-BBody (Get-BResult (Invoke-BTool $provider 'check_index_coverage' @{project=$project;paths=@('part0.ts','Contract.cs','Consumer.cs');format='json'}))
        Assert-B ($coverage.signal -eq 'best_effort' -and $coverage.path_returned -eq 3) 'Coverage scope/count not established.'
        Save-B 'B2-independent-observations' @{job=$job;serving=$serving;coverage=$coverage}
        $null=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='bProbe000'})
        $after=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_repository' @{project=$project;status=$true}))
        Assert-B ($after.state -eq $job.state) 'Search changed the provider job observation.'
    }
    Invoke-BCase 'B3' 'query-paging-provider-oracle-and-typed-completion' {
        $script:oracle=Get-BExactQuery $project $query;Assert-B ($oracle.pages -gt 1 -and $oracle.total -gt 200) 'Fixture did not exercise query paging.'
        $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_query' @{project=$project;query=$query})
        $nodes=@($answer.structuredContent.nodes)
        Assert-B ($nodes.Count -eq $oracle.total) 'Typed query delivered a partial page as full result.'
        for ($i=0;$i -lt $nodes.Count;$i++) {Assert-B ($nodes[$i].name -eq $oracle.rows[$i][0] -and $nodes[$i].file -eq $oracle.rows[$i][1]) 'Projected name/file/order changed.'}
    }
    Invoke-BCase 'B3' 'search-collection-and-cache-consistency' {
        foreach ($attempt in 1..2) {
            $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='bProbe'})
            $nodes=@($answer.structuredContent.nodes);Assert-B ($nodes.Count -eq 225) 'Search collection/cache lost probe symbols.'
            foreach ($node in $nodes) {Assert-B ($manifest.ContainsKey($node.name) -and $manifest[$node.name] -eq $node.file) 'Search returned a foreign identity or wrong file.'}
        }
    }
    Invoke-BCase 'B3' 'trace-cross-file-hub-cursor-and-direct-edges' {
        $root=Get-BGroupedRows (Get-BBody (Get-BResult (Invoke-BTool $provider 'search_graph' @{project=$project;name_pattern='bHub';format='json'})))
        Assert-B ($root.Count -eq 1) 'Hub identity ambiguous.';$qn=$root[0].qualified_name
        $trace=Get-BExactTrace $project $qn
        Assert-B ($trace.pages -gt 1 -and $trace.rows.Count -eq 225) 'Fixture did not exercise the complete hub traversal.'
        $target=@($trace.rows | Where-Object hop -eq 1)[-1].qualified_name
        $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_trace' @{project=$project;from=$qn;to=$target})
        $edges=@($answer.structuredContent.edges);Assert-B ($edges.Count -eq 1) 'Typed trace lost a direct target beyond the default first page.'
        Assert-B ($edges[0].from -ceq $qn -and $edges[0].to -ceq $target -and $edges[0].label -eq 'calls') 'Trace invented/misoriented an edge.'
        Save-B 'B3-trace-oracle' $trace
    }
    Invoke-BCase 'B1' 'architecture-consumed-tables' {
        $raw=Get-BBody (Get-BResult (Invoke-BTool $provider 'get_architecture' @{project=$project;format='json';aspects=@('packages','boundaries')}))
        $answer=Get-BResult (Invoke-BReadyTool $clean 'get_architecture' @{project=$project})
        if ($raw.PSObject.Properties['packages']) {Assert-B (@($answer.structuredContent.modules).Count -eq @($raw.packages.rows).Count) 'Architecture packages lost.'}
        if ($raw.PSObject.Properties['boundaries']) {Assert-B (@($answer.structuredContent.dependencies).Count -eq @($raw.boundaries.rows).Count) 'Architecture boundaries lost.'}
    }
    Invoke-BCase 'B2' 'additional-root-same-names-and-primary-routing' {
        $otherResult=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$other;query='bAdditionalOnly'})
        Assert-B (@($otherResult.structuredContent.nodes).Count -eq 1) 'Additional-root symbol not delivered.'
        $primaryResult=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='bAdditionalOnly'})
        Assert-B (@($primaryResult.structuredContent.nodes).Count -eq 0) 'Additional-root symbol leaked into primary.'
        foreach ($root in @($primary,$additional)) {
            $native=Get-BResult (Invoke-BReadyTool $clean 'provide_code_context' @{workspaceRoot=$root;filePath='Probe.cs';fidelity='high'})
            Assert-B (@($native.content).Count -gt 0 -and ($native | ConvertTo-Json -Depth 100 -Compress).Contains('Probe')) 'C# native boundary unavailable.'
            $nativeJson=$native | ConvertTo-Json -Depth 100 -Compress
            Assert-B ($nativeJson.Contains('OtherPhaseB') -eq ($root -eq $additional)) 'Same-named C# file resolved against the wrong root.'
        }
    }
    Invoke-BCase 'B4' 'raw-single-page-is-lossless-and-bridge-status-current' {
        $toolArgs=@{project=$project;query=$query;format='json';max_rows=17}
        $direct=Get-BBody (Get-BResult (Invoke-BTool $provider 'query_graph' $toolArgs))
        $proxy=Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='query_graph';parameters=$toolArgs})
        $body=Get-BBody $proxy
        Assert-B ($proxy._meta.cbm_status -eq 'available') 'Proxy reported stale startup readiness.'
        Assert-B ($body.returned -eq 17 -and $body.has_more) 'Raw proxy implicitly collected or lost pagination.'
        Assert-B (((Convert-BStable $body) | ConvertTo-Json -Depth 100 -Compress) -ceq ((Convert-BStable $direct) | ConvertTo-Json -Depth 100 -Compress)) 'Raw model-facing body changed protected provider facts.'
    }
    Invoke-BCase 'B4' 'tree-format-is-intentional-and-verbatim' {
        $toolArgs=@{project=$project;name_pattern='bProbe000';format='tree'}
        $direct=Get-BResult (Invoke-BTool $provider 'search_graph' $toolArgs)
        $proxy=Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='search_graph';parameters=$toolArgs})
        $treeText=$proxy.content[0].text
        try {
            $envelope=$treeText | ConvertFrom-Json -Depth 100
            if ($envelope.PSObject.Properties['content']) { $treeText=$envelope.content[0].text }
        } catch { # Intentional tree output is not JSON.
        }
        Assert-B ($treeText -ceq $direct.content[0].text) 'Proxy changed tree layout or token whitespace.'
    }
    Invoke-BCase 'B4' 'soft-failure-remains-failure-and-whitelist-stays-closed' {
        $failed=Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='query_graph';parameters=@{project=$project;query='not valid cypher';format='json'}}
        Assert-B ($failed.result.isError -eq $true -and @($failed.result.content).Count -gt 0) 'Provider error was disguised as empty success.'
        $blocked=Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='index_status';parameters=@{project=$project;format='json'}}
        Assert-B ([bool]$blocked.PSObject.Properties['error']) 'Internal status tool leaked into public proxy whitelist.'
    }
    Invoke-BCase 'B1/B4' 'list-projects-single-page-and-identity' {
        $toolArgs=@{format='json';limit=2;offset=0}
        $direct=Get-BBody (Get-BResult (Invoke-BTool $provider 'list_projects' $toolArgs))
        $proxy=Get-BBody (Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='list_projects';parameters=$toolArgs}))
        Assert-B (((Convert-BStable $direct) | ConvertTo-Json -Depth 100 -Compress) -ceq ((Convert-BStable $proxy) | ConvertTo-Json -Depth 100 -Compress)) 'Project page/identity/continuation changed at proxy boundary.'
    }
    Invoke-BCase 'B4' 'architecture-raw-consumed-aspects-equivalence' {
        $toolArgs=@{project=$project;format='json';aspects=@('packages','boundaries')}
        $direct=Get-BBody (Get-BResult (Invoke-BTool $provider 'get_architecture' $toolArgs))
        $proxy=Get-BBody (Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='get_architecture';parameters=$toolArgs}))
        Assert-B (((Convert-BStable $direct) | ConvertTo-Json -Depth 100 -Compress) -ceq ((Convert-BStable $proxy) | ConvertTo-Json -Depth 100 -Compress)) 'Architecture raw metadata/tables changed.'
    }
    Invoke-BCase 'B4' 'trace-raw-directional-page-equivalence' {
        $hub=Get-BGroupedRows (Get-BBody (Get-BResult (Invoke-BTool $provider 'search_graph' @{project=$project;name_pattern='bHub';format='json'})))
        $toolArgs=@{project=$project;function_name=$hub[0].qualified_name;direction='outbound';depth=3;format='json';limit=17}
        $direct=Get-BBody (Get-BResult (Invoke-BTool $provider 'trace_path' $toolArgs))
        $proxy=Get-BBody (Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='trace_path';parameters=$toolArgs}))
        Assert-B (((Convert-BStable $direct) | ConvertTo-Json -Depth 100 -Compress) -ceq ((Convert-BStable $proxy) | ConvertTo-Json -Depth 100 -Compress)) 'Trace direction/identity/page metadata changed.'
    }
    Invoke-BCase 'B2/B4' 'raw-additional-project-does-not-change-active-wrapper-project' {
        $null=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{project=$project;query='bProbe000'})
        $proxy=Get-BBody (Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='search_graph';parameters=@{project=$other;name_pattern='bAdditionalOnly';format='json'}}))
        $additionalRows=Get-BGroupedRows $proxy
        Assert-B ($additionalRows.Count -eq 1 -and $additionalRows[0].name -eq 'bAdditionalOnly') 'Raw additional-project identity mismatch.'
        $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_search' @{query='bAdditionalOnly'})
        Assert-B (@($answer.structuredContent.nodes).Count -eq 0) 'Raw per-call project mutated persistent typed-wrapper project.'
    }
    Invoke-BCase 'B2/B4' 'explicit-manual-index-success-reaches-mcp' {
        $answer=Get-BBody (Get-BResult (Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='index_repository';parameters=@{repo_path=$primary;mode='fast'}}))
        Assert-B ($answer.status -eq 'indexed' -and $answer.project -eq $project) 'Manual MCP index did not return a bound terminal success.'
    }
    Invoke-BCase 'B3/B4' 'stale-provider-cursor-remains-error-through-proxy' {
        $toolArgs=@{project=$project;query=$query;format='json';max_rows=17}
        $first=Get-BBody (Get-BResult (Invoke-BTool $provider 'query_graph' $toolArgs))
        Assert-B ($first.has_more -and $first.next_cursor) 'Stale-cursor probe needs a real continuation.'
        Write-BText (Join-Path $primary 'cursor-mutated.ts') 'export function bCursorMutation(): number { return 99; }'
        $indexed=Get-BBody (Get-BResult (Invoke-BTool $provider 'index_repository' @{repo_path=$primary;mode='fast'}))
        Assert-B ($indexed.status -eq 'indexed') 'Controlled fixture generation change did not publish.'
        $toolArgs.cursor=$first.next_cursor
        $stale=Invoke-BTool $provider 'query_graph' $toolArgs
        Assert-B ($stale.result.isError -eq $true) 'Provider reused a stale result snapshot.'
        $proxy=Invoke-BReadyTool $clean 'cbm_proxy' @{cbm_tool='query_graph';parameters=$toolArgs}
        Assert-B ($proxy.result.isError -eq $true) 'Proxy hid stale-cursor failure or restarted by live offset.'
        Save-B 'B3-stale-cursor' @{provider=$stale;proxy=$proxy}
    }
    Invoke-BCase 'B5' 'native-csharp-typescript-with-provider-disabled' {
        $nativeRoot=Join-Path $runRoot 'native-only'
        New-Item -ItemType Directory -Force $nativeRoot | Out-Null
        & git -C $nativeRoot init --quiet
        Assert-B ($LASTEXITCODE -eq 0) 'Native fixture Git init failed.'
        $padding=(1..65 | ForEach-Object { "// native preservation line $_" }) -join "`n"
        Write-BText (Join-Path $nativeRoot 'Probe.cs') ($padding+"`nnamespace NativePhaseB; public class NativeProbe { public int Read() { return 17; } }")
        Write-BText (Join-Path $nativeRoot 'probe.ts') ($padding+"`nexport function bNativeOnly(): number { return 17; }")
        Write-BText (Join-Path $nativeRoot '.clean-ctx.json') '{"cbm":{"enabled":false}}'
        $native=Start-BSession 'native-only' $BinaryPath $nativeRoot
        $status=Get-BResult (Invoke-BTool $native 'get_cbm_status' @{})
        Assert-B ($status._meta.cbm_status -eq 'unavailable') 'Disabled provider was reported available.'
        foreach ($item in @(@{file='Probe.cs';symbol='NativeProbe'},@{file='probe.ts';symbol='bNativeOnly'})) {
            $context=Get-BResult (Invoke-BTool $native 'provide_code_context' @{workspaceRoot=$nativeRoot;filePath=$item.file;fidelity='high'})
            Assert-B (($context | ConvertTo-Json -Depth 100 -Compress).Contains($item.symbol)) 'Native context depends on optional provider availability.'
        }
    }
} finally {
    foreach ($session in $sessions) {
        try { Stop-BSession $session }
        catch { $verdicts.Add(@{phase='cleanup';case=$session.Name;result='FAIL';evidence=$_.Exception.Message}) }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Phase B evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
