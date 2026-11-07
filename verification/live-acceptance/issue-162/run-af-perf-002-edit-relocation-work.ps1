# Agent MaxHeadRoom: public edit relocation characterization; no repair budget.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/edit-relocation-work-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Edit characterization: $logs"
    $changed=@('src/edit/locate.rs','src/mcp/tool_handlers/edit.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/mcp/artifact_operation_work_af_perf_002.rs','src/tests/mcp/edit_relocation_work_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='af002_edit_relocation_';minimum=2},
        @{filter='edit::locate_tests::';minimum=1},
        @{filter='issue_136_delete_refuses_unowned_leading_documentation';minimum=1},
        @{filter='stale_external_source_is_rejected_before_intent_and_can_be_explicitly_reconciled';minimum=1},
        @{filter='durable_edit_failure_restores_exact_prior_bytes_and_preserves_live_state';minimum=1},
        @{filter='candidate_compilation_failure_precedes_source_intent_and_live_mutation';minimum=1}
    )){
        $log=Join-Path $logs (($case.filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        Get-Content -LiteralPath $log -Tail 10
        $summary=[regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if($code -ne 0 -or -not $summary.Success -or [int]$summary.Groups[1].Value -lt $case.minimum){throw "Tracked characterization/control failed: $log"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-af-perf-002-edit-relocation-work.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $hashes=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Public edit growth/refusal plus existing locator, metadata, stale-source, durable-failure and candidate-failure controls';files=$hashes;optimizationImplemented=$false} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Characterization complete, not optimization RED: $logs"
} finally {Pop-Location}
