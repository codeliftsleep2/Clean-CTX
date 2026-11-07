# Agent MaxHeadRoom: diff shortcut focused semantic/work/static/cost/lifecycle acceptance.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/diff-shortcut-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Edit diff shortcut focused: $logs"
    & rustfmt --edition 2024 --config skip_children=true src/mcp/tool_helpers.rs src/tests/mcp/diff_work_af_perf_002.rs src/tests/mcp/diff_shortcut_protection_af_perf_002.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
    foreach($case in @(
        @{filter='afperf_diff_shortcut_';count=2},
        @{filter='af002_diff_shortcut_';count=2},
        @{filter='af002_diff_public_';count=2},
        @{filter='overload_insertion_does_not_relabel_unchanged_overloads';count=1},
        @{filter='first_middle_and_final_body_changes_stay_on_their_declarations';count=1},
        @{filter='csharp_modifier_only_change_is_compared_as_declaration_evidence';count=1},
        @{filter='diff_code_context_emits_canonical_envelope';count=1}
    )){
        $log=Join-Path $logs (($case.filter -replace '[^A-Za-z0-9_.-]','_')+'.log')
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE;Get-Content -LiteralPath $log -Tail 8
        if($code -ne 0 -or [IO.File]::ReadAllText($log) -notmatch "test result: ok\. $($case.count) passed; 0 failed; 0 ignored;"){throw "Tracked GREEN missing: $log"}
    }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if($LASTEXITCODE -ne 0){throw "Format check failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock scripts/check-semantic-work-fitness.ps1 verification/live-acceptance/issue-162/run-diff-shortcut-focused.ps1 verification/live-acceptance/issue-162/run-diff-shortcut-mutation.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';category='10 tracked diff shortcut tests and focused checks';files=$files} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "10 tracked tests and focused gates GREEN: $logs"
} finally {Pop-Location}
