# Operator-only final document reconciliation/source acquisition; never grants or publishes.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$out = Join-Path $root ('target/ops-r1/integration-26-documents-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out | Out-Null
$binary = Join-Path $root 'target/debug/clean-ctx.exe'
$profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
$launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
$client = Join-Path $root 'verification/pcg-175/invoke-workspace-knowledge.ps1'
Push-Location $root
try {
    foreach ($guard in @('check-file-sizes.ps1','check-utf8.ps1')) {
        & pwsh -NoProfile -File (Join-Path $root "scripts/$guard") 2>&1 |
            Tee-Object -FilePath (Join-Path $out "$guard.txt")
        if ($LASTEXITCODE -ne 0) { throw "Document guard failed; preserve $out" }
    }
    if ((Get-FileHash $binary).Hash.ToLowerInvariant() -cne 'f03052a27e2a916ddf52dfb0a18deb246b387c386c04584f8dc8d8115c3ff4cd') {
        throw "Authoring binary changed; preserve $out. No runtime started."
    }
    $descriptorPath = Join-Path $out 'descriptor.json'
    & $binary pcg documents describe --profile $profile 1> $descriptorPath 2> (Join-Path $out 'descriptor-stderr.txt')
    if ($LASTEXITCODE -ne 0) { throw "Actual descriptor failed; preserve $out" }
    $descriptor = Get-Content $descriptorPath -Raw | ConvertFrom-Json
    if (-not $descriptor.digest -or -not $descriptor.descriptor) { throw "Descriptor refused; preserve $out" }
    if ($descriptor.descriptor.accepted_state_digest -cne $descriptor.descriptor.proposed_state_digest) {
        throw "Protected state change requires owner review of exact descriptor; preserve $out. No grant submitted."
    }
    $contextRequests = Join-Path $out 'context-requests.json'
    ConvertTo-Json -InputObject @(@{workspaceRoot=$root; mode='context'; id='OPS-001'}) -Depth 20 |
        Set-Content $contextRequests -Encoding utf8NoBOM
    & $client -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $contextRequests -OutputRoot (Join-Path $out 'context') -LaunchDirectory $launch
    $payloadPaths = @(Get-ChildItem (Join-Path $out 'context') -Filter '*.payload.json')
    if ($payloadPaths.Count -ne 1) { throw "Context capture incomplete; preserve $out" }
    $context = Get-Content $payloadPaths[0].FullName -Raw | ConvertFrom-Json -AsHashtable
    if ($context.result_state -cne 'ready' -or -not $context.complete -or -not $context.catalog_current) {
        throw "Actual context refused or incomplete; preserve $out"
    }
    $ids = @('OPS-001','DOC-270','VERIFY-001','AUTH.FINAL_VERIFICATION','PERF-004-DOCTRINE','PERF-005-BOUNDED')
    $units = @($context.units | Where-Object { $_.id -in $ids })
    if ($units.Count -ne $ids.Count) { throw "Applicable direct authorities missing; preserve $out" }
    $requests = @($units | ForEach-Object {
        if (-not $_.unit_complete -or -not $_.acquire_request) { throw 'Missing returned source acquisition request' }
        $request = $_.acquire_request
        $request.workspaceRoot = $root
        $request
    })
    $acquisitionPath = Join-Path $out 'acquire-requests.json'
    ConvertTo-Json -InputObject $requests -Depth 100 | Set-Content $acquisitionPath -Encoding utf8NoBOM
    & $client -WorkspaceRoot $root -Profile $profile -Binary $binary -Requests $acquisitionPath -OutputRoot (Join-Path $out 'acquisition') -LaunchDirectory $launch
    $payloads = @(Get-ChildItem (Join-Path $out 'acquisition') -Filter '*.payload.json')
    if ($payloads.Count -ne $ids.Count) { throw "Source acquisition capture incomplete; preserve $out" }
    [ordered]@{
        category='Actual final document checker and returned source acquisition; requires payload review'
        descriptor_digest=$descriptor.digest; descriptor_sha256=(Get-FileHash $descriptorPath).Hash.ToLowerInvariant()
        protected_state_unchanged=$true; context_generation=$context.generation
        context_ready=$true; acquired_payload_count=$payloads.Count; source_ids=$ids
        authorization_submitted=$false; push_merge_or_publication_performed=$false
    } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $out 'receipt.json') -Encoding utf8NoBOM
    Write-Output "Return final actual document/source receipts: $out"
} finally { Pop-Location }
