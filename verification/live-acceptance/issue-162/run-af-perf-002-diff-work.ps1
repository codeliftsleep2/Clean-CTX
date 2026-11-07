# Agent MaxHeadRoom: public diff characterization, not an optimization budget.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/diff-work-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Diff characterization: $logs"
    $changed=@('src/diff/builder.rs','src/diff/differ.rs','src/mcp/tool_helpers.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/mcp/artifact_operation_work_af_perf_002.rs','src/tests/mcp/diff_work_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='af002_diff_public_';count=2},
        @{filter='overload_insertion_does_not_relabel_unchanged_overloads';count=1},
        @{filter='first_middle_and_final_body_changes_stay_on_their_declarations';count=1},
        @{filter='csharp_modifier_only_change_is_compared_as_declaration_evidence';count=1},
        @{filter='diff_code_context_emits_canonical_envelope';count=1}
    )){
        $log=Join-Path $logs ($case.filter+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        Get-Content -LiteralPath $log -Tail 10
        $summary=[regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if($code -ne 0 -or -not $summary.Success -or [int]$summary.Groups[1].Value -ne $case.count){throw "Tracked characterization/control failed: $log"}
    }
    $rows=@([IO.File]::ReadAllLines((Join-Path $logs 'af002_diff_public_.log')) | Where-Object {$_ -match '^AF002_DIFF '})
    if($rows.Count -ne 40){throw "Expected 40 operation rows, found $($rows.Count): $logs"}
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-af-perf-002-diff-work.ps1 docs/agent/AF_PERF_002_DIFF_DISCOVERY.md)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $hashes=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='Tracked public diff characterization and exact matching/envelope controls';files=$hashes;operationRows=$rows.Count;optimizationImplemented=$false} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Characterization complete, not optimization RED: $logs"
} finally {Pop-Location}
