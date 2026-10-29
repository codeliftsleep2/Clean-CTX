# Agent MaxHeadRoom: neutral authority GREEN before unchanged legacy semantic probe.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/legacy-bootstrap-authority-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozenTest = 'src/tests/mcp/angular_legacy_bootstrap_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/ir/typescript_resolution/declared_return_member.rs src/ir/typescript_resolution/capture.rs src/ir/typescript_resolution/declaration_facets.rs src/ir/typescript_resolution/demanded_occurrences.rs src/ir/typescript_resolution/mod.rs src/angular_meta/bootstrap_references.rs src/mcp/semantic_compilation.rs src/tests/mcp/typescript_declared_return_member_issue_162.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozenTest -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen investigation bytes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    foreach ($filter in @('issue_162_declared_return_member','issue_162_bootstrap_controls','issue_162_config_composition')) {
        $testLog = Join-Path $logDirectory "$filter.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Neutral authority/preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_legacy_bootstrap -- --test-threads=1 *> (Join-Path $logDirectory 'legacy-investigation.log')
    $legacyExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'legacy-investigation.log') -Tail 24
    Write-Output "Legacy probe exit: $legacyExit. Expected association RED only after native prerequisites; production relation remains unapproved. Full logs: $logDirectory"
    exit $legacyExit
} finally { Pop-Location }
