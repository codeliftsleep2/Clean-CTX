# Agent MaxHeadRoom: test-only design comparison; no optimization RED/closure.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/root-inventory-discovery-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/state.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/ir/binary_wire/decode.rs',
        'src/tests/ir/original_binary_encoder_af_perf_002.rs','src/diff/differ.rs','src/diff/overload_lookup.rs',
        'src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{}
    foreach($path in $protected){$before[$path]=(Get-FileHash $path).Hash}
    $tests=@('src/tests/mcp/root_inventory_discovery_af_perf_002.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/acquisition.rs')
    & rustfmt --edition 2024 --config skip_children=true @tests *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Formatting failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $log=Join-Path $logs 'design.log'
    & cargo test --lib --all-features --quiet af002_root_inventory_ -- --test-threads=1 --nocapture *> $log
    if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 2 passed; 0 failed; 0 ignored;'){throw "Tracked design comparison failed: $log"}
    $rows=@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_ROOT_INVENTORY ')}|ForEach-Object{$_.Substring('AF002_ROOT_INVENTORY '.Length)|ConvertFrom-Json})
    if($rows.Count -ne 36){throw 'Incomplete36 operation rows'}
    $controls=@('incomplete_authorized_root_cannot_earn_declaration_completeness')
    foreach($filter in $controls){
        $controlLog=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $controlLog
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($controlLog) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Control failed: $controlLog"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($path in $protected){if((Get-FileHash $path).Hash -ne $before[$path]){throw "Protected source changed: $path"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-root-inventory-discovery.ps1 docs/agent/AF_PERF_002_ROOT_INVENTORY_DISCOVERY.md)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Tracked test-only broad default-path discovery';trackedPasses=3;
        operationRows=$rows;files=$files;protectedIdentities=$before;productionReplacementImplemented=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Three tracked tests GREEN;36 operation rows. Full evidence: $logs"
} finally {Pop-Location}
