# Agent MaxHeadRoom: approved shared neutral written-selection verification; unchanged probes last.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/shared-written-selection-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $frozen = 'src/tests/mcp/angular_load_children_issue_162.rs'
    $beforeHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash
    & rustfmt --edition 2024 --config skip_children=true src/ir/typescript_resolution/arrow_selection.rs src/ir/typescript_resolution/declaration_facets.rs src/tests/mcp/typescript_children_selection_support_issue_162.rs src/tests/mcp/typescript_children_selection_issue_162.rs src/tests/mcp/typescript_children_selection_lifecycle_issue_162.rs src/tests/mcp/typescript_shared_selection_roles_issue_162.rs src/tests/mcp/angular_load_children_semantic_issue_162.rs src/tests/mcp/typescript_load_children_field_issue_162.rs src/mcp/semantic_compilation.rs *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed; logs: $logDirectory" }
    if ((Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash -ne $beforeHash) { throw 'Frozen probes changed' }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Active sizes failed; logs: $logDirectory" }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed; not behavioral RED; logs: $logDirectory" }
    $filters = @('issue_162_children_selection','issue_162_shared_selection','issue_162_lazy_selection','issue_162_children_field','issue_162_lazy_semantic','issue_162_lazy_controls','issue_162_route_component','angular_meta::routing::','issue_162_route_provider','provider_semantics','issue_162_module','issue_162_bootstrap_controls','issue_162_legacy','issue_162_config_composition','issue_162_application_config','issue_162_bootstrap_inline')
    foreach ($filter in $filters) {
        $logName = $filter -replace '[^A-Za-z0-9_.-]', '_'
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 6
        if ($testExit -ne 0) { throw "Capture/preservation failed: $filter; logs: $logDirectory" }
        if (-not (Select-String -LiteralPath $testLog -Pattern 'test result: ok\. [1-9][0-9]* passed;' -Quiet)) { throw "Empty filter: $filter" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; logs: $logDirectory" }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "fmt failed; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_load_children_ -- --test-threads=1 *> (Join-Path $logDirectory 'investigation.log')
    $probeExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'investigation.log') -Tail 24
    Write-Output "Investigation exit: $probeExit. Classify prerequisites separately; production LoadsChildren/target policy/replacement remain unapproved. Full logs: $logDirectory"
    if ($probeExit -ne 0) { throw "Original probes reveal a remaining prerequisite; logs: $logDirectory" }
    & cargo test --lib --all-features --quiet issue_162_children_semantic_ -- --test-threads=1 *> (Join-Path $logDirectory 'semantic-investigation.log')
    $semanticExit = $LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'semantic-investigation.log') -Tail 24
    Write-Output "Semantic investigation exit=$semanticExit; classify selected-role prerequisites before association RED. Full logs: $logDirectory"
    exit $semanticExit
} finally { Pop-Location }
