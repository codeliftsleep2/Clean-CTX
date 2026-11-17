# Operator-owned actual admission/discoverability. Never submits authorization.
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$review=Join-Path $root 'target/ci-verify-r1/descriptor-20261117-152254'
$out=Join-Path $root ('target/ci-verify-r1/admission-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out|Out-Null
Push-Location $root
try {
    foreach($pin in @(Get-Content (Join-Path $review 'candidate-sources.json') -Raw|ConvertFrom-Json)){
        if((Get-FileHash -LiteralPath $pin.path).Hash -cne $pin.sha256){throw "Reviewed candidate changed: $($pin.path)"}
    }
    $descriptor=Get-Content (Join-Path $review 'descriptor.json') -Raw|ConvertFrom-Json
    if($descriptor.digest -cne '9ef404dab3917c2a54d9aa7fef551b18923620ae2dfe5522815c4cbd0df5b5d4'){throw 'Reviewed descriptor drift'}
    $common=@{
        WorkspaceRoot=$root
        Binary=(Join-Path $root 'target/debug/clean-ctx.exe')
        Profile=(Join-Path $root '.clean-ctx/pcg-documents/profile.json')
        LaunchDirectory=(Join-Path $root '.clean-ctx/pcg-documents/authoring')
    }
    $ids=@('VERIFY-001','VERIFY-002','AUTH.FINAL_VERIFICATION')
    $requests=@($ids|ForEach-Object{@{workspaceRoot=$root;mode='context';id=$_}})
    $request=Join-Path $out 'contexts.json'
    ConvertTo-Json -InputObject $requests -Depth 20|Set-Content $request -Encoding utf8NoBOM
    $driver=Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
    & $driver @common -Requests $request -OutputRoot (Join-Path $out 'contexts')
    $generations=[Collections.Generic.HashSet[string]]::new()
    $units=@{}
    foreach($file in Get-ChildItem (Join-Path $out 'contexts') -Filter '*.payload.json'){
        $value=Get-Content -LiteralPath $file.FullName -Raw|ConvertFrom-Json
        if($value.result_state -cne 'ready' -or -not $value.complete -or -not $value.catalog_current){throw "Actual admission refused/incomplete; preserve $out"}
        $null=$generations.Add($value.generation)
        foreach($unit in $value.units){$units[$unit.id]=$unit}
    }
    if($generations.Count -ne 1 -or $generations.Contains($descriptor.descriptor.predecessor_generation)){throw "Expected one newly admitted generation; preserve $out"}
    $refs=[Collections.Generic.List[object]]::new()
    foreach($id in $ids){
        if(-not $units.ContainsKey($id)){throw "Required admitted identity missing: $id"}
        foreach($component in $units[$id].components){$refs.Add($component.reference)}
    }
    if(-not $refs.Count){throw 'No discoverable exact source components'}
    $acquire=Join-Path $out 'acquire.json'
    ConvertTo-Json -InputObject @(@{workspaceRoot=$root;mode='acquire';references=$refs.ToArray()}) -Depth 100|Set-Content $acquire -Encoding utf8NoBOM
    & $driver @common -Requests $acquire -OutputRoot (Join-Path $out 'acquisition')
    $result=Get-Content (Join-Path $out 'acquisition/2.payload.json') -Raw|ConvertFrom-Json
    if($result.result_state -cne 'ready' -or -not $result.complete -or -not $result.catalog_current -or -not $generations.Contains($result.generation)){throw "Admitted source acquisition incomplete; preserve $out"}
    [ordered]@{
        agent='Agent MaxHeadRoom';category='Actual document admission and source discoverability, not CI or empirical performance'
        reviewed_digest=$descriptor.digest;generation=$result.generation;ids=$ids
        acquisition_sha256=(Get-FileHash (Join-Path $out 'acquisition/2.payload.json')).Hash
    }|ConvertTo-Json -Depth 5|Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Return actual admission receipts: $out"
} finally {Pop-Location}
