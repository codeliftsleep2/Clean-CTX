# Agent MaxHeadRoom: narrow paired-cost rerun after matching original observer sites.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/edit-end-cost-review-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Exact-end paired cost review: $logs"
    & rustfmt --edition 2024 --config skip_children=true src/tests/edit/exact_end_acceptance_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    & cargo test --lib --all-features --quiet afperf_edit_end_construction_cost -- --test-threads=1 --nocapture *> (Join-Path $logs 'cost.log')
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText((Join-Path $logs 'cost.log'))
    Get-Content (Join-Path $logs 'cost.log') -Tail 12
    if($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Paired cost/semantic GREEN missing: $logs"}
    $rows=@(Get-Content (Join-Path $logs 'cost.log') | Where-Object {$_.StartsWith('AF002_EDIT_END_COST ')})
    if($rows.Count -ne 8){throw 'Incomplete cost dimensions'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock scripts/check-semantic-work-fitness.ps1 verification/live-acceptance/issue-162/run-edit-end-cost-review.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='One tracked paired-cost semantic test and focused checks';diagnostics='original predicate observer sites now match frozen production; all updates disabled during timing';files=$files} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Cost comparison ready for review, not automatic performance approval: $logs"
} finally {Pop-Location}
