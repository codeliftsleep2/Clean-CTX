# Agent MaxHeadRoom: discovery only; no optimization RED or closure gates.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/rolling-read-discovery-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $manifest=@{
        'src/mcp/tool_handlers/hydration.rs'='f86d89ee4568f1ab19053857df512d7f82542ed4440ed4af0e8b319f927d1e6d'
        'src/mcp/tool_handlers/hydration/filesystem.rs'='28c8066a33acdc07f476904507f06977c1d84269aef267daaf15c6f2c5fc9474'
        'src/mcp/tool_handlers/hydration/filesystem_discovery.rs'='3198da17e775822a3b930196e01d76a7abb1f9d701c7bc847b366e3094f6067f'
    }
    foreach($p in $manifest.Keys){if((Get-FileHash $p).Hash -ne $manifest[$p]){throw "Closed manifest identity drift: $p"}}
    $protected=@('src/mcp/state.rs','src/mcp/state/source_cache.rs','src/mcp/tool_helpers.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')+@($manifest.Keys)
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $packet=@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/rolling_read_discovery_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Packet formatting failed'}
    $instrumented=@{};foreach($p in $packet){$instrumented[$p]=(Get-FileHash $p).Hash}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($group in @(@{filter='af002_rolling_read_';count=2},@{filter='explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore';count=1},@{filter='matching_canonical_epoch_accepts_delta_and_advances_chain';count=1})) {
        $log=Join-Path $logs ($group.filter+'.log')
        & cargo test --lib --all-features --quiet $group.filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch "test result: ok\. $($group.count) passed; 0 failed; 0 ignored;" -or $text -match '(?m)^warning:'){throw "Tracked characterization failure: $log"}
    }
    $prefix='AF002_ROLLING_READ '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_rolling_read_.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 96){throw 'Incomplete public characterization rows'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected source changed: $p"}}
    foreach($p in $packet){if((Get-FileHash $p).Hash -ne $instrumented[$p]){throw "Executed packet drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-rolling-read-discovery.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=4;publicRows=$rows;files=$files;protectedIdentities=$before;packetIdentities=$instrumented;discoveryOnly=$true;composedExecuted=$false}|
        ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Four tracked passes;96 public characterization rows. Evidence: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
