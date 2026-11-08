# Agent MaxHeadRoom: original matcher RED before any retrieval rewrite.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/overload-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    Write-Output "Overload $Phase evidence: $logs"
    $tests=@('src/tests/diff/original_matcher_af_perf_002.rs','src/tests/diff/overload_equivalence_af_perf_002.rs','src/tests/mcp/overload_budget_af_perf_002.rs','src/tests/mcp/diff_work_af_perf_002.rs')
    if($Phase -eq 'After'){
        $before=(Get-ChildItem target/af-perf-002 -Directory|Where-Object {$_.Name -match '^overload-before-\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'receipt.json'))}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original RED required'}
        $r=Get-Content (Join-Path $before 'receipt.json') -Raw|ConvertFrom-Json
        if($r.semanticExit -ne 0 -or $r.budgetExit -eq 0){throw 'Invalid original RED receipt'}
        foreach($file in $tests){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Frozen test drift: $file"}}
    }
    $production=@('src/diff/differ.rs')
    if($Phase -eq 'After'){$production+=@(Get-ChildItem src/diff -Recurse -Filter '*overload*.rs'|ForEach-Object {$_.FullName})}
    & rustfmt --edition 2024 --config skip_children=true @tests @production src/tests/architecture/work_observations_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Formatting failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    if($Phase -eq 'After'){foreach($file in $tests){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw 'Post-format frozen drift'}}}
    foreach($file in $tests){Copy-Item $file (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    Copy-Item src/diff/differ.rs (Join-Path $logs 'production-source.rs')
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-overload-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object {@{path=$_;sha256=(Get-FileHash $_).Hash}})
    & cargo test --lib --all-features --quiet af002_overload_frozen_equivalence -- --test-threads=1 --nocapture *> (Join-Path $logs 'semantics.log')
    $semanticExit=$LASTEXITCODE
    if($semanticExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'semantics.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Frozen semantic GREEN missing: $logs"}
    & cargo test --lib --all-features --quiet af002_overload_public_budget -- --test-threads=1 --nocapture *> (Join-Path $logs 'budget.log')
    $budgetExit=$LASTEXITCODE;$observed=[IO.File]::ReadAllText((Join-Path $logs 'budget.log'))
    Get-Content (Join-Path $logs 'budget.log') -Tail 10
    if($Phase -eq 'Before'){
        if($budgetExit -eq 0 -or $observed -notmatch 'Overload redundant candidate search budget' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended original RED missing: $logs"}
    } elseif($budgetExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Corrected budget GREEN missing: $logs"}
    $rows=@(Get-Content (Join-Path $logs 'budget.log')|Where-Object {$_.StartsWith('AF002_OVERLOAD_SEM ')})
    $work=@(Get-Content (Join-Path $logs 'budget.log')|Where-Object {$_.StartsWith('AF002_OVERLOAD_WORK ')})
    if($rows.Count -ne 45 -or $work.Count -ne 45){throw 'Incomplete 45-case public semantic/work evidence'}
    $frozen=$rows -join "`n"
    [IO.File]::WriteAllText((Join-Path $logs 'semantic-rows.txt'),$frozen,[Text.UTF8Encoding]::new($false))
    if($Phase -eq 'After' -and $frozen -cne [IO.File]::ReadAllText((Join-Path $before 'semantic-rows.txt'))){throw 'STOP: selected pairs/actions/public/cache identity changed'}
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticExit=$semanticExit;budgetExit=$budgetExit;publicCases=45;files=$files}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Expected $Phase result verified: $logs"
} finally {Pop-Location}
