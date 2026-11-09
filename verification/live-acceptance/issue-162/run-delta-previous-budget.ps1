# Agent MaxHeadRoom: genuine original handler-site RED before any delayed-capture correction.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$oldRoot=$env:AF002_DELTA_PREVIOUS_ROOT;$oldOracle=$env:AF002_DELTA_PREVIOUS_ORACLE
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/delta-previous-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $frozen=@('src/tests/mcp/delta_previous_repair_af_perf_002.rs','src/tests/mcp/delta_previous_repair_cases_af_perf_002.rs')
    $packet=$frozen+@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/delta_previous_design_af_perf_002.rs','src/tests/mcp/delta_previous_cases_af_perf_002.rs','src/tests/mcp/rolling_read_discovery_af_perf_002.rs')
    $protected=@('src/mcp/tool_helpers.rs','src/mcp/state/source_cache.rs','src/mcp/state/publication.rs','src/cache.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $protectedHashes=@{};foreach($p in $protected){$protectedHashes[$p]=(Get-FileHash $p).Hash}
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    if($Phase -eq 'Before'){
        $env:AF002_DELTA_PREVIOUS_ROOT=Join-Path $logs 'fixtures'
        Remove-Item Env:AF002_DELTA_PREVIOUS_ORACLE -ErrorAction SilentlyContinue
        foreach($p in ($packet+$protected|Sort-Object -Unique)){
            $dst=Join-Path (Join-Path $logs 'frozen') $p
            New-Item -ItemType Directory -Force -Path (Split-Path $dst)|Out-Null
            Copy-Item -LiteralPath $p -Destination $dst
        }
    } else {
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'delta-previous-before-*'|Where-Object{Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original RED missing'}
        $prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        if($prior.semanticPasses -ne 5 -or $prior.publicCases -ne 380 -or $prior.budgetExit -eq 0){throw 'Invalid original RED'}
        foreach($p in $frozen){if((Get-FileHash $p).Hash -ne (Get-FileHash (Join-Path (Join-Path $before 'frozen') $p)).Hash){throw "Frozen budget drift: $p"}}
        foreach($p in $protected){if((Get-FileHash $p).Hash -ne $prior.protectedIdentities.$p){throw "Protected original identity drift: $p"}}
        $env:AF002_DELTA_PREVIOUS_ROOT=Join-Path $before 'fixtures'
        $env:AF002_DELTA_PREVIOUS_ORACLE=Join-Path $before 'public-oracle.json'
    }
    foreach($p in $packet){[IO.File]::SetLastWriteTimeUtc((Join-Path $repoPath $p),[DateTime]::UtcNow)}
    $executed=@{};foreach($p in ($packet+$protected|Sort-Object -Unique)){$executed[$p]=(Get-FileHash $p).Hash}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-delta-previous-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Original source inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    $files|ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'executed-source-identities.json') -Encoding utf8NoBOM
    if($Phase -eq 'Before'){
        $original=[IO.File]::ReadAllText((Join-Path $repoPath 'src/mcp/tool_handlers/core/delta.rs'))
        if(-not $original.Contains('let capture_before = true;') -or $original.IndexOf('let previous_authority =') -gt $original.IndexOf('if ir_ctx.is_source_unchanged')){throw 'Original early capture missing'}
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw 'Warning-fatal Clippy failed; invalid RED'}
    foreach($filter in @('af002_delta_previous_frozen_semantics','explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore','matching_canonical_epoch_accepts_delta_and_advances_chain','delta_derivation_rejects_a_superseded_baseline_epoch','afperf_diff_source_static')) {
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -or $text -match '(?m)^warning:'){throw "Semantic prerequisite failed; invalid RED: $log"}
    }
    $prefix='AF002_DELTA_PREVIOUS_FROZEN '
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_delta_previous_frozen_semantics.log'))|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    if($rows.Count -ne 380){throw 'Incomplete frozen public/error/lifetime identities'}
    if($Phase -eq 'Before'){
        $rows|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'public-oracle.json') -Encoding utf8NoBOM
        $env:AF002_DELTA_PREVIOUS_ORACLE=Join-Path $logs 'public-oracle.json'
    }
    $log=Join-Path $logs 'budget.log'
    & cargo test --lib --all-features --quiet af002_delta_previous_unused_budget -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith($prefix)}).Count -ne 380){throw 'Incomplete budget identities; invalid RED'}
    if($Phase -eq 'Before'){
        if($code -ne 101 -or $text -notmatch 'Registered delta unused previous-authority budget' -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw 'Intended original unused-clone RED missing'}
    } elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Current-byte unused-clone budget GREEN missing'}
    if($text -match '(?m)^warning:'){throw 'Warnings invalidate acceptance'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $protectedHashes[$p]){throw "Protected source drift: $p"}}
    foreach($p in $executed.Keys){if((Get-FileHash $p).Hash -ne $executed[$p]){throw "Executed source drift: $p"}}
    & cargo fmt --all -- --check *> (Join-Path $logs 'format-check.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting check failed'}
    foreach($file in $files){if((Get-FileHash $file.path).Hash -ne $file.sha256){throw "Source inventory drift: $($file.path)"}}
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticPasses=5;publicCases=380;budgetExit=$code;files=$files;protectedIdentities=$protectedHashes;executedIdentities=$executed}|ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Delta previous $Phase complete. Evidence: $logs"
} finally {Pop-Location;$env:AF002_DELTA_PREVIOUS_ROOT=$oldRoot;$env:AF002_DELTA_PREVIOUS_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding}
