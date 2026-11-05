# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/af-perf-002/proxy-prefilter-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative = 'proxy/src/scrub_patterns.rs'
$source = Join-Path $workspace $relative
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash} })
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'count-before.rs')
# Require the clean behavioral baseline before attributing RED to mutation.
Push-Location $workspace
try {
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_prefilter_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'baseline-tests.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'baseline-tests.log') -Pattern 'test result: ok\. 4 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Clean behavioral baseline missing: $evidence" }
} finally { Pop-Location }
$text = [Text.UTF8Encoding]::new($false,$true).GetString($original)
$mutations = @('pwd','apikey','api-key','accesskey','access-key','privatekey','private-key','authorization')
$results = @()
foreach ($root in $mutations) {
    $line = '    "' + $root + '",'
    $start = $text.IndexOf($line,[StringComparison]::Ordinal)
    if ($start -lt 0 -or $start -ne $text.LastIndexOf($line,[StringComparison]::Ordinal)) { throw "Root mutation scope changed: $root" }
    $mutated = $text.Remove($start,$line.Length)
    [IO.File]::WriteAllText($source,$mutated,[Text.UTF8Encoding]::new($false,$true))
    $mutationLog = Join-Path $evidence ($root + '-red.log')
    Copy-Item -LiteralPath $source -Destination (Join-Path $evidence ($root + '-mutated.rs'))
    Push-Location $workspace
    try {
        & cargo test -p clean-ctx-proxy --lib --all-features --quiet scrub_prefilter_af_perf_002::af002_prefilter_existing_rule_triggers -- --exact --test-threads=1 --nocapture *> $mutationLog
        $redExit = $LASTEXITCODE
        $observed = [IO.File]::ReadAllText($mutationLog)
        $expected = if ($root -eq 'authorization') { 'authorization:Basic' } else { 'assignment:' + $root }
        if ($redExit -eq 0 -or -not $observed.Contains($expected) -or $observed -notmatch 'Prefilter excludes existing rule matches' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;') { throw "Missing root-removal RED: $root, $evidence" }
        $results += [pscustomobject]@{root=$root;redExit=$redExit;expectedOmission=$expected}
    } finally {
        [IO.File]::WriteAllBytes($source,$original)
        Pop-Location
    }
}
if ((Get-FileHash -LiteralPath $source).Hash -ne $activeHash) { throw 'Isolated restoration mismatch' }
Push-Location $workspace
try {
    & cargo test -p clean-ctx-proxy --lib --all-features --quiet af002_prefilter_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'restored-tests.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'restored-tests.log') -Pattern 'test result: ok\. 4 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Restored behavioral GREEN missing: $evidence" }
} finally { Pop-Location }
if ((Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash -ne $activeHash) { throw 'Active source changed during verification' }
[pscustomobject]@{agent='Agent MaxHeadRoom';category='isolated production regression; tracked tests remain authority';mutations=$results;restoredTests=4;activeSourceUnchanged=$true;isolatedSourceRestored=$true;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Eight independent root-removal REDs; exact restored four-test GREEN: $evidence"
