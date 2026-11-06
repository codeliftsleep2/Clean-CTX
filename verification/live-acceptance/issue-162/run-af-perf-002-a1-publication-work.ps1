# Agent MaxHeadRoom: operator-owned A1 default-publication characterization only, no optimization.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs = Join-Path $repoPath ('target/af-perf-002/a1-publication-work-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "Discovery packet: $logs"
    $changed = @('src/tests/architecture/work_observations_af_perf_002.rs','src/mcp/semantic_compilation/artifact.rs','src/mcp/csharp_calls/reconciliation.rs','src/mcp/tool_handlers/core/compress.rs','src/mcp/tool_handlers/hydration/publication.rs','src/mcp/persistence_ir.rs','src/mcp/durable_semantics.rs','src/workspace/index/coverage.rs','src/tests/mcp/artifact_operation_work_af_perf_002.rs','src/tests/mcp/publication_copy_af_perf_002.rs','src/tests/mcp/baseline_publication.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Format failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size failed: $logs" }
    foreach ($selection in @(@{filter='af002_a1_default_';count=2},@{filter='failed_compress_baseline_commit_publishes_no_candidate_live_state';count=1},@{filter='failed_publication_cannot_satisfy_clean_csharp_acquisition';count=1},@{filter='retained_artifacts_project_cross_file_calls_without_reparsing';count=1},@{filter='missing_artifacts_are_compiled_once_then_reused';count=1})) {
        $logName = ($selection.filter -replace '[^A-Za-z0-9_.-]', '_') + '.log'
        $log = Join-Path $logs $logName
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $log
        if ($LASTEXITCODE -ne 0) { Get-Content -LiteralPath $log -Tail 25; throw "Tracked packet failed: $log" }
        $summary = [regex]::Match([IO.File]::ReadAllText($log),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -ne $selection.count) { throw "Missing or ignored tracked tests: $log" }
    }
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
    if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $logs" }
    & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
    if ($LASTEXITCODE -ne 0) { throw "Format check failed: $logs" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
    if ($LASTEXITCODE -ne 0) { throw "Encoding failed: $logs" }
    $paths = @(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-af-perf-002-a1-publication-work.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $files = @($paths | Sort-Object -Unique | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='Six tracked A1 public-copy characterization, failure and existing artifact controls';command='run-af-perf-002-a1-publication-work.ps1';discoveryComplete=$false;files=$files;driverSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Packet complete: $logs. Enumeration is not classification or architectural acceptance."
} finally { Pop-Location }
