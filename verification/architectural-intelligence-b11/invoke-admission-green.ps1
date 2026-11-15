param([string]$WorkspaceRoot = (Get-Location).Path)
# Operator-owned focused B11 verification. No full repository gate.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = Join-Path $root ('target/pcg-b11/admission-green-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$test = Join-Path $root 'src/tests/pcg/document_admission_diagnostics.rs'
$expected = '227f736d9174bf8b60f3bad20310a1f877f3d840817a84ad76d19a506e0e8dd9'
if ((Get-FileHash $test).Hash.ToLowerInvariant() -ne $expected) { throw 'Frozen diagnostic RED tests changed.' }
$frozen = @{
    'src/tests/pcg/document_admission_rule_components.rs'='42624910d2a95c30f29dfc3f9860491f2c1af7bd7ac55153fc2c55c7cdc56666'
    'src/tests/pcg/document_governance_lifecycle.rs'='4f4bdd090e61be292011aa2bb1781dff19a74a950d5bbe3ac6f3f5269c2b8266'
    'src/tests/pcg/document_governance_operator.rs'='2f0f5f5ea0ca2b187ecc250588475b14be02d09840bd96e95192a46220612589'
    'src/tests/pcg/document_admission_authority.rs'='f23d89db2610f4be2872a07e60aecd8d3b7c06085f67d84082a5893f34fe5938'
    'src/tests/pcg/document_governed_membership.rs'='7a9cafe85a1dff140b9622a276920af7d325f6257922084ab8a1b277f44362c9'
    'src/tests/pcg/document_governance_bootstrap.rs'='341ff962a7b017167517bfbef1aae7c9ae226a1342fe6d23ea3688ff555a143b'
}
foreach ($path in $frozen.Keys) {
    if ((Get-FileHash (Join-Path $root $path)).Hash.ToLowerInvariant() -ne $frozen[$path]) { throw "Frozen RED changed: $path" }
}
Push-Location $root
try {
    $formatFiles = @('src/pcg/cli.rs','src/tests/pcg/document_governance_operator.rs','src/tests/pcg/document_governance_lifecycle.rs','src/tests/pcg/knowledge_migration.rs')
    $formatFiles += @(Get-ChildItem src/pcg/documents -File -Filter '*.rs' | ForEach-Object FullName)
    $formatFiles += 'src/tests/pcg/document_governance_associations.rs'
    $formatFiles += 'src/tests/pcg/document_governance_work.rs'
    & rustfmt --edition 2024 --config skip_children=true @formatFiles 2>&1 |
        Tee-Object (Join-Path $output 'format.log')
    if ($LASTEXITCODE -ne 0) { throw 'Scoped formatting failed.' }
    $steps = @(
        @{name='original-diagnostic-regressions';arguments=@('test','--lib','--all-features','b11_','--','--nocapture')},
        @{name='document-regressions';arguments=@('test','--lib','--all-features','pcg::documents::','--','--nocapture')},
        @{name='registered-mcp';arguments=@('test','--lib','--all-features','pcg_documents::tests','--','--nocapture')},
        @{name='scoped-clippy';arguments=@('clippy','-p','clean-ctx','--all-targets','--all-features','--','-D','warnings')}
    )
    $results = @()
    foreach ($step in $steps) {
        & cargo @($step.arguments) 2>&1 | Tee-Object (Join-Path $output ($step.name + '.log'))
        $code = $LASTEXITCODE
        $results += @{step=$step.name;exit_code=$code;arguments=$step.arguments}
        [IO.File]::WriteAllText((Join-Path $output 'results.json'), (ConvertTo-Json -InputObject $results -Depth 6), $utf8)
        if ($code -ne 0) { throw "Focused step $($step.name) failed; preserve its log." }
    }
    if ((Get-FileHash $test).Hash.ToLowerInvariant() -ne $expected) { throw 'Frozen diagnostic tests changed during verification.' }
    foreach ($path in $frozen.Keys) {
        if ((Get-FileHash (Join-Path $root $path)).Hash.ToLowerInvariant() -ne $frozen[$path]) { throw "Frozen RED changed: $path" }
    }
    Write-Output "Focused B11 diagnostic verification complete: $output"
} finally { Pop-Location }
