# AF-PERF-001: tracked tests are the gate; this script selects and preserves evidence.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoPath
try {
    $logDirectory = Join-Path $repoPath ('target/issue-162/af-perf-001-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "AF-PERF-001 logs: $logDirectory"
    foreach ($selection in @(@{filter='af_perf_001_';minimum=14},@{filter='issue_162_replay_p0_';minimum=5},@{filter='af_delegated_001_';minimum=6})) {
        $testLog = Join-Path $logDirectory ($selection.filter + '.log')
        & cargo test --lib --all-features --quiet $selection.filter -- --test-threads=1 --nocapture *> $testLog
        $testExit = $LASTEXITCODE
        Get-Content -LiteralPath $testLog -Tail 40
        if ($testExit -ne 0) { throw "AF-PERF-001 tracked tests failed: $testLog" }
        $summary = [regex]::Match((Get-Content -LiteralPath $testLog -Raw),'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
        if (-not $summary.Success -or [int]$summary.Groups[1].Value -lt $selection.minimum) { throw "Missing/ignored required tests: $testLog" }
    }
    $paths = @(git ls-files --cached --others --exclude-standard -- src/angular_meta src/ir/typescript_resolution src/tests/architecture src/tests/mcp/angular_admission* src/tests/mcp/angular_resolver_replay_p0_issue_162.rs src/mcp/semantic_compilation.rs src/mcp/tool_handlers/query.rs src/lib.rs scripts/check-semantic-work-fitness.ps1)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    $hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';category='tracked Rust AST/direct/lifecycle/P0 fitness tests';files=$hashes;scope='retained admission; unresolved AST findings require review; not full acceptance'} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
} finally { Pop-Location }
