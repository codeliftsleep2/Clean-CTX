# Agent MaxHeadRoom: current-byte acceptance and integrated cost review only.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_SQLITE_TEXT_ORACLE
Push-Location $repoPath
try {
    $before=Join-Path $repoPath 'target/af-perf-002/sqlite-text-before-20261108-173446'
    $prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
    if($prior.semanticPasses -ne 2 -or $prior.publicCases -ne 12 -or $prior.budgetExit -eq 0){throw 'Genuine original RED required'}
    $frozen=@('src/tests/mcp/sqlite_text_repair_af_perf_002.rs','src/tests/mcp/sqlite_text_original_af_perf_002.rs')
    foreach($p in $frozen){if((Get-FileHash $p).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($p)))).Hash){throw "Frozen drift: $p"}}
    if((Get-FileHash src/tests/mcp/sqlite_text_before_af_perf_002.txt).Hash -ne (Get-FileHash (Join-Path $before 'production-sqlite.rs')).Hash){throw 'Original text fixture drift'}
    $env:AF002_SQLITE_TEXT_ORACLE=Join-Path $before 'public-oracle.json'
    $logs=Join-Path $repoPath ('target/af-perf-002/sqlite-text-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $tests=@('src/tests/mcp/sqlite_text_integrated_af_perf_002.rs','src/tests/mcp/sqlite_text_cost_original_af_perf_002.rs','src/tests/mcp/sqlite_work_af_perf_002.rs','src/tests/mcp/sqlite_text_design_af_perf_002.rs','src/tests/architecture/default_operation_observations_af_perf_002.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs')
    & rustfmt --edition 2024 --config skip_children=true @tests *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $groups=@();$passes=0
    foreach($filter in @('af002_sqlite_text_frozen_semantics','af002_sqlite_text_public_identity','af002_sqlite_text_public_budget','afperf_sqlite_text_static','af002_sqlite_text_integrated_cost','mcp::sqlite_store::tests::','mcp::sqlite_store::compatibility_persistence_tests::','save_context_checkpoints_only_the_requested_file_and_reloads_exactly','tracked_and_durable_only_history_are_read_without_publication','ir::binary_wire::tests::','af002_binary_reverse_frozen_equivalence')) {
        $log=Join-Path $logs (($filter -replace ':','_')+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
        if($code -ne 0 -or $text -notmatch 'test result: ok\. ([1-9][0-9]*) passed; 0 failed; 0 ignored;'){throw "Tracked GREEN missing: $log"}
        $count=[int]$Matches[1];$passes+=$count;$groups+=@{filter=$filter;passes=$count}
        Write-Output "$filter : $count passed"
    }
    $prefix='AF002_SQLITE_TEXT_INTEGRATED '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_sqlite_text_integrated_cost.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 12){throw 'Incomplete integrated cost rows'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $frozen){if((Get-FileHash $p).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($p)))).Hash){throw "Frozen drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-sqlite-text-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=$passes;groups=$groups;costRows=$rows;files=$files;mutationExecuted=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 12|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Focused acceptance complete; inspect costs before mutation/composed: $logs"
} finally {Pop-Location;$env:AF002_SQLITE_TEXT_ORACLE=$oldOracle}
