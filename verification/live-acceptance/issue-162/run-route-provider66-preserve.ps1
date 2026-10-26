# Agent MaxHeadRoom: one integrated deterministic/preservation/lint batch, no Node.
param([switch]$SkipExitCases)
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/route-provider66-preserve-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'compile.log')
    if ($LASTEXITCODE -ne 0) { throw "Compile failed; see $logDirectory/compile.log" }
    # Six earlier route controls remain GREEN; run the five newly added controls.
    $filters = @('exit_cases::issue_162_route_provider', 'issue_162_provider',
        'issue_162_view_provider', 'issue_162_string', 'issue_162_module',
        'provider_semantics', 'angular_meta::routing::',
        'angular_template_compatibility', 'ngrx_legacy_transition')
    if ($SkipExitCases) {
        $filters = $filters | Where-Object { $_ -ne 'exit_cases::issue_162_route_provider' }
        Write-Output 'Exit controls skipped; use only after their recorded results and corrected rerun are GREEN.'
    }
    foreach ($testFilter in $filters) {
        $logName = $testFilter.Replace('::', '-')
        $testLog = Join-Path $logDirectory "$logName.log"
        & cargo test --lib --all-features --quiet $testFilter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 5
        if ($testExit -ne 0) { throw "Tests failed: $testFilter; full log: $testLog" }
    }
    & cargo fmt --all -- --check *> (Join-Path $logDirectory 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed; see $logDirectory/fmt.log" }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logDirectory 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed; see $logDirectory/clippy.log" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logDirectory 'utf8.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding guard failed; see $logDirectory/utf8.log" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size guard failed; see $logDirectory/sizes.log" }
    Write-Output "Scoped checkpoint GREEN; full logs: $logDirectory"
} finally {
    Pop-Location
}
