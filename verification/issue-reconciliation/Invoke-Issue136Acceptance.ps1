param([Parameter(Mandatory)][string]$BinaryPath,[ValidateRange(30,600)][int]$CaseTimeoutSeconds=180,[switch]$KeepWorkspace)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath=(Resolve-Path -LiteralPath $BinaryPath).Path
$runRoot=Join-Path $repo ('target/issue-136-live-verification/'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$root=Join-Path $runRoot 'workspace';New-Item -ItemType Directory -Force $root | Out-Null
& git -C $root init --quiet;if ($LASTEXITCODE -ne 0) {throw 'Fixture Git root failed.'}
$sessions=[Collections.Generic.List[object]]::new();$verdicts=[Collections.Generic.List[object]]::new()
$rpcId=0;$deadline=[DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')
Write-BText (Join-Path $root '.clean-ctx.json') '{"cbm":{"enabled":false,"auto_launch":false}}'
Save-B 'provenance' @{binary=$BinaryPath;sha256=(Get-FileHash -LiteralPath $BinaryPath).Hash;pwsh=$PSVersionTable.PSVersion.ToString();provider_enabled=$false}
function Assert-Source([string]$File,[string]$Expected) {
    $actual=[IO.File]::ReadAllBytes((Join-Path $root $File));$wanted=[Text.Encoding]::UTF8.GetBytes($Expected)
    Save-B ($File+'-exact') @{expected_base64=[Convert]::ToBase64String($wanted);actual_base64=[Convert]::ToBase64String($actual)}
    Assert-B ([Convert]::ToBase64String($actual) -ceq [Convert]::ToBase64String($wanted)) "Exact-source mismatch: $File"
}
function Publish-Edit([string]$File) {
    $null=Get-BResult (Invoke-BTool $session 'provide_code_context' @{workspaceRoot=$root;filePath=$File;fidelity='edit'})
}
function Invoke-Edit([string]$File,$Operations,[bool]$Verify=$false) {
    Invoke-BTool $session 'apply_edit' @{workspaceRoot=$root;filePath=$File;operations=$Operations;verify=$Verify}
}
function Assert-EditError($Response,[string]$Kind,[string]$Scenario) {
    Assert-B ($null -ne $Response.PSObject.Properties['error']) "$Scenario unexpectedly succeeded."
    Assert-B ($Response.error.data.kind -eq $Kind) "$Scenario returned $($Response.error.data.kind), expected $Kind."
}
function Write-Fixture([string]$File,[string]$Source) {Write-BText (Join-Path $root $File) $Source;Publish-Edit $File}
try {
    $session=Start-BSession 'issue-136' $BinaryPath $root
    $registry=Get-BResult (Invoke-BRpc $session 'tools/list' @{});Assert-B ('apply_edit' -in $registry.tools.name) 'Registered apply_edit absent.'
    Invoke-BCase '136' 'csharp-signature-selectors-and-legacy-ambiguity' {
        $file='overloads.cs';$source="class Service {`n  int Run(int value) { return 1; }`n  int Run(string value) { return 2; }`n  int Run(bool value) { return 3; }`n}`n"
        Write-Fixture $file $source
        $response=Get-BResult (Invoke-Edit $file @(@{type='replace_body';target='Service.Run(string)';expectedOldText='{ return 2; }';newText='{ return 22; }'}) $true)
        $expected=$source.Replace('{ return 2; }','{ return 22; }');Assert-Source $file $expected
        Assert-B ($response.structuredContent.operations.Count -eq 1) 'Expected one edit receipt.'
        Assert-B ($response.structuredContent.operations[0].target -eq 'Service.Run(string)') 'Receipt lost the signature selector.'
        Assert-B ($response.structuredContent.operations[0].newText -eq '{ return 22; }') 'verify:true omitted exact applied text.'
        Assert-B ($response._meta.syntaxGated -eq $true) 'Successful edit was not syntax gated.'
        $ambiguous=Invoke-Edit $file @(@{type='replace_body';target='Service.Run';expectedOldText='{ return 22; }';newText='{ return 99; }'})
        Assert-EditError $ambiguous 'ambiguous_target' 'legacy overload selector';Assert-Source $file $expected
    }
    Invoke-BCase '136' 'typescript-signature-and-accessor-discrimination' {
        $file='selectors.ts';$source="class Service {`n  run(value: string) { return 's'; }`n  run(value: number) { return 'n'; }`n}`nclass Access {`n  get value() { return 1; }`n  set value(input: number) { this.saved = input; }`n}`n"
        Write-Fixture $file $source
        $null=Get-BResult (Invoke-Edit $file @(@{type='replace_body';target='Service.run(number)';expectedOldText="{ return 'n'; }";newText="{ return 'number'; }"}))
        $expected=$source.Replace("{ return 'n'; }","{ return 'number'; }")
        $null=Get-BResult (Invoke-Edit $file @(@{type='replace_body';target='Access.value(number)';expectedOldText='{ this.saved = input; }';newText='{ this.saved = input + 1; }'}))
        $expected=$expected.Replace('{ this.saved = input; }','{ this.saved = input + 1; }');Assert-Source $file $expected
    }
    Invoke-BCase '136' 'unique-legacy-and-owner-qualification' {
        $file='owners.cs';$source="class Outer {`n  int Run(int value) { return 1; }`n  class Inner { int Run(int value) { return 2; } }`n  int Unique() { return 3; }`n}`n"
        Write-Fixture $file $source
        $null=Get-BResult (Invoke-Edit $file @(@{type='replace_body';target='Outer.Unique';expectedOldText='{ return 3; }';newText='{ return 33; }'}))
        $null=Get-BResult (Invoke-Edit $file @(@{type='replace_body';target='Inner.Run(int)';expectedOldText='{ return 2; }';newText='{ return 22; }'}))
        $expected=$source.Replace('{ return 3; }','{ return 33; }').Replace('{ return 2; }','{ return 22; }');Assert-Source $file $expected
    }
    Invoke-BCase '136' 'diagnostic-taxonomy-and-no-write' {
        $file='diagnostics.cs';$source="class Service {`n  int Value { get; set; }`n  int Run() { return 1; }`n}`n";Write-Fixture $file $source
        $uneditable=Invoke-Edit $file @(@{type='replace_body';target='Service.Value';expectedOldText='';newText='{}'})
        Assert-EditError $uneditable 'uneditable_target' 'property edit';Assert-B ($uneditable.error.data.targetKind -eq 'property_declaration') 'Uneditable response lost parser kind.';Assert-Source $file $source
        $missing=Invoke-Edit $file @(@{type='replace_body';target='Service.Missing';expectedOldText='';newText='{}'})
        Assert-EditError $missing 'not_found' 'missing target';Assert-Source $file $source
        $mismatch=Invoke-Edit $file @(@{type='replace_body';target='Service.Run';expectedOldText='{ stale body }';newText='{ return 9; }'})
        Assert-EditError $mismatch 'unit_mismatch' 'expected text mismatch'
        Assert-B ([string]$mismatch.error.message -match 'does not match its current tracked body') 'Mismatch prose overclaims source history.';Assert-Source $file $source
    }
    Invoke-BCase '136' 'separator-delete-and-documentation-authority' {
        $file='boundaries.cs';$source="class Boundary {`n  void First() {}`n  void Gone() {}`n  /// unowned docs`n  void Documented() {}`n}`n";Write-Fixture $file $source
        $null=Get-BResult (Invoke-Edit $file @(@{type='insert_after';anchor='Boundary.First';unitText='void Added() {}'}))
        $expected=$source.Replace("  void First() {}`n","  void First() {}`n  void Added() {}`n");Assert-Source $file $expected
        $null=Get-BResult (Invoke-Edit $file @(@{type='delete';target='Boundary.Gone';expectedOldText='void Gone() {}'}))
        $expected=$expected.Replace('void Gone() {}','');Assert-Source $file $expected
        $refused=Invoke-Edit $file @(@{type='delete';target='Boundary.Documented';expectedOldText='void Documented() {}'})
        Assert-EditError $refused 'uneditable_target' 'unowned documentation delete';Assert-Source $file $expected
    }
    Invoke-BCase '136' 'atomic-and-stale-source-refusal' {
        $file='safety.ts';$source='class Safety { first() { return 1; } second() { return 2; } }';Write-Fixture $file $source
        $atomic=Invoke-Edit $file @(@{type='replace_body';target='Safety.first';expectedOldText='{ return 1; }';newText='{ return 9; }'},@{type='replace_body';target='Safety.missing';expectedOldText='';newText='{}'})
        Assert-EditError $atomic 'not_found' 'atomic batch';Assert-Source $file $source
        $changed=$source+' // external change';Write-BText (Join-Path $root $file) $changed
        $stale=Invoke-Edit $file @(@{type='replace_body';target='Safety.first';expectedOldText='{ return 1; }';newText='{ return 9; }'})
        Assert-B ($null -ne $stale.PSObject.Properties['error']) 'Stale source unexpectedly succeeded.'
        Assert-B ($stale.error.data.code -eq 'stale_edit_source') 'Stale source returned the wrong diagnostic code.';Assert-Source $file $changed
    }
} finally {
    foreach ($item in $sessions) {Stop-BSession $item};Save-B 'verdicts' $verdicts.ToArray()
    if ($KeepWorkspace) {Write-Host "Retained Issue #136 live workspace: $root"}
}
Write-Host "Issue #136 evidence: $runRoot"
if (@($verdicts | Where-Object result -eq 'FAIL').Count) {exit 1}
Write-Host 'PASS: Issue #136 consolidated live acceptance completed.'