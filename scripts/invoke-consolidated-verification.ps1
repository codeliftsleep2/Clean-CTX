# Agent MaxHeadRoom: CI-owned full correctness execution, once per Cargo target.
# Operator-owned focused checks remain separate. This is not a performance certificate.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$out=Join-Path $root ('target/issue-162/af-perf-001-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out | Out-Null
$names=@('AF002_BINARY_PUBLIC_ORACLE','AF002_SQLITE_TEXT_ORACLE',
    'AF002_MANIFEST_FIXTURE_ROOT','AF002_MANIFEST_ORACLE','AF002_DIFF_SOURCE_ROOT',
    'AF002_DIFF_SOURCE_ORACLE','AF002_DELTA_PREVIOUS_ROOT','AF002_DELTA_PREVIOUS_ORACLE')
$prior=@{}
foreach($name in $names){$prior[$name]=[Environment]::GetEnvironmentVariable($name)}
Push-Location $root
try {
    $env:AF002_BINARY_PUBLIC_ORACLE=Join-Path $root 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
    $env:AF002_SQLITE_TEXT_ORACLE=Join-Path $root 'src/tests/mcp/fixtures/sqlite_text_public_identities.json'
    # Preserve the standalone runner's optional historical local replay configuration.
    foreach($item in @(
        @('manifest-demand-before-20261108-231514','AF002_MANIFEST_FIXTURE_ROOT','AF002_MANIFEST_ORACLE'),
        @('diff-source-before-20261109-103157','AF002_DIFF_SOURCE_ROOT','AF002_DIFF_SOURCE_ORACLE'),
        @('delta-previous-before-20261109-134928','AF002_DELTA_PREVIOUS_ROOT','AF002_DELTA_PREVIOUS_ORACLE')
    )){
        $folder=Join-Path $root ('target/af-perf-002/'+$item[0])
        if(Test-Path (Join-Path $folder 'public-oracle.json')){
            [Environment]::SetEnvironmentVariable($item[1],(Join-Path $folder 'fixtures'))
            [Environment]::SetEnvironmentVariable($item[2],(Join-Path $folder 'public-oracle.json'))
        }
    }
    $environment=@{}
    $oracleIdentities=@{}
    foreach($name in $names){
        $value=[Environment]::GetEnvironmentVariable($name)
        $environment[$name]=$value
        if($value -and -not (Test-Path -LiteralPath $value)){throw "Missing configured oracle/fixture: $name"}
        if($value -and $name.EndsWith('_ORACLE')){
            $oracle=ConvertFrom-Json -InputObject (Get-Content -LiteralPath $value -Raw) -NoEnumerate
            if($oracle -isnot [array] -or $oracle.Count -eq 0){throw "Empty/invalid configured oracle: $name"}
            $oracleIdentities[$name]=@{path=$value;sha256=(Get-FileHash -LiteralPath $value).Hash}
        }
    }
    # CI-only runtime configuration, measured by the retained focused trials.
    # Do not change local development or explicit empirical-study profiles.
    $testProfile=@('--config','profile.test.opt-level=1',
        '--config','profile.test.package."*".opt-level=3',
        '--config','profile.test.debug-assertions=true',
        '--config','profile.test.overflow-checks=true')
    $inventory=@(git ls-files --cached --others --exclude-standard -- src proxy Cargo.toml Cargo.lock scripts verification/ci-verify-r1 .github/workflows docs/agent/verification.md docs/agent/CI_VERIFY_R1.md docs/architecture/knowledge-governance/corpus.csv)
    if($LASTEXITCODE -ne 0){throw 'Source inventory failed'}
    $hashes=@($inventory|Sort-Object -Unique|Where-Object{Test-Path -LiteralPath $_ -PathType Leaf}|ForEach-Object{
        @{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}
    })
    [ordered]@{
        agent='Agent MaxHeadRoom';head=(git rev-parse HEAD);status=(git status --porcelain)
        features='all-features';test_threads=1;environment=$environment;files=$hashes
        oracle_identities=$oracleIdentities
        test_profile_arguments=$testProfile
        listing_command='cargo test --workspace --all-targets --all-features <test_profile_arguments> -- --list'
        execution_command='cargo test --workspace --all-targets --all-features --no-fail-fast <test_profile_arguments> -- --test-threads=1 --format=pretty'
        rustflags=$env:RUSTFLAGS;encoded_rustflags=$env:CARGO_ENCODED_RUSTFLAGS
        build_target=$env:CARGO_BUILD_TARGET;target_directory=$env:CARGO_TARGET_DIR
        scope='Correctness and declared deterministic fitness only; empirical acceptance obligations remain separate.'
    }|ConvertTo-Json -Depth 6|Set-Content (Join-Path $out 'configuration.json') -Encoding utf8NoBOM
    $listing=Join-Path $out 'registration.log'
    $execution=Join-Path $out 'execution.log'
    & python (Join-Path $PSScriptRoot 'verify-ci-membership.py') --root $root --prerequisites-only --output (Join-Path $out 'prerequisites.json')
    if($LASTEXITCODE -ne 0){throw "Mandatory verification prerequisite failed; preserve $out"}
    Write-Output "Compiling and registering the full workspace suite; registration log: $listing"
    & cargo test --workspace --all-targets --all-features @testProfile -- --list *> $listing
    if($LASTEXITCODE -ne 0){throw "Registration/compilation failed; preserve $out"}
    Write-Output "Running workspace correctness and deterministic semantic fitness; live per-test results also retained in $execution"
    & cargo test --workspace --all-targets --all-features --no-fail-fast @testProfile -- --test-threads=1 --format=pretty 2>&1 | Tee-Object -FilePath $execution
    $suiteExit=$LASTEXITCODE
    & python (Join-Path $PSScriptRoot 'verify-ci-membership.py') --root $root --listing $listing --execution $execution --suite-exit $suiteExit --output (Join-Path $out 'membership-result.json')
    $membershipExit=$LASTEXITCODE
    $drift=@($hashes|Where-Object{
        -not (Test-Path -LiteralPath $_.path) -or (Get-FileHash -LiteralPath $_.path).Hash -cne $_.sha256
    })
    if($suiteExit -ne 0 -or $membershipExit -ne 0 -or $drift.Count){
        $drift|ConvertTo-Json -Depth 4|Set-Content (Join-Path $out 'source-drift.json') -Encoding utf8NoBOM
        Get-Content -LiteralPath $execution -Tail 120|Write-Output
        throw "Consolidated correctness/fitness failed; preserve $out"
    }
    # Preserve source inventory and declared scope in the existing receipt location.
    [ordered]@{
        agent='Agent MaxHeadRoom';category='Consolidated tracked correctness and deterministic fitness'
        head=(git rev-parse HEAD);files=$hashes;resumed=$false;reusedGroups=@()
        configuration_sha256=(Get-FileHash (Join-Path $out 'configuration.json')).Hash
        membership_sha256=(Get-FileHash (Join-Path $out 'membership-result.json')).Hash
        scope='Declared inspected fitness families; not empirical performance certification or full architectural acceptance.'
    }|ConvertTo-Json -Depth 6|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Consolidated correctness and semantic fitness passed: $out"
} finally {
    foreach($name in $names){[Environment]::SetEnvironmentVariable($name,$prior[$name])}
    Pop-Location
}
