# Agent MaxHeadRoom: genuine original materialization RED, then unchanged GREEN.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_SQLITE_TEXT_ORACLE
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/sqlite-text-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $frozen=@('src/tests/mcp/sqlite_text_repair_af_perf_002.rs','src/tests/mcp/sqlite_text_original_af_perf_002.rs')
    $protected=@('src/ir/binary_wire.rs','src/ir/string_table.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $protectedHashes=@{}
    foreach($p in $protected){$protectedHashes[$p]=(Get-FileHash $p).Hash}
    & rustfmt --edition 2024 --config skip_children=true @frozen src/tests/mcp/sqlite_work_af_perf_002.rs src/tests/mcp/sqlite_text_design_af_perf_002.rs src/tests/architecture/default_operation_observations_af_perf_002.rs src/mcp/sqlite_store.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    if($Phase -eq 'Before') {
        Remove-Item Env:AF002_SQLITE_TEXT_ORACLE -ErrorAction SilentlyContinue
        Copy-Item src/mcp/sqlite_store.rs (Join-Path $logs 'production-sqlite.rs')
        foreach($p in $frozen){Copy-Item $p (Join-Path $logs ([IO.Path]::GetFileName($p)))}
    } else {
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'sqlite-text-before-*'|Where-Object{Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original RED required'}
        $prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        if($prior.semanticPasses -ne 2 -or $prior.budgetExit -eq 0 -or $prior.publicCases -ne 12){throw 'Invalid original RED'}
        foreach($p in $frozen){if((Get-FileHash $p).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($p)))).Hash){throw "Frozen drift: $p"}}
        $env:AF002_SQLITE_TEXT_ORACLE=Join-Path $before 'public-oracle.json'
    }
    foreach($filter in @('af002_sqlite_text_frozen_semantics','af002_sqlite_text_public_identity')) {
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Semantic prerequisite failed; not optimization RED: $log"}
        Write-Output "$filter GREEN"
    }
    $prefix='AF002_SQLITE_TEXT_PUBLIC '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_sqlite_text_public_identity.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 12){throw 'Incomplete public semantic rows'}
    if($Phase -eq 'Before') {
        $rows|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'public-oracle.json') -Encoding utf8NoBOM
        $env:AF002_SQLITE_TEXT_ORACLE=Join-Path $logs 'public-oracle.json'
    }
    $budgetLog=Join-Path $logs 'budget.log'
    & cargo test --lib --all-features --quiet af002_sqlite_text_public_budget -- --test-threads=1 --nocapture *> $budgetLog
    $budgetExit=$LASTEXITCODE
    $text=[IO.File]::ReadAllText($budgetLog)
    $publicCount=@([IO.File]::ReadAllLines($budgetLog)|Where-Object{$_.StartsWith($prefix)}).Count
    if($publicCount -ne 12){throw 'Incomplete budget semantic rows; not intended RED'}
    if($Phase -eq 'Before') {
        if($budgetExit -eq 0 -or $text -notmatch 'SQLite discarded owned-text materialization budget' -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw 'Intended budget RED missing'}
    } elseif($budgetExit -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Budget GREEN missing'}
    if($text -match '(?m)^warning:'){throw 'Warnings invalidate acceptance'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $protectedHashes[$p]){throw "Protected drift: $p"}}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-sqlite-text-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticPasses=2;publicCases=12;budgetExit=$budgetExit;files=$files;protectedIdentities=$protectedHashes}|
        ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "SQLite text $Phase complete. Full evidence: $logs"
} finally {Pop-Location;$env:AF002_SQLITE_TEXT_ORACLE=$oldOracle}
