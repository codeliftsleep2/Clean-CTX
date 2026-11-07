# Agent MaxHeadRoom: exact-end focused semantic/work/static/cost/lifecycle acceptance.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/edit-end-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Edit exact-end focused: $logs"
    & rustfmt --edition 2024 --config skip_children=true src/edit/locate.rs src/edit/locate/end_join.rs src/tests/edit/exact_end_acceptance_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='afperf_edit_end_';count=3},
        @{filter='af002_edit_end_';count=2},
        @{filter='af002_edit_relocation_refusal_controls';count=1},
        @{filter='edit::locate_tests::';count=10},
        @{filter='edit::delete_tests::';count=10},
        @{filter='stale_external_source_is_rejected_before_intent_and_can_be_explicitly_reconciled';count=1},
        @{filter='durable_edit_failure_restores_exact_prior_bytes_and_preserves_live_state';count=1},
        @{filter='candidate_compilation_failure_precedes_source_intent_and_live_mutation';count=1}
    )){
        $log=Join-Path $logs (($case.filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE;Get-Content -LiteralPath $log -Tail 8
        if($code -ne 0 -or [IO.File]::ReadAllText($log) -notmatch "test result: ok\. $($case.count) passed; 0 failed; 0 ignored;"){throw "Tracked GREEN missing: $log"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock scripts/check-semantic-work-fitness.ps1 verification/live-acceptance/issue-162/run-edit-end-focused.ps1 verification/live-acceptance/issue-162/run-edit-end-mutation.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='29 tracked exact-end tests and focused checks';files=$files} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "29 tracked tests and focused gates GREEN: $logs"
} finally {Pop-Location}
