# Agent MaxHeadRoom: affected current-byte gates, not final composed acceptance.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_BINARY_PUBLIC_ORACLE
function Frozen-Hash([string]$File) {
    if($File -eq 'src/tests/ir/original_binary_encoder_af_perf_002.rs') {
        $adapted=[IO.File]::ReadAllText($File).Replace('binary_clone_original_decode_af_perf_002.rs','../../ir/binary_wire/decode.rs')
        $bytes=[Text.UTF8Encoding]::new($false).GetBytes($adapted)
        return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
    }
    return (Get-FileHash -LiteralPath $File).Hash
}
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/binary-clone-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $before=Join-Path $repoPath 'target/af-perf-002/binary-clone-before-20261108-131355'
    $r=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
    if($r.semanticExit -ne 0 -or $r.publicExit -ne 0 -or $r.budgetExit -eq 0){throw 'Original RED missing'}
    $frozen=@('src/tests/ir/original_binary_encoder_af_perf_002.rs','src/tests/ir/binary_clone_fixtures_af_perf_002.rs',
        'src/tests/ir/binary_encoder_missing_probe_af_perf_002.rs','src/tests/ir/binary_clone_equivalence_af_perf_002.rs',
        'src/tests/mcp/binary_clone_public_af_perf_002.rs','src/tests/mcp/default_operation_work_af_perf_002.rs',
        'src/tests/architecture/default_operation_observations_af_perf_002.rs','src/ir/binary_wire/decode.rs','src/ir/string_table.rs','src/ir/wire.rs')
    foreach($file in $frozen){if((Frozen-Hash $file) -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Frozen source drift: $file"}}
    if((Get-FileHash src/tests/ir/binary_clone_original_decode_af_perf_002.rs).Hash -ne (Get-FileHash (Join-Path $before 'decode.rs')).Hash){throw 'Original decoder copy drift'}
    $fixture='src/tests/mcp/fixtures/binary_clone_public_identities.json'
    $oldIdentities=Get-Content (Join-Path $before 'public-identities.json') -Raw -Encoding utf8|ConvertFrom-Json
    $newIdentities=Get-Content $fixture -Raw -Encoding utf8|ConvertFrom-Json
    if(($oldIdentities|ConvertTo-Json -Depth 50 -Compress) -cne ($newIdentities|ConvertTo-Json -Depth 50 -Compress)){throw 'Tracked original public fixture drift'}
    $env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $repoPath $fixture
    & rustfmt --edition 2024 --config skip_children=true src/lib.rs src/ir/binary_wire.rs src/tests/ir/binary_clone_protection_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $groups=@()
    foreach($filter in @('af002_binary_frozen_semantics','af002_binary_public_identity','af002_binary_public_budget','afperf_binary_clone_static',
        'ir::binary_wire::tests::','ir::binary_wire::v04_tests::',
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
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($file in $frozen){if((Frozen-Hash $file) -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Post-gate frozen source drift: $file"}}
    foreach($item in $r.protectedIdentities.psobject.Properties){if((Get-FileHash $item.Name).Hash -ne $item.Value){throw "Protected identity changed: $($item.Name)"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-binary-clone-focused.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Tracked focused encoder/current public acceptance';groups=$groups;files=$files;
        trackedPasses=($groups.passes|Measure-Object -Sum).Sum;publicCases=40;binaryCases=42;composedExecuted=$false}|
        ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Focused complete: $logs"
} finally {$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle;Pop-Location}
