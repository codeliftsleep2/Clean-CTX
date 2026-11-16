param([string]$ResumeVerifiedFrom)
# Operator-owned disposition-03 focused tests, scoped formatting/lint, guards and admission.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$utf8 = [Text.UTF8Encoding]::new($false,$true)
Push-Location $root
try {
    $normalization = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_FORMAT_NORMALIZATION.json') -Raw -Encoding utf8 | ConvertFrom-Json
    if ((Get-FileHash $normalization.retention_manifest).Hash.ToLowerInvariant() -cne $normalization.retention_manifest_sha256) { throw 'Retained evidence manifest changed.' }
    foreach ($witness in $normalization.witnesses) {
        if ((Get-FileHash $witness.file).Hash.ToLowerInvariant() -cne $witness.proposed_sha256) { throw 'Approved normalized witness changed.' }
    }
    $pins = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_FORMAT_GOVERNANCE_PINS.json') -Raw -Encoding utf8 | ConvertFrom-Json
    if ((& git rev-parse HEAD) -cne $pins.baseline -or (& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Expected mission branch and baseline.' }
    if ((Get-FileHash $pins.source).Hash.ToLowerInvariant() -cne $pins.candidate_sha256) { throw 'Governing candidate changed.' }
    if ((Get-FileHash $pins.index).Hash.ToLowerInvariant() -cne $pins.index_sha256) { throw 'Mission document coverage changed.' }
    if ((Get-FileHash (Join-Path $PSScriptRoot 'ANG_R2_DOCUMENT_COVERAGE.json')).Hash.ToLowerInvariant() -cne $pins.coverage_sha256) { throw 'Covered document identities changed.' }
    foreach ($document in @(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_DOCUMENT_COVERAGE.json') -Raw -Encoding utf8 | ConvertFrom-Json)) {
        if ((Get-FileHash $document.path).Hash.ToLowerInvariant() -cne $document.sha256) { throw "Covered document changed: $($document.path)" }
    }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if ((Get-FileHash $binary).Hash.ToLowerInvariant() -cne $pins.binary_sha256 -or (Get-FileHash $profile).Hash.ToLowerInvariant() -cne $pins.profile_sha256) { throw 'Knowledge executable/profile changed.' }
    $out = Join-Path $root ('target/ang-r2/format-verification-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $paths = @(@(& git diff HEAD --name-only -- src) + @(& git ls-files --others --exclude-standard src) | Sort-Object -Unique | Where-Object {$_ -like '*.rs' -and (Test-Path -LiteralPath $_)})
    ConvertTo-Json -InputObject @($paths) -Depth 10 | Set-Content -LiteralPath (Join-Path $out 'active-rust-files.json') -Encoding utf8NoBOM
    function Source-Pins {
        @($paths | ForEach-Object { [ordered]@{path=$_;sha256=(Get-FileHash $_).Hash.ToLowerInvariant()} })
    }
    $before = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest.json'),$before,$utf8)
    if ($ResumeVerifiedFrom) {
        $prior = (Resolve-Path -LiteralPath $ResumeVerifiedFrom).Path
        $allowed = Join-Path $root 'target/ang-r2/format-verification-'
        if (-not $prior.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Split-Path $prior -Leaf) -notmatch '^format-verification-\d{8}-\d{6}-\d{3}$') { throw 'Invalid prior format capture.' }
        if ([IO.File]::ReadAllText((Join-Path $prior 'source-manifest.json'),$utf8) -cne $before -or [IO.File]::ReadAllText((Join-Path $prior 'source-manifest-after.json'),$utf8) -cne $before) { throw 'Prior Rust source pins differ; cannot reuse verification.' }
        foreach ($name in @('scoped-format','normalized-retirement','normalized-registration-and-relocation','scoped-clippy')) {
            $receiptPath = Join-Path $prior ($name+'-receipt.json')
            $receipt = Get-Content -LiteralPath $receiptPath -Raw -Encoding utf8 | ConvertFrom-Json
            if ($receipt.exit_code -ne 0 -or $receipt.head -cne (& git rev-parse HEAD)) { throw 'Prior verification failed or baseline changed.' }
            foreach ($channel in @('stdout','stderr')) {
                if ((Get-FileHash (Join-Path $prior ($name+'-'+$channel+'.txt'))).Hash.ToLowerInvariant() -cne $receipt.($channel+'_sha256')) { throw 'Prior log hash mismatch.' }
            }
            if (Select-String -LiteralPath (Join-Path $prior ($name+'-stderr.txt')) -Pattern '^warning:' -Quiet) { throw 'Prior compiler warnings.' }
            if ($receipt.expected_test_count -and -not (Select-String -LiteralPath (Join-Path $prior ($name+'-stdout.txt')) -SimpleMatch -Pattern "test result: ok. $($receipt.expected_test_count) passed; 0 failed;" -Quiet)) { throw 'Prior test count mismatch.' }
            [ordered]@{category='Verified prior exact-source result reused; not rerun';receipt=$receiptPath;sha256=(Get-FileHash $receiptPath).Hash.ToLowerInvariant()} |
                ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out ($name+'-reused.json')) -Encoding utf8NoBOM
        }
    }
    $runs = @(
        @{name='scoped-format';exe='rustfmt';arguments=(@('--edition','2024','--config','skip_children=true','--check')+$paths)},
        @{name='normalized-retirement';exe='cargo';expected=1;arguments=@('test','--lib','--all-features','--quiet','issue_162_deleted_owner_legacy_membership_is_retired','--','--test-threads=1','--nocapture')},
        @{name='normalized-registration-and-relocation';exe='cargo';expected=18;arguments=@('test','--lib','--all-features','--quiet','issue_162_resolver_registration_','--','--test-threads=1','--nocapture')},
        @{name='scoped-clippy';exe='cargo';arguments=@('clippy','--lib','--bin','clean-ctx','--tests','--all-features','--','-D','warnings')},
        @{name='utf8';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-utf8.ps1')},
        @{name='file-sizes';exe='pwsh';arguments=@('-NoProfile','-File','scripts/check-file-sizes.ps1')}
    )
    foreach ($run in $runs) {
        if ($ResumeVerifiedFrom -and $run.name -notin @('utf8','file-sizes')) { continue }
        $argumentsForRun = $run.arguments
        $stdout = Join-Path $out ($run.name+'-stdout.txt')
        $stderr = Join-Path $out ($run.name+'-stderr.txt')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        & $run.exe @argumentsForRun 1> $stdout 2> $stderr
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Disposition-03 focused operator evidence; not full CI, field or delivery'
            command=($run.exe+' '+($argumentsForRun -join ' '));exit_code=$code;expected_test_count=$run.expected
            elapsed_seconds=$watch.Elapsed.TotalSeconds;head=(& git rev-parse HEAD)
            source_manifest_sha256=(Get-FileHash (Join-Path $out 'source-manifest.json')).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant();stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $out ($run.name+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve $out" }
        if ($run.exe -eq 'cargo' -and (Select-String -LiteralPath $stderr -Pattern '^warning:' -Quiet)) { throw "Compiler warnings; preserve $out" }
        if ($run.expected -and -not (Select-String -LiteralPath $stdout -SimpleMatch -Pattern "test result: ok. $($run.expected) passed; 0 failed;" -Quiet)) { throw "Unexpected selected tests; preserve $out" }
    }
    $after = ConvertTo-Json -InputObject @(Source-Pins) -Depth 10
    [IO.File]::WriteAllText((Join-Path $out 'source-manifest-after.json'),$after,$utf8)
    if ($before -cne $after) { throw 'Sources changed during verification.' }
    $requests = Join-Path $out 'admission-request.json'
    [IO.File]::WriteAllText($requests,(ConvertTo-Json -InputObject @(@{workspaceRoot=$root;mode='context';id='DOC-270'}) -Depth 10),$utf8)
    & (Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1') -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $requests -OutputRoot (Join-Path $out 'admission') -LaunchDirectory (Join-Path $root '.clean-ctx/pcg-documents/authoring')
    $admission = Get-Content -LiteralPath (Join-Path $out 'admission/2.payload.json') -Raw -Encoding utf8 | ConvertFrom-Json
    if ($admission.result_state -cne 'ready' -or -not $admission.catalog_current -or -not $admission.complete -or @($admission.diagnostics).Count) {
        & $binary pcg documents describe --profile $profile 1> (Join-Path $out 'blocked-descriptor.json') 2> (Join-Path $out 'blocked-descriptor-stderr.txt')
        throw "Actual document admission blocked; preserve $out. No protected grant submitted."
    }
    Write-Host "Return focused, guard and actual admission receipts from $out. No delivery or CI success implied."
} finally { Pop-Location }
