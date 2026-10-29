# Agent MaxHeadRoom: bounded neutral chain only; semantic RED is recorded last.
param([switch]$ResumeAfterRefusalFixtureCorrection, [switch]$ResumeAtLint)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/lazy-arrow-authority-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_lazy_route_component_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    $formatFiles = @(
        'src/ir/typescript_resolution/arrow_selection.rs',
        'src/ir/typescript_resolution/arrow_selection/acquire.rs',
        'src/ir/typescript_resolution/mod.rs','src/ir/typescript_resolution/capture.rs',
        'src/ir/typescript_resolution/declaration_facets.rs','src/ir/typescript_resolution/demanded_occurrences.rs',
        'src/tests/mcp/typescript_lazy_route_field_issue_162.rs',
        'src/tests/mcp/typescript_lazy_selection_support_issue_162.rs',
        'src/tests/mcp/typescript_lazy_selection_issue_162.rs',
        'src/tests/mcp/typescript_lazy_selection_lifecycle_issue_162.rs',
        'src/tests/mcp/angular_lazy_route_semantic_issue_162.rs','src/mcp/semantic_compilation.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @formatFiles *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Original investigation bytes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    if (-not $ResumeAtLint) {
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not semantic RED; logs: $logDirectory" }
    $filters = @('issue_162_lazy_selection','issue_162_lazy_field','issue_162_route_component','angular_meta::routing::','issue_162_route_provider','provider_semantics','issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition')
    if ($ResumeAfterRefusalFixtureCorrection) {
        $filters[0] = 'issue_162_lazy_selection_syntax_export_refusals_and_neutral_roles'
    }
    foreach ($filter in $filters) {
        $logName = $filter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Neutral/preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet angular_lazy_route_component_tests -- --test-threads=1 *> (Join-Path $logDirectory 'original-investigation.log')
    $originalExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'original-investigation.log') -Tail 16
    if ($originalExit -ne 0) { throw "Original probe prerequisite remains; classify before semantics: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_lazy_semantic -- --test-threads=1 *> (Join-Path $logDirectory 'semantic-investigation.log')
    $semanticExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'semantic-investigation.log') -Tail 24
    Write-Output "Semantic investigation exit: $semanticExit. Preserve observed boundary; production lazy semantics remain unapproved. Full logs: $logDirectory"
    exit $semanticExit
} finally { Pop-Location }
