# Agent MaxHeadRoom: real unconditional constructor restored only in isolated source.
$ErrorActionPreference='Stop'
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldEncoding=[Console]::OutputEncoding
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$oldRoot=$env:AF002_MANIFEST_FIXTURE_ROOT;$oldOracle=$env:AF002_MANIFEST_ORACLE
$before=Join-Path $repoPath 'target/af-perf-002/manifest-demand-before-20261108-231514'
$prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
if($prior.semanticPasses -ne 2 -or $prior.publicCases -ne 70 -or $prior.budgetExit -eq 0){throw 'Genuine RED missing'}
Push-Location $repoPath
try {
    $evidence=Join-Path $repoPath ('target/af-perf-002/manifest-demand-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative='src/mcp/tool_handlers/hydration/filesystem.rs'
$source=Join-Path $workspace $relative
$restored=[IO.File]::ReadAllBytes($source)
$env:AF002_MANIFEST_FIXTURE_ROOT=Join-Path $before 'fixtures'
$env:AF002_MANIFEST_ORACLE=Join-Path $before 'public-oracle.json'
function Gate([string]$Name,[string]$Filter,[string]$Red='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){
        if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}
    } elseif($code -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Warning invalidates mutation: $log"}
    if($Filter -in @('af002_manifest_unused_budget','af002_manifest_frozen_public_semantics')){
        if(@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_MANIFEST_PUBLIC ')}).Count -ne 70){throw 'Incomplete frozen public identities'}
    }
}
Push-Location $workspace
try {
    & cargo clippy --all-targets --all-features -- -D warnings *> (Join-Path $evidence 'clippy.log')
    if($LASTEXITCODE -ne 0){throw 'Baseline Clippy failed'}
    Gate 'baseline-static' 'af002_manifest_integrated_static'
    $old=[IO.File]::ReadAllText((Join-Path $before ('frozen/'+$relative)))
    $line=[regex]::Match($old,'(?m)^    let inventories = root_inventories\(roots, &manifest\);\r?$').Value.TrimEnd()
    if(-not $line){throw 'Real original constructor missing'}
    $current=[IO.File]::ReadAllText($source)
    $pattern='(?s)    let inventories = if manifests_required \{.*?\n    \};'
    if([regex]::Matches($current,$pattern).Count -ne 1){throw 'Conditional constructor boundary drift'}
    $mutant=[regex]::Replace($current,$pattern,$line).Replace('    manifests_required: bool,','    _manifests_required: bool,')
    [IO.File]::WriteAllText($source,$mutant,[Text.UTF8Encoding]::new($false))
    Gate 'original-budget' 'af002_manifest_unused_budget' 'Explicit-demand unused per-root manifest budget'
    Gate 'original-static' 'af002_manifest_integrated_static' 'protected_path(caller, scan, fallback, execution)'
    Gate 'original-public-proof' 'af002_manifest_frozen_public_semantics'
    Gate 'original-root-contract' 'af002_root_inventory_contracts'
} finally {
    [IO.File]::WriteAllBytes($source,$restored)
    Pop-Location
    $env:AF002_MANIFEST_FIXTURE_ROOT=$oldRoot;$env:AF002_MANIFEST_ORACLE=$oldOracle
    [Console]::OutputEncoding=$oldEncoding
}
$env:AF002_MANIFEST_FIXTURE_ROOT=Join-Path $before 'fixtures'
$env:AF002_MANIFEST_ORACLE=Join-Path $before 'public-oracle.json'
Push-Location $workspace
try {
    [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
    Gate 'restored-budget' 'af002_manifest_unused_budget'
    Gate 'restored-public-proof' 'af002_manifest_frozen_public_semantics'
    Gate 'restored-static' 'af002_manifest_integrated_static'
    Gate 'restored-root' 'af002_root_inventory_contracts'
    Gate 'restored-incomplete' 'incomplete_authorized_root_cannot_earn_declaration_completeness'
} finally {Pop-Location;$env:AF002_MANIFEST_FIXTURE_ROOT=$oldRoot;$env:AF002_MANIFEST_ORACLE=$oldOracle;[Console]::OutputEncoding=$oldEncoding}
foreach($file in $files){
    if((Get-FileHash (Join-Path $repoPath $file.path)).Hash -ne $file.sha256){throw "Active file drift: $($file.path)"}
    if((Get-FileHash (Join-Path $workspace $file.path)).Hash -ne $file.sha256){throw "Restored file drift: $($file.path)"}
}
@{agent='Agent MaxHeadRoom';baselinePasses=1;intendedFailures=2;preservedSemanticPasses=2;restoredPasses=5;files=$files;activeSourceUnchanged=$true}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED; frozen semantics GREEN; five restored GREEN. Evidence: $evidence"
