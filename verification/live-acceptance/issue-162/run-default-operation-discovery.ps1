# Agent MaxHeadRoom: bounded discovery only; no optimization or composed closure.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/default-operation-discovery-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Discovery receipt: $logs"
    $protected=@('src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs',
        'src/tests/diff/original_matcher_af_perf_002.rs','src/tests/diff/overload_equivalence_af_perf_002.rs',
        'src/tests/mcp/overload_budget_af_perf_002.rs','src/tests/mcp/diff_work_af_perf_002.rs')
    $before=@{}
    foreach($path in $protected){$before[$path]=(Get-FileHash -LiteralPath $path).Hash}
    $changed=@('src/ir/binary_wire.rs','src/mcp/sqlite_store.rs','src/lib.rs',
        'src/tests/architecture/default_operation_observations_af_perf_002.rs',
        'src/tests/mcp/default_operation_work_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Formatting failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='af002_default_public_';count=2},
        @{filter='save_context_checkpoints_only_the_requested_file_and_reloads_exactly';count=1},
        @{filter='tracked_and_durable_only_history_are_read_without_publication';count=1},
        @{filter='test_round_trip_full_ir';count=1}
    )) {
        $log=Join-Path $logs ($case.filter+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        Get-Content -LiteralPath $log -Tail 8
        $summary=[regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if($code -ne 0 -or -not $summary.Success -or [int]$summary.Groups[1].Value -ne $case.count){throw "Tracked GREEN missing: $log"}
    }
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_default_public_.log')) | Where-Object {$_ -match '^AF002_DEFAULT '})
    if($rows.Count -ne 46){throw "Expected 46 discovery rows, found $($rows.Count)"}
    $parsed=@($rows | ForEach-Object { $_.Substring('AF002_DEFAULT '.Length) | ConvertFrom-Json })
    foreach($phase in @('cold_compress','checkpoint','already_durable','checkpoint_history')) {
        if(@($parsed | Where-Object phase -eq $phase).Count -ne 10){throw "Incomplete phase: $phase"}
    }
    if(@($parsed | Where-Object phase -eq 'seeded_metadata_history').Count -ne 6){throw 'Incomplete seeded history rows'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    foreach($path in $protected){if((Get-FileHash -LiteralPath $path).Hash -ne $before[$path]){throw "Protected source changed: $path"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-default-operation-discovery.ps1 docs/agent/AF_PERF_002_DEFAULT_OPERATION_DISCOVERY.md)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Tracked default-operation characterization';trackedPasses=5;
        operationRows=$parsed;files=$hashes;protectedIdentities=$before;optimizationImplemented=$false;
        composedExecuted=$false;limits='Logical work and offered bytes, no allocator/inspected-byte or latency-benefit proof'} |
        ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Discovery complete: five tracked passes and 46 operation intervals; not optimization RED: $logs"
} finally {Pop-Location}
