param([string]$WorkspaceRoot = (Get-Location).Path)
# Operator-owned B9-R1 focused tests/build/production refresh. Full gates are CI-only.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = Join-Path $root ('target/pcg-b9-r1/delivery-refresh-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$results = [Collections.Generic.List[object]]::new()
Push-Location $root
try {
    $steps = @(
        @{name='pcg-regressions';arguments=@('test','--lib','--all-features','pcg::documents::','--','--nocapture')},
        @{name='registered-knowledge-mcp';arguments=@('test','--lib','--all-features','pcg_documents::tests','--','--nocapture')},
        @{name='production-build';arguments=@('build','--all-features','--bin','clean-ctx')}
    )
    foreach ($step in $steps) {
        & cargo @($step.arguments) 2>&1 | Tee-Object (Join-Path $output ($step.name + '.log'))
        $code = $LASTEXITCODE
        $results.Add(@{step=$step.name;arguments=$step.arguments;exit_code=$code})
        [IO.File]::WriteAllText((Join-Path $output 'results.json'), (ConvertTo-Json -InputObject $results.ToArray() -Depth 8), $utf8)
        if ($code -ne 0) { throw "Focused step $($step.name) failed; subsequent steps were not run." }
    }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $hash = (Get-FileHash $binary).Hash.ToLowerInvariant()
    $identity = @{
        agent='Agent MaxHeadRoom';branch=(& git branch --show-current);head=(& git rev-parse HEAD)
        binary=$binary;binary_sha256=$hash
        profile_sha256=(Get-FileHash .clean-ctx/pcg-documents/profile.json).Hash.ToLowerInvariant()
        dirty_paths=@(& git status --porcelain=v1)
        scope='focused B9-R1 tests and production refresh; full gates are CI-only'
    }
    [IO.File]::WriteAllText((Join-Path $output 'identity.json'), ($identity | ConvertTo-Json -Depth 8), $utf8)
    & (Join-Path $PSScriptRoot 'invoke-production-verification.ps1') `
        -WorkspaceRoot $root -OutputRoot (Join-Path $output 'production') -ExpectedBinarySha256 $hash
    Write-Output "B9-R1 focused delivery refresh complete: $output"
} finally {
    Pop-Location
}
