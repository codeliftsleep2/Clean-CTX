# Agent MaxHeadRoom: operator-owned discovery and affected verification, no repair.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs = Join-Path $repoPath ('target/af-perf-002/proxy-scrub-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Discovery packet: $logs"
    $changed = @('proxy/src/lib.rs','proxy/src/scrub.rs','src/tests/proxy/scrub_equal_length_af_perf_002.rs')
    & rustfmt --edition 2021 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size failed: $logs" }
    foreach ($selection in @(@{filter='af002_proxy_';count=6},@{filter='scrub::tests::';count=24},@{filter='pipeline::tests::';count=5},@{filter='transform::tests::';count=17})) {
        $logName = ($selection.filter -replace '[^A-Za-z0-9_.-]', '_') + '.log'
        $log = Join-Path $logs $logName
        & cargo test -p clean-ctx-proxy --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $log
        if ($LASTEXITCODE -ne 0) { Get-Content -LiteralPath $log -Tail 25; throw "Tracked packet failed: $log" }
        $summary = [regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -ne $selection.count) { throw "Missing or ignored tracked tests: $log" }
    }
    & cargo clippy -p clean-ctx-proxy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $logs" }
    & cargo fmt -p clean-ctx-proxy -- --check *> (Join-Path $logs 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Format check failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding failed: $logs" }
    $paths = @(git ls-files --cached --others --exclude-standard -- proxy src/tests/proxy Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-proxy-scrub-focused.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $files = @($paths | Sort-Object -Unique | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='proxy scrub correctness regressions and existing scrub/pipeline/transform controls; focused lint';command='run-proxy-scrub-focused.ps1';discoveryComplete=$false;files=$files;driverSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Packet complete: $logs. Enumeration is not classification or architectural acceptance."
} finally { Pop-Location }
