# Mission 24 package-scoped lint capture; no test execution or repository-wide gate.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
Push-Location $root
try {
    $env:OPS_R1_STAGE = 'green'
    $env:OPS_R1_GREEN_SOURCE = 'hardening-24e'
    $adapter = 'verification/ops-r1/proposals/shared-acquisition/execution/capture.py'
    $before = @(& python $adapter --verify-sources-only)
    if ($LASTEXITCODE -ne 0 -or $before.Count -ne 1) { throw 'Source preflight failed.' }
    $out = 'target/ops-r1/mission-24-clippy-final'
    if (Test-Path -LiteralPath $out) { throw 'Fresh lint output required.' }
    New-Item -ItemType Directory -Path $out | Out-Null
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & cargo clippy --lib --tests --all-features -- -D warnings 1> "$out/stdout.txt" 2> "$out/stderr.txt"
    $code = $LASTEXITCODE
    $watch.Stop()
    $after = @(& python $adapter --verify-sources-only)
    $identityCode = $LASTEXITCODE
    [ordered]@{
        command = 'cargo clippy --lib --tests --all-features -- -D warnings'
        exit_code = $code
        elapsed_seconds = $watch.Elapsed.TotalSeconds
        source_tree_sha256 = $before[0]
        post_source_identity_verified = ($identityCode -eq 0 -and $after.Count -eq 1 -and $before[0] -ceq $after[0])
        adapter_sha256 = (Get-FileHash -LiteralPath $adapter).Hash.ToLowerInvariant()
        stdout_sha256 = (Get-FileHash -LiteralPath "$out/stdout.txt").Hash.ToLowerInvariant()
        stderr_sha256 = (Get-FileHash -LiteralPath "$out/stderr.txt").Hash.ToLowerInvariant()
        tests_executed = $false
    } | ConvertTo-Json | Set-Content -LiteralPath "$out/receipt.json" -Encoding utf8NoBOM
    if ($code -ne 0 -or $identityCode -ne 0 -or $before[0] -cne $after[0]) { throw "Lint failed; preserve $out" }
    Write-Output "$out/receipt.json"
} finally { Pop-Location }
