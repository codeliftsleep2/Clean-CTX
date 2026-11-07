# Agent MaxHeadRoom: isolated real shortcut clone restoration; active source untouched.
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $before=(Get-ChildItem target/af-perf-002 -Directory | Where-Object {$_.Name -match '^diff-shortcut-before-\d{8}-\d{6}$' -and (Test-Path (Join-Path $_.FullName 'receipt.json'))} | Sort-Object Name -Descending | Select-Object -First 1).FullName
    if(-not $before){throw 'Original RED required'}
    $r=Get-Content (Join-Path $before 'receipt.json') -Raw|ConvertFrom-Json
    if($r.semanticExit -ne 0 -or $r.budgetExit -eq 0){throw 'Invalid original RED'}
    $evidence=Join-Path $repoPath ('target/af-perf-002/diff-shortcut-mutation-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
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
        if(Test-Path -LiteralPath $path -PathType Leaf){
            New-Item -ItemType Directory -Force -Path (Split-Path $destination)|Out-Null
            Copy-Item -LiteralPath $path -Destination $destination
        } elseif(Test-Path -LiteralPath $destination -PathType Leaf){Remove-Item -LiteralPath $destination}
    }
    $hashes=@($paths|Sort-Object -Unique|Where-Object {Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
} finally {Pop-Location}
$relative='src/mcp/tool_helpers.rs'
$source=Join-Path $workspace $relative
$original=[IO.File]::ReadAllBytes($source)
$activeHash=(Get-FileHash (Join-Path $repoPath $relative)).Hash
[IO.File]::WriteAllBytes((Join-Path $evidence 'production-before.rs'),$original)
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
    Invoke-Gate 'baseline-static' 'afperf_diff_shortcut_static' 1
    $text=[Text.UTF8Encoding]::new($false,$true).GetString($original)
    $start='&& let Some(baseline_snap) = cache.get_baseline(&cache_key)'
    if([regex]::Matches($text,[regex]::Escape($start)).Count -ne 1){throw 'Mutation site changed'}
    $frozen=[IO.File]::ReadAllText((Join-Path $before 'production-source.rs')).Replace("`r`n","`n")
    $a=$frozen.IndexOf('&& let Some(baseline_snap) = cache')
    $b=$frozen.IndexOf('.cloned()',$a)+'.cloned()'.Length
    if($a -lt 0 -or $b -le $a){throw 'Original clone site missing'}
    $text=$text.Replace($start,$frozen.Substring($a,$b-$a))
    [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false))
    & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence 'clone.diff')
    if($LASTEXITCODE -ne 1){throw 'Real mutation diff missing'}
    Invoke-Gate 'mutant-budget' 'af002_diff_shortcut_budget' 1 'Diff shortcut redundant snapshot clone budget'
    Invoke-Gate 'mutant-static' 'afperf_diff_shortcut_static' 1 'Diff shortcut static clone protection'
    Invoke-Gate 'mutant-frozen' 'afperf_diff_shortcut_frozen_public' 1
} finally {[IO.File]::WriteAllBytes($source,$original);Pop-Location}
if((Get-FileHash $source).Hash -ne $activeHash){throw 'Exact restoration failed'}
Push-Location $workspace
try {
    Invoke-Gate 'restored-budget-semantic' 'af002_diff_shortcut_' 2
    Invoke-Gate 'restored-static-frozen' 'afperf_diff_shortcut_' 2
} finally {Pop-Location}
if((Get-FileHash (Join-Path $repoPath $relative)).Hash -ne $activeHash){throw 'Active source changed'}
@{agent='Agent MaxHeadRoom';intendedFailures=2;frozenSemanticPasses=1;restoredPasses=4;activeSourceUnchanged=$true;files=$hashes}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED, frozen semantic GREEN, exact restored four GREEN: $evidence"
