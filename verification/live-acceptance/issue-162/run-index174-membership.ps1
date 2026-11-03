# Agent MaxHeadRoom: four tracked intended-budget regressions; not characterization PASS.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
 $logs=Join-Path $repoPath ('target/af-perf-002/index174-'+$Phase.ToLowerInvariant()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
 New-Item -ItemType Directory -Path $logs | Out-Null
 Write-Output "#174 logs: $logs"
 $paths=@('src/lib.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/tests/workspace/index_membership_budget_af_perf_002.rs','src/tests/workspace/index_membership_lifecycle_af_perf_002.rs','src/workspace/index.rs','src/workspace/index/membership.rs','src/workspace/index/edges.rs','src/workspace/index/remove.rs','src/workspace/index/ngrx_refinement.rs')
 $formatPaths=@($paths | Where-Object {Test-Path -LiteralPath $_})
 & rustfmt --edition 2024 --config skip_children=true @formatPaths *> (Join-Path $logs 'format.log')
 if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
 & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
 if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
 foreach($path in $paths){
  $destination=Join-Path (Join-Path $logs 'source') $path
  New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
  Copy-Item -LiteralPath $path -Destination $destination
 }
 & cargo test --lib --all-features --quiet af174_ -- --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
 $code=$LASTEXITCODE
 Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 65
 $hashes=@($paths | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
 [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;category='tracked linear-budget RED/GREEN';exitCode=$code;files=$hashes} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
 if($Phase -eq 'Before'){Write-Output 'Expected three quadratic-budget REDs, one semantic control GREEN. Compilation/fixture failures do not qualify.'}
 if($Phase -eq 'After' -and ($code -ne 0 -or -not (Select-String -LiteralPath (Join-Path $logs 'tests.log') -Pattern 'test result: ok\. 9 passed; 0 failed; 0 ignored;' -Quiet))){throw "#174 not GREEN: $logs"}
 exit $code
} finally {Pop-Location}
