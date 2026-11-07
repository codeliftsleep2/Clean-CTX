# AF-PERF-001: tracked tests are the gate; this script selects and preserves evidence.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/af-perf-001-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "AF-PERF-001 logs: $logDirectory"
    foreach ($selection in @(@{filter='af_perf_001_';minimum=14},@{filter='issue_162_replay_p0_';minimum=5},@{filter='af_delegated_001_';minimum=6},@{filter='af174_';minimum=9},@{filter='af002_a3_';minimum=4},@{filter='af173_binding_';minimum=4},@{filter='compression_token_work::c1_budget_public_raw_count_once';minimum=1},@{filter='content_economics::tests::c1_equivalence_counted_and_recounted_paths';minimum=1},@{filter='afperf_c1_';minimum=2},@{filter='ir::hierarchical::placement_repair::';minimum=2},@{filter='afperf_hierarchy_';minimum=1},@{filter='mcp::tool_handlers::stats::format_budget_af_perf_002::';minimum=1},@{filter='af002_stats_static_format_sinks';minimum=1},@{filter='artifact_operation_work_af_perf_002::a1_clone_budget::';minimum=2},@{filter='afperf_a1_';minimum=2},@{filter='af002_a4_lookup_';minimum=2},@{filter='afperf_a4_';minimum=2},@{filter='af002_a4_public_lookup_budget';minimum=1},@{filter='af002_edit_end_';minimum=2},@{filter='afperf_edit_end_';minimum=3})) {
        $testLog = Join-Path $logDirectory (($selection.filter -replace '[^A-Za-z0-9_.-]', '_') + '.log')
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 40
        if ($testExit -ne 0) { throw "AF-PERF-001 tracked tests failed: $testLog" }
        $summary = [regex]::Match((Get-Content -LiteralPath $testLog -Raw),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -lt $selection.minimum) { throw "Missing/ignored required tests: $testLog" }
    }
    # Proxy correctness regressions are a separate package, not main-library tests.
    $proxyLog = Join-Path $logDirectory 'proxy_scrub_correctness.log'
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_proxy_ -- --test-threads=1 --nocapture *> $proxyLog
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath $proxyLog -Pattern 'test result: ok\. 6 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Proxy scrub correctness gate failed: $proxyLog" }
    $prefilterLog = Join-Path $logDirectory 'proxy_prefilter_correctness.log'
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_prefilter_ -- --test-threads=1 --nocapture *> $prefilterLog
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath $prefilterLog -Pattern 'test result: ok\. 4 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Proxy prefilter correctness gate failed: $prefilterLog" }
    $paths = @(git ls-files --cached --others --exclude-standard -- src/edit/locate.rs src/edit/locate/end_join.rs src/tests/edit/exact_end_budget_af_perf_002.rs src/tests/edit/original_end_join_af_perf_002.rs src/tests/edit/exact_end_acceptance_af_perf_002.rs src/tests/mcp/edit_relocation_work_af_perf_002.rs src/ir/csharp_resolution/compilation/dependencies.rs src/ir/csharp_resolution/compilation/dependency_inputs.rs src/tests/ir/a4_lookup_budget_af_perf_002.rs src/tests/ir/a4_original_obligations_af_perf_002.rs src/tests/ir/a4_lookup_acceptance_af_perf_002.rs src/tests/mcp/dependency_footprint_af_perf_002.rs src/tests/mcp/a1_clone_budget_af_perf_002.rs src/tests/mcp/a1_clone_static_af_perf_002.rs src/tests/mcp/a1_clone_frozen_af_perf_002.rs src/tests/mcp/fixtures/a1_clone_semantic_responses.json src/mcp/persistence_ir.rs src/mcp/tool_handlers/core/provide_persistence.rs src/mcp/tool_handlers/stats/mod.rs src/tests/mcp/stats_format_budget_af_perf_002.rs src/tests/mcp/stats_format_static_af_perf_002.rs src/tests/mcp/fixtures/stats_format_responses.json proxy/src/scrub.rs proxy/src/scrub_patterns.rs proxy/src/pipeline.rs proxy/src/transform.rs proxy/src/lib.rs src/tests/proxy src/ir/hierarchical.rs src/ir/hierarchical src/tests/ir/hierarchy_placement_budget_af_perf_002.rs src/mcp/tool_handlers/core/compress.rs src/mcp/content_economics.rs src/mcp/tool_helpers.rs src/tests/mcp/content_economics.rs src/tests/mcp/c1_public_response_af_perf_002.rs src/tests/mcp/fixtures/c1_public_responses.json src/tests/mcp/compression_token_work_af_perf_002.rs src/ir/csharp_exact_calls.rs src/tests/ir/csharp_exact_calls.rs src/tests/mcp/csharp_a3_budget_af_perf_002.rs src/tests/mcp/artifact_operation_work_af_perf_002.rs src/tests/mcp/ngrx173_binding_semantics.rs src/tests/mcp/ngrx_issue_152_semantic_envelope.rs src/workspace src/tests/workspace src/angular_meta src/ir/typescript_resolution src/tests/architecture src/tests/mcp/angular_admission* src/tests/mcp/angular_resolver_replay_p0_issue_162.rs src/mcp/semantic_compilation.rs src/mcp/tool_handlers/query.rs src/lib.rs scripts/check-semantic-work-fitness.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='tracked Rust AST/direct/lifecycle/P0 fitness tests';files=$hashes;scope='retained admission; unresolved AST findings require review; not full acceptance'} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
} finally { Pop-Location }
