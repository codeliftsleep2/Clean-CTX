# Agent MaxHeadRoom: intended RED before approved hierarchy placement lookup reuse.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/hierarchy-budget-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Hierarchy $Phase logs: $logs"
    $changed=@('src/ir/hierarchical/encode.rs','src/ir/hierarchical.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/ir/hierarchy_placement_budget_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-hierarchy-placement-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath 'src/tests/ir/hierarchy_placement_budget_af_perf_002.rs' -Destination (Join-Path $logs 'budget-source.rs')
    Copy-Item -LiteralPath 'src/ir/hierarchical/encode.rs' -Destination (Join-Path $logs 'production-source.rs')
    & cargo test --lib --all-features --quiet ir::hierarchical::placement_repair::hierarchy_placement_forward_and_refusal_equivalence -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'equivalence.log')
    $equivalenceExit=$LASTEXITCODE
    if($equivalenceExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'equivalence.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Ownership equivalence control failed or missing: $logs"}
    & cargo test --lib --all-features --quiet ir::hierarchical::placement_repair::hierarchy_placement_bounded_work -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked complete-operation budget RED/GREEN';testExit=$testExit;equivalenceExit=$equivalenceExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 18
    if($Phase -eq 'Before'){
        if($testExit -eq 0 -or $observed -notmatch 'Hierarchy placement work exceeds one lookup per member' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;') {throw "Missing intended placement RED: $logs"}
        Write-Output 'Observed intended placement RED and ownership equivalence GREEN. Production placement remains unchanged.'
    } elseif($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;') {
        throw "Post-repair budget gate not GREEN: $logs"
    }
} finally {Pop-Location}
