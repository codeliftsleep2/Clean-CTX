# Agent MaxHeadRoom: original shortcut clone RED, then unchanged semantic/work GREEN.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/diff-shortcut-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Diff shortcut $Phase evidence: $logs"
    $tests=@('src/tests/mcp/diff_shortcut_budget_af_perf_002.rs','src/tests/mcp/diff_work_af_perf_002.rs')
    $production='src/mcp/tool_helpers.rs'
    if($Phase -eq 'After'){
        $before=(Get-ChildItem target/af-perf-002 -Directory | Where-Object {$_.Name -match '^diff-shortcut-before-\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'receipt.json'))} | Sort-Object Name -Descending | Select-Object -First 1).FullName
        if(-not $before){throw 'Verified original RED required'}
        $r=Get-Content (Join-Path $before 'receipt.json') -Raw | ConvertFrom-Json
        if($r.phase -ne 'Before' -or $r.semanticExit -ne 0 -or $r.budgetExit -eq 0){throw 'Invalid original RED receipt'}
        foreach($file in $tests){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Budget/fixture drift: $file"}}
    }
    & rustfmt --edition 2024 --config skip_children=true @tests $production *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    if($Phase -eq 'After'){
        foreach($file in $tests){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Post-format budget drift: $file"}}
    }
    Copy-Item $production (Join-Path $logs 'production-source.rs')
    foreach($file in $tests){Copy-Item $file (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-diff-shortcut-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    & cargo test --lib --all-features --quiet af002_diff_shortcut_semantics -- --test-threads=1 --nocapture *> (Join-Path $logs 'semantics.log')
    $semanticExit=$LASTEXITCODE
    $semanticText=[IO.File]::ReadAllText((Join-Path $logs 'semantics.log'))
    if($semanticExit -ne 0 -or $semanticText -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Exact semantic GREEN missing: $logs"}
    & cargo test --lib --all-features --quiet af002_diff_shortcut_budget -- --test-threads=1 --nocapture *> (Join-Path $logs 'budget.log')
    $budgetExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'budget.log'))
    Get-Content (Join-Path $logs 'budget.log') -Tail 10
    if($Phase -eq 'Before'){
        if($budgetExit -eq 0 -or $observed -notmatch 'Diff shortcut redundant snapshot clone budget' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Genuine original budget RED missing: $logs"}
    } elseif($budgetExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Corrected budget GREEN missing: $logs"}
    $semanticRows=@(Get-Content (Join-Path $logs 'semantics.log') | Where-Object {$_.StartsWith('AF002_SHORTCUT_SEM ')})
    $budgetRows=@(Get-Content (Join-Path $logs 'budget.log') | Where-Object {$_.StartsWith('AF002_SHORTCUT_SEM ')})
    $workRows=@(Get-Content (Join-Path $logs 'budget.log') | Where-Object {$_.StartsWith('AF002_SHORTCUT_WORK ')})
    if($semanticRows.Count -ne 11 -or $budgetRows.Count -ne 11 -or $workRows.Count -ne 11){throw 'Incomplete semantic/work rows'}
    $frozen=$semanticRows -join "`n"
    if($frozen -cne ($budgetRows -join "`n")){throw 'Semantic/budget run identity mismatch'}
    [IO.File]::WriteAllText((Join-Path $logs 'semantic-rows.txt'),$frozen,[Text.UTF8Encoding]::new($false))
    if($Phase -eq 'After' -and $frozen -cne [IO.File]::ReadAllText((Join-Path $before 'semantic-rows.txt'))){throw 'STOP: frozen complete public/cache identity changed'}
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticExit=$semanticExit;budgetExit=$budgetExit;cases=11;files=$files} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Verified expected $Phase result: $logs"
} finally {Pop-Location}
