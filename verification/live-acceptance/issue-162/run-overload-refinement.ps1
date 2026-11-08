# Agent MaxHeadRoom: overload signature refinement comparison focused semantic/work/static/cost/lifecycle acceptance.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/overload-refinement-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Overload signature refinement comparison focused: $logs"
    & rustfmt --edition 2024 --config skip_children=true src/diff/differ.rs src/tests/architecture/work_observations_af_perf_002.rs src/tests/diff/overload_alternatives_af_perf_002.rs src/tests/diff/overload_alternative_lookup_af_perf_002.rs src/tests/diff/overload_alternative_comparator_af_perf_002.rs src/tests/diff/overload_observed_original_af_perf_002.rs src/tests/diff/overload_refinement_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='af002_overload_refinement_';count=3}
    )){
        $log=Join-Path $logs (($case.filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE;Get-Content -LiteralPath $log -Tail 8
        if($code -ne 0 -or [IO.File]::ReadAllText($log) -notmatch "test result: ok\. $($case.count) passed; 0 failed; 0 ignored;"){throw "Tracked GREEN missing: $log"}
    }
    $content=[IO.File]::ReadAllText((Join-Path $logs 'af002_overload_refinement_.log'))
    if([regex]::Matches($content,'(?m)^AF002_REFINE_COST ').Count -ne 14 -or [regex]::Matches($content,'(?m)^AF002_REFINE_LAYOUT ').Count -ne 1){throw 'Incomplete alternative work/cost rows'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock scripts/check-semantic-work-fitness.ps1 verification/live-acceptance/issue-162/run-overload-refinement.ps1 docs/agent/AF_PERF_002_OVERLOAD_REFINEMENT.md)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='3 tracked refinement-investigation overload signature refinement comparison tests and focused checks';files=$files} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "3 tracked refinement-investigation tests and focused gates GREEN: $logs"
} finally {Pop-Location}
