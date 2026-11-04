# Agent MaxHeadRoom: intended RED before approved #173 original-position prepass.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/ngrx173-budget-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "#173 $Phase logs: $logs"
    $changed=@('src/workspace/index/ngrx_refinement.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/workspace/index_ngrx_noop_af_perf_002.rs','src/tests/workspace/index_ngrx_binding_budget_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-ngrx173-binding-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath 'src/tests/workspace/index_ngrx_binding_budget_af_perf_002.rs' -Destination (Join-Path $logs 'budget-source.rs')
    Copy-Item -LiteralPath 'src/workspace/index/ngrx_refinement.rs' -Destination (Join-Path $logs 'production-source.rs')
    & cargo test --lib --all-features --quiet af173_binding_budget_ -- --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked complete-operation budget RED/GREEN';testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 18
    if($Phase -eq 'Before'){
        if($testExit -eq 0 -or $observed -notmatch 'AF173 unused binding collection' -or $observed -notmatch 'test result: FAILED\. 1 passed; 1 failed; 0 ignored;') {throw "Missing intended no-family budget RED: $logs"}
        Write-Output 'Observed intended no-family RED and demanding/fresh mixed control GREEN. Production remains eager until reviewed.'
    } elseif($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 2 passed; 0 failed; 0 ignored;') {
        throw "Post-repair budget gate not GREEN: $logs"
    }
} finally {Pop-Location}
