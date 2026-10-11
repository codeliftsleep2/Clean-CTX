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
        foreach ($name in @('index_repository','search_graph','query_graph','trace_path','get_architecture','index_status','check_index_coverage')) { Assert-B ($name -in $registry.tools.name) "Missing provider tool: $name" }
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
        Assert-B ($answer.PSObject.Properties['structuredContent']) 'Typed response is not ready/structured.'
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
        $first=Get-BBody (Get-BResult (Invoke-BTool $provider 'trace_path' @{project=$project;function_name=$qn;direction='both';depth=3;limit=17;format='json'}))
        Assert-B ($first.has_more -and $first.next_cursor) 'Fixture did not exercise trace cursor.'
        $next=Get-BBody (Get-BResult (Invoke-BTool $provider 'trace_path' @{project=$project;function_name=$qn;direction='both';depth=3;limit=17;format='json';cursor=$first.next_cursor}))
        Save-B 'B3-trace-pages' @{first=$first;next=$next}
        $answer=Get-BResult (Invoke-BReadyTool $clean 'graph_trace' @{project=$project;from=$qn})
        $edges=@($answer.structuredContent.edges);Assert-B ($edges.Count -eq 225) 'Typed trace omitted cross-file direct calls.'
        foreach ($edge in $edges) {Assert-B ($edge.from -eq $qn -and $edge.label -eq 'calls') 'Trace invented/misoriented an edge.'}
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
        Assert-B (($body | ConvertTo-Json -Depth 100 -Compress) -eq ($direct | ConvertTo-Json -Depth 100 -Compress)) 'Raw model-facing body changed protected provider facts.'
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
        Assert-B ($blocked.PSObject.Properties['error']) 'Internal status tool leaked into public proxy whitelist.'
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
