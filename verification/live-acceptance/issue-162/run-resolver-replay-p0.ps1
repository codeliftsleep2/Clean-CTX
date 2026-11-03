# Agent MaxHeadRoom: fixed direct-authority P0 work-count/timing gate, not full filters.
param([ValidateSet('Before','After')][string]$Phase = 'Before')
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/resolver-replay-p0-' + $Phase.ToLowerInvariant() + '-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & rustfmt --edition 2024 --config skip_children=true src/angular_meta/route_resolvers.rs src/angular_meta/route_resolvers/replay_metrics.rs src/ir/typescript_resolution/relation_evidence.rs src/ir/typescript_resolution/declaration_facets/resolve_mappings.rs src/tests/mcp/angular_resolver_replay_p0_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size gate failed: $logDirectory" }
    # Exactly one Cargo invocation. libtest timing excludes compilation.
    & cargo test --lib --all-features --quiet issue_162_replay_p0_ -- --test-threads=1 --nocapture *> (Join-Path $logDirectory 'tests.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'tests.log') -Tail 36
    $paths = @('src/tests/mcp/angular_resolver_replay_p0_issue_162.rs','src/angular_meta/route_resolvers.rs','src/ir/typescript_resolution/relation_evidence.rs','src/ir/typescript_resolution/declaration_facets/resolve_mappings.rs')
    $hashes = @($paths | ForEach-Object { [pscustomobject]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash } })
    [pscustomobject]@{ agent = 'Agent MaxHeadRoom'; phase = $Phase; category = 'tracked work-count and fixed-fixture timing'; testExit = $testExit; files = $hashes } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
    if ($Phase -eq 'Before') { Write-Output "Expected architecture RED: projection-per-edge and duplicate mapping replay. Compilation/fixture failures do not qualify. Return logs: $logDirectory" }
    if ($Phase -eq 'After' -and $testExit -ne 0) { throw "P0 correction not GREEN: $logDirectory" }
    exit $testExit
} finally { Pop-Location }
