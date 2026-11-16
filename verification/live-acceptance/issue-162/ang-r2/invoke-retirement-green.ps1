param([string]$ResumeFrom)
# Operator-owned bounded shared-retirement and affected preservation tests.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    $frozen = 'src/tests/mcp/angular_deleted_owner_membership_issue_162.rs'
    if ((Get-FileHash $frozen).Hash.ToLowerInvariant() -cne '69c27a304b59029f7aee0ed2bfd511eb47c2702c0a8a43bb22d09653368b797f') { throw 'Frozen RED source changed.' }
    $out = Join-Path $root ('target/ang-r2/retirement-green-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $utf8 = [Text.UTF8Encoding]::new($false,$true)
    function Source-Pins {
        $paths = @(& git ls-files '*.rs' '*.toml' 'Cargo.lock') + @(& git ls-files --others --exclude-standard src)
        @($paths | Sort-Object -Unique | Where-Object {Test-Path -LiteralPath $_} | ForEach-Object {
            [ordered]@{path=$_;sha256=(Get-FileHash $_).Hash.ToLowerInvariant()}
        })
    }
    $before = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest.json'),$before,$utf8)
    $resumeRoot = $null
    if ($ResumeFrom) {
        $resumeRoot = (Resolve-Path -LiteralPath $ResumeFrom).Path
        $allowed = Join-Path $root 'target/ang-r2/retirement-green-'
        if (-not $resumeRoot.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path $resumeRoot -Leaf) -notmatch '^retirement-green-\d{8}-\d{6}-\d{3}$') { throw 'Invalid resume receipt directory.' }
        if ([IO.File]::ReadAllText((Join-Path $resumeRoot 'source-manifest.json'),$utf8) -cne $before) { throw 'Resume source manifest differs; do not reuse results.' }
    }
    $runs = @(
        @{filter='issue_162_deleted_owner_legacy_membership_is_retired';expected=1},
        @{filter='issue_162_retirement_';expected=9},
        @{filter='hard_link_';expected=3},
        @{filter='issue_162_resolver_registration_';expected=18},
        @{filter='issue_162_resolver78_';expected=6},
        @{filter='issue_162_resolver_semantic_key_only_public_collision';expected=1},
        @{filter='historical_replay_compatibility';expected=5},
        @{filter='failed_candidate_publication_remains_retryable_after_successful_discovery';expected=1}
    )
    foreach ($run in $runs) {
        if ($resumeRoot) {
            $prior = Join-Path $resumeRoot $run.filter
            $priorReceipt = Join-Path $prior 'receipt.json'
            $reuse = Join-Path $resumeRoot ($run.filter+'-reused.json')
            if (-not (Test-Path -LiteralPath $priorReceipt) -and (Test-Path -LiteralPath $reuse)) {
                $link = Get-Content -LiteralPath $reuse -Raw -Encoding utf8 | ConvertFrom-Json
                $priorReceipt = (Resolve-Path -LiteralPath $link.receipt).Path
                if (-not $priorReceipt.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Get-FileHash $priorReceipt).Hash.ToLowerInvariant() -cne $link.sha256) { throw 'Invalid reused receipt reference.' }
                $prior = Split-Path $priorReceipt -Parent
            }
            if (Test-Path -LiteralPath $priorReceipt) {
                $receipt = Get-Content -LiteralPath $priorReceipt -Raw -Encoding utf8 | ConvertFrom-Json
                $expectedCommand = 'cargo test --lib --all-features --quiet ' + $run.filter + ' -- --test-threads=1 --nocapture'
                if ($receipt.exit_code -ne 0 -or $receipt.command -cne $expectedCommand -or $receipt.head -cne (& git rev-parse HEAD) -or $receipt.frozen_witness_sha256 -cne (Get-FileHash $frozen).Hash.ToLowerInvariant() -or $receipt.source_manifest_sha256 -cne (Get-FileHash (Join-Path $out 'source-manifest.json')).Hash.ToLowerInvariant()) { throw 'Prior receipt is not reusable.' }
                foreach ($channel in @('stdout','stderr')) {
                    if ((Get-FileHash (Join-Path $prior ($channel+'.txt'))).Hash.ToLowerInvariant() -cne $receipt.($channel+'_sha256')) { throw 'Prior log hash mismatch.' }
                }
                if (Select-String -LiteralPath (Join-Path $prior 'stderr.txt') -Pattern '^warning:' -Quiet) { throw 'Prior run has warnings.' }
                if (-not (Select-String -LiteralPath (Join-Path $prior 'stdout.txt') -SimpleMatch -Pattern "test result: ok. $($run.expected) passed; 0 failed;" -Quiet)) { throw 'Prior selected test count differs.' }
                [ordered]@{category='Reused exact prior passing operator receipt; not a new execution';receipt=$priorReceipt;sha256=(Get-FileHash $priorReceipt).Hash.ToLowerInvariant();prior_expected_test_count=$receipt.expected_test_count;verified_test_count=$run.expected} |
                    ConvertTo-Json | Set-Content (Join-Path $out ($run.filter+'-reused.json')) -Encoding utf8NoBOM
                Write-Host "Reusing verified PASS: $($run.filter)"
                continue
            }
        }
        $folder = Join-Path $out $run.filter
        New-Item -ItemType Directory -Path $folder | Out-Null
        $argumentsForRun = @('test','--lib','--all-features','--quiet',$run.filter,'--','--test-threads=1','--nocapture')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & cargo @argumentsForRun 1> (Join-Path $folder 'stdout.txt') 2> (Join-Path $folder 'stderr.txt')
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Focused operator Rust evidence; not production stdio, CI or field evidence'
            command=('cargo '+($argumentsForRun -join ' '));exit_code=$code;expected_test_count=$run.expected
            elapsed_seconds=$watch.Elapsed.TotalSeconds;head=(& git rev-parse HEAD)
            frozen_witness_sha256=(Get-FileHash $frozen).Hash.ToLowerInvariant()
            source_manifest_sha256=(Get-FileHash (Join-Path $out 'source-manifest.json')).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash (Join-Path $folder 'stdout.txt')).Hash.ToLowerInvariant()
            stderr_sha256=(Get-FileHash (Join-Path $folder 'stderr.txt')).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content (Join-Path $folder 'receipt.json') -Encoding utf8NoBOM
        if ($code -ne 0) { throw "Focused run failed; preserve $out" }
        if (Select-String -LiteralPath (Join-Path $folder 'stderr.txt') -Pattern '^warning:' -Quiet) { throw "Warnings; preserve $out" }
        if (-not (Select-String -LiteralPath (Join-Path $folder 'stdout.txt') -SimpleMatch -Pattern "test result: ok. $($run.expected) passed; 0 failed;" -Quiet)) { throw "Unexpected test selection; preserve $out" }
    }
    $after = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest-after.json'),$after,$utf8)
    if ($before -cne $after) { throw "Sources changed during verification; preserve $out" }
    Write-Host "Return receipts from $out. Production build/stdio and governed admission remain pending."
} finally { Pop-Location }
