# Agent MaxHeadRoom: original public work RED before exact-end production edits.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/edit-end-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Edit exact-end $Phase evidence: $logs"
    $tests=@('src/tests/edit/exact_end_budget_af_perf_002.rs','src/tests/edit/original_end_join_af_perf_002.rs','src/tests/mcp/edit_relocation_work_af_perf_002.rs')
    if($Phase -eq 'After'){
        $before=(Get-ChildItem target/af-perf-002 -Directory | Where-Object {$_.Name -match '^edit-end-before-\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'receipt.json'))} | Sort-Object Name -Descending | Select-Object -First 1).FullName
        if(-not $before){throw 'Verified original RED evidence required'}
        $r=Get-Content (Join-Path $before 'receipt.json') -Raw | ConvertFrom-Json
        if($r.phase -ne 'Before' -or $r.semanticExit -ne 0 -or $r.budgetExit -eq 0){throw 'Invalid original RED receipt'}
        foreach($file in $tests){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Budget/oracle drift: $file"}}
    }
    $production=@('src/edit/locate.rs')
    if($Phase -eq 'After'){$production+='src/edit/locate/end_join.rs'}
    & rustfmt --edition 2024 --config skip_children=true @tests @production src/tests/architecture/work_observations_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-edit-end-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    Copy-Item src/edit/locate.rs (Join-Path $logs 'production-source.rs')
    if($Phase -eq 'After'){Copy-Item src/edit/locate/end_join.rs (Join-Path $logs 'end-join-source.rs')}
    foreach($file in $tests){Copy-Item $file (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    & cargo test --lib --all-features --quiet af002_edit_end_frozen_equivalence -- --test-threads=1 --nocapture *> (Join-Path $logs 'semantics.log')
    $semanticExit=$LASTEXITCODE
    if($semanticExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'semantics.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Frozen semantic GREEN missing: $logs"}
    & cargo test --lib --all-features --quiet af002_edit_end_public_budget -- --test-threads=1 --nocapture *> (Join-Path $logs 'budget.log')
    $budgetExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'budget.log'))
    Get-Content (Join-Path $logs 'budget.log') -Tail 10
    if($Phase -eq 'Before'){
        if($budgetExit -eq 0 -or $observed -notmatch 'Edit exact-end redundant full-input join budget' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Genuine original budget RED missing: $logs"}
    } elseif($budgetExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Repaired budget GREEN missing: $logs"}
    $rows=@(Get-Content (Join-Path $logs 'budget.log') | Where-Object {$_.StartsWith('AF002_EDIT ')})
    if($rows.Count -ne 12){throw 'Incomplete public semantic/work rows'}
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticExit=$semanticExit;budgetExit=$budgetExit;publicRows=12;files=$files} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Verified expected $Phase result: $logs"
} finally {Pop-Location}
