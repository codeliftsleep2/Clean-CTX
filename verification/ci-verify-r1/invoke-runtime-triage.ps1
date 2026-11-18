# Operator-owned fixed diagnostic batches. No full-suite execution.
param([switch]$OptimizedDependenciesTrial,[switch]$OptimizedRootTrial)
if($OptimizedRootTrial){$OptimizedDependenciesTrial=$true}
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/runtime-triage-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
$runs=[Collections.Generic.List[object]]::new()
function Invoke-Capture([string]$name,[string]$exe,[string[]]$argumentsForRun) {
    $info=[Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$exe
    $info.WorkingDirectory=$root
    $info.UseShellExecute=$false
    $info.CreateNoWindow=$true
    $info.RedirectStandardOutput=$true
    $info.RedirectStandardError=$true
    foreach($argument in $argumentsForRun){$info.ArgumentList.Add($argument)}
    $process=[Diagnostics.Process]::new()
    $process.StartInfo=$info
    $clock=[Diagnostics.Stopwatch]::StartNew()
    Write-Output "START $name"
    [void]$process.Start()
    $stdout=$process.StandardOutput.ReadToEndAsync()
    $stderr=$process.StandardError.ReadToEndAsync()
    $nextNotice=60
    while(-not $process.WaitForExit(1000)){
        if($clock.Elapsed.TotalSeconds -ge $nextNotice){
            Write-Output "STILL RUNNING $name : $([int]$clock.Elapsed.TotalSeconds)s (no timeout or test suppression)"
            $nextNotice+=60
        }
    }
    $clock.Stop()
    $utf8=[Text.UTF8Encoding]::new($false)
    $stdoutPath=Join-Path $out ($name+'.stdout.txt')
    $stderrPath=Join-Path $out ($name+'.stderr.txt')
    [IO.File]::WriteAllText($stdoutPath,$stdout.GetAwaiter().GetResult(),$utf8)
    [IO.File]::WriteAllText($stderrPath,$stderr.GetAwaiter().GetResult(),$utf8)
    $runs.Add(@{name=$name;exe=$exe;arguments=$argumentsForRun;seconds=$clock.Elapsed.TotalSeconds;exitCode=$process.ExitCode;stdout_sha256=(Get-FileHash $stdoutPath).Hash;stderr_sha256=(Get-FileHash $stderrPath).Hash})
    @{category='Focused per-test elapsed-time diagnosis, not CI or empirical production acceptance';head=(& git -C $root rev-parse HEAD);runs=$runs.ToArray()}|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    $code=$process.ExitCode
    $process.Dispose()
    Write-Output "END $name : $($clock.Elapsed.TotalSeconds)s, exit=$code"
    if($code -ne 0){Get-Content $stderrPath -Tail 40;Get-Content $stdoutPath -Tail 40;throw "Failure; preserve $out"}
}
Push-Location $root
try {
    $sources=@(git ls-files --cached --others --exclude-standard -- src Cargo.toml Cargo.lock verification/ci-verify-r1 scripts)
    @($sources|Sort-Object -Unique|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})|ConvertTo-Json -Depth 5|Set-Content (Join-Path $out 'sources.json') -Encoding utf8NoBOM
    $compileArguments=@('test','--lib','--all-features','--no-run','--message-format=json')
    if($OptimizedDependenciesTrial){
        # Root crate, debug assertions, overflow checks and test selection stay unchanged.
        # Trial only: no Cargo.toml or repository profile default is modified.
        $compileArguments+=@('--config','profile.test.package."*".opt-level=3')
    }
    if($OptimizedRootTrial){$compileArguments+=@('--config','profile.test.opt-level=1')}
    Invoke-Capture 'compile' 'cargo' $compileArguments
    $artifacts=@(Get-Content (Join-Path $out 'compile.stdout.txt')|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.executable -and $_.profile.test -and $_.target.name -eq 'clean_ctx' -and $_.target.kind -contains 'lib'})
    if($artifacts.Count -ne 1){throw 'Expected one actual Cargo library test executable'}
    $binary=$artifacts[0].executable
    @{path=$binary;sha256=(Get-FileHash -LiteralPath $binary).Hash;rustflags=$env:RUSTFLAGS;profile=$artifacts[0].profile;dependency_optimization_trial=[bool]$OptimizedDependenciesTrial;root_optimization_trial=[bool]$OptimizedRootTrial;features='all-features'}|ConvertTo-Json|Set-Content (Join-Path $out 'binary.json') -Encoding utf8NoBOM
    Invoke-Capture 'listing' $binary @('--list')
    $listing=Get-Content (Join-Path $out 'listing.stdout.txt')
    $tests=@(
        'mcp::tool_handlers::core::csharp_skeleton_degradation_tests::csharp_structural_fidelities_declare_bounded_semantic_loss',
        'mcp::tool_handlers::core::delta::delta_design::repair::af002_delta_previous_frozen_semantics',
        'mcp::tool_handlers::core::delta::delta_design::integrated::af002_delta_previous_integrated_cost',
        'mcp::tool_handlers::core::delta::delta_design::cases::af002_delta_previous_persistent',
        'artifact_operation_work_af_perf_002::dependency_footprint::af002_a4_public_lookup_budget'
    )
    foreach($removed in @(
        'mcp::tool_handlers::core::delta::delta_design::repair::af002_delta_previous_unused_budget',
        'artifact_operation_work_af_perf_002::dependency_footprint::af002_a4_default_footprint_dimensions'
    )){
        if($listing -ccontains ($removed+': test')){throw "Duplicate corpus still registered: $removed"}
    }
    if($OptimizedDependenciesTrial){
        $tests=@($tests|Where-Object{$_ -match 'af002_delta_previous_frozen_semantics|af002_a4_public_lookup_budget'})
        if($tests.Count -ne 2){throw 'Expected two exact unchanged slow witnesses'}
    }
    $index=0
    foreach($test in $tests){
        if(@($listing|Where-Object{$_ -ceq ($test+': test')}).Count -ne 1){throw "Missing/duplicate registration: $test"}
        $index++
        Write-Output "TEST $test"
        Invoke-Capture ('test-'+$index) $binary @('--exact',$test,'--test-threads=1','--format=pretty')
        $log=Join-Path $out ('test-'+$index+'.stdout.txt')
        if(-not (Select-String -LiteralPath $log -SimpleMatch -Pattern 'test result: ok. 1 passed; 0 failed; 0 ignored;' -Quiet)){throw "Unexpected selection; preserve $out"}
    }
    Invoke-Capture 'membership-enforcement' $binary @('--exact','ci_verify_r1::ci_verify_r1_required_execution_and_failure_contracts','--test-threads=1','--format=pretty')
    if(-not (Select-String -LiteralPath (Join-Path $out 'membership-enforcement.stdout.txt') -SimpleMatch -Pattern 'test result: ok. 1 passed; 0 failed; 0 ignored;' -Quiet)){throw 'Missing membership enforcement execution'}
    Write-Output "Return runtime triage receipts: $out"
} finally {Pop-Location}
