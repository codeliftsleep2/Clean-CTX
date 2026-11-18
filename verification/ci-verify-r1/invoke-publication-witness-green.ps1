# Operator-owned focused frozen-source repair verification; no full suite.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/publication-witness-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    $protected=@{
        'src/mcp/state/publication.rs'='b9bc08c1b22d0ce4f757d3c9229dc067ae1524462bb429c8b4bc044b7e219cae'
        'src/mcp/state/source_cache.rs'='1a6de2005ec48fd5cc6217fcd4a5d75c794a73533aac9a6221972e576a67f5c1'
        'src/mcp/tool_helpers.rs'='bfda853b86811854966818b4471eccba98fe1790f26a6b6466099e71e21b02c7'
    }
    function Assert-ProtectedSources {
        foreach($path in $protected.Keys){
            $text=[IO.File]::ReadAllText((Join-Path $root $path)).Replace("`r`n","`n")
            $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($text))).ToLowerInvariant()
            if($hash -cne $protected[$path]){throw "Frozen source mismatch: $path"}
        }
    }
    Assert-ProtectedSources
    & rustfmt --edition 2024 --config skip_children=true src/tests/mcp/semantic_publication_clock_support.rs
    if($LASTEXITCODE -ne 0){throw 'Focused regression formatting failed'}
    Assert-ProtectedSources
    $newTest='mcp::state::publication::snapshot_test_support::ci_runtime_publication_hooks_preserve_owner_order_and_failure'
    $frozenTest='mcp::tool_handlers::core::delta::delta_design::integrated::afperf_delta_previous_static'
    $runs=@(
        @{name='all-features';arguments=@('test','--lib','--all-features','--','--exact','--test-threads=1',$frozenTest,$newTest);expectedTests=2},
        @{name='csharp-only';arguments=@('test','--lib','--no-default-features','--features','csharp','--','--exact','--test-threads=1',$newTest);expectedTests=1},
        @{name='scoped-clippy';arguments=@('clippy','--lib','--tests','--all-features','--','-D','warnings');expectedTests=0}
    )
    $sourcePaths=@('src/mcp/state/publication.rs','src/tests/mcp/semantic_publication_clock_support.rs','src/tests/mcp/delta_previous_integrated_af_perf_002.rs')
    $pins=@($sourcePaths|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    $results=[Collections.Generic.List[object]]::new()
    foreach($run in $runs){
        $log=Join-Path $out ($run.name+'.log')
        $argumentsForRun=$run.arguments
        & cargo @argumentsForRun *> $log
        $code=$LASTEXITCODE
        $complete=$code -eq 0
        if($run.expectedTests){$summary="test result: ok. $($run.expectedTests) passed; 0 failed; 0 ignored;";$complete=$complete -and (Select-String -LiteralPath $log -SimpleMatch -Pattern $summary -Quiet)}
        $warnings=[bool](Select-String -LiteralPath $log -Pattern '^warning:' -Quiet)
        $results.Add(@{name=$run.name;arguments=$argumentsForRun;exitCode=$code;complete_selection=$complete;warnings=$warnings;log_sha256=(Get-FileHash -LiteralPath $log).Hash})
        @{category='Focused tracked regression/lint, not full CI';head=(git rev-parse HEAD);sources=$pins;runs=$results.ToArray()}|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 30
        if(-not $complete -or $warnings){throw "$($run.name) failed; preserve $out"}
    }
    Assert-ProtectedSources
    foreach($pin in $pins){if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw 'Source drift during focused verification'}}
    Write-Output "Return focused frozen-witness, publication ordering and lint receipts: $out"
} finally {Pop-Location}
