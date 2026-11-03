# Agent MaxHeadRoom: new delegated-path tracked RED/GREEN gate, not broad acceptance.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logDirectory=Join-Path $repoPath ('target/issue-162/delegated-provider-replay-' + $Phase.ToLowerInvariant() + '-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logDirectory | Out-Null
    Write-Output "Full logs: $logDirectory"
    $paths=@('src/angular_meta/mod.rs','src/angular_meta/delegated_work_metrics.rs','src/angular_meta/admission_work_metrics.rs','src/angular_meta/bootstrap_references.rs','src/angular_meta/application_config_providers.rs','src/angular_meta/config_composition.rs','src/angular_meta/provider_imported.rs','src/angular_meta/provider_pair_authority.rs','src/angular_meta/provider_position.rs','src/ir/typescript_resolution/relation_evidence.rs','src/tests/architecture/delegated_provider_work_af_perf_001.rs','src/tests/architecture/semantic_work_static.rs','src/tests/mcp/angular_admission_lifecycle_af_perf_001.rs')
    & rustfmt --edition 2024 --config skip_children=true @paths *> (Join-Path $logDirectory 'format.log')
    if ($LASTEXITCODE -ne 0) { throw "Formatting failed: $logDirectory" }
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logDirectory 'sizes.log')
    if ($LASTEXITCODE -ne 0) { throw "Size gate failed: $logDirectory" }
    foreach ($path in $paths) {
        $destination=Join-Path (Join-Path $logDirectory 'source') $path
        New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
        Copy-Item -LiteralPath $path -Destination $destination
    }
    & cargo test --lib --all-features --quiet af_delegated_001_ -- --test-threads=1 --nocapture *> (Join-Path $logDirectory 'tests.log')
    $testExit=$LASTEXITCODE
    Get-Content -LiteralPath (Join-Path $logDirectory 'tests.log') -Tail 65
    $hashes=@($paths | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash} })
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked delegated production-admission tests';testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logDirectory 'receipt.json') -Encoding utf8NoBOM
    if ($Phase -eq 'Before') { Write-Output 'Expected RED: complete delegated search/construction count exceeds one for two candidates. Compilation/fixture failures do not qualify.' }
    if ($Phase -eq 'After') {
        if ($testExit -ne 0) { throw "Delegated correction not GREEN: $logDirectory" }
        if (-not (Select-String -LiteralPath (Join-Path $logDirectory 'tests.log') -Pattern 'test result: ok\. 6 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Missing required tests: $logDirectory" }
    }
    exit $testExit
} finally { Pop-Location }
