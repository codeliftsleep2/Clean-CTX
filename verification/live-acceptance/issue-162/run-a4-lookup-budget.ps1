# Agent MaxHeadRoom: original RED must precede production lookup edits.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/a4-lookup-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs | Out-Null
    Write-Output "A4 $Phase evidence: $logs"
    $changed=@('src/tests/ir/a4_lookup_budget_af_perf_002.rs','src/tests/ir/a4_original_obligations_af_perf_002.rs','src/tests/mcp/dependency_footprint_af_perf_002.rs','src/tests/architecture/work_observations_af_perf_002.rs')
    if($Phase -eq 'After'){
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'a4-lookup-before-*' | Where-Object {Test-Path (Join-Path $_.FullName 'receipt.json')} | Sort-Object Name -Descending | Select-Object -First 1).FullName
        if(-not $before){throw 'Verified original Before evidence required'}
        $original=Get-Content (Join-Path $before 'receipt.json') -Raw | ConvertFrom-Json
        if($original.phase -ne 'Before' -or $original.results[0].exit -ne 0 -or $original.results[1].exit -eq 0 -or $original.results[2].exit -eq 0){throw 'Invalid original RED receipt'}
        foreach($file in $changed[0..2]){
            $snapshot=Join-Path $before ([IO.Path]::GetFileName($file))
            if($file -eq $changed[0]){
                $frozen=[IO.File]::ReadAllText($snapshot)
                $current=[IO.File]::ReadAllText((Join-Path $repoPath $file))
                if($current -cne $frozen.Replace('mod original;','pub(super) mod original;')){throw 'Budget differs beyond oracle module sharing visibility'}
            } elseif((Get-FileHash -LiteralPath $file).Hash -ne (Get-FileHash -LiteralPath $snapshot).Hash){throw "Original budget/oracle changed: $file"}
        }
        $changed+=@('src/ir/csharp_resolution/compilation/dependencies.rs','src/ir/csharp_resolution/compilation/dependency_inputs.rs')
    }
    & rustfmt --edition 2024 --config skip_children=true @changed *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-a4-lookup-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($paths | Sort-Object -Unique | ForEach-Object{[pscustomobject]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    Copy-Item -LiteralPath src/ir/csharp_resolution/compilation/dependencies.rs -Destination (Join-Path $logs 'production-source.rs')
    foreach($file in $changed){Copy-Item -LiteralPath $file -Destination (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    $cases=@(
        @{filter='af002_a4_lookup_frozen_equivalence';log='semantics.log';red=''},
        @{filter='af002_a4_lookup_construction_budget';log='budget.log';red='A4 redundant full-input retrieval budget'},
        @{filter='af002_a4_public_lookup_budget';log='public.log';red='A4 redundant public retrieval budget'}
    )
    $results=@()
    foreach($case in $cases){
        $log=Join-Path $logs $case.log
        & cargo test --lib --all-features --quiet $case.filter -- --test-threads=1 --nocapture *> $log
        $code=$LASTEXITCODE
        $observed=[IO.File]::ReadAllText($log)
        $results+=@{filter=$case.filter;exit=$code}
        Get-Content -LiteralPath $log -Tail 8
        if($Phase -eq 'Before' -and $case.red){
            if($code -eq 0 -or $observed -notmatch [regex]::Escape($case.red) -or $observed -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Missing intended RED: $log"}
            $prefix=if($case.log -eq 'budget.log'){'AF002_A4_BUDGET '}else{'AF002_A4 '}
            $expected=if($case.log -eq 'budget.log'){10}else{57}
            $rows=@(Get-Content -LiteralPath $log | Where-Object {$_.StartsWith($prefix)})
            if($rows.Count -ne $expected){throw "Incomplete semantic/work rows: $log"}
        } else {
            if($code -ne 0 -or $observed -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Required GREEN missing: $log"}
        }
    }
    @{agent='Agent MaxHeadRoom';phase=$Phase;results=$results;files=$hashes} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Verified expected $Phase outcomes: $logs"
} finally {Pop-Location}
