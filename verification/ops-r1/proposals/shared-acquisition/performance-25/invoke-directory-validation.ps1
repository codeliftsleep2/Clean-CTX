# Focused operator verification; existing exact-child monitor owns all test execution.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../../..'))
$isolated = Join-Path $root 'target/ops-r1/m3-isolated-23'
$adapter = Join-Path $isolated 'verification/ops-r1/proposals/shared-acquisition/execution/capture.py'
$build = Join-Path $isolated 'target/ops-r1/performance-25-directory-green-build-01/build-receipt.json'
$out = Join-Path $root ('target/ops-r1/directory-validation-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out -ErrorAction Stop | Out-Null
$savedStage = $env:OPS_R1_STAGE
$savedVersion = $env:OPS_R1_GREEN_SOURCE
$savedTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = Join-Path $isolated 'target'
$env:OPS_R1_STAGE = 'green'
$env:OPS_R1_GREEN_SOURCE = 'performance-25-directory-green'
$cases = @(
    'cost-constraints',
    'directory-work',
    'precached_host_unit_project_and_owner_only_proof',
    'seed_is_not_sibling_disk_authority',
    'divergent_overlay_and_private_cache_are_not_overwritten',
    'failed_partial_and_poisoned_leader_do_not_grant_or_poison',
    'standalone_capture_and_missing_module_qualification',
    'source_and_package_changes_invalidate_proof',
    'config_roots_and_route_changes_stay_independent',
    'request_directory_and_inherited_project_remain_authoritative',
    'same_config_different_automatic_type_lookup_stays_distinct',
    'multiple_projects_and_operations_do_not_share_authority',
    'typescript_persistence'
)
$receipts = @()
Push-Location $root
try {
    & pwsh -NoProfile -File (Join-Path (Split-Path $adapter) 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/performance-25-directory-green-build-01
    if ($LASTEXITCODE -ne 0) { throw "Build failed; no tests started; preserve $out" }
    foreach ($case in $cases) {
        Write-Host "START $case"
        $result = @(& python $adapter --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
        $code = $LASTEXITCODE
        $result | Set-Content -LiteralPath (Join-Path $out "$case-adapter.txt") -Encoding utf8NoBOM
        if ($result.Count -ne 1) { throw "Unexpected adapter output; preserve $out" }
        $receiptPath = Join-Path $result[0] 'receipt.json'
        $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
        $receipts += [ordered]@{
            case = $case
            receipt = $receiptPath
            sha256 = (Get-FileHash -LiteralPath $receiptPath).Hash.ToLowerInvariant()
            status = $receipt.status
        }
        $receipts | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $out 'receipts.json') -Encoding utf8NoBOM
        if ($code -ne 0 -or $receipt.status -cne 'TRACKED_TEST_COMPLETED' -or -not $receipt.post_source_and_binary_identity_verified) {
            throw "Focused control failed: $case; preserve $out"
        }
        Write-Host "PASS $case"
    }
    & python (Join-Path $PSScriptRoot 'run-growth.py') performance-25-directory-green $build flat-scripts-4 flat-scripts-16 directories-scripts-16 flat-scripts-64 directories-scripts-64 flat-modules-16 directories-modules-16 2>&1 |
        Tee-Object -FilePath (Join-Path $out 'growth-output.txt')
    if ($LASTEXITCODE -ne 0) { throw "Growth comparison stopped; preserve $out" }
    Write-Output "Return focused receipts and growth evidence: $out"
} finally {
    Pop-Location
    $env:OPS_R1_STAGE = $savedStage
    $env:OPS_R1_GREEN_SOURCE = $savedVersion
    $env:CARGO_TARGET_DIR = $savedTarget
}
