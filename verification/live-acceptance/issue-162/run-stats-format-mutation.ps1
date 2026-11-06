# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/af-perf-002/stats-format-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
$workspace = Join-Path $evidence 'workspace'
New-Item -ItemType Directory -Path $evidence | Out-Null
& git clone --local --no-hardlinks $repoPath $workspace *> (Join-Path $evidence 'clone.log')
if ($LASTEXITCODE -ne 0) { throw "Isolated clone failed: $evidence" }
# Overlay exact current files, including untracked regression assets. Ignored target
# and local credentials are excluded by Git's own inventory; do not copy .git.
Push-Location $repoPath
try {
    $paths = @(git ls-files --cached --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw 'Source inventory failed' }
    foreach ($path in $paths) {
        $destination = [IO.Path]::GetFullPath((Join-Path $workspace $path))
        $workspacePrefix = [IO.Path]::GetFullPath($workspace) + [IO.Path]::DirectorySeparatorChar
        if (-not $destination.StartsWith($workspacePrefix,[StringComparison]::OrdinalIgnoreCase)) { throw 'Overlay escaped isolated workspace' }
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
            Copy-Item -LiteralPath $path -Destination $destination
        } elseif (Test-Path -LiteralPath $destination -PathType Leaf) {
            Remove-Item -LiteralPath $destination
        }
    }
} finally { Pop-Location }
$relative = 'src/mcp/tool_handlers/stats/mod.rs'
$source = Join-Path $workspace $relative
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash} })
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'production-before.rs')
function Invoke-StatsGate([string]$Name,[string]$Filter,[int]$Count,[bool]$Red,[string]$Message='') {
    $log=Join-Path $evidence ($Name+'.log')
    & cargo test --lib --all-features --quiet $Filter -- --test-threads=1 --nocapture *> $log
    $code=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText($log)
    if($Red){
        if($code -eq 0 -or $observed -notmatch [regex]::Escape($Message) -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended mutation RED missing: $log"}
    } elseif($code -ne 0 -or $observed -notmatch "test result: ok\. $Count passed; 0 failed; 0 ignored;") {throw "GREEN missing: $log"}
}
Push-Location $workspace
try {
    Invoke-StatsGate 'baseline' 'af002_stats_' 3 $false
    $text=[Text.UTF8Encoding]::new($false,$true).GetString($original)
    $needle='if format != "json" {'
    if(([regex]::Matches($text,[regex]::Escape($needle))).Count -ne 1){throw 'Mutation boundary changed'}
    $text=$text.Replace($needle,'if true {')
    [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false,$true))
    Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'production-mutated.rs')
    & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence 'mutation.diff')
    if($LASTEXITCODE -ne 1){throw 'Expected real production diff'}
    Invoke-StatsGate 'mutation-budget' 'af002_stats_json_zero_unused_text_budget' 1 $true 'unused stats text render budget'
    Invoke-StatsGate 'mutation-static' 'af002_stats_static_format_sinks' 1 $true 'stats format static'
    Invoke-StatsGate 'mutation-preservation' 'af002_stats_format_and_file_growth' 1 $false
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    Pop-Location
}
if((Get-FileHash -LiteralPath $source).Hash -ne $activeHash){throw 'Exact restoration mismatch'}
Push-Location $workspace
try { Invoke-StatsGate 'restored' 'af002_stats_' 3 $false } finally {Pop-Location}
if((Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash -ne $activeHash){throw 'Active source changed'}
[pscustomobject]@{agent='Agent MaxHeadRoom';category='isolated real unconditional-render mutation';intendedFailures=2;restoredTests=3;activeSourceUnchanged=$true;isolatedSourceRestored=$true;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED; exact restored three-test GREEN: $evidence"
