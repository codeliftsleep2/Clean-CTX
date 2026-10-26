# Agent MaxHeadRoom: rerun only the corrected control, then resume pending gates.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $testLog = Join-Path $repoPath 'target/issue-162/route-provider66-root-repair-rerun.log'
    & cargo test --lib --all-features --quiet issue_162_route_provider_stale_barrel_config_export_and_root_repair -- --test-threads=1 *> $testLog
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath $testLog -Tail 12
    if ($testExit -ne 0) { throw "Corrected control failed; see $testLog" }
    & pwsh -NoProfile -ExecutionPolicy Bypass (Join-Path $PSScriptRoot 'run-route-provider66-preserve.ps1') -SkipExitCases
    if ($LASTEXITCODE -ne 0) { throw 'Preservation/lint batch stopped; inspect its printed log directory.' }
} finally {
    Pop-Location
}
