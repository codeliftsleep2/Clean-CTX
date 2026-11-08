# Agent MaxHeadRoom: test-only design comparison; no optimization RED/closure.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/binary-reverse-design-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/ir/binary_wire.rs','src/ir/string_table.rs','src/ir/binary_wire/decode.rs',
        'src/tests/ir/original_binary_encoder_af_perf_002.rs','src/diff/differ.rs','src/diff/overload_lookup.rs',
        'src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{}
    foreach($path in $protected){$before[$path]=(Get-FileHash $path).Hash}
    $tests=@('src/tests/ir/binary_reverse_design_af_perf_002.rs','src/tests/ir/binary_reverse_table_af_perf_002.rs',
        'src/tests/ir/binary_reverse_encoder_af_perf_002.rs','src/tests/ir/binary_reverse_fixtures_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @tests src/lib.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Formatting failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $log=Join-Path $logs 'design.log'
    & cargo test --lib --all-features --quiet af002_binary_reverse_design_ -- --test-threads=1 --nocapture *> $log
    if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 2 passed; 0 failed; 0 ignored;'){throw "Tracked design comparison failed: $log"}
    $rows=@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_BINARY_REVERSE ')}|ForEach-Object{$_.Substring('AF002_BINARY_REVERSE '.Length)|ConvertFrom-Json})
    if($rows.Count -ne 12){throw 'Incomplete12 cost dimensions'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($path in $protected){if((Get-FileHash $path).Hash -ne $before[$path]){throw "Protected source changed: $path"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-binary-reverse-design.ps1 docs/agent/AF_PERF_002_BINARY_REVERSE_DESIGN.md)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Tracked test-only encoder reverse-map design comparison';trackedPasses=2;
        costRows=$rows;files=$files;protectedIdentities=$before;productionReplacementImplemented=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 12|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Two design tests GREEN;12 cost rows. Full evidence: $logs"
} finally {Pop-Location}
