# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/af-perf-002/index174-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative = 'src/workspace/index/membership.rs'
$source = Join-Path $workspace $relative
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash} })
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'membership-before.rs')
$text = [Text.UTF8Encoding]::new($false,$true).GetString($original)
$anchor = 'if !members.contains(key) {'
if ([regex]::Matches($text,[regex]::Escape($anchor)).Count -ne 1) { throw 'Mutation anchor changed; review required' }
$mutation = @'
if !self.name_index.get(&entity.name).is_some_and(|keys| keys.iter().any(|existing| {
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|s| s.comparisons[0] += 1);
    existing == key
})) {
'@
$text = $text.Replace($anchor,$mutation)
[IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false,$true))
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'membership-mutated.rs')
Push-Location $workspace
try {
    & git diff --no-index -- (Join-Path $evidence 'membership-before.rs') $source *> (Join-Path $evidence 'mutation.diff')
    if ($LASTEXITCODE -ne 1) { throw 'Expected production mutation diff' }
    & cargo test --lib --all-features --quiet af174_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'mutation-tests.log')
    $redExit = $LASTEXITCODE
    $observed = [IO.File]::ReadAllText((Join-Path $evidence 'mutation-tests.log'))
    if ($redExit -eq 0 -or $observed -notmatch 'AF174 quadratic membership dimension=same-name-identities' -or $observed -notmatch 'AF174 prohibited insertion scan' -or $observed -notmatch 'test result: FAILED') { throw "Missing intended counter/static RED: $evidence" }
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    Pop-Location
}
if ((Get-FileHash -LiteralPath $source).Hash -ne $activeHash) { throw 'Isolated restoration mismatch' }
Push-Location $workspace
try {
    & cargo test --lib --all-features --quiet af174_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'restored-tests.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'restored-tests.log') -Pattern 'test result: ok\. 9 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Restored GREEN missing: $evidence" }
} finally { Pop-Location }
if ((Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash -ne $activeHash) { throw 'Active source changed during verification' }
[pscustomobject]@{agent='Agent MaxHeadRoom';category='isolated production regression; tracked tests remain authority';redExit=$redExit;restoredTests=9;activeSourceUnchanged=$true;isolatedSourceRestored=$true;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Counter and static RED; restored nine-test GREEN: $evidence"
