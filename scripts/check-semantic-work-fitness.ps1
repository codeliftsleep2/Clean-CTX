# AF-PERF-001: tracked tests are the gate; this script selects and preserves evidence.
param([string]$ResumeLogDirectory='')
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$oldBinaryOracle=$env:AF002_BINARY_PUBLIC_ORACLE
$oldSqliteOracle=$env:AF002_SQLITE_TEXT_ORACLE
$oldManifestRoot=$env:AF002_MANIFEST_FIXTURE_ROOT
$oldManifestOracle=$env:AF002_MANIFEST_ORACLE
$oldConsoleEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $repoPath 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
    $env:AF002_SQLITE_TEXT_ORACLE=Join-Path $repoPath 'src/tests/mcp/fixtures/sqlite_text_public_identities.json'
    # Local closure replays the exact original path-dependent proof oracle.
    # Fresh checkouts still exercise tracked reachability/work/static contracts.
    $manifestBefore=Join-Path $repoPath 'target/af-perf-002/manifest-demand-before-20261108-231514'
    if(Test-Path (Join-Path $manifestBefore 'public-oracle.json')) {
        $env:AF002_MANIFEST_FIXTURE_ROOT=Join-Path $manifestBefore 'fixtures'
        $env:AF002_MANIFEST_ORACLE=Join-Path $manifestBefore 'public-oracle.json'
    }
    $reusedGroups=@()
    $resumeProof=$null
    if($ResumeLogDirectory) {
        $logDirectory=(Resolve-Path -LiteralPath $ResumeLogDirectory).Path
        $allowed=[IO.Path]::GetFullPath((Join-Path $repoPath 'target/issue-162'))+[IO.Path]::DirectorySeparatorChar
        if(-not $logDirectory.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path $logDirectory -Leaf) -notmatch '^af-perf-001-\d{8}-\d{6}$'){throw 'Invalid resume evidence directory'}
        if(Test-Path (Join-Path $logDirectory 'receipt.json')){throw 'Already completed; no resume required'}
        $resumeProof=(Get-ChildItem (Join-Path $repoPath 'target/af-perf-002') -Directory -Filter 'binary-clone-mutation-*'|Where-Object{Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $resumeProof){throw 'No verified source inventory for resume'}
        $proof=Get-Content (Join-Path $resumeProof 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        $sourceFiles=@($proof.files|Where-Object{$_.path -match '^(src/|proxy/|Cargo\.toml$|Cargo\.lock$)'})
        if($sourceFiles.Count -lt 100){throw 'Incomplete resume source inventory'}
        foreach($file in $sourceFiles){if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Source changed; cannot reuse tests: $($file.path)"}}
    } else {
        $logDirectory = Join-Path $repoPath ('target/issue-162/af-perf-001-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
        New-Item -ItemType Directory -Path $logDirectory | Out-Null
    }
    Write-Output "AF-PERF-001 logs: $logDirectory"
    foreach ($selection in @(@{filter='af_perf_001_';minimum=14},@{filter='issue_162_replay_p0_';minimum=5},@{filter='af_delegated_001_';minimum=6},@{filter='af174_';minimum=9},@{filter='af002_a3_';minimum=4},@{filter='af173_binding_';minimum=4},@{filter='compression_token_work::c1_budget_public_raw_count_once';minimum=1},@{filter='content_economics::tests::c1_equivalence_counted_and_recounted_paths';minimum=1},@{filter='afperf_c1_';minimum=2},@{filter='ir::hierarchical::placement_repair::';minimum=2},@{filter='afperf_hierarchy_';minimum=1},@{filter='mcp::tool_handlers::stats::format_budget_af_perf_002::';minimum=1},@{filter='af002_stats_static_format_sinks';minimum=1},@{filter='artifact_operation_work_af_perf_002::a1_clone_budget::';minimum=2},@{filter='afperf_a1_';minimum=2},@{filter='af002_a4_lookup_';minimum=2},@{filter='afperf_a4_';minimum=2},@{filter='af002_a4_public_lookup_budget';minimum=1},@{filter='af002_edit_end_';minimum=2},@{filter='afperf_edit_end_';minimum=3},@{filter='af002_diff_shortcut_';minimum=2},@{filter='afperf_diff_shortcut_';minimum=2},@{filter='af002_overload_frozen_equivalence';minimum=1},@{filter='af002_overload_public_budget';minimum=1},@{filter='afperf_overload_static';minimum=1},@{filter='afperf_overload_full_equality_lookup';minimum=1},@{filter='af002_overload_integrated_frozen_public';minimum=1},@{filter='af002_binary_frozen_semantics';minimum=1},@{filter='af002_binary_public_identity';minimum=1},@{filter='af002_binary_public_budget';minimum=1},@{filter='afperf_binary_clone_static';minimum=1},@{filter='af002_binary_reverse_frozen_equivalence';minimum=1},@{filter='af002_binary_reverse_public_budget';minimum=1},@{filter='afperf_binary_reverse_static';minimum=1},@{filter='afperf_binary_reverse_accessor_law';minimum=1},@{filter='af002_sqlite_text_frozen_semantics';minimum=1},@{filter='af002_sqlite_text_public_identity';minimum=1},@{filter='af002_sqlite_text_public_budget';minimum=1},@{filter='afperf_sqlite_text_static';minimum=1},@{filter='afperf_sqlite_text_validation_law';minimum=1},@{filter='af002_manifest_frozen_public_semantics';minimum=1},@{filter='af002_manifest_unused_budget';minimum=1},@{filter='af002_manifest_integrated_static';minimum=1},@{filter='af002_root_inventory_contracts';minimum=1},@{filter='af002_proof_manifest_unknown_demand';minimum=1},@{filter='incomplete_authorized_root_cannot_earn_declaration_completeness';minimum=1})) {
        $testLog = Join-Path $logDirectory (($selection.filter -replace '[^A-Za-z0-9_.-]', '_') + '.log')
        if($ResumeLogDirectory -and (Test-Path -LiteralPath $testLog)) {
            $prior=[regex]::Match([IO.File]::ReadAllText($testLog),'(?m)^test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
            if($prior.Success -and [int]$prior.Groups[1].Value -ge $selection.minimum) {
                $reusedGroups+=@{filter=$selection.filter;passes=[int]$prior.Groups[1].Value;sha256=(Get-FileHash $testLog).Hash}
                Write-Output "Preserved $($selection.filter): $($prior.Value)"
                continue
            }
            Move-Item -LiteralPath $testLog -Destination ($testLog+'.interrupted-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
        }
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $testLog
        $testExit = $LASTEXITCODE
        if ($testExit -ne 0) { throw "AF-PERF-001 tracked tests failed: $testLog" }
        $summary = [regex]::Match((Get-Content -LiteralPath $testLog -Raw),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -lt $selection.minimum) { throw "Missing/ignored required tests: $testLog" }
        Write-Output "$($selection.filter): $($summary.Value)"
    }
    # Proxy correctness regressions are a separate package, not main-library tests.
    $proxyLog = Join-Path $logDirectory 'proxy_scrub_correctness.log'
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_proxy_ -- --test-threads=1 --nocapture *> $proxyLog
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath $proxyLog -Pattern 'test result: ok\. 6 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Proxy scrub correctness gate failed: $proxyLog" }
    $prefilterLog = Join-Path $logDirectory 'proxy_prefilter_correctness.log'
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_prefilter_ -- --test-threads=1 --nocapture *> $prefilterLog
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath $prefilterLog -Pattern 'test result: ok\. 4 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Proxy prefilter correctness gate failed: $prefilterLog" }
    $paths = @(git ls-files --cached --others --exclude-standard -- src/mcp/tool_handlers/hydration.rs src/mcp/tool_handlers/hydration src/tests/mcp/proof_manifest* src/tests/mcp/manifest_* src/tests/mcp/root_inventory_discovery_af_perf_002.rs src/mcp/sqlite_store.rs src/mcp/sqlite_store src/tests/mcp/sqlite_text* src/tests/mcp/fixtures/sqlite_text_public_identities.json src/ir/binary_wire.rs src/ir/binary_wire src/ir/string_table.rs src/ir/wire.rs src/tests/ir/binary_clone* src/tests/ir/binary_reverse* src/tests/ir/original_binary_reverse_af_perf_002.rs src/tests/ir/fixtures/binary_reverse_original_string_table.txt src/tests/mcp/binary_reverse_budget_af_perf_002.rs src/tests/ir/original_binary_encoder_af_perf_002.rs src/tests/ir/binary_encoder_missing_probe_af_perf_002.rs src/tests/mcp/binary_clone_public_af_perf_002.rs src/tests/mcp/default_operation_work_af_perf_002.rs src/tests/mcp/fixtures/binary_clone_public_identities.json src/edit/locate.rs src/edit/locate/end_join.rs src/tests/edit/exact_end_budget_af_perf_002.rs src/tests/edit/original_end_join_af_perf_002.rs src/tests/edit/exact_end_acceptance_af_perf_002.rs src/tests/mcp/edit_relocation_work_af_perf_002.rs src/ir/csharp_resolution/compilation/dependencies.rs src/ir/csharp_resolution/compilation/dependency_inputs.rs src/tests/ir/a4_lookup_budget_af_perf_002.rs src/tests/ir/a4_original_obligations_af_perf_002.rs src/tests/ir/a4_lookup_acceptance_af_perf_002.rs src/tests/mcp/dependency_footprint_af_perf_002.rs src/tests/mcp/a1_clone_budget_af_perf_002.rs src/tests/mcp/a1_clone_static_af_perf_002.rs src/tests/mcp/a1_clone_frozen_af_perf_002.rs src/tests/mcp/fixtures/a1_clone_semantic_responses.json src/mcp/persistence_ir.rs src/mcp/tool_handlers/core/provide_persistence.rs src/mcp/tool_handlers/stats/mod.rs src/tests/mcp/stats_format_budget_af_perf_002.rs src/tests/mcp/stats_format_static_af_perf_002.rs src/tests/mcp/fixtures/stats_format_responses.json proxy/src/scrub.rs proxy/src/scrub_patterns.rs proxy/src/pipeline.rs proxy/src/transform.rs proxy/src/lib.rs src/tests/proxy src/ir/hierarchical.rs src/ir/hierarchical src/tests/ir/hierarchy_placement_budget_af_perf_002.rs src/mcp/tool_handlers/core/compress.rs src/mcp/content_economics.rs src/mcp/tool_helpers.rs src/tests/mcp/diff_shortcut_budget_af_perf_002.rs src/tests/mcp/diff_work_af_perf_002.rs src/tests/mcp/diff_shortcut_protection_af_perf_002.rs src/tests/mcp/fixtures/diff_shortcut_responses.json src/diff src/tests/diff src/tests/mcp/overload_budget_af_perf_002.rs src/tests/mcp/content_economics.rs src/tests/mcp/c1_public_response_af_perf_002.rs src/tests/mcp/fixtures/c1_public_responses.json src/tests/mcp/compression_token_work_af_perf_002.rs src/ir/csharp_exact_calls.rs src/tests/ir/csharp_exact_calls.rs src/tests/mcp/csharp_a3_budget_af_perf_002.rs src/tests/mcp/artifact_operation_work_af_perf_002.rs src/tests/mcp/ngrx173_binding_semantics.rs src/tests/mcp/ngrx_issue_152_semantic_envelope.rs src/workspace src/tests/workspace src/angular_meta src/ir/typescript_resolution src/tests/architecture src/tests/mcp/angular_admission* src/tests/mcp/angular_resolver_replay_p0_issue_162.rs src/mcp/semantic_compilation.rs src/mcp/tool_handlers/query.rs src/lib.rs scripts/check-semantic-work-fitness.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='tracked Rust AST/direct/lifecycle/P0 fitness tests';files=$hashes;scope='retained admission; unresolved AST findings require review; not full acceptance';resumed=[bool]$ResumeLogDirectory;resumeSourceProof=$resumeProof;reusedGroups=$reusedGroups} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
} finally { $env:AF002_BINARY_PUBLIC_ORACLE=$oldBinaryOracle;$env:AF002_SQLITE_TEXT_ORACLE=$oldSqliteOracle;$env:AF002_MANIFEST_FIXTURE_ROOT=$oldManifestRoot;$env:AF002_MANIFEST_ORACLE=$oldManifestOracle;[Console]::OutputEncoding=$oldConsoleEncoding; Pop-Location }
