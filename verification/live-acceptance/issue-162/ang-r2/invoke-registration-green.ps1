param([switch]$RepairOnly)
# Operator-owned focused ANG-R2 GREEN and membership-preservation capture.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$witness = 'src/tests/mcp/angular_resolver_registration_public_issue_162.rs'
$frozenHash = '8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch.' }
    if ((Get-FileHash -LiteralPath $witness).Hash.ToLowerInvariant() -cne $frozenHash) { throw 'Frozen public witness changed.' }
    $out = Join-Path $root ('target/ang-r2/registration-green-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    function Source-Pins {
        $paths = @(& git diff HEAD --name-only) + @(& git ls-files --others --exclude-standard src)
        $paths += @('Cargo.toml', 'Cargo.lock', 'build.rs', $witness)
        @($paths | Sort-Object -Unique | Where-Object { ($_ -like '*.rs' -or $_ -like 'Cargo.*') -and (Test-Path -LiteralPath $_) } | ForEach-Object {
            [ordered]@{path=$_; sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}
        })
    }
    $before = ConvertTo-Json -InputObject @(Source-Pins) -Depth 20
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest.json'), $before, $utf8)
    $runs = @(
        @{filter='issue_162_resolver_registration_'; expected=18; category='New capability GREEN, lifecycle, compatibility and bounded-work tests'},
        @{filter='issue_162_resolver78_'; expected=6; category='Existing membership controls and lifecycle preservation at generation 79'},
        @{filter='issue_162_resolver_semantic_key_only_public_collision'; expected=1; category='Original unqualified public membership collision, unchanged'}
    )
    if ($RepairOnly) {
        $repairs = @(
            'complete_admission_rejects_missing_extra_crossed',
            'conservative_producer_hint_is_not_actual_scope',
            'generation78_sqlite_payload_requires_reacquisition',
            'missing_sqlite_collection_is_not_empty_success',
            'query_work_scales_by_owner_and_records',
            'sqlite_restart_key_target_and_retirement'
        )
        $runs = @($repairs | ForEach-Object {
            @{filter=('issue_162_resolver_registration_' + $_); expected=1; category='Repaired session-lifecycle test; cold query still runs first'}
        }) + @($runs[1], $runs[2])
    }
    foreach ($run in $runs) {
        $folder = Join-Path $out $run.filter
        New-Item -ItemType Directory -Path $folder | Out-Null
        $commandArgs = @('test', '--lib', '--all-features', '--quiet', $run.filter, '--', '--test-threads=1', '--nocapture')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & cargo @commandArgs 1> (Join-Path $folder 'stdout.txt') 2> (Join-Path $folder 'stderr.txt')
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category=$run.category; command=('cargo ' + ($commandArgs -join ' '))
            repair_only=[bool]$RepairOnly
            exit_code=$code; elapsed_seconds=$watch.Elapsed.TotalSeconds
            head=(& git rev-parse HEAD); expected_test_count=$run.expected
            frozen_witness_sha256=(Get-FileHash -LiteralPath $witness).Hash.ToLowerInvariant()
            source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'source-manifest.json')).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash -LiteralPath (Join-Path $folder 'stdout.txt')).Hash.ToLowerInvariant()
            stderr_sha256=(Get-FileHash -LiteralPath (Join-Path $folder 'stderr.txt')).Hash.ToLowerInvariant()
        } | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $folder 'receipt.json') -Encoding utf8NoBOM
        Write-Host "Captured $($run.filter): exit $code; $folder"
        if ($code -ne 0) { throw "Focused run failed; preserve $out" }
        if (Select-String -LiteralPath (Join-Path $folder 'stderr.txt') -Pattern '^warning:' -Quiet) { throw "Warnings found; preserve $out" }
        if (-not (Select-String -LiteralPath (Join-Path $folder 'stdout.txt') -SimpleMatch "test result: ok. $($run.expected) passed; 0 failed;" -Quiet)) { throw 'Unexpected selected tests.' }
    }
    $after = ConvertTo-Json -InputObject @(Source-Pins) -Depth 20
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest-after.json'), $after, $utf8)
    if ($before -cne $after) { throw "Sources changed during verification; preserve $out" }
    if ((Get-FileHash -LiteralPath $witness).Hash.ToLowerInvariant() -cne $frozenHash) { throw 'Frozen witness changed during verification.' }
    Write-Host "Return all receipts and logs from $out. No production-binary or CI acceptance is implied."
} finally { Pop-Location }
