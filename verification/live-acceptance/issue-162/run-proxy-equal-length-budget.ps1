# Agent MaxHeadRoom: regression evidence only; no live server or external traffic.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/proxy-equal-length-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Regression evidence: $logs"
    & rustfmt --edition 2021 --config skip_children=true proxy/src/lib.rs src/tests/proxy/scrub_equal_length_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- proxy src/tests/proxy Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-proxy-equal-length-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath proxy/src/scrub.rs -Destination (Join-Path $logs 'production-source.rs')
    Copy-Item -LiteralPath src/tests/proxy/scrub_equal_length_af_perf_002.rs -Destination (Join-Path $logs 'test-source.rs')
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_proxy_equal_length_ -- --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked synthetic scrub correctness regression';testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 30
    if($Phase -eq 'Before') {
        if($testExit -eq 0 -or $observed -notmatch 'equal-length direct redaction must survive' -or $observed -notmatch 'equal-length pipeline redaction must survive' -or $observed -notmatch 'test result: FAILED\. 0 passed; 2 failed; 0 ignored;'){throw "Missing intended correctness RED: $logs"}
    } elseif($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 2 passed; 0 failed; 0 ignored;') { throw "Repaired regression not GREEN: $logs" }
} finally {Pop-Location}
