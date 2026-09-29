param([string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path)

$ErrorActionPreference = "Stop"
$captures = Join-Path $RepositoryRoot "target\context-compression-verification\captures"
function Require([bool]$condition, [string]$message) { if (-not $condition) { throw "FAIL: $message" } }
function Payload([string]$id) {
    $text = Get-Content -Raw (Join-Path $captures "$id\control-full.txt")
    $remainder = $text -replace "^[^`r`n]*`r?`n", ""
    $pathmap = $remainder.LastIndexOf("`n§PATHMAP", [StringComparison]::Ordinal)
    Require ($pathmap -ge 0) "$id missing PATHMAP"
    return $remainder.Substring(0, $pathmap) | ConvertFrom-Json -Depth 100
}
function MethodId($payload, [string]$ownerName, [string]$methodName) {
    $owner = @($payload.classes | Where-Object name -eq $ownerName)
    Require ($owner.Count -eq 1) "$ownerName owner count"
    return @($owner[0].methods | Where-Object name -eq $methodName | ForEach-Object id)
}
function Callees($payload, [string]$methodId) {
    return @($payload.calls | Where-Object caller_method_id -eq $methodId | ForEach-Object callee_written_name)
}

$angular = Payload "edge-angular-arrows-edit"
$load = @(MethodId $angular "EdgeComponent" "load")
$cancel = @(MethodId $angular "EdgeComponent" "cancel")
$refresh = @(MethodId $angular "EdgeComponent" "refresh")
Require ($load.Count -eq 1 -and $cancel.Count -eq 1 -and $refresh.Count -eq 1) "bound arrows need distinct canonical methods"
Require ($load[0] -ne $cancel[0] -and $cancel[0] -ne $refresh[0]) "arrow IDs collided"
foreach ($callee in @("pipe","map","transform","tap","audit","subscribe","save","log")) {
    Require ((Callees $angular $load[0]) -contains $callee) "load missing nested call $callee"
}
foreach ($callee in @("queueMicrotask","abort")) {
    Require ((Callees $angular $cancel[0]) -contains $callee) "cancel missing nested call $callee"
}
$spread = @($angular.calls | Where-Object { $_.caller_method_id -eq $refresh[0] -and $_.callee_written_name -eq "external" })
Require ($spread.Count -eq 1 -and $spread[0].has_spread -eq $true -and $spread[0].callee_resolution -eq "unresolved") "spread/unresolved evidence"
Require (@($angular.semantic_edges | Where-Object { $_.relation -eq "Injects" -and $_.subject.name -eq "EdgeComponent" -and $_.object.name -eq "Repository" }).Count -eq 1) "Angular DI edge"

$csharp = Payload "edge-csharp-lambdas-edit"
$runs = @(MethodId $csharp "EdgeController" "Run")
Require ($runs.Count -eq 2 -and $runs[0] -ne $runs[1]) "C# overload IDs"
$runMethods = @($csharp.classes | Where-Object name -eq "EdgeController" | ForEach-Object methods | Where-Object name -eq "Run")
Require (@($runMethods | Where-Object { $null -eq $_.body -or $null -eq $_.body_start -or $null -eq $_.body_end }).Count -eq 0) "C# exact bodies/spans"
foreach ($callee in @("Normalize","Audit","Notify")) {
    Require (@($csharp.calls | Where-Object { $runs -contains $_.caller_method_id -and $_.callee_written_name -eq $callee }).Count -ge 1) "C# missing lambda call $callee"
}
Require (@($csharp.semantic_edges | Where-Object relation -eq "HasRoute").Count -ge 1) "ASP.NET HasRoute edge"
Require (@($csharp.semantic_edges | Where-Object relation -eq "ControllerAction").Count -ge 1) "ASP.NET ControllerAction edge"

Write-Host "PASS: live TypeScript/Angular and C# edge-case CONTROL-FULL captures are correct."
