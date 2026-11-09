# Agent MaxHeadRoom: real original early-capture restoration in isolated production.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$oldRoot=$env:AF002_DELTA_PREVIOUS_ROOT;$oldOracle=$env:AF002_DELTA_PREVIOUS_ORACLE
$before=Join-Path $repoPath 'target/af-perf-002/delta-previous-before-20261109-134928'
$prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
if($prior.semanticPasses -ne 5 -or $prior.publicCases -ne 380 -or $prior.budgetExit -eq 0){throw 'Genuine original RED missing'}
Push-Location $repoPath
try {
    $evidence=Join-Path $repoPath ('target/af-perf-002/delta-previous-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    $workspace=Join-Path $evidence 'workspace'
    New-Item -ItemType Directory -Path $evidence|Out-Null
    & git clone --local --no-hardlinks $repoPath $workspace *> (Join-Path $evidence 'clone.log')
    if($LASTEXITCODE -ne 0){throw 'Isolated clone failed'}
    $paths=@(git ls-files --cached --others --exclude-standard)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $prefix=[IO.Path]::GetFullPath($workspace)+[IO.Path]::DirectorySeparatorChar
    foreach($p in $paths){
        $dst=[IO.Path]::GetFullPath((Join-Path $workspace $p))
        if(-not $dst.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Overlay escaped workspace'}
        if(Test-Path -LiteralPath $p -PathType Leaf){New-Item -ItemType Directory -Force -Path (Split-Path $dst)|Out-Null;Copy-Item -LiteralPath $p -Destination $dst}
        elseif(Test-Path -LiteralPath $dst -PathType Leaf){Remove-Item -LiteralPath $dst}
    }
    $files=@($paths|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
} finally {Pop-Location}
$relative='src/mcp/tool_handlers/core/delta.rs'
$source=Join-Path $workspace $relative;$restored=[IO.File]::ReadAllBytes($source)
$env:AF002_DELTA_PREVIOUS_ROOT=Join-Path $before 'fixtures'
$env:AF002_DELTA_PREVIOUS_ORACLE=Join-Path $before 'public-oracle.json'
function Gate([string]$Name,[string]$Filter,[string]$Red='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}}
    elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Warning invalidates acceptance: $log"}
    if($Filter -in @('af002_delta_previous_unused_budget','af002_delta_previous_frozen_semantics')){
        if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_DELTA_PREVIOUS_FROZEN ')}).Count -ne 380){throw 'Incomplete frozen public identities'}
    }
}
Push-Location $workspace
try {
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $evidence 'clippy.log')
    if($LASTEXITCODE -ne 0){throw 'Baseline Clippy failed'}
    Gate 'baseline-static' 'afperf_delta_previous_static'
    $old=[IO.File]::ReadAllText((Join-Path $before ('frozen/'+$relative))).Replace("`r`n","`n")
    $startOriginal=$old.IndexOf('    let previous_authority = (capture_before')
    $endOriginal=$old.IndexOf('    if prev_version > 0',$startOriginal)
    if($startOriginal -lt 0 -or $endOriginal -le $startOriginal){throw 'Real original early capture missing'}
    $early=$old.Substring($startOriginal,$endOriginal-$startOriginal).Replace('capture_before && ','')
    $current=[IO.File]::ReadAllText($source).Replace("`r`n","`n")
    $start=$current.IndexOf('    #[cfg(all(test, feature = "typescript"))]'+[char]10+'    let previous_authority =')
    if($start -lt 0){
        $normalized=$current.Replace("`r`n","`n")
        $start=$normalized.IndexOf('    #[cfg(all(test, feature = "typescript"))]'+[char]10+'    let previous_authority =')
        $current=$normalized
    }
    $end=$current.IndexOf('    if prev_version > 0',$start)
    if($start -lt 0 -or $end -le $start){throw 'Current historical reference arm missing'}
    $keep='    #[cfg(all(test, feature = "typescript"))]'+[Environment]::NewLine+'    let _historical_reference: fn() -> bool = delta_design::historical_early;'+[Environment]::NewLine
    $mutant=$current.Substring(0,$start)+$keep+$early+$current.Substring($end)
    $late=$mutant.IndexOf('    let capture_previous = ||')
    $release=$mutant.IndexOf('    #[cfg(all(test, feature = "typescript"))]'+[char]10+'    delta_design::changed',$late)
    if($late -lt 0 -or $release -le $late){throw 'Actual delayed capture boundary missing'}
    $mutant=$mutant.Substring(0,$late)+$mutant.Substring($release)
    [IO.File]::WriteAllText($source,$mutant,[Text.UTF8Encoding]::new($false))
    Gate 'original-budget' 'af002_delta_previous_unused_budget' 'Registered delta unused previous-authority budget'
    Gate 'original-static' 'afperf_delta_previous_static' 'Delta capture/guard/read/publication static protection'
    Gate 'original-public-cache-lifetime' 'af002_delta_previous_frozen_semantics'
    Gate 'original-persisted-restart' 'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore'
    Gate 'original-epoch' 'matching_canonical_epoch_accepts_delta_and_advances_chain'
    Gate 'original-superseded' 'delta_derivation_rejects_a_superseded_baseline_epoch'
    Gate 'original-diff-protection' 'afperf_diff_source_static'
} finally {
    [IO.File]::WriteAllBytes($source,$restored);Pop-Location
    $env:AF002_DELTA_PREVIOUS_ROOT=$oldRoot;$env:AF002_DELTA_PREVIOUS_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding
}
$env:AF002_DELTA_PREVIOUS_ROOT=Join-Path $before 'fixtures'
$env:AF002_DELTA_PREVIOUS_ORACLE=Join-Path $before 'public-oracle.json'
Push-Location $workspace
try {
    [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
    Gate 'restored-budget' 'af002_delta_previous_unused_budget'
    Gate 'restored-public-cache-lifetime' 'af002_delta_previous_frozen_semantics'
    Gate 'restored-static' 'afperf_delta_previous_static'
    Gate 'restored-persisted-restart' 'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore'
    Gate 'restored-epoch' 'matching_canonical_epoch_accepts_delta_and_advances_chain'
    Gate 'restored-superseded' 'delta_derivation_rejects_a_superseded_baseline_epoch'
    Gate 'restored-diff-protection' 'afperf_diff_source_static'
    Gate 'restored-lock-order' 'af002_delta_previous_lock_boundary'
} finally {Pop-Location;$env:AF002_DELTA_PREVIOUS_ROOT=$oldRoot;$env:AF002_DELTA_PREVIOUS_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding}
foreach($file in $files){
    if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Active file drift: $($file.path)"}
    if((Get-FileHash (Join-Path $workspace $file.path)).Hash -ne $file.sha256){throw "Byte-exact isolated restoration drift: $($file.path)"}
}
@{agent='Agent MaxHeadRoom';baselinePasses=1;intendedFailures=2;preservedSemanticPasses=5;restoredPasses=8;files=$files;activeSourceUnchanged=$true}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED; frozen semantic/persistence GREEN; eight exact restored GREEN. Evidence: $evidence"
