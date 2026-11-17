# Agent MaxHeadRoom: operator-owned focused preservation check, not a full gate.
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $root
try {
    $out=Join-Path $root ('target/ci-verify-r1/mixed-preservation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $files=@(
        'src/tests/mcp/delta_previous_integrated_af_perf_002.rs'
        'src/tests/mcp/diff_source_integrated_af_perf_002.rs'
        'src/tests/mcp/sqlite_text_integrated_af_perf_002.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @files
    if ($LASTEXITCODE -ne 0) { throw 'Focused formatting failed' }
    $selection=@(
        'mcp::tool_handlers::core::delta::delta_design::integrated::af002_delta_previous_integrated_cost'
        'mcp::tool_handlers::core::delta::ownership_design::integrated::af002_diff_source_integrated_cost'
        'mcp::sqlite_store::work_af_perf_002::text_integrated::af002_sqlite_text_integrated_cost'
    )
    $log=Join-Path $out 'tracked-tests.log'
    & cargo test --lib --all-features -- --exact --test-threads=1 @selection *> $log
    $code=$LASTEXITCODE
    $ok=$code -eq 0 -and (Select-String -LiteralPath $log -Pattern '^test result: ok\. 3 passed; 0 failed; 0 ignored;' -Quiet)
    [ordered]@{agent='Agent MaxHeadRoom';head=(git rev-parse HEAD);category='Three mandatory mixed-test assertion phases; no empirical performance result';exitCode=$code;passed=$ok;selection=$selection;files=@($files|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}});log_sha256=(Get-FileHash $log).Hash} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath $log -Tail 25
    if (-not $ok) { throw "Focused preservation failed; preserve $out" }
    Write-Output "Return focused receipts: $out"
} finally { Pop-Location }
