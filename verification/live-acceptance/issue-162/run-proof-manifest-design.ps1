# Agent MaxHeadRoom: test-only design; no optimization RED or closure gates.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/proof-manifest-design-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/state.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $tests=@('src/tests/mcp/root_inventory_discovery_af_perf_002.rs','src/tests/mcp/proof_manifest_design_af_perf_002.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/test_support.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/acquisition.rs')
    & rustfmt --edition 2024 --config skip_children=true @tests *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    $instrumented=@{};foreach($p in $tests){$instrumented[$p]=(Get-FileHash $p).Hash}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $log=Join-Path $logs 'design.log'
    & cargo test --lib --all-features --quiet af002_proof_manifest_ -- --test-threads=1 --nocapture *> $log
    if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 3 passed; 0 failed; 0 ignored;'){throw "Tracked design failure: $log"}
    $costPrefix='AF002_PROOF_MANIFEST_COST ';$publicPrefix='AF002_PROOF_MANIFEST_PUBLIC '
    $costRows=@();$publicRows=@()
    foreach($line in [IO.File]::ReadAllLines($log)) {
        if($line.StartsWith($costPrefix)){$costRows+=($line.Substring($costPrefix.Length)|ConvertFrom-Json)}
        if($line.StartsWith($publicPrefix)){$publicRows+=($line.Substring($publicPrefix.Length)|ConvertFrom-Json)}
    }
    $durableRows=@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_PROOF_MANIFEST_DURABLE ')}|ForEach-Object{$_.Substring('AF002_PROOF_MANIFEST_DURABLE '.Length)|ConvertFrom-Json})
    if($durableRows.Count -ne 1){throw 'Missing enabled-persistence proof control'}
    if($costRows.Count -ne 24 -or $publicRows.Count -ne 60){throw 'Incomplete comparison populations'}
    foreach($filter in @('incomplete_authorized_root_cannot_earn_declaration_completeness','af002_root_inventory_contracts')) {
        $control=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $control
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($control) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Control failure: $control"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected source drift: $p"}}
    foreach($p in $tests){if((Get-FileHash $p).Hash -ne $instrumented[$p]){throw "Instrumented source drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-proof-manifest-design.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=5;costRows=$costRows;publicRows=$publicRows;durableRows=$durableRows;files=$files;protectedIdentities=$before;testOnlySwitches=$instrumented;productionOptimizationImplemented=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Five tracked passes;60 public and24 cost rows. Evidence: $logs"
} finally {Pop-Location}
