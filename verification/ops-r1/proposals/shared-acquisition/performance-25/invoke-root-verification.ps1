# Operator-only scoped root verification; no CI gate, runtime server or publication.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../../..'))
$execution = Join-Path $PSScriptRoot '../execution'
$out = Join-Path $root ('target/ops-r1/performance-root-verification-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out -ErrorAction Stop | Out-Null
$savedTarget = $env:CARGO_TARGET_DIR
$savedVersion = $env:OPS_R1_GREEN_SOURCE
$savedStage = $env:OPS_R1_STAGE
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
$env:OPS_R1_GREEN_SOURCE = 'performance-25-root'
$env:OPS_R1_STAGE = 'green'
Push-Location $root
try {
    $files = @(
        'src/ir/typescript_resolution/host.rs',
        'src/ir/typescript_resolution/acquisition_inputs.rs',
        'src/ir/typescript_resolution/shared_acquisition.rs',
        'src/tests/ir/ops_r1_shared_acquisition_controls.rs',
        'src/tests/ir/ops_r1_shared_acquisition_cost.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @files 2>&1 | Tee-Object -FilePath (Join-Path $out 'format.txt')
    if ($LASTEXITCODE -ne 0) { throw "Focused formatting failed; preserve $out" }
    & python (Join-Path $PSScriptRoot 'prepare-root-verification.py')
    if ($LASTEXITCODE -ne 0) { throw "Root identity preparation failed; preserve $out" }
    & pwsh -NoProfile -File (Join-Path $execution 'invoke-build-capture.ps1') -Stage green -OutputDirectory target/ops-r1/performance-25-root-build-01
    if ($LASTEXITCODE -ne 0) { throw "Root compilation failed; preserve $out" }
    $build = Join-Path $root 'target/ops-r1/performance-25-root-build-01/build-receipt.json'
    $receipts = @()
    foreach ($case in @('cost-constraints', 'directory-work', 'fitness', 'typescript_persistence')) {
        Write-Host "START root $case"
        $result = @(& python (Join-Path $execution 'capture.py') --build-receipt $build --case $case --wall-seconds 60 --memory-mib 512 --stop-wait 5)
        $code = $LASTEXITCODE
        $result | Set-Content -LiteralPath (Join-Path $out "$case-adapter.txt") -Encoding utf8NoBOM
        if ($result.Count -ne 1) { throw "Unexpected capture output; preserve $out" }
        $path = Join-Path $result[0] 'receipt.json'
        $receipt = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
        $receipts += [ordered]@{case=$case; receipt=$path; sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant(); status=$receipt.status}
        $receipts | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $out 'receipts.json') -Encoding utf8NoBOM
        if ($code -ne 0 -or $receipt.status -cne 'TRACKED_TEST_COMPLETED' -or -not $receipt.post_source_and_binary_identity_verified) {
            throw "Root regression failed: $case; preserve $out"
        }
    }
    & cargo clippy --lib --tests --all-features -- -D warnings 2>&1 | Tee-Object -FilePath (Join-Path $out 'clippy.txt')
    $clippyCode = $LASTEXITCODE
    [ordered]@{command='cargo clippy --lib --tests --all-features -- -D warnings'; exit_code=$clippyCode} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out 'clippy-receipt.json') -Encoding utf8NoBOM
    if ($clippyCode -ne 0) { throw "Scoped Clippy failed; preserve $out" }
    Write-Output "Return root focused receipts and scoped Clippy: $out"
} finally {
    Pop-Location
    $env:CARGO_TARGET_DIR = $savedTarget
    $env:OPS_R1_GREEN_SOURCE = $savedVersion
    $env:OPS_R1_STAGE = $savedStage
}
