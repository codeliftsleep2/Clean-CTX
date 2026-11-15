param([string]$WorkspaceRoot = (Get-Location).Path, [switch]$Authority, [switch]$Governance, [switch]$Bootstrap, [switch]$Replacement, [switch]$Lifecycle, [switch]$RuleComponents, [switch]$ProxyControls)
# Operator-owned narrow behavioral RED. No build campaign or full gate.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = Join-Path $root ('target/pcg-b11/admission-red-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    if ($ProxyControls) {
        & cargo test -p clean-ctx-proxy --all-features --lib -- `
            cache::tests::test_inject_skips_when_client_breakpoints_exist `
            cache::tests::test_inject_skips_when_client_breakpoints_on_system `
            cache::tests::test_inject_skips_when_client_breakpoints_on_messages `
            cache::tests::test_injects_tools_breakpoint `
            cache::tests::test_system_breakpoint_on_large_block `
            cache::tests::test_injects_messages_breakpoint --nocapture 2>&1 |
            Tee-Object (Join-Path $output 'owner05-proxy-controls.log')
        $proxyCode = $LASTEXITCODE
        $proxySelection = (Get-Content -LiteralPath (Join-Path $output 'owner05-proxy-controls.log') -Raw) -match 'test result: ok\. 6 passed; 0 failed;'
        [IO.File]::WriteAllText((Join-Path $output 'owner05-proxy-result.json'), (@{exit_code=$proxyCode;selected_count_verified=$proxySelection;scope='six existing direct cache controls; no proxy implementation change'} | ConvertTo-Json), $utf8)
        if ($proxyCode -ne 0) { throw 'Existing proxy controls failed; preserve their receipt.' }
        if (-not $proxySelection) { throw 'Expected six existing proxy controls; preserve selection output.' }
    }
    $files = @('src/tests/pcg/document_admission_diagnostics.rs','src/pcg/documents/mod.rs')
    if ($Authority) { $files = @('src/tests/pcg/document_admission_authority.rs','src/pcg/documents/mod.rs') }
    if ($Governance) { $files = @('src/tests/pcg/document_admission_authority.rs','src/tests/pcg/document_governed_membership.rs','src/pcg/documents/mod.rs') }
    if ($Bootstrap) { $files = @('src/tests/pcg/document_governance_bootstrap.rs','src/pcg/documents/mod.rs') }
    if ($Replacement) { $files = @('src/tests/pcg/document_governance_operator.rs') }
    if ($Lifecycle) { $files = @('src/tests/pcg/document_governance_lifecycle.rs','src/pcg/documents/mod.rs') }
    if ($RuleComponents) { $files = @('src/tests/pcg/document_admission_rule_components.rs','src/pcg/documents/mod.rs') }
    & rustfmt --edition 2024 --config skip_children=true @files 2>&1 | Tee-Object (Join-Path $output 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Scoped formatting failed; RED tests not run.' }
    $hashes = @($files | ForEach-Object { @{path=$_;sha256=(Get-FileHash $_).Hash.ToLowerInvariant()} })
    [IO.File]::WriteAllText((Join-Path $output 'identity.json'), ($hashes | ConvertTo-Json -Depth 5), $utf8)
    $filter = if ($Authority) { 'admission_authority::b11_' } else { 'admission_diagnostics::b11_' }
    $filters = @($filter)
    if ($Governance) { $filters = @('admission_authority::b11_','governed_membership::b11_') }
    if ($Bootstrap) { $filters = @('governance_bootstrap::b11_') }
    if ($Replacement) { $filters = @('b11_protected_component_replacement_requires_reviewable_exact_authorization') }
    if ($Lifecycle) { $filters = @('governance_lifecycle::b11_') }
    if ($RuleComponents) { $filters = @('admission_rule_components::b11_') }
    & cargo test --lib --all-features -- @filters --nocapture 2>&1 | Tee-Object (Join-Path $output 'tests.log')
    $code = $LASTEXITCODE
    [IO.File]::WriteAllText((Join-Path $output 'result.json'), (@{exit_code=$code;filters=$filters;expected=if($RuleComponents){'one behavioral failure: summary-only governing references omit the accepted constraint component'}elseif($Lifecycle){'one behavioral removal failure; relocation is a preservation control'}elseif($Replacement){'one behavioral failure: valid protected replacement cannot produce a reviewable descriptor'}elseif($Bootstrap){'one behavioral failure: candidate registry silently becomes accepted without operator bootstrap'}elseif($Governance){'four behavioral failures: missing authority references and missing registry enforcement'}elseif($Authority){'two behavioral failures on missing governing authority metadata'}else{'two behavioral failures on missing location metadata'};scope='tracked B11 RED before the corresponding production fix'} | ConvertTo-Json), $utf8)
    Write-Output "B11 RED evidence saved: $output (exit $code). Preserve failures for inspection."
} finally { Pop-Location }
