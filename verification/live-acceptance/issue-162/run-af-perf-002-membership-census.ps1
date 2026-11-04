# Agent MaxHeadRoom: operator-owned discovery and affected verification, no repair.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs = Join-Path $repoPath ('target/af-perf-002/membership-census-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Discovery packet: $logs"
    $changed = @('src/lib.rs','src/tests/architecture/membership_census_af_perf_002.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/workspace/index/declarations.rs','src/tests/workspace/index_enclosing_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size failed: $logs" }
    foreach ($selection in @(@{filter='af002_membership_source_census';count=1},@{filter='af002_enclosing_';count=2},@{filter='af174_';count=9})) {
        $log = Join-Path $logs ($selection.filter + '.log')
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $log
        if ($LASTEXITCODE -ne 0) { Get-Content -LiteralPath $log -Tail 25; throw "Tracked packet failed: $log" }
        $summary = [regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -ne $selection.count) { throw "Missing or ignored tracked tests: $log" }
    }
    $census = Join-Path $repoPath 'target/af-perf-002/source-wide-membership-census'
    Copy-Item -LiteralPath $census -Destination (Join-Path $logs 'census') -Recurse
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $logs" }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Format check failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding failed: $logs" }
    $files = Get-Content -LiteralPath (Join-Path $logs 'census/files.json') -Raw | ConvertFrom-Json
    foreach ($file in $files) {
        if ((Get-FileHash -LiteralPath $file.path).Hash -ne $file.sha256) { throw "Source changed during packet: $($file.path)" }
    }
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='12 tracked discovery/preservation tests plus focused lint';command='run-af-perf-002-membership-census.ps1';discoveryComplete=$false;files=$files;driverSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Packet complete: $logs. Enumeration is not classification or architectural acceptance."
} finally { Pop-Location }
