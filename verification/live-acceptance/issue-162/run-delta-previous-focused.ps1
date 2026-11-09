# Agent MaxHeadRoom: inspect integrated cost before mutation/composed closure.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/delta-previous-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/tool_helpers.rs','src/mcp/state/source_cache.rs','src/mcp/state/publication.rs','src/cache.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $packet=@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/delta_previous_design_af_perf_002.rs','src/tests/mcp/delta_previous_cases_af_perf_002.rs','src/tests/mcp/delta_previous_integrated_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    $executed=@{};foreach($p in $packet){$executed[$p]=(Get-FileHash $p).Hash}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-delta-previous-budget.ps1 -Phase After *> (Join-Path $logs 'after.log')
    if($LASTEXITCODE -ne 0){throw "Current frozen After failed: $logs"}
    foreach($filter in @('afperf_delta_previous_static','af002_delta_previous_integrated_cost','af002_delta_previous_lock_boundary')){
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -or $text -match '(?m)^warning:'){throw "Focused tracked failure: $log"}
    }
    $prefix='AF002_DELTA_PREVIOUS_INTEGRATED_COST '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_delta_previous_integrated_cost.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 16){throw 'Incomplete integrated costs'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected drift: $p"}}
    foreach($p in $executed.Keys){if((Get-FileHash $p).Hash -ne $executed[$p]){throw "Executed source drift: $p"}}
    $workPrefix='AF002_DELTA_PREVIOUS_INTEGRATED_WORK '
    $workRows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_delta_previous_integrated_cost.log'))|Where-Object{$_.StartsWith($workPrefix)}|ForEach-Object{$_.Substring($workPrefix.Length)|ConvertFrom-Json})
    if($workRows.Count -ne 16){throw 'Incomplete enabled integrated work controls'}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-delta-previous-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=9;costRows=$rows;workRows=$workRows;executedIdentities=$executed;files=$files;protectedIdentities=$before;costReviewRequired=$true;mutationExecuted=$false;composedExecuted=$false}|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Nine tracked passes;16 integrated cost rows. Inspect before mutation/composed: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
