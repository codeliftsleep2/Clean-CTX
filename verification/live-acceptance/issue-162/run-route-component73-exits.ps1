# Agent MaxHeadRoom: supplemental test-only exits; prior 144 passes do not repeat.
param([switch]$ResumeAfterConfigFixtureCorrection)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-component73-exits-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_route_component_membership_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/angular_route_component_exit_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen probes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; logs: $logDirectory" }
    $filter = if ($ResumeAfterConfigFixtureCorrection) { 'issue_162_route_component_exit_package_and_config_stale_restore_repair' } else { 'issue_162_route_component_exit' }
    & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> (Join-Path $logDirectory 'exits.log')
    $testExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'exits.log') -Tail 12
    if ($testExit -ne 0) { throw "Supplemental exits failed; logs: $logDirectory" }
    $expectedCount = if ($ResumeAfterConfigFixtureCorrection) { 1 } else { 4 }
    $resultPattern = 'test result: ok\. ' + $expectedCount + ' passed; 0 failed; 0 ignored;'
    if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'exits.log') -Pattern $resultPattern -Quiet)) { throw 'Unexpected supplemental batch count' }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed; logs: $logDirectory" }
    Write-Output "Supplemental exits/lint GREEN. Binary and field acceptance pending. Full logs: $logDirectory"
} finally { Pop-Location }
