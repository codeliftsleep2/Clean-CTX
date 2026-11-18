param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180)
# Complete written-registration delivery oracle. No activation/preprocessor inference.
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path $BinaryPath).Path
$runRoot=Join-Path $repo ('target/di-omission/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force $runRoot | Out-Null
$root=Join-Path $runRoot 'fixture';New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet
if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new();$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false}}'
$cases=[Collections.Generic.List[object]]::new()
foreach ($count in @(0,9,10,11,40)) {
    $calls=@();$facts=@()
    for ($i=0;$i -lt $count;$i++) {
        $id=$i.ToString('D3');$calls+="services.AddScoped<IReg$id, Impl$id>();";$facts+="T @di = IReg$id → AddScoped<Impl$id>"
    }
    $cases.Add(@{name="count-$count";calls=($calls -join "`n");facts=$facts;extra='';conditions='unconditional'})
}
$forty=$cases[4];$cases.Add(@{name='unrelated-metadata';calls=$forty.calls;facts=$forty.facts;extra='private ILogger<RegistrationProbe> logger;';conditions='unconditional'})
$cases.Add(@{name='distinct-and-existing-order';calls='services.AddSingleton<IShared, First>(); services.AddScoped<IShared, First>(); services.AddScoped<IShared, Second>(); services.AddTransient<IThird, Third>();';facts=@('T @di = IShared → AddScoped<First>','T @di = IShared → AddScoped<Second>','T @di = IShared → AddSingleton<First>','T @di = IThird → AddTransient<Third>');extra='';conditions='unconditional'})
$cases.Add(@{name='conditional-written-mentions';calls="#if DEBUG`nservices.AddScoped<IDebug, DebugImpl>();`n#else`nservices.AddScoped<IRelease, ReleaseImpl>();`n#endif";facts=@('T @di = IDebug → AddScoped<DebugImpl>','T @di = IRelease → AddScoped<ReleaseImpl>');extra='';conditions='activity NOT ESTABLISHED; preserve existing written-mention extraction only'})
foreach ($case in $cases) {
    $case.file=$case.name+'.cs'
    $case.source=("// unchanged padding`n"*65)+"using Microsoft.Extensions.DependencyInjection;`nusing Microsoft.Extensions.Logging;`npublic class RegistrationProbe {`n $($case.extra)`n public void Register() {`n$($case.calls)`n }`n}`n"
    Write-BText (Join-Path $root $case.file) $case.source
}
Save-B 'source-manifest' $cases.ToArray()
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash $BinaryPath).Hash;provider_enabled=$false;interpretation='complete delivery of existing recognized written facts, not runtime container completeness'}
try {
    $session=Start-BSession 'di' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    Assert-B ('provide_code_context' -in $registry.tools.name) 'Registered context tool absent.'
    foreach ($fidelity in @('medium','high')) {
        foreach ($case in $cases) {
            Invoke-BCase '135' "$fidelity-$($case.name)" {
                $answer=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$case.file;fidelity=$fidelity})
                Save-B "$fidelity-$($case.name)-result" $answer
                $facts=@($answer.content[0].text -split "`r?`n" | Where-Object {$_.StartsWith('T @di = ')})
                Save-B "$fidelity-$($case.name)-oracle" @{expected=$case.facts;observed=$facts;conditions=$case.conditions}
                if ($answer._meta.content_kind -eq 'raw_passthrough') {
                    Assert-B ('document' -in $answer._meta.byte_exact) 'Fallback does not establish byte-exact source.'
                    Assert-B ($answer.content[0].text -ceq $case.source) 'Fallback does not deliver the exact authored source.'
                    $calls=[regex]::Matches($answer.content[0].text,'services\.Add(?:Scoped|Singleton|Transient|DbContext)<[^>]*>\(\);').Count
                    Assert-B ($calls -eq $case.facts.Count) 'Exact source does not contain the expected written-call count.'
                    Save-B "$fidelity-$($case.name)-delivery" @{mode='byte_exact_source';visible=$calls;expected=$case.facts.Count;conditions=$case.conditions}
                } else {
                Assert-B ($facts.Count -eq $case.facts.Count) "DI written-fact count $($facts.Count)/$($case.facts.Count)."
                for ($i=0;$i -lt $facts.Count;$i++) {Assert-B ($facts[$i] -ceq $case.facts[$i]) 'DI fact identity/content/order changed.'}
                if ($case.name -eq 'unrelated-metadata') {Assert-B ($answer.content[0].text.Contains('T @log = ILogger<RegistrationProbe>')) 'Unrelated logging metadata was lost.'}
                }
            }
        }
    }
} finally {
    foreach ($session in $sessions) {Stop-BSession $session}
    Save-B 'verdicts' $verdicts.ToArray();Write-Host "DI evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
