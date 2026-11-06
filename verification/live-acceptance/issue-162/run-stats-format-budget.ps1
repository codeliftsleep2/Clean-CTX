# Agent MaxHeadRoom: format-gated stats budget, original RED before production editing.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/stats-budget-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Stats $Phase logs: $logs"
    $changed=@('src/mcp/tool_handlers/stats/mod.rs','src/tests/mcp/stats_format_budget_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-stats-format-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath $changed[0] -Destination (Join-Path $logs 'production-source.rs')
    Copy-Item -LiteralPath $changed[1] -Destination (Join-Path $logs 'budget-source.rs')
    & cargo test --lib --all-features --quiet mcp::tool_handlers::stats::format_budget_af_perf_002::af002_stats_json_zero_unused_text_budget -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 12
    if($Phase -eq 'Before'){
        if($testExit -eq 0 -or $observed -notmatch 'unused stats text render budget' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Missing intended stats render RED: $logs"}
        Write-Output 'Verified intended unused-text RED; production remains unchanged pending inspection.'
    } elseif($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){
        throw "Stats budget not GREEN: $logs"
    }
    if($Phase -eq 'After'){
        $baseline=Join-Path $repoPath 'target/af-perf-002/stats-budget-before-20261105-231919/tests.log'
        $beforeRows=@(Get-Content -LiteralPath 'src/tests/mcp/fixtures/stats_format_responses.json' -Raw -Encoding utf8 | ConvertFrom-Json)
        $afterRows=@(Get-Content -LiteralPath (Join-Path $logs 'tests.log') | Where-Object { $_.StartsWith('AF002_STATS_RESPONSE ') } | ForEach-Object { $_.Substring(21) | ConvertFrom-Json })
        if($beforeRows.Count -ne 30 -or $afterRows.Count -ne 30){throw 'Missing complete response rows'}
        for($i=0;$i -lt 30;$i++){
            $old=$beforeRows[$i] | ConvertTo-Json -Depth 100 -Compress
            $new=$afterRows[$i] | ConvertTo-Json -Depth 100 -Compress
            if($old -cne $new){throw "Response identity mismatch at row $i : $logs"}
        }
        [pscustomobject]@{baseline=$baseline;exactNormalizedResponses=30;normalization='elapsed duration and unordered JSON file arrays only';encodingRecovery='expected source literals recovered exactly from HEAD; frozen raw evidence retained'} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $logs 'response-comparison.json') -Encoding utf8NoBOM
        Write-Output '30 complete normalized response rows match frozen Before evidence.'
    }
} finally {Pop-Location}
