# Agent MaxHeadRoom: actual integrated cost/semantic gates; inspect before mutation.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_BINARY_PUBLIC_ORACLE
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/binary-reverse-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    Write-Output "Integrated reverse-map review: $logs"
    $before=Join-Path $repoPath 'target/af-perf-002/binary-reverse-before-20261108-160709'
    $r=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
    if($r.semanticPasses -ne 3 -or $r.budgetExit -eq 0 -or $r.publicCases -ne 40){throw 'Genuine original RED required'}
    $frozen=@('src/tests/mcp/binary_reverse_budget_af_perf_002.rs','src/tests/mcp/binary_clone_public_af_perf_002.rs',
        'src/tests/mcp/default_operation_work_af_perf_002.rs','src/tests/architecture/default_operation_observations_af_perf_002.rs',
        'src/tests/ir/original_binary_reverse_af_perf_002.rs','src/tests/ir/binary_reverse_original_decode_af_perf_002.rs',
        'src/tests/ir/binary_reverse_equivalence_af_perf_002.rs','src/tests/ir/binary_reverse_design_af_perf_002.rs',
        'src/tests/ir/binary_clone_equivalence_af_perf_002.rs','src/tests/ir/binary_clone_fixtures_af_perf_002.rs',
        'src/tests/ir/binary_reverse_fixtures_af_perf_002.rs','src/tests/mcp/fixtures/binary_clone_public_identities.json',
        'src/ir/binary_wire/decode.rs','src/ir/wire.rs')
    foreach($file in $frozen){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Frozen drift: $file"}}
    if((Get-FileHash src/tests/ir/fixtures/binary_reverse_original_string_table.txt).Hash -ne (Get-FileHash (Join-Path $before 'production-string-table.rs')).Hash){throw 'Original table fixture drift'}
    $env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $repoPath 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
    & rustfmt --edition 2024 --config skip_children=true src/lib.rs src/ir/string_table.rs src/ir/binary_wire.rs src/tests/ir/binary_clone_protection_af_perf_002.rs src/tests/ir/binary_encoder_missing_probe_af_perf_002.rs src/tests/ir/binary_reverse_integrated_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $groups=@()
    foreach($filter in @('af002_binary_reverse_integrated_cost','afperf_binary_reverse_static','afperf_binary_reverse_accessor_law',
        'af002_binary_frozen_semantics','af002_binary_reverse_frozen_equivalence','af002_binary_public_identity',
        'af002_binary_reverse_public_budget','af002_binary_public_budget','afperf_binary_clone_static',
        'ir::binary_wire::tests::','ir::binary_wire::v04_tests::','ir::string_table::tests::',
        'save_context_checkpoints_only_the_requested_file_and_reloads_exactly',
        'tracked_and_durable_only_history_are_read_without_publication',
        'registered_edit_is_byte_exact_and_commits_source_durable_and_live_together',
        'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore')) {
        $log=Join-Path $logs (($filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        $summary=[regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if($code -ne 0 -or -not $summary.Success -or [int]$summary.Groups[1].Value -lt 1){throw "Tracked GREEN missing: $log"}
        $groups+=@{filter=$filter;passes=[int]$summary.Groups[1].Value}
        Write-Output "$filter : $($summary.Groups[1].Value) passes"
    }
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_binary_reverse_integrated_cost.log'))|Where-Object{$_.StartsWith('AF002_BINARY_REVERSE_INTEGRATED ')}|ForEach-Object{$_.Substring('AF002_BINARY_REVERSE_INTEGRATED '.Length)|ConvertFrom-Json})
    if($rows.Count -ne 12){throw 'Incomplete12 integrated cost populations'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($file in $frozen){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Post-gate frozen drift: $file"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-binary-reverse-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=($groups.passes|Measure-Object -Sum).Sum;groups=$groups;costRows=$rows;
        files=$files;mutationExecuted=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 12|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Focused complete; inspect integrated costs before mutation/composed: $logs"
} finally {$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle;Pop-Location}
