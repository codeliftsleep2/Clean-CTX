# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/issue-162/af-perf-001-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$source = Join-Path $workspace 'src/ir/typescript_resolution/relation_evidence.rs'
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath 'src/ir/typescript_resolution/relation_evidence.rs')).Hash
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'relation_evidence-before.rs')
$text = [IO.File]::ReadAllText($source)
$pattern = 'let projected = children_projection[\s\S]*?return crate::angular_meta::route_load_children::admits_projected\(\s*projected,\s*edge,?\s*\);'
$matches = [regex]::Matches($text,$pattern)
if ($matches.Count -ne 1) { throw "Mutation anchor changed; review required: $evidence" }
$text = [regex]::Replace($text,$pattern,'let projected = crate::angular_meta::route_load_children::project(self); return crate::angular_meta::route_load_children::admits_projected(&projected, edge);')
# Remove only the now-unused local slot, so warnings do not masquerade as RED.
$text = [regex]::Replace($text,'#\[cfg\(feature = "angular"\)\]\s*let mut children_projection: Option<Vec<SemanticEdge>> = None;','')
[IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false))
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'relation_evidence-mutated.rs')
Push-Location $workspace
try {
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1 *> (Join-Path $evidence 'mutation-gate.log')
    $gateExit = $LASTEXITCODE
    $logs = @(Get-ChildItem -LiteralPath (Join-Path $workspace 'target/issue-162') -Recurse -Filter 'af_perf_001_.log' -File)
    if ($gateExit -eq 0 -or $logs.Count -ne 1) { throw "Mutation did not fail the mandatory gate: $evidence" }
    $observed = Get-Content -LiteralPath $logs[0].FullName -Raw
    if ($observed -notmatch 'AF-PERF-001 projection-per-edge consumer=7 candidates=2') { throw "Missing intended production work-count RED: $evidence" }
    if ($observed -notmatch 'AF-PERF-001 complete projection per candidate') { throw "Missing intended AST amplification RED: $evidence" }
    Copy-Item -LiteralPath $logs[0].FullName -Destination (Join-Path $evidence 'tracked-mutation-tests.log')
    Write-Output "Intentional real production amplification detected by counters and AST gate: $evidence"
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    Pop-Location
}
[pscustomobject]@{agent='Agent MaxHeadRoom';category='operator mutation of real production dispatch; tracked tests remain authority';gateExit=$gateExit;activeSourceUnchanged=((Get-FileHash -LiteralPath (Join-Path $repoPath 'src/ir/typescript_resolution/relation_evidence.rs')).Hash -eq $activeHash);isolatedSourceRestored=((Get-FileHash -LiteralPath $source).Hash -eq (Get-FileHash -LiteralPath (Join-Path $evidence 'relation_evidence-before.rs')).Hash)} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
