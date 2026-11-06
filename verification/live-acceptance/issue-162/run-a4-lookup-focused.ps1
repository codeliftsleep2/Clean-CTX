# Agent MaxHeadRoom: user-owned A4 focused acceptance; no production mutation.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/a4-lookup-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "A4 focused evidence: $logs"
    $changed=@('src/ir/csharp_resolution/compilation/dependencies.rs','src/ir/csharp_resolution/compilation/dependency_inputs.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/ir/a4_lookup_acceptance_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='afperf_a4_';count=2},
        @{filter='af002_a4_lookup_';count=2},
        @{filter='af002_a4_public_lookup_budget';count=1},
        @{filter='retained_artifacts_project_cross_file_calls_without_reparsing';count=1},
        @{filter='removal_and_signature_change_revoke_exact_calls';count=1},
        @{filter='newly_introduced_ambiguity_revokes_and_removal_reestablishes';count=1},
        @{filter='failed_preparation_remains_dirty_and_returns_no_current_authority';count=1}
    )){
        $log=Join-Path $logs (($case.filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        Get-Content -LiteralPath $log -Tail 10
        if($code -ne 0 -or [IO.File]::ReadAllText($log) -notmatch "test result: ok\. $($case.count) passed; 0 failed; 0 ignored;"){throw "Tracked GREEN missing: $log"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock scripts/check-semantic-work-fitness.ps1 verification/live-acceptance/issue-162/run-a4-lookup-focused.ps1 verification/live-acceptance/issue-162/run-a4-lookup-mutation.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Nine tracked A4 semantic/work/static/cost/lifecycle controls';files=$hashes} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Nine tracked tests and focused gates complete: $logs"
} finally {Pop-Location}
