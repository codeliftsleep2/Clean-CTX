# Operator-owned scoped integration only. No mutation campaigns, full gate or delivery.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$out=Join-Path $root ('target/ops-r1/integration-31-'+(Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$savedTarget=$env:CARGO_TARGET_DIR
$savedStage=$env:OPS_R1_STAGE
$savedVersion=$env:OPS_R1_GREEN_SOURCE
Push-Location $root
try {
    $files=@(git diff --name-only HEAD -- src | Where-Object {$_.EndsWith('.rs')})
    $files+=@(git ls-files --others --exclude-standard -- src | Where-Object {$_.EndsWith('.rs')})
    $files=@($files | Sort-Object -Unique)
    if(-not $files.Count){throw 'No integration Rust source selected.'}
    & rustfmt --edition 2024 --config skip_children=true @files 2>&1 | Tee-Object (Join-Path $out 'format.txt')
    if($LASTEXITCODE -ne 0){throw "Formatting failed; preserve $out"}
    & rustfmt --edition 2024 --config skip_children=true --check @files 2>&1 | Tee-Object (Join-Path $out 'format-check.txt')
    if($LASTEXITCODE -ne 0){throw "Formatting check failed; preserve $out"}
    & python (Join-Path $PSScriptRoot 'prepare.py') $out
    if($LASTEXITCODE -ne 0){throw "Reconciliation failed; preserve $out"}
    $source=Get-Content (Join-Path $out 'source.json') -Raw | ConvertFrom-Json
    $env:CARGO_TARGET_DIR=Join-Path $root 'target'
    $env:OPS_R1_STAGE='green'
    $env:OPS_R1_GREEN_SOURCE='integration-31'
    if($source.source_changed){
        & pwsh -NoProfile -File (Join-Path $source.execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/integration-31-build-01
        if($LASTEXITCODE -ne 0){throw "Compilation failed; preserve $out"}
        $build=Join-Path $root 'target/ops-r1/integration-31-build-01/build-receipt.json'
        $cases=@('public-work','phase-edit','conflicting-facts','retained-currentness','seal','qualification','cost-constraints','directory-work','fitness','seed_is_not_sibling_disk_authority','divergent_overlay_and_private_cache_are_not_overwritten','failed_partial_and_poisoned_leader_do_not_grant_or_poison','precached_unaccounted_values_never_seal_from_nonempty_witnesses','source_and_package_changes_invalidate_proof','config_roots_and_route_changes_stay_independent','same_config_different_automatic_type_lookup_stays_distinct','typescript_persistence','resolver_positive','repeated-requests')
        $receipts=@()
        foreach($case in $cases){
            $result=@(& python (Join-Path $source.execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
            $code=$LASTEXITCODE
            if($result.Count -ne 1){throw "Missing capture output; preserve $out"}
            $path=Join-Path $result[0] 'receipt.json'
            $r=Get-Content $path -Raw | ConvertFrom-Json
            $receipts+=@{case=$case;receipt=$path;sha256=(Get-FileHash $path).Hash.ToLowerInvariant();status=$r.status}
            ConvertTo-Json -InputObject $receipts -Depth 10 | Set-Content (Join-Path $out 'tests.json') -Encoding utf8NoBOM
            if($code -ne 0 -or $r.status -cne 'TRACKED_TEST_COMPLETED' -or -not $r.post_source_and_binary_identity_verified){throw "Focused regression failed: $case; preserve $out"}
        }
    }
    & cargo clippy --lib --tests --all-features -- -D warnings 2>&1 | Tee-Object (Join-Path $out 'clippy.txt')
    if($LASTEXITCODE -ne 0){throw "Scoped lint failed; preserve $out"}
    # Existing actual document descriptor/context/source route includes repository guards.
    & pwsh -NoProfile -File (Join-Path $root 'verification/ops-r1/proposals/integration-26/invoke-final-documents.ps1') 2>&1 | Tee-Object (Join-Path $out 'documents-output.txt')
    if($LASTEXITCODE -ne 0){throw "Document checker/refusal requires review; preserve $out"}
    [ordered]@{agent='Agent MaxHeadRoom';source_tree_sha256=$source.source_tree_sha256;source_changed=$source.source_changed;scoped_lint_exit=0;document_payload_review_required=$true;ci_success_claimed=$false;field_acceptance_claimed=$false;delivery_performed=$false} | ConvertTo-Json | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Return integration-31 evidence: $out"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR=$savedTarget
    $env:OPS_R1_STAGE=$savedStage
    $env:OPS_R1_GREEN_SOURCE=$savedVersion
}
