# Agent MaxHeadRoom: costs must be inspected before mutation/composed closure.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/manifest-demand-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/state.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $hashes=@{};foreach($p in $protected){$hashes[$p]=(Get-FileHash $p).Hash}
    $format=@('src/tests/mcp/manifest_demand_integrated_af_perf_002.rs','src/tests/mcp/proof_manifest_design_af_perf_002.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs')
    & rustfmt --edition 2024 --config skip_children=true @format *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-manifest-demand-budget.ps1 -Phase After *> (Join-Path $logs 'after.log')
    if($LASTEXITCODE -ne 0){throw "Frozen After failed: $logs"}
    $groups=@(@{filter='af002_manifest_integrated_';count=2},@{filter='incomplete_authorized_root_cannot_earn_declaration_completeness';count=1},@{filter='af002_proof_manifest_unknown_demand';count=1})
    foreach($group in $groups){
        $log=Join-Path $logs ($group.filter+'.log')
        & cargo test --lib --all-features --quiet $group.filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch "test result: ok\. $($group.count) passed; 0 failed; 0 ignored;" -or $text -match '(?m)^warning:'){throw "Tracked focused failure: $log"}
    }
    $prefix='AF002_MANIFEST_INTEGRATED_COST '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_manifest_integrated_.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 24){throw 'Incomplete integrated cost populations'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $hashes[$p]){throw "Protected source drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-manifest-demand-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=7;costRows=$rows;files=$files;protectedIdentities=$hashes;costReviewRequired=$true;mutationExecuted=$false;composedExecuted=$false}|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Seven tracked passes;24 integrated cost rows. Inspect before closure gates: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
