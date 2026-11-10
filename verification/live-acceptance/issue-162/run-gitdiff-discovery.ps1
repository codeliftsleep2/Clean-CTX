# Agent MaxHeadRoom: replay cleanup regression plus next discovery, no optimization.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/gitdiff-discovery-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    $closure=Join-Path $repoPath 'target/issue-162/af-perf-001-20261109-161615/receipt.json'
    if(Test-Path $closure){$prior=Get-Content $closure -Raw -Encoding utf8|ConvertFrom-Json;foreach($file in $prior.files){if((Get-FileHash $file.path).Hash -ne $file.sha256){throw "Protected closure drift: $($file.path)"}}}
    $packet=@('src/mcp/tool_handlers/persistence/replay.rs','src/tests/mcp/replay_footer_observation_af_perf_002.rs','src/gitdiff/engine.rs','src/gitdiff/workspace.rs','src/gitdiff/runner.rs','src/tests/gitdiff/default_work_discovery_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @packet *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Format failed'}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-gitdiff-discovery.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $files=@($paths|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    $files|ConvertTo-Json -Depth 8|Set-Content (Join-Path $logs 'executed-source-identities.json') -Encoding utf8NoBOM
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    $total=0;$groups=@()
    foreach($spec in @(@{filter='af002_replay_footer_observes_dictionary_after_economics';minimum=1},@{filter='af002_gitdiff_';minimum=3},@{filter='diff_commits_exposes_loss_and_comparison_dimensions_in_public_metadata';minimum=1},@{filter='gitdiff::workspace::tests::';minimum=1},@{filter='gitdiff::runner::tests::';minimum=1},@{filter='gitdiff::engine::tests::';minimum=1})) {
        $name=$spec.filter -replace '[^A-Za-z0-9_.-]','_';$log=Join-Path $logs ($name+'.log')
        & cargo test --lib --all-features --quiet $spec.filter -- --test-threads=1 --nocapture *> $log
        $text=[IO.File]::ReadAllText($log);$summary=[regex]::Match($text,'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if($LASTEXITCODE -ne 0 -or -not $summary.Success -or [int]$summary.Groups[1].Value -lt $spec.minimum -or $text -match '(?m)^warning:'){throw "Tracked gate failure: $log"}
        $count=[int]$summary.Groups[1].Value;$total+=$count;$groups+=@{filter=$spec.filter;passes=$count}
    }
    $text=[IO.File]::ReadAllLines((Join-Path $logs 'af002_gitdiff_.log'))
    $rows=@($text|Where-Object{$_.StartsWith('AF002_GITDIFF_DISCOVERY ')}|ForEach-Object{$_.Substring('AF002_GITDIFF_DISCOVERY '.Length)|ConvertFrom-Json})
    $limits=@($text|Where-Object{$_.StartsWith('AF002_GITDIFF_LIMIT ')}|ForEach-Object{$_.Substring('AF002_GITDIFF_LIMIT '.Length)|ConvertFrom-Json})
    if($rows.Count -ne 30 -or $limits.Count -ne 3){throw 'Incomplete registered work/limit rows'}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log');if($LASTEXITCODE -ne 0){throw 'Format check failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log');if($LASTEXITCODE -ne 0){throw 'Size failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log');if($LASTEXITCODE -ne 0){throw 'Encoding failed'}
    foreach($file in $files){if((Get-FileHash $file.path).Hash -ne $file.sha256){throw "Executed source drift: $($file.path)"}}
    if(Test-Path $closure){foreach($file in $prior.files){if((Get-FileHash $file.path).Hash -ne $file.sha256){throw "Protected closure drift: $($file.path)"}}}
    @{agent='Agent MaxHeadRoom';trackedPasses=$total;groups=$groups;publicRows=$rows;limitRows=$limits;files=$files;productionOptimization=$false}|ConvertTo-Json -Depth 100|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "$total tracked passes across six groups; replay cleanup and30 Git work/3 limit rows. Evidence: $logs"
} finally {Pop-Location;[Console]::OutputEncoding=$oldEncoding}
