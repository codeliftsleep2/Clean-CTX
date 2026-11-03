# Agent MaxHeadRoom: exactly eight approved residual tracked tests, no broad campaign.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory=Join-Path $repoPath ('target/issue-162/resolver78-residual-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Phase A logs: $logDirectory"
    $filters=@('issue_162_route_component_controls_forged_crossed_borrowed_retained_proof','issue_162_route_component_controls_owned_refusal_and_provider_independence','issue_162_lazy_controls_forged_native_field_and_selection_proof','issue_162_lazy_controls_owned_unsupported_and_duplicate_no_weak_fallback','issue_162_lazy_controls_mixed_eager_lazy_provider_independent_retirement','issue_162_children_controls_forged_selection_and_separate_target_roles','issue_162_children_controls_owned_refusals_never_legacy_fallback','issue_162_route_provider_proof_forgery_serde_and_shared_strategy_policies')
    $sourcePaths=@(Get-ChildItem -LiteralPath (Join-Path $repoPath 'src') -Filter '*.rs' -Recurse -File | ForEach-Object { [IO.Path]::GetRelativePath($repoPath,$_.FullName).Replace('\','/') }) + @('Cargo.toml','Cargo.lock','build.rs')
    $sourcePaths=@($sourcePaths | Where-Object {Test-Path -LiteralPath $_} | Sort-Object -Unique)
    $hashes=@($sourcePaths | ForEach-Object {[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash}})
    $results=[Collections.Generic.List[object]]::new()
    $receipt=[ordered]@{agent='Agent MaxHeadRoom';category='Phase A tracked residual Rust tests';status='INCOMPLETE';commandHash=(Get-FileHash -LiteralPath $PSCommandPath).Hash;files=$hashes;results=$results}
    $receiptPath=Join-Path $logDirectory 'receipt.json'
    $receipt | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $receiptPath -Encoding utf8NoBOM
    foreach($filter in $filters){
        $log=Join-Path $logDirectory ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $log
        $code=$LASTEXITCODE
        $results.Add([pscustomobject]@{filter=$filter;command="cargo test --lib --all-features --quiet $filter -- --test-threads=1";exitCode=$code;log=$log;sha256=(Get-FileHash -LiteralPath $log).Hash})
        $receipt | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $receiptPath -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 4
        if($code -ne 0 -or -not (Select-String -LiteralPath $log -Pattern 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -Quiet)){throw "Residual test failed/missing: $log"}
    }
    foreach($item in $hashes){if((Get-FileHash -LiteralPath $item.path).Hash -ne $item.sha256){throw "Source changed during Phase A: $($item.path)"}}
    $receipt.status='PASS'
    $receipt | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $receiptPath -Encoding utf8NoBOM
    Write-Output "Phase A PASS: 8 tracked tests. Receipt: $receiptPath"
} finally {Pop-Location}
