# Operator-owned knowledge acquisition only. No build, test or protected grant.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$binary = Join-Path $root 'target/debug/clean-ctx.exe'
if ((Get-FileHash $binary).Hash.ToLowerInvariant() -cne '5d00fea65d44bf7522d123e10a543b46a92842c77a1e60caee918605ec1549bd') { throw 'Expected verified ANG-R2 executable; binary drift.' }
$out = Join-Path $root ('target/ang-r2/retirement-authority-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$ids = @('WSC-003','WSC-004','PERSIST-001','PERSIST-002','PUB-001','AUTH.PERSISTENCE_RECOVERY','AUTH.ANGULAR_CANONICAL_RELATIONS','AUTH.ANGULAR_RESOLVER')
$requests = @($ids | ForEach-Object { @{workspaceRoot=$root; mode='context'; id=$_} })
$requestFile = Join-Path $out 'contexts.json'
[IO.File]::WriteAllText($requestFile,(ConvertTo-Json -InputObject $requests -Depth 100),$utf8)
$client = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
$common = @{WorkspaceRoot=$root; Profile=(Join-Path $root '.clean-ctx/pcg-documents/profile.json'); Binary=$binary; LaunchDirectory=(Join-Path $root '.clean-ctx/pcg-documents/authoring')}
& $client @common -Requests $requestFile -OutputRoot (Join-Path $out 'contexts')
$generations = [Collections.Generic.HashSet[string]]::new()
$units = @{}
$payloads = @(Get-ChildItem (Join-Path $out 'contexts') -Filter '*.payload.json')
if ($payloads.Count -ne $ids.Count) { throw "Incomplete context responses; preserve $out" }
foreach ($file in $payloads) {
    $payload = [IO.File]::ReadAllText($file.FullName,$utf8) | ConvertFrom-Json -AsHashtable
    if ($payload.result_state -cne 'ready' -or -not $payload.catalog_current -or -not $payload.complete) { throw "Context incomplete; preserve $out" }
    $null = $generations.Add($payload.generation)
    foreach ($unit in $payload.units) { $units[$unit.id] = $unit }
}
if ($generations.Count -ne 1) { throw 'Context generations differ.' }
$references = [Collections.Generic.List[object]]::new()
foreach ($id in ($units.Keys | Sort-Object)) {
    foreach ($component in $units[$id].components) {
        if ($component.role -notin @('observation','historical_applicability')) { $references.Add($component.reference) }
    }
}
if ($references.Count -eq 0) { throw 'No explicit source references.' }
$acquire = [Collections.Generic.List[object]]::new()
for ($i=0; $i -lt $references.Count; $i+=24) {
    $last = [Math]::Min($references.Count-1,$i+23)
    $acquire.Add(@{workspaceRoot=$root; mode='acquire'; references=@($references[$i..$last])})
}
$acquireFile = Join-Path $out 'acquire.json'
[IO.File]::WriteAllText($acquireFile,(ConvertTo-Json -InputObject @($acquire.ToArray()) -Depth 100),$utf8)
& $client @common -Requests $acquireFile -OutputRoot (Join-Path $out 'acquisition')
$acquired = @(Get-ChildItem (Join-Path $out 'acquisition') -Filter '*.payload.json')
if ($acquired.Count -ne $acquire.Count) { throw 'Incomplete acquisition responses.' }
foreach ($file in $acquired) {
    $payload = [IO.File]::ReadAllText($file.FullName,$utf8) | ConvertFrom-Json -AsHashtable
    if ($payload.result_state -cne 'ready' -or -not $payload.catalog_current -or -not $payload.complete -or -not $generations.Contains($payload.generation)) { throw "Acquisition incomplete; preserve $out" }
}
Write-Host "Return context and acquired source receipts from $out. No authorization grant or production implementation performed."
