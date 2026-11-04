# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/af-perf-002/hierarchy-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative = 'src/ir/hierarchical/encode.rs'
$source = Join-Path $workspace $relative
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash} })
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'count-before.rs')
# Establish clean static baseline before attributing any later RED to mutation.
Push-Location $workspace
try {
    & cargo test --lib --all-features --quiet afperf_hierarchy_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'baseline-static.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'baseline-static.log') -Pattern 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Clean static baseline missing before mutation: $evidence" }
} finally { Pop-Location }
$text = [Text.UTF8Encoding]::new($false,$true).GetString($original)
$baseline = Join-Path $repoPath 'target/af-perf-002/hierarchy-budget-before-20261104-164550/production-source.rs'
if (-not (Test-Path -LiteralPath $baseline)) { throw 'Original RED source is required' }
$before = [IO.File]::ReadAllText($baseline)
$start = $text.IndexOf('            CoreOp::DefMethod(cid, mid, name) => {')
$end = $text.IndexOf('            CoreOp::DefField(cid, fid, name) => {')
$oldStart = $before.IndexOf('            CoreOp::DefMethod(cid, mid, name) => {')
$oldEnd = $before.IndexOf('            CoreOp::DefField(cid, fid, name) => {')
if ($start -lt 0 -or $end -le $start -or $oldStart -lt 0 -or $oldEnd -le $oldStart) { throw 'Mutation boundary changed; review required' }
$oldArm = $before.Substring($oldStart,$oldEnd-$oldStart)
$scan = @"
classes.iter().position(|c| {
                    #[cfg(test)]
                    crate::work_observations_af_perf_002::update(|counts| counts.hierarchy_class_comparisons += 1);
                    c.id == *cid
                })
"@
$oldArm = $oldArm.Replace('find_class_by_id(&classes, cid)', $scan.Trim())
$text = $text.Substring(0,$start) + $oldArm + $text.Substring($end)
[IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false,$true))
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'count-mutated.rs')
Push-Location $workspace
try {
    & git diff --no-index -- (Join-Path $evidence 'count-before.rs') $source *> (Join-Path $evidence 'mutation.diff')
    if ($LASTEXITCODE -ne 1) { throw 'Expected production mutation diff' }
    & cargo test --lib --all-features --quiet hierarchy_placement_bounded_work -- --test-threads=1 --nocapture *> (Join-Path $evidence 'mutation-budget.log')
    $budgetRedExit = $LASTEXITCODE
    & cargo test --lib --all-features --quiet afperf_hierarchy_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'mutation-tests.log')
    $redExit = $LASTEXITCODE
    $observed = [IO.File]::ReadAllText((Join-Path $evidence 'mutation-tests.log'))
    $budgetObserved = [IO.File]::ReadAllText((Join-Path $evidence 'mutation-budget.log'))
    if ($budgetRedExit -eq 0 -or $redExit -eq 0 -or $budgetObserved -notmatch 'Hierarchy placement work exceeds one lookup per member' -or $budgetObserved -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;' -or $observed -notmatch 'hierarchy placement unclassified call' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;') { throw "Missing intended budget/static RED: $evidence" }
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    Pop-Location
}
if ((Get-FileHash -LiteralPath $source).Hash -ne $activeHash) { throw 'Isolated restoration mismatch' }
Push-Location $workspace
try {
    & cargo test --lib --all-features --quiet hierarchy_placement_bounded_work -- --test-threads=1 --nocapture *> (Join-Path $evidence 'restored-budget.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'restored-budget.log') -Pattern 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Restored budget GREEN missing: $evidence" }
    & cargo test --lib --all-features --quiet afperf_hierarchy_ -- --test-threads=1 --nocapture *> (Join-Path $evidence 'restored-tests.log')
    if ($LASTEXITCODE -ne 0 -or -not (Select-String -LiteralPath (Join-Path $evidence 'restored-tests.log') -Pattern 'test result: ok\. 1 passed; 0 failed; 0 ignored;' -Quiet)) { throw "Restored semantic/static GREEN missing: $evidence" }
} finally { Pop-Location }
if ((Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash -ne $activeHash) { throw 'Active source changed during verification' }
[pscustomobject]@{agent='Agent MaxHeadRoom';category='isolated production regression; tracked tests remain authority';redExit=$redExit;restoredTests=2;activeSourceUnchanged=$true;isolatedSourceRestored=$true;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Counter and static RED; restored two-test GREEN: $evidence"
