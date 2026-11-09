# Agent MaxHeadRoom: real original source-copy restoration in isolated production.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding;[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$oldRoot=$env:AF002_DIFF_SOURCE_ROOT;$oldOracle=$env:AF002_DIFF_SOURCE_ORACLE
$before=Join-Path $repoPath 'target/af-perf-002/diff-source-before-20261109-103157'
$prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
if($prior.semanticPasses -ne 2 -or $prior.publicCases -ne 225 -or $prior.budgetExit -eq 0){throw 'Genuine original RED missing'}
Push-Location $repoPath
try {
    $evidence=Join-Path $repoPath ('target/af-perf-002/diff-source-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$env:AF002_DIFF_SOURCE_ROOT=Join-Path $before 'fixtures'
$env:AF002_DIFF_SOURCE_ORACLE=Join-Path $before 'public-oracle.json'
function Gate([string]$Name,[string]$Filter,[string]$Red='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}}
    elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Warning invalidates acceptance: $log"}
    if($Filter -in @('af002_diff_source_copy_budget','af002_diff_source_frozen_semantics')){
        if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_DIFF_SOURCE_PUBLIC ')}).Count -ne 225){throw 'Incomplete frozen public identities'}
    }
}
Push-Location $workspace
try {
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $evidence 'clippy.log')
    if($LASTEXITCODE -ne 0){throw 'Baseline Clippy failed'}
    Gate 'baseline-static' 'afperf_diff_source_static'
    $old=[IO.File]::ReadAllText((Join-Path $before ('frozen/'+$relative)))
    $copy=[regex]::Match($old,'let source = s\.as_str\(\)\.to_string\(\);').Value
    if(-not $copy){throw 'Real original source conversion missing'}
    $current=[IO.File]::ReadAllText($source)
    $start=$current.IndexOf('        Ok(s) => {',$current.IndexOf('pub(crate) fn handle_diff_code_context('))
    $end=$current.IndexOf('        Err(e) => {',$start)
    if($start -lt 0 -or $end -le $start){throw 'Actual handler ownership arm missing'}
    $arm=@"
        Ok(s) => {
            #[cfg(all(test, feature = "typescript"))]
            {
                // Keep replaced test machinery type-checked; never invoke it.
                let _current_owner: fn(std::sync::Arc<String>) -> ownership_design::Source = ownership_design::current;
                $copy
                ownership_design::repair::owned_copy(&source);
                ownership_design::owned(source)
            }
            #[cfg(not(all(test, feature = "typescript")))]
            {
                s.as_str().to_string()
            }
        }
"@
    [IO.File]::WriteAllText($source,$current.Substring(0,$start)+$arm+[Environment]::NewLine+$current.Substring($end),[Text.UTF8Encoding]::new($false))
    Gate 'original-budget' 'af002_diff_source_copy_budget' 'Registered diff full-source-copy budget'
    Gate 'original-static' 'afperf_diff_source_static' 'Diff acquired-source ownership static protection'
    Gate 'original-public-cache-lifetime' 'af002_diff_source_frozen_semantics'
    Gate 'original-persisted-restart' 'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore'
} finally {
    [IO.File]::WriteAllBytes($source,$restored);Pop-Location
    $env:AF002_DIFF_SOURCE_ROOT=$oldRoot;$env:AF002_DIFF_SOURCE_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding
}
$env:AF002_DIFF_SOURCE_ROOT=Join-Path $before 'fixtures'
$env:AF002_DIFF_SOURCE_ORACLE=Join-Path $before 'public-oracle.json'
Push-Location $workspace
try {
    [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
    Gate 'restored-budget' 'af002_diff_source_copy_budget'
    Gate 'restored-public-cache-lifetime' 'af002_diff_source_frozen_semantics'
    Gate 'restored-static' 'afperf_diff_source_static'
    Gate 'restored-persisted-restart' 'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore'
    Gate 'restored-lifetime-design' 'af002_diff_ownership_errors_and_snapshot_lifetime'
} finally {Pop-Location;$env:AF002_DIFF_SOURCE_ROOT=$oldRoot;$env:AF002_DIFF_SOURCE_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding}
foreach($file in $files){
    if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Active file drift: $($file.path)"}
    if((Get-FileHash (Join-Path $workspace $file.path)).Hash -ne $file.sha256){throw "Byte-exact isolated restoration drift: $($file.path)"}
}
@{agent='Agent MaxHeadRoom';baselinePasses=1;intendedFailures=2;preservedSemanticPasses=2;restoredPasses=5;files=$files;activeSourceUnchanged=$true}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED; frozen semantic/persistence GREEN; five exact restored GREEN. Evidence: $evidence"
