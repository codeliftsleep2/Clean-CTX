# Agent MaxHeadRoom: isolated restoration of each real production join, never active source.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $before=(Get-ChildItem target/af-perf-002 -Directory | Where-Object {$_.Name -match '^edit-end-before-\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'receipt.json'))} | Sort-Object Name -Descending | Select-Object -First 1).FullName
    if(-not $before){throw 'Genuine original source required'}
    $receipt=Get-Content (Join-Path $before 'receipt.json') -Raw | ConvertFrom-Json
    if($receipt.phase -ne 'Before' -or $receipt.semanticExit -ne 0 -or $receipt.budgetExit -eq 0){throw 'Invalid original RED receipt'}
    $evidence=Join-Path $repoPath ('target/af-perf-002/edit-end-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    $workspace=Join-Path $evidence 'workspace'
    New-Item -ItemType Directory -Path $evidence | Out-Null
    & git clone --local --no-hardlinks $repoPath $workspace *> (Join-Path $evidence 'clone.log')
    if($LASTEXITCODE -ne 0){throw 'Isolated clone failed'}
    $paths=@(git ls-files --cached --others --exclude-standard)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $prefix=[IO.Path]::GetFullPath($workspace)+[IO.Path]::DirectorySeparatorChar
    foreach($path in $paths){
        $destination=[IO.Path]::GetFullPath((Join-Path $workspace $path))
        if(-not $destination.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Overlay escaped isolated workspace'}
        if(Test-Path -LiteralPath $path -PathType Leaf){
            New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
            Copy-Item -LiteralPath $path -Destination $destination
        } elseif(Test-Path -LiteralPath $destination -PathType Leaf){Remove-Item -LiteralPath $destination}
    }
} finally {Pop-Location}
$relative='src/edit/locate.rs'
$helperRelative='src/edit/locate/end_join.rs'
$source=Join-Path $workspace $relative
$original=[IO.File]::ReadAllBytes($source)
$activeHash=(Get-FileHash (Join-Path $repoPath $relative)).Hash
$helperHash=(Get-FileHash (Join-Path $repoPath $helperRelative)).Hash
$hashes=@($paths|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf}|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash}})
[IO.File]::WriteAllBytes((Join-Path $evidence 'production-before.rs'),$original)
$frozen=[IO.File]::ReadAllText((Join-Path $before 'production-source.rs'))
function Between([string]$Text,[string]$Start,[string]$End){
    $a=$Text.IndexOf($Start,[StringComparison]::Ordinal)
    if($a -lt 0){throw "Missing original join start: $Start"}
    $b=$Text.IndexOf($End,$a,[StringComparison]::Ordinal)
    if($b -lt 0){throw "Missing original join end: $End"}
    $Text.Substring($a,$b-$a).Trim()
}
$attachment=Between $frozen 'let declaration = declarations' '            unit.declaration_span'
$editable=Between $frozen 'let editable = table.units.iter().any' '            if !editable'
function Replace-Exact([string]$Text,[string]$Old,[string]$New){
    if([regex]::Matches($Text,[regex]::Escape($Old)).Count -ne 1){throw "Mutation site changed: $Old"}
    $Text.Replace($Old,$New)
}
function Invoke-Gate([string]$Name,[string]$Filter,[int]$Count,[string]$Red=''){
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){
        if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}
    } elseif($code -ne 0 -or $text -notmatch "test result: ok\. $Count passed; 0 failed; 0 ignored;"){throw "GREEN missing: $log"}
}
Push-Location $workspace
try {
    Invoke-Gate 'baseline-static' 'afperf_edit_end_static_known_joins' 1
    foreach($site in @('attachment','editability')){
        $text=[Text.UTF8Encoding]::new($false,$true).GetString($original)
        if($site -eq 'attachment'){
            # Retain an owned original Vec solely in the isolated waste mutation.
            $text=Replace-Exact $text 'EndJoin::new(declarations, &table.units)' 'EndJoin::new(declarations.clone(), &table.units)'
            $text=Replace-Exact $text 'let declaration = end_join.declaration(unit);' $attachment
        } else {$text=Replace-Exact $text 'let editable = end_join.editable(start, end);' $editable}
        [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false))
        & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence ($site+'.diff'))
        if($LASTEXITCODE -ne 1){throw 'Real production mutation diff missing'}
        Invoke-Gate "$site-budget" 'af002_edit_end_public_budget' 1 'Edit exact-end redundant full-input join budget'
        Invoke-Gate "$site-static" 'afperf_edit_end_static_known_joins' 1 'Edit exact-end static protection'
        Invoke-Gate "$site-semantic" 'af002_edit_end_frozen_equivalence' 1
        [IO.File]::WriteAllBytes($source,$original)
    }
} finally {[IO.File]::WriteAllBytes($source,$original);Pop-Location}
if((Get-FileHash $source).Hash -ne $activeHash -or (Get-FileHash (Join-Path $workspace $helperRelative)).Hash -ne $helperHash){throw 'Exact restoration mismatch'}
Push-Location $workspace
try {
    Invoke-Gate 'restored-budget-semantic' 'af002_edit_end_' 2
    Invoke-Gate 'restored-raw-cost-static' 'afperf_edit_end_' 3
} finally {Pop-Location}
if((Get-FileHash (Join-Path $repoPath $relative)).Hash -ne $activeHash -or (Get-FileHash (Join-Path $repoPath $helperRelative)).Hash -ne $helperHash){throw 'Active production changed'}
@{agent='Agent MaxHeadRoom';category='Independent real attachment/editability join restorations';intendedFailures=4;semanticPreservationPasses=2;restoredPasses=5;activeSourceUnchanged=$true;files=$hashes} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Four intended RED, two semantic passes, exact restored five GREEN: $evidence"
