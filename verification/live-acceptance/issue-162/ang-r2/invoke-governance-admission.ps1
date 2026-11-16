# Operator-run normal knowledge reconciliation and affected-context acquisition.
# Never creates authorization; no tests, builds or production capability pilot.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$pins = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ANG_R2_CANDIDATE_PINS.json') -Raw | ConvertFrom-Json
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    if ((& git rev-parse HEAD) -cne $pins.baseline) { throw 'Baseline changed; review again.' }
    foreach ($pin in $pins.files) {
        if ((Get-FileHash -LiteralPath (Join-Path $root $pin.path)).Hash.ToLowerInvariant() -cne $pin.candidate_sha256) {
            throw "Candidate drift: $($pin.path)"
        }
    }
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
    if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -cne $pins.binary_sha256) { throw 'Binary drift.' }
    if ((Get-FileHash -LiteralPath $profile).Hash.ToLowerInvariant() -cne $pins.profile_sha256) { throw 'Profile drift.' }
    $out = Join-Path $root ('target/ang-r2/governance-admission-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    $requests = @('AUTH.ANGULAR_RESOLVER', 'AUTH.ANGULAR_CANONICAL_RELATIONS', 'AUTH.TOOL_WORKFLOW') | ForEach-Object {
        @{ workspaceRoot = $root; mode = 'context'; id = $_ }
    }
    $requestFile = Join-Path $out 'contexts.json'
    [IO.File]::WriteAllText($requestFile, (ConvertTo-Json -InputObject @($requests) -Depth 100), $utf8)
    $client = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
    $common = @{WorkspaceRoot=$root; Profile=$profile; Binary=$binary; LaunchDirectory=(Join-Path $root '.clean-ctx/pcg-documents/authoring')}
    $contextOut = Join-Path $out 'contexts'
    & $client @common -Requests $requestFile -OutputRoot $contextOut
    $generations = [Collections.Generic.HashSet[string]]::new()
    $units = @{}
    foreach ($file in Get-ChildItem -LiteralPath $contextOut -Filter '*.payload.json') {
        $payload = [IO.File]::ReadAllText($file.FullName, $utf8) | ConvertFrom-Json -AsHashtable
        if ($payload.result_state -cne 'ready' -or -not $payload.catalog_current -or -not $payload.complete) {
            throw "Admission/context incomplete; preserve $out"
        }
        $null = $generations.Add($payload.generation)
        foreach ($unit in $payload.units) { $units[$unit.id] = $unit }
    }
    if ($generations.Count -ne 1 -or $generations.Contains($pins.accepted_generation)) {
        throw "Expected one new admitted generation; preserve $out"
    }
    $references = [Collections.Generic.List[object]]::new()
    foreach ($id in ($units.Keys | Sort-Object)) {
        foreach ($component in $units[$id].components) {
            if ($component.role -notin @('observation', 'historical_applicability')) {
                $references.Add($component.reference)
            }
        }
    }
    if ($references.Count -eq 0) { throw 'No explicit source references returned.' }
    $acquire = [Collections.Generic.List[object]]::new()
    for ($i = 0; $i -lt $references.Count; $i += 24) {
        $last = [Math]::Min($references.Count - 1, $i + 23)
        $acquire.Add(@{workspaceRoot=$root; mode='acquire'; references=@($references[$i..$last])})
    }
    $acquireFile = Join-Path $out 'acquire.json'
    [IO.File]::WriteAllText($acquireFile, (ConvertTo-Json -InputObject @($acquire.ToArray()) -Depth 100), $utf8)
    & $client @common -Requests $acquireFile -OutputRoot (Join-Path $out 'acquisition')
    foreach ($file in Get-ChildItem -LiteralPath (Join-Path $out 'acquisition') -Filter '*.payload.json') {
        $payload = [IO.File]::ReadAllText($file.FullName, $utf8) | ConvertFrom-Json -AsHashtable
        if ($payload.result_state -cne 'ready' -or -not $payload.catalog_current -or -not $payload.complete -or -not $generations.Contains($payload.generation)) {
            throw "Acquisition incomplete or generation changed; preserve $out"
        }
    }
    Write-Host "Normal reconciliation and source acquisition captured: $out"
    Write-Host 'Return receipts for exact admission/history/context review before Phase B.'
} finally { Pop-Location }
