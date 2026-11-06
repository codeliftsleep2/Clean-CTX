# Agent MaxHeadRoom: tracked original RED before approved handler-only clone removal.
param([ValidateSet('Before','After')][string]$Phase='Before',[string]$BeforeEvidence='')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/a1-handler-clone-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "A1 $Phase evidence: $logs"
    $changed=@('src/tests/mcp/a1_clone_budget_af_perf_002.rs','src/tests/mcp/artifact_operation_work_af_perf_002.rs')
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-a1-handler-clone-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath 'src/mcp/tool_handlers/core/compress.rs' -Destination (Join-Path $logs 'production-source.rs')
    Copy-Item -LiteralPath 'src/tests/mcp/a1_clone_budget_af_perf_002.rs' -Destination (Join-Path $logs 'budget-source.rs')
    Copy-Item -LiteralPath 'src/mcp/persistence_ir.rs' -Destination (Join-Path $logs 'normalization-source.rs')
    & cargo test --lib --all-features --quiet artifact_operation_work_af_perf_002::a1_clone_budget::af002_a1_normalized_input_identity -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'identity.log')
    $identityExit=$LASTEXITCODE
    if($identityExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'identity.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Physical normalization equivalence missing: $logs"}
    & cargo test --lib --all-features --quiet artifact_operation_work_af_perf_002::a1_clone_budget::af002_a1_public_redundant_candidate_budget -- --exact --test-threads=1 --nocapture *> (Join-Path $logs 'tests.log')
    $testExit=$LASTEXITCODE
    $observed=[IO.File]::ReadAllText((Join-Path $logs 'tests.log'))
    [pscustomobject]@{agent='Agent MaxHeadRoom';phase=$Phase;identityExit=$identityExit;testExit=$testExit;files=$hashes} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Tail 12
    if($Phase -eq 'Before'){
        if($testExit -eq 0 -or $observed -notmatch 'A1 redundant handler candidate clone:' -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Missing intended original clone RED: $logs"}
        $rows=@(Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Encoding utf8 | Where-Object { $_.StartsWith('AF002_A1_CLONE_ORACLE ') })
        if($rows.Count -ne 11){throw 'Missing original complete semantic rows; not valid RED'}
        Write-Output 'Genuine public clone budget RED and physical normalization identity GREEN. Production still unchanged.'
    } else {
        if($testExit -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Repaired public clone budget not GREEN: $logs"}
        if(-not $BeforeEvidence){
            $BeforeEvidence=(Get-ChildItem target/af-perf-002 -Directory | Where-Object {$_.Name -match '^a1-handler-clone-before-\d{8}-\d{6}$'} | Sort-Object Name -Descending | Select-Object -First 1).FullName
        }
        if(-not $BeforeEvidence){throw 'Frozen Before evidence required'}
        $originalReceipt=Get-Content -LiteralPath (Join-Path $BeforeEvidence 'receipt.json') -Raw -Encoding utf8 | ConvertFrom-Json
        if($originalReceipt.phase -ne 'Before' -or $originalReceipt.testExit -eq 0 -or $originalReceipt.identityExit -ne 0){throw 'Invalid original RED receipt'}
        if((Get-FileHash -LiteralPath (Join-Path $BeforeEvidence 'budget-source.rs')).Hash -ne (Get-FileHash -LiteralPath 'src/tests/mcp/a1_clone_budget_af_perf_002.rs').Hash){throw 'Budget source changed since original RED'}
        if((Get-FileHash -LiteralPath (Join-Path $BeforeEvidence 'normalization-source.rs')).Hash -ne (Get-FileHash -LiteralPath 'src/mcp/persistence_ir.rs').Hash){throw 'Persistence normalizer changed outside approved scope'}
        $prefix='AF002_A1_CLONE_ORACLE '
        $old=@(Get-Content -LiteralPath (Join-Path $BeforeEvidence 'tests.log') -Encoding utf8 | Where-Object {$_.StartsWith($prefix)} | ForEach-Object {$_.Substring($prefix.Length) | ConvertFrom-Json})
        $new=@(Get-Content -LiteralPath (Join-Path $logs 'tests.log') -Encoding utf8 | Where-Object {$_.StartsWith($prefix)} | ForEach-Object {$_.Substring($prefix.Length) | ConvertFrom-Json})
        if($old.Count -ne 11 -or $new.Count -ne 11){throw 'Missing complete semantic comparison rows'}
        for($i=0;$i -lt 11;$i++){
            if(($old[$i] | ConvertTo-Json -Depth 100 -Compress) -cne ($new[$i] | ConvertTo-Json -Depth 100 -Compress)){throw "Semantic identity differs at row $i : $logs"}
        }
        [pscustomobject]@{baseline=$BeforeEvidence;matchedCases=11;normalization='temporary root paths; normalized physical-owner binary validated against exact actual persisted bytes; actual cache breakers independently validated'} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $logs 'comparison.json') -Encoding utf8NoBOM
        Write-Output 'Unchanged budget GREEN; eleven complete durable/public semantic rows match.'
    }
} finally {Pop-Location}
