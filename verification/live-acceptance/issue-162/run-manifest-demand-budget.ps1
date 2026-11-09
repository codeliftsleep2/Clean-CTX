# Agent MaxHeadRoom: original-only semantic baseline before any release correction.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
$oldConsoleEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldRoot=$env:AF002_MANIFEST_FIXTURE_ROOT;$oldOracle=$env:AF002_MANIFEST_ORACLE
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/manifest-demand-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $frozen=@('src/tests/mcp/proof_manifest_repair_af_perf_002.rs','src/tests/mcp/manifest_baseline_lifecycle_af_perf_002.rs')
    $production=@('src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/acquisition.rs','src/mcp/tool_handlers/hydration/execution.rs','src/mcp/tool_handlers/query/entities.rs','src/mcp/tool_handlers/query/edges.rs','src/mcp/tool_handlers/hydration/test_support.rs')
    $protected=@('src/mcp/state.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $protectedHashes=@{};foreach($p in $protected){$protectedHashes[$p]=(Get-FileHash $p).Hash}
    if($Phase -eq 'After'){$production+= 'src/mcp/tool_handlers/hydration/filesystem_discovery.rs'}
    $tests=$frozen+@('src/tests/mcp/proof_manifest_design_af_perf_002.rs','src/tests/mcp/root_inventory_discovery_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @tests @production *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    if($Phase -eq 'Before') {
        $env:AF002_MANIFEST_FIXTURE_ROOT=Join-Path $logs 'fixtures'
        Remove-Item Env:AF002_MANIFEST_ORACLE -ErrorAction SilentlyContinue
        foreach($p in ($frozen+$production+$tests|Sort-Object -Unique)) {
            $target=Join-Path (Join-Path $logs 'frozen') $p
            New-Item -ItemType Directory -Force -Path (Split-Path $target)|Out-Null
            Copy-Item -LiteralPath $p -Destination $target
        }
    } else {
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'manifest-demand-before-*'|Where-Object{Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original RED required'}
        $prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        if($prior.semanticPasses -ne 2 -or $prior.publicCases -ne 70 -or $prior.budgetExit -eq 0){throw 'Invalid original RED'}
        foreach($p in $frozen){if((Get-FileHash $p).Hash -ne (Get-FileHash (Join-Path (Join-Path $before 'frozen') $p)).Hash){throw "Frozen oracle/budget drift: $p"}}
        $env:AF002_MANIFEST_FIXTURE_ROOT=Join-Path $before 'fixtures'
        $env:AF002_MANIFEST_ORACLE=Join-Path $before 'public-oracle.json'
    }
    # Restored source copies can retain mtimes older than Cargo's last build.
    # Force dependency freshness without altering any frozen source bytes.
    foreach($p in ($production+$tests|Sort-Object -Unique)) {
        [IO.File]::SetLastWriteTimeUtc((Join-Path $repoPath $p),[DateTime]::UtcNow)
    }
    foreach($filter in @('af002_root_inventory_contracts','af002_manifest_frozen_public_semantics')) {
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Semantic prerequisite failed; not optimization RED: $log"}
        Write-Output "$filter GREEN"
    }
    $prefix='AF002_MANIFEST_PUBLIC '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_manifest_frozen_public_semantics.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 70){throw 'Incomplete ordinary/proof/provider/invalidation/durable baseline'}
    if($Phase -eq 'Before') {
        $rows|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'public-oracle.json') -Encoding utf8NoBOM
        $env:AF002_MANIFEST_ORACLE=Join-Path $logs 'public-oracle.json'
    }
    $budgetLog=Join-Path $logs 'budget.log'
    & cargo test --lib --all-features --quiet af002_manifest_unused_budget -- --test-threads=1 --nocapture *> $budgetLog
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($budgetLog)
    if(@([IO.File]::ReadAllLines($budgetLog)|Where-Object{$_.StartsWith($prefix)}).Count -ne 70){throw 'Incomplete budget identities; invalid RED'}
    if($Phase -eq 'Before') {
        if($code -eq 0 -or $text -notmatch 'Explicit-demand unused per-root manifest budget' -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw 'Intended unused-work RED missing'}
    } elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Current-byte budget GREEN missing'}
    if($text -match '(?m)^warning:'){throw 'Warnings invalidate acceptance'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $protectedHashes[$p]){throw "Protected drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-manifest-demand-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticPasses=2;publicCases=70;budgetExit=$code;files=$files;protectedIdentities=$protectedHashes}|
        ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Manifest demand $Phase complete. Evidence: $logs"
} finally {Pop-Location;$env:AF002_MANIFEST_FIXTURE_ROOT=$oldRoot;$env:AF002_MANIFEST_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldConsoleEncoding}
