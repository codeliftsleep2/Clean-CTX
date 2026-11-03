# Agent MaxHeadRoom: focused tracked admission-work baseline, not full acceptance.
param([ValidateSet('Before','After')][string]$Phase = 'Before')
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/cross-relation-replay-' + $Phase.ToLowerInvariant() + '-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $paths = @('src/angular_meta/admission_work_metrics.rs','src/angular_meta/mod.rs','src/angular_meta/route_guards.rs','src/angular_meta/route_components.rs','src/angular_meta/route_lazy_components.rs','src/angular_meta/route_load_children.rs','src/angular_meta/route_providers.rs','src/angular_meta/provider_pair_authority.rs','src/tests/mcp/angular_admission_work_af_perf_001.rs','src/mcp/semantic_compilation.rs','src/ir/typescript_resolution/relation_evidence.rs')
    & rustfmt --edition 2024 --config skip_children=true @paths *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size gate failed: $logDirectory" }
    $snapshot = Join-Path $logDirectory 'source'
    foreach ($path in $paths) {
        $destination = Join-Path $snapshot $path
        New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
        Copy-Item -LiteralPath $path -Destination $destination
    }
    & cargo test --lib --all-features --quiet af_perf_001_ -- --test-threads=1 --nocapture *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'tests.log') -Tail 65
    $hashes = @($paths | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked Rust architectural work-count tests';testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
    if ($Phase -eq 'Before') { Write-Output 'Expected RED: per-edge counts exceed one at candidate growth. Compilation/fixture failures do not qualify. Return this log directory for inspection.' }
    if ($Phase -eq 'After' -and $testExit -ne 0) { throw "Focused gate failed: $logDirectory" }
    exit $testExit
} finally { Pop-Location }
