param(
    [string]$WorkspaceRoot = (Get-Location).Path,
    [string]$OutputRoot,
    [switch]$NamespaceOnly,
    [switch]$ScopeAndNamespace,
    [switch]$ModuleAndWork,
    [switch]$ModuleFixtureCorrections,
    [switch]$RemainingFailures,
    [switch]$CSharpDependencies,
    [switch]$WithClippy,
    [switch]$LiveCbmOnly
)
# Operator-owned execution. The agent must not launch this script.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
if (-not $OutputRoot) {
    $OutputRoot = Join-Path $root ('target/pcg-b9-r1/failure-recheck-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
}
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh failure verification directory.' }
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    $inventory = Import-Csv verification/architectural-intelligence-b9-r1/B9_R1_GATE_FAILURE_INVENTORY.csv
    # Preserve the failed live-provider receipt. Resolve deterministic failures first.
    $tests = @($inventory | Where-Object disposition -ne 'live_provider_timeout_unresolved' | ForEach-Object test)
    if ($tests.Count -ne 80) { throw 'Expected the original 80 non-live failures.' }
    if ($NamespaceOnly) { $tests = @('ir::typescript_resolution::') }
    if ($ScopeAndNamespace) {
        if ($NamespaceOnly) { throw 'Choose one focused selection.' }
        $tests = @(
            'ir::typescript_resolution::',
            'workspace_query_scope::',
            'workspace_query_scope_entities::',
            'workspace_query_scope_traversal::',
            'workspace_query_scope_provenance::',
            'workspace_query_within_path'
        )
    }
    $moduleSelection = $NamespaceOnly -or $ScopeAndNamespace
    if ($ModuleAndWork) {
        if ($moduleSelection) { throw 'Choose one focused selection.' }
        $moduleSelection = $true
        $tests = @(
            'tests_issue_157::',
            'tests_issue_157_dependencies::',
            'inventory_work::',
            'delta::discovery_work::',
            'delta::delta_design::',
            'tests_exact_identity_coverage::'
        )
    }
    if ($ModuleFixtureCorrections) {
        if ($moduleSelection) { throw 'Choose one focused selection.' }
        $tests = @($inventory | Where-Object test -Match 'issue_157_(array_retargets_membership_by_existing_import_identity|durable_projection_retains_and_validates_contributors|exact_file_precedence_is_a_resolution_dependency|warm_query_reuses_compilation_and_ignores_unrelated_content)$' | ForEach-Object test)
        if ($tests.Count -ne 4) { throw 'Expected exactly four corrected module fixtures.' }
    }
    if ($RemainingFailures) {
        if ($moduleSelection -or $ModuleFixtureCorrections) { throw 'Choose one focused selection.' }
        $tests = @($inventory | Where-Object latest_verified -eq 'FAILED' | ForEach-Object test)
        if (-not $tests.Count) { throw 'No unresolved non-live failures in the inventory.' }
    }
    if ($CSharpDependencies) {
        if ($moduleSelection -or $ModuleFixtureCorrections -or $RemainingFailures) { throw 'Choose one focused selection.' }
        $moduleSelection = $true
        $tests = @(
            'tests_entities_auto_compile::',
            'workspace_query_csharp_dependencies::',
            'tests_hydration_fidelity::',
            'tests_issue_155::',
            'tests_issue_155_dependencies::',
            'tests_issue_155_blockers::',
            'tests_issue_156::',
            'tests_issue_156_dependencies::',
            'csharp_call_artifact_reconciliation::',
            'tests_performance_work_counts::'
        )
    }
    if ($LiveCbmOnly) {
        if ($moduleSelection -or $ModuleFixtureCorrections -or $RemainingFailures) { throw 'Choose one focused selection.' }
        $tests = @('cbm::tests::graph_intel::live_intel_queries_propagate_errors_on_unknown_project')
    }
    [IO.File]::WriteAllText((Join-Path $output 'selected-tests.json'), (ConvertTo-Json -InputObject $tests), $utf8)
    $frozen = 'src/tests/pcg/knowledge_reference_first.rs'
    $expected = 'fbe348e3d0ccdb200d300fd5374cbcc66c598d64e4dfcd7bedd092cbf74e21e2'
    if ((Get-FileHash $frozen).Hash.ToLowerInvariant() -ne $expected) { throw 'Frozen B9-R1 regression changed.' }
    $namespaceRegression = 'src/tests/ir/typescript_erased_namespace_queries_issue_162.rs'
    $namespaceHash = 'f2887776d37c571d6a7df7558e803b850f7d623c7ad8d666b48294f6bde7a8df'
    if ((Get-FileHash $namespaceRegression).Hash.ToLowerInvariant() -ne $namespaceHash) {
        throw 'Original namespace RED regression changed.'
    }
    $fidelityRegression = 'src/tests/mcp/workspace_query_entities_auto_compile.rs'
    $fidelityHash = '34e2e299d9caeaa8d0ffc1252997f6d1b25ceaab095e8d122ddae2d35ce86fa2'
    if ((Get-FileHash $fidelityRegression).Hash.ToLowerInvariant() -ne $fidelityHash) {
        throw 'Captured C# fidelity regression changed.'
    }
    $files = @(& git diff --name-only -- src | Where-Object { $_.EndsWith('.rs') })
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate changed Rust files.' }
    & rustfmt --edition 2024 --config skip_children=true @files 2>&1 |
        Tee-Object (Join-Path $output 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Scoped formatting failed; tests were not run.' }
    if ((Get-FileHash $frozen).Hash.ToLowerInvariant() -ne $expected) { throw 'Frozen regression changed during formatting.' }
    $arguments = @('test', '--lib', '--all-features', '--') + $tests
    if (-not $moduleSelection) { $arguments += '--exact' }
    $arguments += @('--nocapture', '--test-threads=1')
    & cargo @arguments 2>&1 | Tee-Object (Join-Path $output 'tests.log')
    $code = $LASTEXITCODE
    $receipt = [ordered]@{
        agent = 'Agent MaxHeadRoom'
        command = @('cargo') + $arguments
        expected_test_count = $(if ($moduleSelection) { 'Complete selected modules, including the original regressions' } else { $tests.Count })
        exit_code = $code
        evidence_class = 'operator-run tracked Rust tests; not the full repository gate'
        live_provider_test = $(if ($LiveCbmOnly) { 'Selected; inspect captured output for installed-provider execution or fixture skip' } else { 'Not rerun; original timeout remains unresolved' })
    }
    [IO.File]::WriteAllText((Join-Path $output 'result.json'), ($receipt | ConvertTo-Json -Depth 5), $utf8)
    Write-Output "Failure verification evidence: $output"
    if ($code -ne 0) { throw "Focused failures remain ($code). Preserve tests.log and result.json." }
    $countLine = Select-String -LiteralPath (Join-Path $output 'tests.log') -Pattern '^running (\d+) tests$' | Select-Object -Last 1
    if ($moduleSelection) {
        if (-not $countLine -or [int]$countLine.Matches[0].Groups[1].Value -lt 3) {
            throw 'Focused module selection ran fewer than three tests.'
        }
    } elseif (-not $countLine -or [int]$countLine.Matches[0].Groups[1].Value -ne $tests.Count) {
        throw 'Unexpected selected test count; inspect the capture before claiming success.'
    }
    if ($WithClippy) {
        & cargo clippy -p clean-ctx --all-targets --all-features -- -D warnings 2>&1 |
            Tee-Object (Join-Path $output 'clippy.log')
        $receipt['clippy_exit_code'] = $LASTEXITCODE
        [IO.File]::WriteAllText((Join-Path $output 'result.json'), ($receipt | ConvertTo-Json -Depth 5), $utf8)
        if ($LASTEXITCODE -ne 0) { throw 'Scoped Clippy failed; preserve clippy.log.' }
    }
} finally {
    Pop-Location
}
