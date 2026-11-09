# Agent MaxHeadRoom: restore real original discarded text conversion in isolated production only.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_SQLITE_TEXT_ORACLE
$before=Join-Path $repoPath 'target/af-perf-002/sqlite-text-before-20261108-173446'
$r=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
if($r.semanticPasses -ne 2 -or $r.publicCases -ne 12 -or $r.budgetExit -eq 0){throw 'Genuine original RED required'}
Push-Location $repoPath
try {
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/sqlite_text_integrated_af_perf_002.rs
    if($LASTEXITCODE -ne 0){throw 'Law test formatting failed'}
    $evidence=Join-Path $repoPath ('target/af-perf-002/sqlite-text-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    $workspace=Join-Path $evidence 'workspace'
    New-Item -ItemType Directory -Path $evidence|Out-Null
    & git clone --local --no-hardlinks $repoPath $workspace *> (Join-Path $evidence 'clone.log')
    if($LASTEXITCODE -ne 0){throw 'Isolated clone failed'}
    $paths=@(git ls-files --cached --others --exclude-standard)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $prefix=[IO.Path]::GetFullPath($workspace)+[IO.Path]::DirectorySeparatorChar
    foreach($path in $paths){
        $destination=[IO.Path]::GetFullPath((Join-Path $workspace $path))
        if(-not $destination.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Overlay escaped workspace'}
        if(Test-Path -LiteralPath $path -PathType Leaf){New-Item -ItemType Directory -Force -Path (Split-Path $destination)|Out-Null;Copy-Item -LiteralPath $path -Destination $destination}
        elseif(Test-Path -LiteralPath $destination -PathType Leaf){Remove-Item -LiteralPath $destination}
    }
    $files=@($paths|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
} finally {Pop-Location}
$relative='src/mcp/sqlite_store.rs'
$source=Join-Path $workspace $relative
$restored=[IO.File]::ReadAllBytes($source)
$activeHash=(Get-FileHash (Join-Path $repoPath $relative)).Hash
$env:AF002_SQLITE_TEXT_ORACLE=Join-Path $before 'public-oracle.json'
function Gate([string]$Name,[string]$Filter,[string]$Red='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}}
    elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Unexpected warning: $log"}
    if($Filter -in @('af002_sqlite_text_public_budget','af002_sqlite_text_public_identity')){
        if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_SQLITE_TEXT_PUBLIC ')}).Count -ne 12){throw "Incomplete public cases: $log"}
    }
}
Push-Location $workspace
try {
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $evidence 'clippy.log')
    if($LASTEXITCODE -ne 0){throw 'Warning-fatal baseline Clippy failed'}
    Gate 'baseline-static' 'afperf_sqlite_text_static'
    Gate 'baseline-law' 'afperf_sqlite_text_validation_law'
    # Restore the complete real original method; keep module registration for the law.
    # Only the original materialization/observation path is restored.
    $old=[IO.File]::ReadAllText((Join-Path $before 'production-sqlite.rs'))
    $current=[IO.File]::ReadAllText($source)
    $start=$old.IndexOf('    fn load_latest(');$end=$old.IndexOf('    fn has_context(',$start)
    $newStart=$current.IndexOf('    fn load_latest(');$newEnd=$current.IndexOf('    fn has_context(',$newStart)
    if($start -lt 0 -or $end -le $start -or $newStart -lt 0 -or $newEnd -le $newStart){throw 'Original method boundaries missing'}
    $mutant=$current.Substring(0,$newStart)+$old.Substring($start,$end-$start)+$current.Substring($newEnd)
    [IO.File]::WriteAllText($source,$mutant,[Text.UTF8Encoding]::new($false))
    Gate 'map-budget' 'af002_sqlite_text_public_budget' 'SQLite discarded owned-text materialization budget'
    Gate 'map-static' 'afperf_sqlite_text_static' 'SQLite validated-text ownership static protection'
    Gate 'original-semantics' 'af002_sqlite_text_frozen_semantics'
    Gate 'map-public' 'af002_sqlite_text_public_identity'
} finally {[IO.File]::WriteAllBytes($source,$restored);Pop-Location;$env:AF002_SQLITE_TEXT_ORACLE=$oldOracle}
if((Get-FileHash $source).Hash -ne $activeHash){throw 'Exact restoration failed'}
$env:AF002_SQLITE_TEXT_ORACLE=Join-Path $before 'public-oracle.json'
Push-Location $workspace
try {
    Gate 'restored-budget' 'af002_sqlite_text_public_budget'
    Gate 'restored-static' 'afperf_sqlite_text_static'
    Gate 'restored-law' 'afperf_sqlite_text_validation_law'
    Gate 'restored-equivalence' 'af002_sqlite_text_frozen_semantics'
    Gate 'restored-public' 'af002_sqlite_text_public_identity'
} finally {Pop-Location;$env:AF002_SQLITE_TEXT_ORACLE=$oldOracle}
foreach($file in $files){
    if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Active file changed: $($file.path)"}
    if((Get-FileHash (Join-Path $workspace $file.path)).Hash -ne $file.sha256){throw "Isolated restoration drift: $($file.path)"}
}
@{agent='Agent MaxHeadRoom';intendedFailures=2;preservedSemanticPublicPasses=2;restoredPasses=5;baselinePasses=2;
    activeSourceUnchanged=$true;productionSha256=$activeHash;files=$files}|
    ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED, semantic/public GREEN, exact restored five GREEN: $evidence"
