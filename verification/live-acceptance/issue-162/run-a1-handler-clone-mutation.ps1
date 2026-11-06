# Operator-only sensitivity proof in an isolated copy. Never mutates active source/index.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repoPath = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$evidence = Join-Path $repoPath ('target/af-perf-002/a1-handler-clone-mutation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
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
$relative = 'src/mcp/tool_handlers/core/compress.rs'
$source = Join-Path $workspace $relative
$original = [IO.File]::ReadAllBytes($source)
$activeHash = (Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash
$hashes = @($paths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath (Join-Path $workspace $_) -PathType Leaf } | ForEach-Object { [pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $_)).Hash} })
Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'production-before.rs')
function Invoke-A1Gate([string]$Name,[string]$Filter,[int]$Count,[bool]$Red,[string]$Message='') {
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
    Invoke-A1Gate 'baseline' 'afperf_a1_static_handler_clone_sinks' 1 $false
    $text=[Text.UTF8Encoding]::new($false,$true).GetString($original)
    $needle='        let path_alias = state.get_or_create_alias(semantic_owner.clone());'
    if(([regex]::Matches($text,[regex]::Escape($needle))).Count -ne 1){throw 'Clone mutation boundary changed'}
    $replacement=@"
        #[cfg(test)]
        crate::work_observations_af_perf_002::publication_ir_copy("handler-durable-candidate", &ir);
        let durable_candidate = ir.clone();
$needle
"@
    $text=$text.Replace($needle,$replacement)
    $call="                &ir,`n                &semantic_edges,"
    if(([regex]::Matches($text,[regex]::Escape($call))).Count -ne 1){throw 'Borrowed persistence argument changed'}
    $text=$text.Replace($call,"                &durable_candidate,`n                &semantic_edges,")
    [IO.File]::WriteAllText($source,$text,[Text.UTF8Encoding]::new($false,$true))
    Copy-Item -LiteralPath $source -Destination (Join-Path $evidence 'production-mutated.rs')
    & git diff --no-index -- (Join-Path $evidence 'production-before.rs') $source *> (Join-Path $evidence 'mutation.diff')
    if($LASTEXITCODE -ne 1){throw 'Expected real production diff'}
    Invoke-A1Gate 'mutation-budget' 'af002_a1_public_redundant_candidate_budget' 1 $true 'A1 redundant handler candidate clone:'
    Invoke-A1Gate 'mutation-static' 'afperf_a1_static_handler_clone_sinks' 1 $true 'A1 clone static:'
    Invoke-A1Gate 'mutation-preservation' 'afperf_a1_frozen_public_and_durable_identity' 1 $false
} finally {
    [IO.File]::WriteAllBytes($source,$original)
    Pop-Location
}
if((Get-FileHash -LiteralPath $source).Hash -ne $activeHash){throw 'Exact restoration mismatch'}
Push-Location $workspace
try {
    Invoke-A1Gate 'restored-budget' 'artifact_operation_work_af_perf_002::a1_clone_budget::' 2 $false
    Invoke-A1Gate 'restored-semantic-static' 'afperf_a1_' 2 $false
} finally {Pop-Location}
if((Get-FileHash -LiteralPath (Join-Path $repoPath $relative)).Hash -ne $activeHash){throw 'Active source changed'}
[pscustomobject]@{agent='Agent MaxHeadRoom';category='isolated real redundant IR clone restoration';intendedFailures=2;restoredTests=4;activeSourceUnchanged=$true;isolatedSourceRestored=$true;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'receipt.json') -Encoding utf8NoBOM
Write-Output "Two intended RED; exact restored four-test GREEN: $evidence"
