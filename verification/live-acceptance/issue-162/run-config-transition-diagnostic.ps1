# Agent MaxHeadRoom: no build, no full lifecycle repetition; existing 120s client.
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
try {
    $binaryPath = Join-Path $PWD 'target/debug/clean-ctx.exe'
    $largeReceipt = Get-Content -LiteralPath 'target/issue-162/load-children75-config-transition-timeout/receipt.json' -Raw | ConvertFrom-Json
    $boundedReceipt = Get-Content -LiteralPath 'target/issue-162/load-children75-modules-binary-receipt.json' -Raw | ConvertFrom-Json
    if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash -ne $boundedReceipt.binary.sha256) { throw 'Recorded generation-75 binary hash mismatch' }
    $largeReport = $largeReceipt.files[1].path
    $boundedReport = $boundedReceipt.reports[-1].path
    & node --check verification/live-acceptance/issue-162/config-transition-diagnostic.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Diagnostic JavaScript syntax failed' }
    & node verification/live-acceptance/issue-162/config-transition-diagnostic.mjs $binaryPath $largeReport $boundedReport
    if ($LASTEXITCODE -ne 0) { throw 'Diagnostic recorded a timeout/failure; inspect its report, not a semantic acceptance result' }
} finally { Pop-Location }
