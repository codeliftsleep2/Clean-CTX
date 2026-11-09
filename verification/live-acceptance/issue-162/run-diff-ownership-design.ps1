# Agent MaxHeadRoom: test-only design; no production correction or closure gates.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/diff-ownership-design-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/tool_helpers.rs','src/mcp/state/source_cache.rs','src/cache.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $packet=@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/diff_ownership_design_af_perf_002.rs','src/tests/mcp/diff_ownership_cases_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Packet formatting failed'}
    $instrumented=@{};foreach($p in $packet){$instrumented[$p]=(Get-FileHash $p).Hash}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($group in @(@{filter='af002_diff_ownership_';count=4},@{filter='af002_rolling_read_';count=2},@{filter='explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore';count=1})) {
        $log=Join-Path $logs ($group.filter+'.log')
        & cargo test --lib --all-features --quiet $group.filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch "test result: ok\. $($group.count) passed; 0 failed; 0 ignored;" -or $text -match '(?m)^warning:'){throw "Tracked design failure: $log"}
    }
    $rows=@{};$textLines=[IO.File]::ReadAllLines((Join-Path $logs 'af002_diff_ownership_.log'))
    foreach($kind in @('PUBLIC','WORK','LIFETIME','PERSISTENT','COST')) {
        $prefix='AF002_DIFF_OWNERSHIP_'+$kind+' '
        $rows[$kind]=@($textLines|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    }
    if($rows.PUBLIC.Count -ne 210 -or $rows.WORK.Count -ne 434 -or $rows.LIFETIME.Count -ne 5 -or $rows.PERSISTENT.Count -ne 7 -or $rows.COST.Count -ne 20){throw 'Incomplete paired populations'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected source drift: $p"}}
    foreach($p in $packet){if((Get-FileHash $p).Hash -ne $instrumented[$p]){throw "Executed packet drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-diff-ownership-design.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=7;rows=$rows;files=$files;protectedIdentities=$before;packetIdentities=$instrumented;testOnlyAlternative=$true;productionOptimizationImplemented=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Seven tracked passes;210 paired public rows;20 cost rows;5 lifetime controls. Evidence: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
