# Agent MaxHeadRoom: integrated tracked Rust checkpoint; no Node/runtime pilot.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath 'target/issue-162'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    & rustfmt --edition 2024 --config skip_children=true src/mcp/tool_handlers/core/compress.rs src/tests/mcp/angular_application_config_controls_issue_162.rs *> (Join-Path $logDirectory 'application-config67-repair-format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Repair formatting failed; see target/issue-162/application-config67-repair-format.log' }
    & cargo test --lib --all-features --no-run --quiet *> (Join-Path $logDirectory 'application-config67-compile.log')
    if ($LASTEXITCODE -ne 0) { throw 'Compile failed; see target/issue-162/application-config67-compile.log' }
    foreach ($testFilter in @('issue_162_application_config_provider','issue_162_route_provider','angular_template_compatibility','ngrx_legacy_transition')) {
        $testLog = Join-Path $logDirectory "application-config67-$testFilter.log"
        & cargo test --lib --all-features --quiet $testFilter -- --test-threads=1 *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 8
        if ($testExit -ne 0) { throw "Tracked tests failed: $testFilter; full log: $testLog" }
    }
    Write-Output 'Requested compile/focused/route/compatibility commands GREEN; logs retained under target/issue-162/application-config67-*'
} finally {
    Pop-Location
}
