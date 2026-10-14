param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [Parameter(Mandatory)][string]$CorpusPath,
    [ValidateRange(60, 3600)][int]$RunTimeoutSeconds = 1800
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$benchmarkRoot = $PSScriptRoot
$repo = (Resolve-Path (Join-Path $benchmarkRoot '../..')).Path
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$CorpusPath = (Resolve-Path -LiteralPath $CorpusPath).Path
$oraclePath = Join-Path $benchmarkRoot 'oracle.json'
$corpusManifestPath = Join-Path $benchmarkRoot 'corpus.json'
$oracle = Get-Content -LiteralPath $oraclePath -Raw | ConvertFrom-Json -Depth 100
$corpusManifest = Get-Content -LiteralPath $corpusManifestPath -Raw | ConvertFrom-Json -Depth 100
$pinned = $corpusManifest.repositories[0]
$scopeRelative = [string]$pinned.scope.included_roots[0]
$scopePath = Join-Path $CorpusPath $scopeRelative
$cleanCtxCommit = (& git -C $repo rev-parse HEAD).Trim()
$timestamp = [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss')
$runRoot = Join-Path $repo "target/ngrx-recall-benchmark/$timestamp-$($cleanCtxCommit.Substring(0,8))"
$sessionRoot = Join-Path $runRoot 'session-root'
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($RunTimeoutSeconds)
. (Join-Path $benchmarkRoot '../cbm/scripts/PhaseBSession.ps1')
$requestCapture = [Collections.Generic.List[object]]::new()
$responseCapture = [Collections.Generic.List[object]]::new()
$frameSequence = 0

# Override the shared session helper's RPC function so this benchmark preserves
# every frame in order in addition to the helper's normal per-frame evidence.
function Invoke-BRpc($Session, [string]$Method, [hashtable]$Params) {
    Assert-B (-not $Session.Terminal) "Session $($Session.Name) is terminal."
    $script:rpcId++
    $request = @{jsonrpc='2.0';id=$script:rpcId;method=$Method;params=$Params}
    $script:requestCapture.Add($request)
    Save-B ("$($Session.Name)-request-$rpcId") $request
    $Session.Process.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 40 -Compress))
    $Session.Process.StandardInput.Flush()
    while ($true) {
        $read = $Session.Process.StandardOutput.ReadLineAsync()
        $remaining = [Math]::Max(1, [int]($script:deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $read.Wait($remaining)) {
            $Session.Terminal = $true
            throw "MCP response deadline exceeded in $($Session.Name)/$Method; inspect the saved request and stderr."
        }
        $line = $read.Result
        if ($null -eq $line) { $Session.Terminal=$true; throw "Session $($Session.Name) closed stdout." }
        Assert-B ([Text.Encoding]::UTF8.GetByteCount($line) -le 8MB) 'Harness response capture exceeded 8 MiB.'
        $response = $line | ConvertFrom-Json -Depth 100
        $script:frameSequence++
        $script:responseCapture.Add($response)
        Save-B ("$($Session.Name)-frame-$frameSequence") $response
        if ($response.PSObject.Properties['id'] -and $response.id -eq $rpcId) { return $response }
        Assert-B ([DateTime]::UtcNow -lt $script:deadline) 'Notification stream exhausted the run deadline.'
    }
}

function Invoke-GitText([string[]]$Arguments) {
    $text = & git @Arguments
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ') failed." }
    return ($text -join "`n").Trim()
}

function Relative-Path([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $base = [IO.Path]::GetFullPath($CorpusPath)
    if ($full.StartsWith('\\?\', [StringComparison]::Ordinal)) { $full = $full.Substring(4) }
    if ($base.StartsWith('\\?\', [StringComparison]::Ordinal)) { $base = $base.Substring(4) }
    $relative = [IO.Path]::GetRelativePath($base, $full).Replace('\', '/')
    if ($relative -eq '..' -or $relative.StartsWith('../', [StringComparison]::Ordinal)) {
        return $null
    }
    return $relative
}

function Entity-Key($Entity) {
    return @(
        [string]$Entity.domain,
        [string]$Entity.kind,
        [string]$Entity.name,
        [string]$Entity.file
    ) -join '|'
}

function Pair-Key($Pair) {
    return @(
        [string]$Pair.relation,
        [string]$Pair.source.domain,
        [string]$Pair.source.kind,
        [string]$Pair.source.name,
        [string]$Pair.source.file,
        [string]$Pair.target.domain,
        [string]$Pair.target.kind,
        [string]$Pair.target.name,
        [string]$Pair.target.file
    ) -join '|'
}

function Normalize-Edge($Edge) {
    $sourceFile = Relative-Path ([string]$Edge.subject.file)
    $targetFile = Relative-Path ([string]$Edge.object.file)
    if ($null -eq $sourceFile -or $null -eq $targetFile) { return $null }
    if (-not $sourceFile.StartsWith(($scopeRelative.Replace('\', '/') + '/'), [StringComparison]::Ordinal)) {
        return $null
    }
    if (-not $targetFile.StartsWith(($scopeRelative.Replace('\', '/') + '/'), [StringComparison]::Ordinal)) {
        return $null
    }
    return [pscustomobject]@{
        relation = [string]$Edge.relation
        source = [pscustomobject]@{
            domain = [string]$Edge.subject.domain
            kind = [string]$Edge.subject.entity_type
            name = [string]$Edge.subject.name
            file = $sourceFile.Substring($scopeRelative.Length + 1).Replace('\', '/')
        }
        target = [pscustomobject]@{
            domain = [string]$Edge.object.domain
            kind = [string]$Edge.object.entity_type
            name = [string]$Edge.object.name
            file = $targetFile.Substring($scopeRelative.Length + 1).Replace('\', '/')
        }
    }
}

function Add-ObservedEdges($Edges, [Collections.Generic.Dictionary[string,object]]$Observed) {
    foreach ($edge in @($Edges)) {
        if ([string]$edge.relation -notin @('Dispatches', 'Selects', 'HandlesAction', 'TriggersReducer')) {
            continue
        }
        $normalized = Normalize-Edge $edge
        if ($null -eq $normalized) { continue }
        $key = Pair-Key $normalized
        if (-not $Observed.ContainsKey($key)) { $Observed[$key] = $normalized }
    }
}

function Invoke-Forward($Session, $Source, [string]$Label) {
    $result = Get-BResult (Invoke-BTool $Session 'workspace_query' @{
        workspaceRoot = $CorpusPath
        withinPath = $scopePath
        type = 'forward_edges'
        domain = [string]$Source.domain
        entity_type = [string]$Source.kind
        name = [string]$Source.name
    })
    Save-B $Label $result
    return @($result.structuredContent.edges)
}

function Write-JsonLines([string]$Path, $Values) {
    $lines = foreach ($value in @($Values)) { $value | ConvertTo-Json -Depth 100 -Compress }
    Write-BText $Path (($lines -join "`n") + $(if ($lines.Count) { "`n" } else { '' }))
}

function Consolidate-RpcEvidence {
    Write-JsonLines (Join-Path $runRoot 'requests.jsonl') $requestCapture.ToArray()
    Write-JsonLines (Join-Path $runRoot 'responses.jsonl') $responseCapture.ToArray()
}

New-Item -ItemType Directory -Force $sessionRoot | Out-Null
Write-BText (Join-Path $sessionRoot '.clean-ctx.json') '{"cbm":{"enabled":false},"additional_roots":[]}'

$actualCorpusCommit = Invoke-GitText @('-C', $CorpusPath, 'rev-parse', 'HEAD')
Assert-B ($actualCorpusCommit -ceq [string]$pinned.commit) "Corpus commit mismatch: $actualCorpusCommit"
$corpusDirty = Invoke-GitText @('-C', $CorpusPath, 'status', '--porcelain', '--untracked-files=all')
Assert-B ([string]::IsNullOrEmpty($corpusDirty)) 'Corpus checkout is dirty.'
$repoDirty = Invoke-GitText @('-C', $repo, 'status', '--porcelain', '--untracked-files=all')
Assert-B ([string]::IsNullOrEmpty($repoDirty)) 'Clean-CTX checkout is dirty; commit the frozen harness before scoring.'

$actualOracleHash = (Get-FileHash -LiteralPath $oraclePath -Algorithm SHA256).Hash.ToLowerInvariant()
Assert-B ($actualOracleHash -ceq [string]$corpusManifest.oracle_sha256) 'Frozen oracle checksum mismatch.'
Assert-B ([string]$oracle.benchmark_version -ceq [string]$corpusManifest.benchmark_version) 'Benchmark version mismatch.'

$files = @(
    Get-ChildItem -LiteralPath $scopePath -Recurse -File -Filter '*.ts' |
        Where-Object {
            $_.Name -notlike '*.spec.ts' -and
            -not ($_.Attributes -band [IO.FileAttributes]::ReparsePoint)
        } |
        ForEach-Object { Relative-Path $_.FullName }
)
[Array]::Sort($files, [StringComparer]::Ordinal)
Assert-B ($files.Count -eq [int]$pinned.scope.expected_included_ts_files) "Scope expected $($pinned.scope.expected_included_ts_files) files; found $($files.Count)."

$provenance = [ordered]@{
    schema = 'clean-ctx/ngrx-recall-benchmark-provenance'
    schema_version = 1
    benchmark_version = [string]$oracle.benchmark_version
    oracle_sha256 = $actualOracleHash
    clean_ctx_commit = $cleanCtxCommit
    clean_ctx_dirty = $false
    binary = $BinaryPath
    binary_sha256 = (Get-FileHash -LiteralPath $BinaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    corpus_url = [string]$pinned.canonical_url
    corpus_commit = $actualCorpusCommit
    corpus_dirty = $false
    corpus_path = $CorpusPath
    scope_root = $scopeRelative.Replace('\', '/')
    scope_file_count = $files.Count
    fidelity = 'high'
    cbm_enabled = $false
    optional_providers_enabled = $false
    fresh_state = "new session root $sessionRoot"
    hydration_order = 'repository-relative ordinal lexical order'
    query_order = 'oracle pair ID order, then remaining source identities in ordinal key order'
    public_tools = @('provide_code_context', 'workspace_query')
    started_utc = [DateTime]::UtcNow.ToString('o')
    host = [Environment]::MachineName
    platform = [Environment]::OSVersion.VersionString
    powershell = $PSVersionTable.PSVersion.ToString()
}
Save-B 'provenance' $provenance
Save-B 'scope-files' $files

$observed = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
$queriedSources = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$declaredSources = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
$session = $null
$completed = $false

try {
    $session = Start-BSession 'benchmark' $BinaryPath $sessionRoot
    $requestCapture.Insert(1, [pscustomobject]@{
        jsonrpc = '2.0'
        method = 'notifications/initialized'
        params = [pscustomobject]@{}
    })
    $registry = Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    foreach ($tool in @('provide_code_context', 'workspace_query')) {
        Assert-B ($tool -in @($registry.tools.name)) "Registered public tool absent: $tool"
    }

    foreach ($file in $files) {
        $null = Get-BResult (Invoke-BTool $session 'provide_code_context' @{
            workspaceRoot = $CorpusPath
            filePath = $file
            fidelity = 'high'
        })
    }

    foreach ($file in $files) {
        $result = Get-BResult (Invoke-BTool $session 'workspace_query' @{
            workspaceRoot = $CorpusPath
            withinPath = $scopePath
            type = 'entities_in_file'
            file_path = $file
            fidelity = 'high'
        })
        foreach ($entity in @($result.structuredContent.entities)) {
            $relative = Relative-Path ([string]$entity.file)
            if ($null -eq $relative -or -not $relative.StartsWith(($scopeRelative.Replace('\', '/') + '/'), [StringComparison]::Ordinal)) { continue }
            $kind = [string]$entity.entity_type
            $eligible = ($entity.domain -eq 'angular' -and $kind -in @('Component', 'Service')) -or
                ($entity.domain -eq 'ngrx' -and $kind -in @('Effect', 'Action'))
            if (-not $eligible) { continue }
            $source = [pscustomobject]@{
                domain = [string]$entity.domain
                kind = $kind
                name = [string]$entity.name
                file = $relative.Substring($scopeRelative.Length + 1).Replace('\', '/')
            }
            $key = Entity-Key $source
            if (-not $declaredSources.ContainsKey($key)) { $declaredSources[$key] = $source }
        }
    }

    foreach ($pair in @($oracle.pairs)) {
        $sourceKey = Entity-Key $pair.source
        $edges = Invoke-Forward $session $pair.source "expected-$($pair.id)-forward"
        $null = $queriedSources.Add($sourceKey)
        Add-ObservedEdges $edges $observed
    }

    $remainingKeys = @($declaredSources.Keys | Where-Object { -not $queriedSources.Contains($_) })
    [Array]::Sort($remainingKeys, [StringComparer]::Ordinal)
    $sweepIndex = 0
    foreach ($key in $remainingKeys) {
        $sweepIndex++
        Add-ObservedEdges (Invoke-Forward $session $declaredSources[$key] "sweep-$('{0:D3}' -f $sweepIndex)-forward") $observed
    }
    foreach ($fallback in @(
        [pscustomobject]@{domain='angular';kind='DispatchSite';name='store';file=''},
        [pscustomobject]@{domain='angular';kind='SelectSite';name='store';file=''}
    )) {
        Add-ObservedEdges (Invoke-Forward $session $fallback "fallback-$($fallback.kind)-forward") $observed
    }

    $expectedKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($pair in @($oracle.pairs)) { $null = $expectedKeys.Add((Pair-Key $pair)) }
    $observedValues = @($observed.Values | Sort-Object relation, @{Expression={$_.source.file}}, @{Expression={$_.source.name}}, @{Expression={$_.target.file}}, @{Expression={$_.target.name}})
    $falseNegatives = @($oracle.pairs | Where-Object { -not $observed.ContainsKey((Pair-Key $_)) })
    $unexpected = @($observedValues | Where-Object { -not $expectedKeys.Contains((Pair-Key $_)) })

    $scores = [Collections.Generic.List[object]]::new()
    foreach ($relation in @('Dispatches', 'Selects', 'HandlesAction', 'TriggersReducer')) {
        $expected = @($oracle.pairs | Where-Object relation -eq $relation)
        $found = @($expected | Where-Object { $observed.ContainsKey((Pair-Key $_)) })
        $observedRelation = @($observedValues | Where-Object relation -eq $relation)
        $recall = if ($expected.Count) { $found.Count / $expected.Count } else { 0.0 }
        $precision = if ($observedRelation.Count) { $found.Count / $observedRelation.Count } else { 0.0 }
        $scores.Add([pscustomobject]@{
            relation = $relation
            expected = $expected.Count
            true_positives = $found.Count
            observed = $observedRelation.Count
            false_negatives = $expected.Count - $found.Count
            unexpected_observed = @($observedRelation | Where-Object { -not $expectedKeys.Contains((Pair-Key $_)) }).Count
            recall = $recall
            precision_diagnostic = $precision
            threshold = 0.90
            pass = $recall -ge 0.90
        })
    }
    $score = [ordered]@{
        schema = 'clean-ctx/ngrx-recall-benchmark-score'
        schema_version = 1
        benchmark_version = [string]$oracle.benchmark_version
        oracle_sha256 = $actualOracleHash
        relations = $scores.ToArray()
        all_relations_pass = @($scores | Where-Object { -not $_.pass }).Count -eq 0
        unexpected_pairs_require_manual_review = $unexpected.Count
    }
    $falsePositiveReview = @($unexpected | ForEach-Object {
        [pscustomobject]@{
            pair = $_
            classification = 'PENDING_MANUAL_REVIEW'
            note = 'Frozen oracle was not mutated; classify after the scored run.'
        }
    })
    Save-B 'observed' $observedValues
    Save-B 'score' $score
    Save-B 'false-negatives' $falseNegatives
    Save-B 'false-positive-review' $falsePositiveReview
    $completed = $true
} finally {
    if ($null -ne $session) {
        try { Stop-BSession $session } catch { Write-Warning $_.Exception.Message }
    }
    $provenance.completed_utc = [DateTime]::UtcNow.ToString('o')
    $provenance.completed = $completed
    Save-B 'provenance' $provenance
    Consolidate-RpcEvidence
    Write-Host "NgRx recall benchmark evidence: $runRoot"
}

if (-not $completed) { exit 2 }
if (@($score.relations | Where-Object { -not $_.pass }).Count) { exit 1 }
