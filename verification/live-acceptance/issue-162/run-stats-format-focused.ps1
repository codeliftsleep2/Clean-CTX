# Agent MaxHeadRoom: operator-owned format-guard focused acceptance.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs = Join-Path $repoPath ('target/af-perf-002/stats-focused-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Discovery packet: $logs"
    $changed = @('src/mcp/tool_handlers/stats/mod.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/mcp/stats_work_af_perf_002.rs','src/tests/mcp/stats_format_budget_af_perf_002.rs','src/tests/mcp/stats_format_static_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size failed: $logs" }
    foreach ($selection in @(@{filter='af002_stats_';count=3},@{filter='mcp::tool_handlers::stats::read_only_tests::';count=5})) {
        $logName = ($selection.filter -replace '[^A-Za-z0-9_.-]', '_') + '.log'
        $log = Join-Path $logs $logName
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $log
        if ($LASTEXITCODE -ne 0) { Get-Content -LiteralPath $log -Tail 25; throw "Tracked packet failed: $log" }
        $summary = [regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -ne $selection.count) { throw "Missing or ignored tracked tests: $log" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $logs" }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Format check failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding failed: $logs" }
    $paths = @(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-stats-format-focused.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $files = @($paths | Sort-Object -Unique | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='Eight tracked stats format budget/static/response and read-only lifecycle tests';command='run-stats-format-focused.ps1';discoveryComplete=$false;files=$files;driverSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Packet complete: $logs. Enumeration is not classification or architectural acceptance."
} finally { Pop-Location }
