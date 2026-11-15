param([string]$WorkspaceRoot = (Get-Location).Path, [string]$ReuseBuiltBinaryFrom)
# Operator-owned build and production acquisition; no real-workspace authorization or full gates.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = Join-Path $root ('target/pcg-b11/production-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function Save-Json([string]$Name, $Value) {
    [IO.File]::WriteAllText((Join-Path $output $Name), (ConvertTo-Json -InputObject $Value -Depth 100), $utf8)
}
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Unexpected branch; no build or reconciliation started.' }
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    $settings = Get-Content -LiteralPath $profile -Raw | ConvertFrom-Json
    $helper = Join-Path $PSScriptRoot 'capture-production.py'
    & python $helper snapshot $settings.database_path (Join-Path $output 'catalog-before.json')
    if ($LASTEXITCODE -ne 0) { throw 'Baseline catalog capture failed.' }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    if (Test-Path -LiteralPath $binary) {
        Copy-Item -LiteralPath $binary -Destination (Join-Path $output 'pre-build-clean-ctx.exe')
        Save-Json 'pre-build-executable.json' @{sha256=(Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant()}
    }
    $sourcePaths = @(& git diff --name-only) + @(& git ls-files --others --exclude-standard)
    $sourceHashes = @($sourcePaths | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object {
        @{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}
    })
    Save-Json 'working-source-hashes.json' $sourceHashes
    if ($ReuseBuiltBinaryFrom) {
        $previous = (Resolve-Path -LiteralPath $ReuseBuiltBinaryFrom).Path
        $previousIdentity = Get-Content -LiteralPath (Join-Path $previous 'identity.json') -Raw | ConvertFrom-Json
        $previousBuild = Get-Content -LiteralPath (Join-Path $previous 'build-result.json') -Raw | ConvertFrom-Json
        if ($previousBuild.exit_code -ne 0) { throw 'Previous production build did not pass.' }
        if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne $previousIdentity.executable_sha256) {
            throw 'Built executable changed; previous receipt cannot be reused.'
        }
        if ((& git rev-parse HEAD) -cne $previousIdentity.head) { throw 'Source HEAD changed since the build.' }
        $inputPattern = '\.rs$|(^|/)Cargo\.(toml|lock)$|(^|/)rust-toolchain(\.toml)?$|^\.cargo/'
        $beforeInputs = @(Get-Content -LiteralPath (Join-Path $previous 'working-source-hashes.json') -Raw | ConvertFrom-Json | Where-Object path -Match $inputPattern)
        $nowInputs = @($sourceHashes | Where-Object path -Match $inputPattern)
        if ($beforeInputs.Count -ne $nowInputs.Count) { throw 'Changed build-input set differs from the recorded build.' }
        foreach ($item in $beforeInputs) {
            $match = @($nowInputs | Where-Object { $_.path -ceq $item.path -and $_.sha256 -ceq $item.sha256 })
            if ($match.Count -ne 1) { throw "Build input changed: $($item.path)" }
        }
        Save-Json 'build-result.json' @{exit_code=0;reused_from=$previous;executable_sha256=$previousIdentity.executable_sha256;qualification='No build rerun; executable identity and changed Rust/Cargo inputs verified against the successful build receipt'}
    } else {
        & cargo build -p clean-ctx --bin clean-ctx --all-features 2>&1 | Tee-Object (Join-Path $output 'build.log')
        Save-Json 'build-result.json' @{exit_code=$LASTEXITCODE;arguments=@('build','-p','clean-ctx','--bin','clean-ctx','--all-features')}
        if ($LASTEXITCODE -ne 0) { throw 'Production build failed; acquisition not started.' }
    }
    Save-Json 'identity.json' @{
        executable_sha256=(Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant()
        profile_sha256=(Get-FileHash -LiteralPath $profile).Hash.ToLowerInvariant()
        head=(& git rev-parse HEAD);branch=(& git branch --show-current)
        actual_governance_activation='not requested; no authorization submission'
    }
    & (Join-Path $PSScriptRoot 'invoke-production-fixtures.ps1') -WorkspaceRoot $root -Binary $binary `
        -OutputRoot (Join-Path $output 'governance-fixtures')
    $selectionPath = Join-Path $PSScriptRoot 'B11_FINAL_PROTECTED_COMPONENT_PINS.csv'
    $selection = Import-Csv -LiteralPath $selectionPath
    Copy-Item -LiteralPath $selectionPath -Destination (Join-Path $output 'selection-pins.csv')
    $wanted = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($item in $selection) { [void]$wanted.Add("$($item.id)|$($item.role)|$($item.label)|$($item.selector)") }
    if ($wanted.Count -ne 342) { throw 'Final reviewed protected selection changed; inspect before runtime capture.' }
    $ids = @('AUTH.PROXY_TRANSFORMS','AUTH.CACHE_HINTS','AUTH.PERSISTENCE_RECOVERY',
        'AUTH.FINAL_VERIFICATION','AUTH.TOOL_WORKFLOW','AUTH.PCG_DOCUMENTS')
    $queries = @($ids | ForEach-Object { @{workspaceRoot=$root;mode='context';id=$_} })
    $queries += @{workspaceRoot=$root;mode='search'}
    Save-Json 'requests.json' $queries
    $driver = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
    $launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
    & $driver -WorkspaceRoot $root -Profile $profile -Binary $binary `
        -Requests (Join-Path $output 'requests.json') -OutputRoot (Join-Path $output 'graph') -LaunchDirectory $launch
    $references = [Collections.Generic.List[object]]::new()
    $seen = [Collections.Generic.HashSet[string]]::new()
    $selected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    for ($index=0; $index -lt $queries.Count; $index++) {
        $payload = Get-Content (Join-Path $output "graph/$($index+2).payload.json") -Raw | ConvertFrom-Json
        if (-not $payload.complete -or -not $payload.catalog_current -or $payload.result_state -ne 'ready') {
            throw "Production context $index incomplete or blocked; preserve its receipt."
        }
        foreach ($unit in $payload.units) {
            foreach ($component in $unit.components) {
                $reference = $component.reference
                $key = "$($unit.id)|$($reference.role)|$($reference.label)|$($reference.selector)"
                $historicalPolicy = $unit.id -ceq 'AUTH.PROXY_TRANSFORMS' -and $reference.selector -ceq 'PROXY-superseded-stripping'
                if ($wanted.Contains($key)) { [void]$selected.Add($key) }
                if (($wanted.Contains($key) -or $historicalPolicy) -and $seen.Add("$($unit.id)|$($unit.revision)|$($component.ordinal)")) {
                    $references.Add($reference)
                }
            }
        }
    }
    if ($selected.Count -ne $wanted.Count) { throw 'Graph descriptors omit reviewed protected components; preserve output.' }
    if (-not $references.Count) { throw 'No exact component references returned.' }
    Save-Json 'acquire.json' @(@{workspaceRoot=$root;mode='acquire';references=$references.ToArray()})
    & $driver -WorkspaceRoot $root -Profile $profile -Binary $binary `
        -Requests (Join-Path $output 'acquire.json') -OutputRoot (Join-Path $output 'acquisition') -LaunchDirectory $launch
    & python $helper snapshot $settings.database_path (Join-Path $output 'catalog-after.json')
    if ($LASTEXITCODE -ne 0) { throw 'Final catalog capture failed.' }
    & python $helper verify $root $output
    if ($LASTEXITCODE -ne 0) { throw 'Production preservation or exact reconstruction check failed.' }
    Write-Output "B11 production acquisition captured: $output"
} finally { Pop-Location }
