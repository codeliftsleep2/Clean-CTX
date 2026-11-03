# Agent MaxHeadRoom: affected index semantics and mandatory architecture gate.
param([switch]$ResumeAtLint)
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
 $logs=Join-Path $repoPath ('target/af-perf-002/index174-focused-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
 New-Item -ItemType Directory -Path $logs | Out-Null
 Write-Output "Focused logs: $logs"
 & rustfmt --edition 2024 --config skip_children=true src/tests/architecture/semantic_work_static.rs src/tests/architecture/index_membership_static_af_perf_002.rs *> (Join-Path $logs 'format.log')
 if($LASTEXITCODE -ne 0){throw 'Format failed'}
 & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
 if($LASTEXITCODE -ne 0){throw 'Size failed'}
 if(-not $ResumeAtLint){
  & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1 *> (Join-Path $logs 'fitness.log')
  if($LASTEXITCODE -ne 0){Get-Content (Join-Path $logs 'fitness.log') -Tail 35;throw "Fitness failed: $logs"}
  foreach($filter in @('workspace::index::','af_perf_002_populated_reconciliation_and_survivors')){
   $log=Join-Path $logs (($filter -replace ':','_')+'.log')
   & cargo test --lib --all-features --quiet $filter -- --test-threads=1 *> $log
   if($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath $log -Pattern 'test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored;' -Quiet)){throw "Index preservation failed: $log"}
  }
 }
 & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $logs 'clippy.log')
 if($LASTEXITCODE -ne 0){throw "Clippy failed: $logs"}
 & cargo fmt --all -- --check *> (Join-Path $logs 'fmt.log')
 if($LASTEXITCODE -ne 0){throw "fmt failed: $logs"}
 & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1 *> (Join-Path $logs 'encoding.log')
 if($LASTEXITCODE -ne 0){throw "Encoding failed: $logs"}
 Write-Output "Focused checks completed: $logs. Mutation sensitivity remains separate."
} finally {Pop-Location}
