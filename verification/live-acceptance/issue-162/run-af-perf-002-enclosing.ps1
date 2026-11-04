# Agent MaxHeadRoom: Phase 1 observations only, no optimization acceptance claim.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
 $logs=Join-Path $repoPath ('target/af-perf-002/enclosing-packet3-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
 New-Item -ItemType Directory -Path $logs | Out-Null
 Write-Output "Baseline logs: $logs"
 $paths=@('src/lib.rs','src/tests/architecture/work_observations_af_perf_002.rs','src/workspace/index/declarations.rs','src/tests/workspace/index_enclosing_af_perf_002.rs')
 & rustfmt --edition 2024 --config skip_children=true @paths *> (Join-Path $logs 'format.log')
 if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
 & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
 if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
 foreach($filter in @('af002_enclosing_','af174_')){
  $log=Join-Path $logs ($filter+'.log')
  & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
  $code=$LASTEXITCODE
  Get-Content -LiteralPath $log -Tail 12
  if($code -ne 0 -or -not (Select-String -LiteralPath $log -Pattern 'test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored;' -Quiet)){throw "Baseline failed: $log"}
 }
 $paths += @(git ls-files --cached --others --exclude-standard -- src/workspace/index src/workspace/index.rs src/tests/architecture/index_membership_static_af_perf_002.rs src/tests/workspace/index_membership*)
 $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
 [pscustomobject]@{agent='Agent MaxHeadRoom';category='tracked Phase 1 observations; not repair acceptance';files=$hashes;scope='enclosing add_edges declaration comparisons and no-op reconciliation; prior membership budget preservation; synchronous thread-local only'} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
} finally {Pop-Location}
