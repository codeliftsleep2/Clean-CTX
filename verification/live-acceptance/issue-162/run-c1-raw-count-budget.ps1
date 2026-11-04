# Agent MaxHeadRoom: intended RED before approved C1 raw count reuse.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/c1-budget-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "C1 $Phase logs: $logs"
    $changed=@('src/mcp/tool_handlers/core/compress.rs','src/tests/mcp/compression_token_work_af_perf_002.rs','src/tests/mcp/content_economics.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-c1-raw-count-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath 'src/tests/mcp/compression_token_work_af_perf_002.rs' -Destination (Join-Path $logs 'budget-source.rs')
    Copy-Item -LiteralPath 'src/mcp/tool_handlers/core/compress.rs' -Destination (Join-Path $logs 'production-source.rs')
    & cargo test --lib --all-features --quiet mcp::content_economics::tests::c1_equivalence_counted_and_recounted_paths -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'equivalence.log')
    $equivalenceExit=$LASTEXITCODE
    if($equivalenceExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'equivalence.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Counting equivalence control failed or missing: $logs"}
    & cargo test --lib --all-features --quiet artifact_operation_work_af_perf_002::compression_token_work::c1_budget_public_raw_count_once -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked complete-operation budget RED/GREEN';testExit=$testExit;equivalenceExit=$equivalenceExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 18
    if($Phase -eq 'Before'){
        if($testExit -eq 0 -or $observed -notmatch 'C1 redundant raw recount' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;') {throw "Missing intended raw recount RED: $logs"}
        Write-Output 'Observed intended public raw recount RED and counting equivalence GREEN. Handler remains unchanged until review.'
    } elseif($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;') {
        throw "Post-repair budget gate not GREEN: $logs"
    }
} finally {Pop-Location}
