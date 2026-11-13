param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot
)
# User-owned format/test/lint/build batch. Agent must not start this script.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output=[IO.Path]::GetFullPath($OutputRoot)
if(Test-Path -LiteralPath $output){throw 'Use a fresh B9-R1 verification directory.'}
New-Item -ItemType Directory -Path $output|Out-Null
$utf8=[Text.UTF8Encoding]::new($false,$true)
$frozenHash='fbe348e3d0ccdb200d300fd5374cbcc66c598d64e4dfcd7bedd092cbf74e21e2'
$baseBinaryHash='74e1be0de109610a35e06c1f369f4662aed0af48c74d44be941fa38a1ac4cd81'
Push-Location $root
try {
    if((Get-FileHash target/debug/clean-ctx.exe).Hash.ToLowerInvariant() -ne $baseBinaryHash){throw 'Baseline executable changed; report identity before rebuilding.'}
    Copy-Item target/debug/clean-ctx.exe (Join-Path $output 'accepted-b10-clean-ctx.exe')
    $files=@(
        'src/mcp/tool_handlers/pcg_documents.rs','src/pcg/documents/knowledge.rs',
        'src/pcg/documents/knowledge_query.rs','src/pcg/documents/knowledge_store.rs',
        'src/pcg/documents/knowledge_acquisition.rs','src/pcg/documents/knowledge_source.rs',
        'src/pcg/documents/mod.rs','src/tests/pcg/knowledge_cases.rs',
        'src/tests/pcg/knowledge_reference_first.rs','src/tests/pcg/knowledge_acquisition_cases.rs',
        'src/tests/mcp/pcg_knowledge.rs','src/tests/mcp/pcg_knowledge_schema.rs'
    )
    & rustfmt --edition 2024 --config skip_children=true @files 2>&1|Tee-Object (Join-Path $output 'format.log')
    if($LASTEXITCODE -ne 0){throw 'Scoped formatting failed.'}
    if((Get-FileHash src/tests/pcg/knowledge_reference_first.rs).Hash.ToLowerInvariant() -ne $frozenHash){throw 'Frozen RED test bytes changed; stop and report before tests.'}
    $steps=@(
        @{name='reference-green';args=@('test','--lib','--all-features','reference_first','--','--nocapture')},
        @{name='document-regressions';args=@('test','--lib','--all-features','pcg::documents::','--','--nocapture')},
        @{name='registered-mcp';args=@('test','--lib','--all-features','pcg_documents::tests','--','--nocapture')},
        @{name='clippy';args=@('clippy','-p','clean-ctx','--all-targets','--all-features','--','-D','warnings')},
        @{name='production-build';args=@('build','--bin','clean-ctx','--all-features')}
    )
    $results=[Collections.Generic.List[object]]::new()
    foreach($step in $steps){
        $arguments=$step.args
        Write-Output "Running $($step.name): cargo $($arguments -join ' ')"
        & cargo @arguments 2>&1|Tee-Object (Join-Path $output "$($step.name).log")
        $code=$LASTEXITCODE
        $results.Add([ordered]@{name=$step.name;arguments=$arguments;exit_code=$code})
        [IO.File]::WriteAllText((Join-Path $output 'steps.json'),(ConvertTo-Json -InputObject $results.ToArray() -Depth 8),$utf8)
        if($code -ne 0){throw "Step $($step.name) failed ($code); later steps were not run."}
    }
    $identity=[ordered]@{head=(& git rev-parse HEAD);source_state='working tree; not a committed-delivery claim';binary_sha256=(Get-FileHash target/debug/clean-ctx.exe).Hash.ToLowerInvariant();profile_sha256=(Get-FileHash .clean-ctx/pcg-documents/profile.json).Hash.ToLowerInvariant();frozen_red_test_sha256=$frozenHash;scope='focused tests and scoped Clippy/build; full repository gate remains separate'}
    [IO.File]::WriteAllText((Join-Path $output 'build-identity.json'),($identity|ConvertTo-Json -Depth 4),$utf8)
    Write-Output "Focused B9-R1 verification complete: $output"
} finally {Pop-Location}
