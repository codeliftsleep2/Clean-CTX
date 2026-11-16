# Operator-owned focused behavioral RED capture; no production implementation.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/registration-red-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $commandArgs = @('test', '--lib', '--all-features', '--quiet', 'issue_162_resolver_registration_public_cold_', '--', '--test-threads=1', '--nocapture')
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & cargo @commandArgs 1> (Join-Path $out 'stdout.txt') 2> (Join-Path $out 'stderr.txt')
    $code = $LASTEXITCODE
    $watch.Stop()
    [ordered]@{
        category = 'Focused tracked test invocation; outcome requires review'
        command = 'cargo ' + ($commandArgs -join ' ')
        exit_code = $code
        elapsed_seconds = $watch.Elapsed.TotalSeconds
        head = (& git rev-parse HEAD)
        test_sha256 = (Get-FileHash -LiteralPath 'src/tests/mcp/angular_resolver_registration_public_issue_162.rs').Hash.ToLowerInvariant()
        registration_sha256 = (Get-FileHash -LiteralPath 'src/mcp/semantic_compilation.rs').Hash.ToLowerInvariant()
        expected_test_count = 2
    } | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Host "Focused run captured: $out"
    Write-Host "Exit code: $code. Return the receipt and both logs; no automatic RED claim."
} finally { Pop-Location }
