# Agent MaxHeadRoom: delayed-capture design only, no release optimization.
param([string]$ResumeCostDirectory='')
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/delta-previous-design-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $protected=@('src/mcp/tool_helpers.rs','src/mcp/state/source_cache.rs','src/cache.rs','src/diff/differ.rs','src/diff/overload_lookup.rs','src/mcp/tool_handlers/hydration.rs','src/mcp/tool_handlers/hydration/filesystem.rs','src/mcp/tool_handlers/hydration/filesystem_discovery.rs','src/ir/binary_wire.rs','src/ir/string_table.rs','src/mcp/sqlite_store.rs','src/mcp/sqlite_store/metadata_text.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $before=@{};foreach($p in $protected){$before[$p]=(Get-FileHash $p).Hash}
    $packet=@('src/mcp/tool_handlers/core/delta.rs','src/tests/mcp/rolling_read_discovery_af_perf_002.rs','src/tests/mcp/delta_previous_design_af_perf_002.rs','src/tests/mcp/delta_previous_cases_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    $executed=@{};foreach($p in $packet){$executed[$p]=(Get-FileHash $p).Hash}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $costProof=$null
    $groups=@(@{filter='af002_delta_previous_';count=5})
    if($ResumeCostDirectory){
        $prior=(Resolve-Path -LiteralPath $ResumeCostDirectory).Path
        $allowed=[IO.Path]::GetFullPath((Join-Path $repoPath 'target/af-perf-002'))+[IO.Path]::DirectorySeparatorChar
        if(-not $prior.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path $prior -Leaf) -notmatch '^delta-previous-design-\d{8}-\d{6}$'){throw 'Invalid cost evidence directory'}
        $costProof=Get-Content (Join-Path $prior 'cost-inspection.json') -Raw -Encoding utf8|ConvertFrom-Json
        if(-not $costProof.costTestPassed -or $costProof.costRows.Count -ne 16){throw 'Completed fixed cost packet missing'}
        foreach($file in $costProof.files){
            $saved=Join-Path (Join-Path $prior 'inspected-sources') $file.path
            if((Get-FileHash $saved).Hash -ne $file.sha256){throw 'Inspected cost source drift'}
            $old=[IO.File]::ReadAllText($saved);$current=[IO.File]::ReadAllText((Join-Path $repoPath $file.path))
            if($file.path.EndsWith('delta_previous_design_af_perf_002.rs')){
                $old=$old.Substring(0,$old.IndexOf('fn identity('));$current=$current.Substring(0,$current.IndexOf('fn identity('))
            } elseif($file.path.EndsWith('delta_previous_cases_af_perf_002.rs')){
                $old=$old.Substring($old.IndexOf('fn local('));$current=$current.Substring($current.IndexOf('fn local('))
            }
            if($old -cne $current){throw "Timed code changed; cannot reuse costs: $($file.path)"}
        }
        $groups=@('af002_delta_previous_lock_boundary','af002_delta_previous_public','af002_delta_previous_errors_and_interleavings','af002_delta_previous_persistent')|ForEach-Object{@{filter=$_;count=1}}
    }
    $groups+=@(@{filter='explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore';count=1},@{filter='matching_canonical_epoch_accepts_delta_and_advances_chain';count=1},@{filter='delta_derivation_rejects_a_superseded_baseline_epoch';count=1},@{filter='afperf_diff_source_static';count=1})
    foreach($group in $groups){
        $log=Join-Path $logs ($group.filter+'.log')
        & cargo test --lib --all-features --quiet $group.filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log)
        if($LASTEXITCODE -ne 0 -or $text -notmatch "test result: ok\. $($group.count) passed; 0 failed; 0 ignored;" -or $text -match '(?m)^warning:'){throw "Tracked design failure: $log"}
    }
    $rows=@{};$lines=@($groups|ForEach-Object{[IO.File]::ReadAllLines((Join-Path $logs ($_.filter+'.log')))})
    foreach($kind in @('PUBLIC','PERSISTENT','INTERLEAVING','COST')){
        $prefix='AF002_DELTA_PREVIOUS_'+$kind+' '
        $rows[$kind]=@($lines|Where-Object{$_.StartsWith($prefix)}|ForEach-Object{$_.Substring($prefix.Length)|ConvertFrom-Json})
    }
    if($costProof){$rows.COST=@($costProof.costRows)}
    if($rows.PUBLIC.Count -ne 240 -or $rows.PERSISTENT.Count -ne 11 -or $rows.INTERLEAVING.Count -ne 2 -or $rows.COST.Count -ne 16){throw 'Incomplete paired design populations'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($p in $protected){if((Get-FileHash $p).Hash -ne $before[$p]){throw "Protected drift: $p"}}
    foreach($p in $packet){if((Get-FileHash $p).Hash -ne $executed[$p]){throw "Executed source drift: $p"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-delta-previous-design.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';trackedPasses=9;newTrackedPasses=$(if($costProof){8}else{9});reusedCostPasses=$(if($costProof){1}else{0});costSource=$ResumeCostDirectory;rows=$rows;files=$files;protectedIdentities=$before;instrumentedIdentities=$executed;productionOptimizationImplemented=$false;composedExecuted=$false}|
        ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Nine tracked passes;240 paired public rows;16 cost rows. Evidence: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
