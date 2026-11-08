# Agent MaxHeadRoom: genuine original encoder clone RED and frozen public identities.
param([ValidateSet('Before','After')][string]$Phase='Before')
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$repoPath=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$oldOracle=$env:AF002_BINARY_PUBLIC_ORACLE
function Frozen-Hash([string]$File) {
    if($File -eq 'src/tests/ir/original_binary_encoder_af_perf_002.rs') {
        $adapted=[IO.File]::ReadAllText($File).Replace('binary_clone_original_decode_af_perf_002.rs','../../ir/binary_wire/decode.rs')
        $bytes=[Text.UTF8Encoding]::new($false).GetBytes($adapted)
        return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
    }
    return (Get-FileHash -LiteralPath $File).Hash
}
Push-Location $repoPath
try {
    $logs=Join-Path $repoPath ('target/af-perf-002/binary-clone-'+$Phase.ToLower()+'-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Path $logs|Out-Null
    Write-Output "Binary clone $Phase evidence: $logs"
    $tests=@('src/tests/ir/original_binary_encoder_af_perf_002.rs',
        'src/tests/ir/binary_clone_fixtures_af_perf_002.rs','src/tests/ir/binary_encoder_missing_probe_af_perf_002.rs',
        'src/tests/ir/binary_clone_equivalence_af_perf_002.rs','src/tests/mcp/binary_clone_public_af_perf_002.rs',
        'src/tests/mcp/default_operation_work_af_perf_002.rs','src/tests/architecture/default_operation_observations_af_perf_002.rs')
    $dependencies=@('src/ir/binary_wire/decode.rs','src/ir/string_table.rs','src/ir/wire.rs',
        'src/tests/mcp/apply_edit_transaction.rs','src/tests/mcp/delta_fidelity_persistence.rs')
    $frozen=@($tests)+@($dependencies)
    $protected=@('src/diff/differ.rs','src/diff/overload_lookup.rs','src/edit/locate.rs','src/edit/locate/end_join.rs')
    $protectedBefore=@{}
    foreach($file in $protected){$protectedBefore[$file]=(Get-FileHash $file).Hash}
    if($Phase -eq 'After') {
        $before=(Get-ChildItem target/af-perf-002 -Directory -Filter 'binary-clone-before-*'|
            Where-Object {Test-Path (Join-Path $_.FullName 'receipt.json')}|Sort-Object Name -Descending|Select-Object -First 1).FullName
        if(-not $before){throw 'Genuine original RED required before production ownership change'}
        $original=Get-Content (Join-Path $before 'receipt.json') -Raw -Encoding utf8|ConvertFrom-Json
        if($original.semanticExit -ne 0 -or $original.publicExit -ne 0 -or $original.budgetExit -eq 0 -or $original.publicCases -ne 40){throw 'Invalid original RED receipt'}
        foreach($file in $frozen){if((Frozen-Hash $file) -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Frozen source drift: $file"}}
    }
    & rustfmt --edition 2024 --config skip_children=true @tests src/ir/binary_wire.rs src/lib.rs *> (Join-Path $logs 'format.log')
    if($LASTEXITCODE -ne 0){throw "Format failed: $logs"}
    & pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1 *> (Join-Path $logs 'sizes.log')
    if($LASTEXITCODE -ne 0){throw "Size failed: $logs"}
    if($Phase -eq 'After') {
        foreach($file in $frozen){if((Frozen-Hash $file) -ne (Get-FileHash (Join-Path $before ([IO.Path]::GetFileName($file)))).Hash){throw "Post-format frozen source drift: $file"}}
    }
    foreach($file in $frozen){Copy-Item -LiteralPath $file -Destination (Join-Path $logs ([IO.Path]::GetFileName($file)))}
    Copy-Item src/ir/binary_wire.rs (Join-Path $logs 'production-source.rs')
    $env:AF002_BINARY_PUBLIC_ORACLE=if($Phase -eq 'After'){Join-Path $before 'public-identities.json'}else{$null}
    & cargo test --lib --all-features --quiet af002_binary_frozen_semantics -- --test-threads=1 --nocapture *> (Join-Path $logs 'semantics.log')
    $semanticExit=$LASTEXITCODE
    if($semanticExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'semantics.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Frozen binary GREEN missing: $logs"}
    $binaryRows=@([IO.File]::ReadAllLines((Join-Path $logs 'semantics.log'))|Where-Object {$_.StartsWith('AF002_BINARY_CASE ')})
    if($binaryRows.Count -lt 37){throw 'Incomplete binary opcode/payload cases'}
    $binaryText=$binaryRows -join "`n"
    [IO.File]::WriteAllText((Join-Path $logs 'binary-cases.txt'),$binaryText,[Text.UTF8Encoding]::new($false))
    if($Phase -eq 'After' -and $binaryText -cne [IO.File]::ReadAllText((Join-Path $before 'binary-cases.txt'))){throw 'Frozen complete binary outputs changed'}
    foreach($filter in @('registered_edit_is_byte_exact_and_commits_source_durable_and_live_together',
        'explicit_delta_missing_baseline_preserves_high_fidelity_across_restart_restore')) {
        $log=Join-Path $logs ($filter+'.log')
        & cargo test --lib --all-features --quiet $filter -- --test-threads=1 --nocapture *> $log
        if($LASTEXITCODE -ne 0 -or [IO.File]::ReadAllText($log) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Existing encoder caller control failed: $log"}
    }
    & cargo test --lib --all-features --quiet af002_binary_public_identity -- --test-threads=1 --nocapture *> (Join-Path $logs 'public.log')
    $publicExit=$LASTEXITCODE
    if($publicExit -ne 0 -or [IO.File]::ReadAllText((Join-Path $logs 'public.log')) -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Frozen registered public GREEN missing: $logs"}
    $publicRows=@([IO.File]::ReadAllLines((Join-Path $logs 'public.log'))|Where-Object {$_.StartsWith('AF002_BINARY_PUBLIC ')}|ForEach-Object {$_.Substring('AF002_BINARY_PUBLIC '.Length)|ConvertFrom-Json})
    if($publicRows.Count -ne 40){throw 'Incomplete public identity rows'}
    $identities=@($publicRows|ForEach-Object {$_.identity})
    $identities|ConvertTo-Json -Depth 50|Set-Content (Join-Path $logs 'public-identities.json') -Encoding utf8NoBOM
    if($Phase -eq 'Before'){$env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $logs 'public-identities.json'}
    & cargo test --lib --all-features --quiet af002_binary_public_budget -- --test-threads=1 --nocapture *> (Join-Path $logs 'budget.log')
    $budgetExit=$LASTEXITCODE
    $budgetText=[IO.File]::ReadAllText((Join-Path $logs 'budget.log'))
    Get-Content (Join-Path $logs 'budget.log') -Tail 9
    $workRows=@([IO.File]::ReadAllLines((Join-Path $logs 'budget.log'))|Where-Object {$_.StartsWith('AF002_BINARY_PUBLIC ')}|ForEach-Object {$_.Substring('AF002_BINARY_PUBLIC '.Length)|ConvertFrom-Json})
    if($workRows.Count -ne 40){throw 'Incomplete public budget rows; not intended RED'}
    if($Phase -eq 'Before') {
        if($budgetExit -eq 0 -or $budgetText -notmatch 'Binary encoder owned-vector clone budget' -or $budgetText -notmatch 'test result: FAILED\. 0 passed; 1 failed; 0 ignored;'){throw "Intended original RED missing: $logs"}
    } elseif($budgetExit -ne 0 -or $budgetText -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw "Repaired budget GREEN missing: $logs"}
    foreach($file in $protected){if((Get-FileHash $file).Hash -ne $protectedBefore[$file]){throw "Protected identity changed: $file"}}
    $paths=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/live-acceptance/issue-162/run-binary-clone-budget.ps1)
    if($LASTEXITCODE -ne 0){throw 'Inventory failed'}
    $files=@($paths|Sort-Object -Unique|ForEach-Object {@{path=$_;sha256=(Get-FileHash $_).Hash}})
    @{agent='Agent MaxHeadRoom';phase=$Phase;semanticExit=$semanticExit;publicExit=$publicExit;
        budgetExit=$budgetExit;publicCases=40;binaryCases=$binaryRows.Count;callerControlPasses=2;workRows=$workRows;files=$files;protectedIdentities=$protectedBefore}|
        ConvertTo-Json -Depth 50|Set-Content (Join-Path $logs 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Expected $Phase result verified; not final acceptance: $logs"
} finally {$env:AF002_BINARY_PUBLIC_ORACLE=$oldOracle; Pop-Location}
