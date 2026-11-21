# Operator-only scoped integration checks. No grant, commit, push, merge or publication.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$out = Join-Path $root ('target/ops-r1/integration-26-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$savedTarget = $env:CARGO_TARGET_DIR
$savedVersion = $env:OPS_R1_GREEN_SOURCE
$savedStage = $env:OPS_R1_STAGE
Push-Location $root
try {
    $files = @(git diff --name-only 0dd8d750 -- src | Where-Object { $_.EndsWith('.rs') })
    if ($LASTEXITCODE -ne 0 -or -not $files.Count) { throw 'Integration source selection failed.' }
    & rustfmt --edition 2024 --config skip_children=true --check @files 2>&1 |
        Tee-Object -FilePath (Join-Path $out 'format-before.txt')
    if ($LASTEXITCODE -ne 0) {
        & rustfmt --edition 2024 --config skip_children=true @files 2>&1 |
            Tee-Object -FilePath (Join-Path $out 'format-normalization.txt')
        if ($LASTEXITCODE -ne 0) { throw "Formatting failed; preserve $out" }
    }
    & rustfmt --edition 2024 --config skip_children=true --check @files 2>&1 |
        Tee-Object -FilePath (Join-Path $out 'format-final.txt')
    if ($LASTEXITCODE -ne 0) { throw "Formatting acceptance failed; preserve $out" }
    & python (Join-Path $PSScriptRoot 'prepare-integration.py') $out
    if ($LASTEXITCODE -ne 0) { throw "Source reconciliation failed; preserve $out" }
    $source = Get-Content (Join-Path $out 'integration-source.json') -Raw | ConvertFrom-Json
    $focused = @()
    if ($source.source_changed) {
        $env:CARGO_TARGET_DIR = Join-Path $root 'target'
        $env:OPS_R1_STAGE = 'green'
        $env:OPS_R1_GREEN_SOURCE = 'integration-26'
        & pwsh -NoProfile -File (Join-Path $source.execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/integration-26-build-01
        if ($LASTEXITCODE -ne 0) { throw "Integration compilation failed; preserve $out" }
        $build = Join-Path $root 'target/ops-r1/integration-26-build-01/build-receipt.json'
        foreach ($case in @('cost-constraints','directory-work','fitness','typescript_persistence','source_and_package_changes_invalidate_proof','config_roots_and_route_changes_stay_independent','same_config_different_automatic_type_lookup_stays_distinct')) {
            Write-Host "START integration $case"
            $result = @(& python (Join-Path $source.execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
            $code = $LASTEXITCODE
            if ($result.Count -ne 1) { throw "Capture output incomplete; preserve $out" }
            $path = Join-Path $result[0] 'receipt.json'
            $value = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
            $focused += [ordered]@{case=$case; receipt=$path; sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant(); status=$value.status}
            $focused | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $out 'focused-receipts.json') -Encoding utf8NoBOM
            if ($code -ne 0 -or $value.status -cne 'TRACKED_TEST_COMPLETED' -or -not $value.post_source_and_binary_identity_verified) {
                throw "Integration test failed: $case; preserve $out"
            }
        }
        & cargo clippy --lib --tests --all-features -- -D warnings 2>&1 |
            Tee-Object -FilePath (Join-Path $out 'clippy.txt')
        $clippy = $LASTEXITCODE
        @{command='cargo clippy --lib --tests --all-features -- -D warnings';exit_code=$clippy} |
            ConvertTo-Json | Set-Content (Join-Path $out 'clippy-receipt.json') -Encoding utf8NoBOM
        if ($clippy -ne 0) { throw "Scoped Clippy failed; preserve $out" }
    }
    foreach ($definition in @('scripts/tests/check-file-sizes.tests.ps1','scripts/check-file-sizes.ps1','scripts/check-utf8.ps1')) {
        $name = [IO.Path]::GetFileName($definition)
        & pwsh -NoProfile -File (Join-Path $root $definition) 2>&1 |
            Tee-Object -FilePath (Join-Path $out "$name.txt")
        if ($LASTEXITCODE -ne 0) { throw "Integration guard failed: $definition; preserve $out" }
    }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne 'f03052a27e2a916ddf52dfb0a18deb246b387c386c04584f8dc8d8115c3ff4cd') {
        throw "Authoring binary identity changed; preserve $out. No runtime started."
    }
    $descriptor = Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptor 2> (Join-Path $out 'descriptor-stderr.txt')
    $descriptorExit = $LASTEXITCODE
    $requests = Join-Path $out 'knowledge-requests.json'
    ConvertTo-Json -InputObject @(@{workspaceRoot=$root;mode='context';id='OPS-001'}) -Depth 10 |
        Set-Content $requests -Encoding utf8NoBOM
    $admission = 'REFUSED_OR_INCOMPLETE'
    try {
        & (Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1') -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $requests -OutputRoot (Join-Path $out 'knowledge') -LaunchDirectory (Join-Path $root '.clean-ctx/pcg-documents/authoring')
        $payloads = @(Get-ChildItem (Join-Path $out 'knowledge') -Filter '*.payload.json')
        if ($payloads.Count -eq 1) {
            $payload = Get-Content $payloads[0].FullName -Raw | ConvertFrom-Json
            if ($payload.result_state -ceq 'ready' -and $payload.complete -and $payload.catalog_current) {
                $admission = 'CONTEXT_READY_SOURCE_ACQUISITION_AND_PUBLICATION_NOT_CLAIMED'
            }
        }
    } catch {
        $_.ToString() | Set-Content (Join-Path $out 'knowledge-error.txt') -Encoding utf8NoBOM
    }
    [ordered]@{
        agent='Agent MaxHeadRoom'; category='Scoped integration and actual governance checker capture'
        source_tree_sha256=$source.source_tree_sha256; source_changed=$source.source_changed
        accepted_results_reused=(-not $source.source_changed); fresh_focused_count=$focused.Count
        descriptor_exit_code=$descriptorExit; descriptor_sha256=(Get-FileHash $descriptor).Hash.ToLowerInvariant()
        authoring_binary_sha256=(Get-FileHash $binary).Hash.ToLowerInvariant()
        profile_sha256=(Get-FileHash $profile).Hash.ToLowerInvariant(); admission_standing=$admission
        authorization_submitted=$false; protected_declarations_changed=$false
        ci_success_claimed=$false; commit_push_merge_or_publication_performed=$false
    } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Return integration receipts and actual governance result: $out"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
    $env:OPS_R1_STAGE=$savedStage
}
