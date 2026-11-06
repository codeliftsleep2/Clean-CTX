# Agent MaxHeadRoom: independently restore each real production retrieval in isolation.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence=Join-Path $repoPath ('target/af-perf-002/a4-lookup-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
$workspace=Join-Path $evidence 'workspace'
New-Item -ItemType Directory -Path $evidence | Out-Null
& git clone --local --no-hardlinks $repoPath $workspace *> (Join-Path $evidence 'clone.log')
if($LASTEXITCODE -ne 0){throw 'Isolated clone failed'}
Push-Location $repoPath
try {
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
$relative='src/ir/csharp_resolution/compilation/dependencies.rs'
$helperRelative='src/ir/csharp_resolution/compilation/dependency_inputs.rs'
$source=Join-Path $workspace $relative
$helper=Join-Path $workspace $helperRelative
$original=[IO.File]::ReadAllBytes($source)
$originalHelper=[IO.File]::ReadAllBytes($helper)
$activeHash=(Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$helperHash=(Get-FileHash -LiteralPath (Join-Path $repoPath $helperRelative)).Hash
$hashes=@($paths | Sort-Object -Unique | Where-Object {Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf} | ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash}})
[IO.File]::WriteAllBytes((Join-Path $evidence 'production-before.rs'),$original)
[IO.File]::WriteAllBytes((Join-Path $evidence 'lookup-before.rs'),$originalHelper)
function Invoke-Gate([string]$Name,[string]$Filter,[int]$Count,[string]$Red=''){
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE
    $text=[IO.File]::ReadAllText($log)
    if($Red){
        if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}
    } elseif($code -ne 0 -or $text -notmatch "test result: ok\. $Count passed; 0 failed; 0 ignored;"){throw "GREEN missing: $log"}
}
function Replace-Exact([string]$Text,[string]$Old,[string]$New,[int]$Count=1){
    if([regex]::Matches($Text,[regex]::Escape($Old)).Count -ne $Count){throw "Mutation site changed: $Old"}
    $Text.Replace($Old,$New)
}
$seed=@'
let _ = inputs.seeds(seed).count();
let seeds: BTreeSet<_> = inputs.original_units.iter().flat_map(|unit| &unit.classes)
    .filter(|input| {
        crate::work_observations_af_perf_002::update(|c| c.a4.seed_class_visits += 1);
        input.part.file == seed
    }).map(|input| input.part.key.clone()).collect();
    for key in &seeds {
'@
$family=@'
for input in inputs.original_units.iter().flat_map(|unit| &unit.classes).filter(|input| {
    crate::work_observations_af_perf_002::update(|c| c.a4.family_class_visits += 1);
    input.part.key == *key
}) {
'@
Push-Location $workspace
try {
    Invoke-Gate 'baseline-static' 'afperf_a4_static_original_input_retrieval' 1
    foreach($site in @('seed','family','base')){
        $text=[Text.UTF8Encoding]::new($false,$true).GetString($original)
        $helperText=[Text.UTF8Encoding]::new($false,$true).GetString($originalHelper)
        # Only mutated isolated source gains access to the original full units.
        $helperText=Replace-Exact $helperText "pub(super) struct OriginalInputs<'a> {" "pub(super) struct OriginalInputs<'a> {`n    pub(super) original_units: &'a [CSharpCompilationUnit],"
        $helperText=Replace-Exact $helperText "impl<'a> OriginalInputs<'a> {" "impl<'a> OriginalInputs<'a> {`n    pub(super) fn with_original_units(units: &'a [CSharpCompilationUnit]) -> Self { Self { original_units: units, ..Self::default() } }"
        $text=Replace-Exact $text 'let mut inputs = OriginalInputs::default();' 'let mut inputs = OriginalInputs::with_original_units(units);' 2
        switch($site){
            'seed' {$text=Replace-Exact $text 'for key in inputs.seeds(seed) {' $seed}
            'family' {$text=Replace-Exact $text 'for input in inputs.family(key) {' $family}
            'base' {
                $text=Replace-Exact $text 'let input = inputs.family(key).iter().copied().find(|input| {' 'let input = inputs.original_units.iter().flat_map(|unit| &unit.classes).find(|input| {'
                $text=Replace-Exact $text 'input.part.base_references.contains(written)' "{ crate::work_observations_af_perf_002::update(|c| c.a4.base_find_visits += 1); input.part.key == *key && input.part.base_references.contains(written) }"
            }
        }
        [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($helper,$helperText,[Text.UTF8Encoding]::new($false))
        & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence ($site+'.diff'))
        if($LASTEXITCODE -ne 1){throw 'Real production mutation diff missing'}
        Invoke-Gate "$site-budget" 'af002_a4_lookup_construction_budget' 1 'A4 redundant full-input retrieval budget'
        Invoke-Gate "$site-static" 'afperf_a4_static_original_input_retrieval' 1 'A4 retrieval static protection'
        Invoke-Gate "$site-semantic" 'af002_a4_lookup_frozen_equivalence' 1
        [IO.File]::WriteAllBytes($source,$original)
        [IO.File]::WriteAllBytes($helper,$originalHelper)
    }
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    [IO.File]::WriteAllBytes($helper,$originalHelper)
    Pop-Location
}
if((Get-FileHash $source).Hash -ne $activeHash -or (Get-FileHash $helper).Hash -ne $helperHash){throw 'Exact restoration mismatch'}
Push-Location $workspace
try {
    Invoke-Gate 'restored-unit' 'af002_a4_lookup_' 2
    Invoke-Gate 'restored-static-cost' 'afperf_a4_' 2
    Invoke-Gate 'restored-public' 'af002_a4_public_lookup_budget' 1
} finally {Pop-Location}
if((Get-FileHash (Join-Path $repoPath $relative)).Hash -ne $activeHash -or (Get-FileHash (Join-Path $repoPath $helperRelative)).Hash -ne $helperHash){throw 'Active source changed'}
@{agent='Agent MaxHeadRoom';category='Three independent real retrieval mutations';intendedFailures=6;semanticPreservationPasses=3;restoredPasses=5;activeSourceUnchanged=$true;files=$hashes} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Six intended RED, three semantic preservation passes, exact restored five GREEN: $evidence"
