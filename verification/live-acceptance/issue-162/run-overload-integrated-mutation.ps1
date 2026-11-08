# Agent MaxHeadRoom: independent isolated restoration of each original candidate scan.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$before=Join-Path $repoPath 'target/af-perf-002/overload-before-20261107-142854'
$r=Get-Content (Join-Path $before 'receipt.json') -Raw|ConvertFrom-Json
if($r.semanticExit -ne 0 -or $r.budgetExit -eq 0){throw 'Genuine original RED required'}
Push-Location $repoPath
try {
    $evidence=Join-Path $repoPath ('target/af-perf-002/overload-integrated-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
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
    $hashes=@($paths|Sort-Object -Unique|Where-Object {Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
} finally {Pop-Location}
$relative='src/diff/differ.rs';$source=Join-Path $workspace $relative
$original=[IO.File]::ReadAllBytes($source)
$helperRelative='src/diff/overload_lookup.rs'
$activeHash=(Get-FileHash (Join-Path $repoPath $relative)).Hash
$helperHash=(Get-FileHash (Join-Path $repoPath $helperRelative)).Hash
[IO.File]::WriteAllBytes((Join-Path $evidence 'production-before.rs'),$original)
$frozen=[IO.File]::ReadAllText((Join-Path $before 'production-source.rs')).Replace("`r`n","`n")
function Header([string]$Text,[int]$Start){
    $a=$Text.IndexOf('                    if let Some((cur_idx, current_method)) =',$Start,[StringComparison]::Ordinal)
    if($a -lt 0){throw 'Original scan header missing'}
    $pair=$Text.IndexOf('crate::work_observations_af_perf_002::overload_pair',$a,[StringComparison]::Ordinal)
    if($pair -lt 0){throw 'Original pair observation missing'}
    $end=$Text.LastIndexOf('                        #[cfg(test)]',$pair,[StringComparison]::Ordinal)
    @{text=$Text.Substring($a,$end-$a);end=$pair}
}
$first=Header $frozen 0;$second=Header $frozen $first.end
function Replace-Header([string]$Text,[string]$Marker,[string]$Replacement){
    $a=$Text.IndexOf($Marker,[StringComparison]::Ordinal)
    if($a -lt 0 -or $Text.IndexOf($Marker,$a+1,[StringComparison]::Ordinal) -ge 0){throw 'Production mutation marker drift'}
    $pair=$Text.IndexOf('crate::work_observations_af_perf_002::overload_pair',$a,[StringComparison]::Ordinal)
    $end=$Text.LastIndexOf('                        #[cfg(test)]',$pair,[StringComparison]::Ordinal)
    if($pair -lt 0 -or $end -le $a){throw 'Production mutation end missing'}
    $Text.Substring(0,$a)+$Replacement+$Text.Substring($end)
}
function Gate([string]$Name,[string]$Filter,[int]$Count,[string]$Red=''){
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    if($Red){if($code -eq 0 -or $text -notmatch [regex]::Escape($Red) -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended RED missing: $log"}}
    elseif($code -ne 0 -or $text -notmatch "test result: ok\. $Count passed; 0 failed; 0 ignored;"){throw "GREEN missing: $log"}
    if($text -match '(?m)^warning:'){throw "Mutation introduced warning: $log"}
}
Push-Location $workspace
try {
    Gate 'baseline-static' 'afperf_overload_static' 1
    foreach($site in @('exact','signature')){
        $text=[Text.UTF8Encoding]::new($false,$true).GetString($original).Replace("`r`n","`n")
        if($site -eq 'exact'){$text=Replace-Header $text '                    if let Some(cur_idx) = lookup.exact(base_method, &cur_used) {' $first.text}
        else {
            $text=Replace-Header $text '                    if let Some(cur_idx) = lookup.signature(&base_method.sig, &cur_used) {' $second.text
            $text=$text.Replace('let mut lookup = MethodLookup::new(g2);','let lookup = MethodLookup::new(g2);')
        }
        [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false))
        & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence ($site+'.diff'))
        if($LASTEXITCODE -ne 1){throw 'Real original-scan mutation diff missing'}
        Gate "$site-budget" 'af002_overload_public_budget' 1 'Overload redundant candidate search budget'
        Gate "$site-static" 'afperf_overload_static' 1 'Overload retrieval static protection'
        Gate "$site-semantic" 'af002_overload_frozen_equivalence' 1
        Gate "$site-public" 'af002_overload_integrated_frozen_public' 1
        [IO.File]::WriteAllBytes($source,$original)
    }
} finally {[IO.File]::WriteAllBytes($source,$original);Pop-Location}
if((Get-FileHash $source).Hash -ne $activeHash -or (Get-FileHash (Join-Path $workspace $helperRelative)).Hash -ne $helperHash){throw 'Exact restoration failed'}
Push-Location $workspace
try {
    Gate 'restored-budget' 'af002_overload_public_budget' 1
    Gate 'restored-semantic' 'af002_overload_frozen_equivalence' 1
    Gate 'restored-static' 'afperf_overload_static' 1
    Gate 'restored-full-equality' 'afperf_overload_full_equality_lookup' 1
    Gate 'restored-public' 'af002_overload_integrated_frozen_public' 1
} finally {Pop-Location}
if((Get-FileHash (Join-Path $repoPath $relative)).Hash -ne $activeHash -or (Get-FileHash (Join-Path $repoPath $helperRelative)).Hash -ne $helperHash){throw 'Active production changed'}
@{agent='Agent MaxHeadRoom';intendedFailures=4;preservedSemanticPublicPasses=4;restoredPasses=5;activeSourceUnchanged=$true;files=$hashes}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Four intended RED; four semantic/public passes; exact restored five GREEN: $evidence"
