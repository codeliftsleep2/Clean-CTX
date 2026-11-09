# Agent MaxHeadRoom: inspect integrated cost before mutation/composed closure.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/diff-source-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/tool_helpers.rs','src/mcp/state/source_cache.rs','src/cache.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $packet=@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/diff_ownership_design_af_perf_002.rs','src/tests/mcp/diff_ownership_cases_af_perf_002.rs','src/tests/mcp/diff_source_integrated_af_perf_002.rs','src/tests/mcp/rolling_read_discovery_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-diff-source-budget.ps1 -Phase After *> (Join-Path $logs 'after.log')
    if($LASTEXITCODE -ne 0){throw "Current frozen After failed: $logs"}
    foreach($filter in @('afperf_diff_source_static','af002_diff_source_integrated_cost','af002_diff_ownership_public_equivalence','af002_diff_ownership_errors_and_snapshot_lifetime','af002_diff_ownership_persistence_and_restart')){
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -or $text -match '(?m)^warning:'){throw "Focused tracked failure: $log"}
    }
    $prefix='AF002_DIFF_SOURCE_INTEGRATED_COST '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_diff_source_integrated_cost.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 20){throw 'Incomplete integrated costs'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-diff-source-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=8;costRows=$rows;files=$files;protectedIdentities=$before;costReviewRequired=$true;mutationExecuted=$false;composedExecuted=$false}|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Eight tracked passes;20 integrated cost rows. Inspect before mutation/composed: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
