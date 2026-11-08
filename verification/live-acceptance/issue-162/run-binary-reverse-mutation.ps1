# Agent MaxHeadRoom: restore real original reverse map in isolated production only.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_BINARY_PUBLIC_ORACLE
$before=Join-Path $repoPath 'target/af-perf-002/binary-reverse-before-20261108-160709'
$r=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
if($r.semanticPasses -ne 3 -or $r.publicCases -ne 40 -or $r.budgetExit -eq 0){throw 'Genuine original RED required'}
Push-Location $repoPath
try {
    $evidence=Join-Path $repoPath ('target/af-perf-002/binary-reverse-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative='src/ir/binary_wire.rs'
$source=Join-Path $workspace $relative
$restored=[IO.File]::ReadAllBytes($source)
$activeHash=(Get-FileHash (Join-Path $repoPath $relative)).Hash
$env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $workspace 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
function Gate([string]$Name,[string]$Filter,[string]$Red='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}}
    elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Unexpected warning: $log"}
    if($Filter -in @('af002_binary_reverse_public_budget','af002_binary_public_identity')){
        if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_BINARY_PUBLIC ')}).Count -ne 40){throw "Incomplete public cases: $log"}
    }
}
Push-Location $workspace
try {
    Gate 'baseline-static' 'afperf_binary_reverse_static'
    # Restore the complete real original encoder bytes; its only algorithmic
    # difference from the accepted encoder is redundant map construction and lookup.
    [IO.File]::WriteAllBytes($source,[IO.File]::ReadAllBytes((Join-Path $before 'production-encoder.rs')))
    Gate 'map-budget' 'af002_binary_reverse_public_budget' 'Binary encoder redundant reverse-map budget'
    Gate 'map-static' 'afperf_binary_reverse_static' 'Binary encoder reverse-map reuse static protection'
    Gate 'map-binary' 'af002_binary_frozen_semantics'
    Gate 'map-equivalence' 'af002_binary_reverse_frozen_equivalence'
    Gate 'map-public' 'af002_binary_public_identity'
} finally {[IO.File]::WriteAllBytes($source,$restored);Pop-Location;$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle}
if((Get-FileHash $source).Hash -ne $activeHash){throw 'Exact restoration failed'}
$env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $workspace 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
Push-Location $workspace
try {
    Gate 'restored-budget' 'af002_binary_reverse_public_budget'
    Gate 'restored-static' 'afperf_binary_reverse_static'
    Gate 'restored-binary' 'af002_binary_frozen_semantics'
    Gate 'restored-equivalence' 'af002_binary_reverse_frozen_equivalence'
    Gate 'restored-public' 'af002_binary_public_identity'
} finally {Pop-Location;$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle}
foreach($file in $files){
    if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Active file changed: $($file.path)"}
    if((Get-FileHash (Join-Path $workspace $file.path)).Hash -ne $file.sha256){throw "Isolated restoration drift: $($file.path)"}
}
@{agent='Agent MaxHeadRoom';intendedFailures=2;preservedBinaryPublicPasses=3;restoredPasses=5;baselineStaticPasses=1;
    activeSourceUnchanged=$true;productionSha256=$activeHash;files=$files}|
    ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED, binary/public GREEN, exact restored five GREEN: $evidence"
