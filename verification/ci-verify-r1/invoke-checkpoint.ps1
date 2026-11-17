# Agent MaxHeadRoom: focused operator checkpoint; no full repository suite.
# Read-only B11 descriptor only. Never submits owner authorization or pushes.
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$out=Join-Path $root ('target/ci-verify-r1/checkpoint-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Write-Output "CI-VERIFY-R1 checkpoint: $out"
Push-Location $root
try {
    $files=@(git ls-files --modified --others --exclude-standard -- '*.rs')
    if($LASTEXITCODE -ne 0 -or -not $files.Count){throw 'No current Rust candidate files'}
    & rustfmt --edition 2024 --config skip_children=true @files
    if($LASTEXITCODE -ne 0){throw 'Focused formatting failed'}
    $priorPath=Join-Path $root 'target/ci-verify-r1/mixed-preservation-20261117-133714/receipt.json'
    $prior=Get-Content -LiteralPath $priorPath -Raw|ConvertFrom-Json
    if(-not $prior.passed -or $prior.exitCode -ne 0){throw 'No verified predecessor companion capture'}
    foreach($file in $prior.files){
        if((Get-FileHash -LiteralPath $file.path).Hash -cne $file.sha256){throw "Previously verified companion changed: $($file.path)"}
    }
    $priorLog=Join-Path (Split-Path $priorPath) 'tracked-tests.log'
    if((Get-FileHash $priorLog).Hash -cne $prior.log_sha256){throw 'Prior companion log drift'}
    [ordered]@{category='Retained prior three-companion execution, not a new run';receipt=$priorPath;sha256=(Get-FileHash $priorPath).Hash}|ConvertTo-Json|Set-Content (Join-Path $out 'retained-companions.json') -Encoding utf8NoBOM
    function Invoke-Step([string]$Name,[string]$Executable,[string[]]$Arguments){
        $log=Join-Path $out ($Name+'.log')
        $started=Get-Date
        & $Executable @Arguments *> $log
        $code=$LASTEXITCODE
        [ordered]@{name=$Name;executable=$Executable;arguments=$Arguments;exitCode=$code;seconds=((Get-Date)-$started).TotalSeconds;log_sha256=(Get-FileHash $log).Hash}|
            ConvertTo-Json -Depth 5|Set-Content (Join-Path $out ($Name+'-receipt.json')) -Encoding utf8NoBOM
        Get-Content -LiteralPath $log -Tail 20
        if($code -ne 0){throw "$Name failed; preserve $out"}
    }
    $selection=@(
        'diff::differ::overload_acceptance::afperf_overload_paired_cost'
        'diff::differ::overload_alternatives::integrated::af002_overload_integrated_cost'
        'diff::differ::overload_alternatives::af002_overload_alternative_costs'
        'diff::differ::overload_alternatives::refinement::af002_overload_refinement_costs'
        'diff::differ::overload_alternatives::source_forms::af002_overload_source_forms_cost'
        'binary_reverse_design_af_perf_002::af002_binary_reverse_design_cost'
        'binary_reverse_integrated_af_perf_002::af002_binary_reverse_integrated_cost'
        'mcp::tool_handlers::hydration::filesystem::inventory_work::comparison::integrated::af002_manifest_integrated_cost'
        'mcp::sqlite_store::work_af_perf_002::startup_projection_design::af002_startup_projection_costs'
        'mcp::sqlite_store::work_af_perf_002::text_design::af002_sqlite_text_design_costs'
        'ir::csharp_resolution::compilation::dependencies::a4_lookup_acceptance::afperf_a4_paired_construction_cost'
        'edit::locate::exact_end_acceptance::afperf_edit_end_construction_cost'
        'mcp::tool_handlers::core::delta::delta_design::cases::af002_delta_previous_cost'
        'mcp::tool_handlers::core::delta::ownership_design::cases::af002_diff_ownership_local_cost'
        'mcp::tool_handlers::hydration::filesystem::inventory_work::comparison::af002_proof_manifest_actual_comparison'
        'ci_verify_r1::ci_verify_r1_required_execution_and_failure_contracts'
    )
    Invoke-Step 'mandatory-companions-and-enforcement' 'cargo' (@('test','--lib','--all-features','--','--exact','--test-threads=1')+$selection)
    $testLog=Join-Path $out 'mandatory-companions-and-enforcement.log'
    if(-not (Select-String -LiteralPath $testLog -Pattern '^test result: ok\. 16 passed; 0 failed; 0 ignored;' -Quiet)){throw 'Incomplete/ignored focused selection'}
    Invoke-Step 'measurement-registration-only' 'pwsh' @('-NoProfile','-File','verification/ci-verify-r1/invoke-measurements.ps1','-ListOnly')
    Invoke-Step 'root-library-tests-clippy' 'cargo' @('clippy','--lib','--tests','--all-features','--','-D','warnings')
    Invoke-Step 'utf8' 'pwsh' @('-NoProfile','-File','scripts/check-utf8.ps1')
    Invoke-Step 'active-size' 'pwsh' @('-NoProfile','-File','scripts/check-file-sizes.ps1')
    $pins=@(git ls-files --modified --others --exclude-standard|ForEach-Object{@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash}})
    $pins|ConvertTo-Json -Depth 4|Set-Content (Join-Path $out 'candidate-sources.json') -Encoding utf8NoBOM
    $binary=Join-Path $root 'target/debug/clean-ctx.exe'
    $profile=Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    $baseline=Get-Content (Join-Path $root 'target/ci-verify-r1/authority-20261117-132802-523/baseline.json') -Raw|ConvertFrom-Json
    if((Get-FileHash $binary).Hash.ToLowerInvariant() -cne $baseline.executable_sha256 -or (Get-FileHash $profile).Hash.ToLowerInvariant() -cne $baseline.profile_sha256){throw 'Production authoring binary/profile changed; inspect actual identities'}
    $descriptor=Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptor 2> (Join-Path $out 'descriptor-stderr.txt')
    if($LASTEXITCODE -ne 0){throw "Actual B11 descriptor/check failed; preserve $out"}
    $value=Get-Content -LiteralPath $descriptor -Raw|ConvertFrom-Json
    if(-not $value.digest -or -not $value.descriptor -or $value.authorization_recorded -or $value.accepted_publication){throw "Invalid read-only descriptor; preserve $out"}
    foreach($pin in $pins){if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw "Candidate source drift: $($pin.path)"}}
    Write-Output "Return checkpoint receipts and exact descriptor for review: $out"
    Write-Output 'No authorization, publication, empirical campaign, commit or push performed.'
} finally {Pop-Location}
