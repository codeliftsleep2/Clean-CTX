# Operator-owned focused diagnosis of the preserved stdio failure.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $out = Join-Path $root ('target/ang-r2/deleted-owner-regression-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $filter = 'issue_162_deleted_owner_legacy_membership_is_retired'
    $argumentsForRun = @('test','--lib','--all-features','--quiet',$filter,'--','--test-threads=1','--nocapture')
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & cargo @argumentsForRun 1> (Join-Path $out 'stdout.txt') 2> (Join-Path $out 'stderr.txt')
    $code = $LASTEXITCODE
    $watch.Stop()
    [ordered]@{
        category='Focused tracked legacy-only regression; expected behavioral result is not assumed'
        command=('cargo ' + ($argumentsForRun -join ' ')); exit_code=$code
        elapsed_seconds=$watch.Elapsed.TotalSeconds; head=(& git rev-parse HEAD)
        test_sha256=(Get-FileHash 'src/tests/mcp/angular_deleted_owner_membership_issue_162.rs').Hash.ToLowerInvariant()
        stdout_sha256=(Get-FileHash (Join-Path $out 'stdout.txt')).Hash.ToLowerInvariant()
        stderr_sha256=(Get-FileHash (Join-Path $out 'stderr.txt')).Hash.ToLowerInvariant()
    } | ConvertTo-Json | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Host "Exit $code; return preserved evidence from $out. No production-binary build, stdio session or full CI gate was run by this wrapper."
    if ($code -ne 0) { throw "Regression run failed; preserve $out" }
} finally { Pop-Location }
