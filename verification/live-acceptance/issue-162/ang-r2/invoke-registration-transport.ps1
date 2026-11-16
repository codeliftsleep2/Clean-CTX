# Operator-owned bounded producer/transport and existing membership verification.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/registration-transport-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    foreach ($filter in @('issue_162_resolver_registration_transport_', 'issue_162_resolver78_')) {
        $run = Join-Path $out $filter
        New-Item -ItemType Directory -Path $run | Out-Null
        $commandArgs = @('test', '--lib', '--all-features', '--quiet', $filter, '--', '--test-threads=1', '--nocapture')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & cargo @commandArgs 1> (Join-Path $run 'stdout.txt') 2> (Join-Path $run 'stderr.txt')
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category = 'Tracked producer/transport or membership tests; not public registration acceptance'
            command = 'cargo ' + ($commandArgs -join ' ')
            exit_code = $code
            elapsed_seconds = $watch.Elapsed.TotalSeconds
            head = (& git rev-parse HEAD)
            expected_test_count = $(if ($filter -ceq 'issue_162_resolver_registration_transport_') { 2 } else { 6 })
            transport_test_sha256 = (Get-FileHash -LiteralPath 'src/tests/mcp/angular_resolver_registration_transport_issue_162.rs').Hash.ToLowerInvariant()
        } | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $run 'receipt.json') -Encoding utf8NoBOM
        Write-Host "Run captured: $run; exit $code"
        if ($code -ne 0) { throw "Focused run failed; preserve $out" }
        if (Select-String -LiteralPath (Join-Path $run 'stderr.txt') -Pattern '^warning:' -Quiet) {
            throw "Warnings found; preserve $out"
        }
        $expected = if ($filter -ceq 'issue_162_resolver_registration_transport_') { 2 } else { 6 }
        if (-not (Select-String -LiteralPath (Join-Path $run 'stdout.txt') -SimpleMatch "test result: ok. $expected passed; 0 failed;" -Quiet)) {
            throw "Unexpected selected tests; preserve $out"
        }
    }
    Write-Host "Return both receipts and logs from $out"
} finally { Pop-Location }
