# Agent MaxHeadRoom: independent genuine RED before accessor/retrieval changes.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_BINARY_PUBLIC_ORACLE
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/binary-reverse-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    Write-Output "Reverse-map $Phase evidence: $logs"
    $tests=@('src/tests/mcp/binary_reverse_budget_af_perf_002.rs','src/tests/mcp/binary_clone_public_af_perf_002.rs',
        'src/tests/mcp/default_operation_work_af_perf_002.rs','src/tests/architecture/default_operation_observations_af_perf_002.rs',
        'src/tests/ir/original_binary_reverse_af_perf_002.rs','src/tests/ir/binary_reverse_original_decode_af_perf_002.rs',
        'src/tests/ir/binary_reverse_equivalence_af_perf_002.rs','src/tests/ir/binary_reverse_design_af_perf_002.rs')
    $frozen=$tests+@('src/tests/ir/binary_clone_equivalence_af_perf_002.rs','src/tests/ir/binary_clone_fixtures_af_perf_002.rs',
        'src/tests/ir/binary_reverse_fixtures_af_perf_002.rs','src/tests/mcp/fixtures/binary_clone_public_identities.json',
        'src/ir/binary_wire/decode.rs','src/ir/wire.rs')
    if($Phase -eq 'After') {
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'binary-reverse-before-*'|Where-Object{Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original reconstruction RED required'}
        $prior=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        if($prior.semanticPasses -ne 3 -or $prior.budgetExit -eq 0 -or $prior.publicCases -ne 40){throw 'Invalid original RED receipt'}
        foreach($file in $frozen){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Frozen drift: $file"}}
    }
    & rustfmt --edition 2024 --config skip_children=true @tests src/lib.rs src/ir/string_table.rs src/ir/binary_wire.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Formatting failed'}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw 'Size failed'}
    if($Phase -eq 'After'){foreach($file in $frozen){if((Get-FileHash $file).Hash -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Post-format frozen drift: $file"}}}
    foreach($file in $frozen){Copy-Item -LiteralPath $file -Destination (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    Copy-Item src/ir/binary_wire.rs (Join-Path $logs 'production-encoder.rs')
    Copy-Item src/ir/string_table.rs (Join-Path $logs 'production-string-table.rs')
    $env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $repoPath 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
    foreach($filter in @('af002_binary_frozen_semantics','af002_binary_reverse_frozen_equivalence','af002_binary_public_identity')) {
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Prerequisite semantic GREEN missing: $log"}
        Write-Output "$filter : PASS"
    }
    $log=Join-Path $logs 'budget.log'
    & cargo test --lib --all-features --quiet af002_binary_reverse_public_budget -- --test-threads=1 --nocapture *> $log
    $budgetExit=$LASTEXITCODE;$text=[IO.File]::ReadAllText($log)
    $rows=@([IO.File]::ReadAllLines($log)|Where-Object{$_.StartsWith('AF002_BINARY_PUBLIC ')}|ForEach-Object{$_.Substring('AF002_BINARY_PUBLIC '.Length)|ConvertFrom-Json})
    if($rows.Count -ne 40){throw 'Incomplete40 public cases; not optimization RED'}
    if($Phase -eq 'Before') {
        if($budgetExit -eq 0 -or $text -notmatch 'Binary encoder redundant reverse-map budget' -or $text -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw 'Intended original reconstruction RED missing'}
    } elseif($budgetExit -ne 0 -or $text -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Repaired reconstruction budget GREEN missing'}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-binary-reverse-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticPasses=3;budgetExit=$budgetExit;publicCases=40;workRows=$rows;files=$files}|
        ConvertTo-Json -Depth 50|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Expected $Phase result verified; full evidence: $logs"
} finally {$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle;Pop-Location}
