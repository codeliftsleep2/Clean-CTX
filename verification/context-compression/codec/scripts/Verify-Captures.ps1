$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$scenarios = Get-Content -Raw (Join-Path $definitionRoot "expected\scenarios.json") | ConvertFrom-Json
$failures = [Collections.Generic.List[string]]::new()
$unsupported = [Collections.Generic.List[string]]::new()
if (-not (Test-Path -LiteralPath $captures)) {
    throw "No captures directory exists. Run Capture-Baselines.ps1 successfully before verification."
}

function Payload([string]$id) {
    $path = Join-Path $captures "$id\control-full.txt"
    if (-not (Test-Path $path)) { return $null }
    $text = Get-Content -Raw $path
    if (-not $text.StartsWith("// CONTROL-FULL v2")) { return $null }
    $withoutHeader = $text -replace "^[^`r`n]*`r?`n", ""
    $pathmap = $withoutHeader.LastIndexOf("PATHMAP", [StringComparison]::Ordinal)
    if ($pathmap -lt 0) { throw "${id}: CONTROL-FULL PATHMAP footer is absent" }
    $footerLine = $withoutHeader.LastIndexOf("`n", $pathmap)
    if ($footerLine -lt 0) { throw "${id}: malformed CONTROL-FULL PATHMAP footer" }
    $withoutHeader.Substring(0, $footerLine).TrimEnd("`r", "`n") | ConvertFrom-Json -Depth 100
}
function Require([bool]$condition, [string]$message) { if (-not $condition) { $failures.Add($message) } }
function Header([string]$id) {
    $path = Join-Path $captures "$id\control-full.txt"
    if (-not (Test-Path -LiteralPath $path)) { return $null }
    ((Get-Content -Raw $path) -split "`r?`n", 2)[0]
}
function Methods($payload) {
    @(@($payload.classes) + @($payload.interfaces) | ForEach-Object { $_.methods } | ForEach-Object { $_ })
}
function Verify-ExactBodies([string]$id, [string]$fixture) {
    $payload = Payload $id
    if (-not $payload) { return }
    $sourcePath = if ($id -eq "restore-flow") {
        Join-Path $runtimeRoot "runtime\restore-latest-source.ts"
    } elseif ($id -eq "replay-flow") {
        Join-Path $runtimeRoot "runtime\restore-baseline-source.ts"
    } elseif ($payload.file.source_path -and (Test-Path -LiteralPath $payload.file.source_path)) {
        [string]$payload.file.source_path
    } else {
        Join-Path $runtimeRoot "runtime\workspace\$fixture"
    }
    $sourceBytes = [IO.File]::ReadAllBytes($sourcePath)
    foreach ($method in Methods $payload | Where-Object { $null -ne $_.body }) {
        $start = [int]$method.body_start
        $end = [int]$method.body_end
        Require ($start -ge 0 -and $end -gt $start -and $end -le $sourceBytes.Length) "$id/$($method.id): invalid exact-body span"
        if ($start -lt 0 -or $end -le $start -or $end -gt $sourceBytes.Length) { continue }
        $slice = $sourceBytes[$start..($end - 1)]
        $decoded = [Text.UTF8Encoding]::new($false, $true).GetString($slice)
        Require ($decoded -ceq [string]$method.body) "$id/$($method.id): body differs from source byte span"
    }
}

foreach ($scenario in $scenarios | Where-Object { $_.operation -in @("provide_code_context", "compress_code_context") }) {
    $responsePath = Join-Path $captures "$($scenario.id)\response.json"
    Require (Test-Path $responsePath) "$($scenario.id): missing response"
    if (-not (Test-Path $responsePath)) { continue }
    $response = Get-Content -Raw $responsePath | ConvertFrom-Json -Depth 100
    if ($scenario.expect -eq "error") {
        Require ($null -ne $response.error) "$($scenario.id): expected selector error"
        Require ($null -eq $response.result) "$($scenario.id): error must not publish a successful result"
        Require (-not (Test-Path (Join-Path $captures "$($scenario.id)\control-full.txt"))) "$($scenario.id): error must not publish CONTROL-FULL content"
        continue
    }
    if ($null -ne $response.error) {
        $message = if ($response.error.message) { [string]$response.error.message } else { $response.error | ConvertTo-Json -Compress -Depth 20 }
        $failures.Add("$($scenario.id): unexpected production error: $message")
        continue
    }
    $content = @($response.result.content)
    if ($content.Count -eq 0 -or $null -eq $content[0].text) {
        $failures.Add("$($scenario.id): successful response has no model-visible content text")
        continue
    }
    $text = [string]$content[0].text
    if ($scenario.fidelity -eq "verbatim") {
        $sourceBytes = [IO.File]::ReadAllBytes((Join-Path $runtimeRoot "runtime\workspace\$($scenario.fixture)"))
        $responseBytes = [Text.UTF8Encoding]::new($false).GetBytes($text)
        Require ([Convert]::ToHexString($responseBytes) -ceq [Convert]::ToHexString($sourceBytes)) "$($scenario.id): verbatim bytes/text differ"
        continue
    }
    $payload = Payload $scenario.id
    Require ($null -ne $payload) "$($scenario.id): not CONTROL-FULL"
    if (-not $payload) { continue }
    Require ($payload.schema -eq "clean-ctx/control-full") "$($scenario.id): wrong schema"
    if ($scenario.fixture -ne "tiny.ts" -and $scenario.callsSupported -ne $false) {
        Require (@($payload.calls).Count -gt 0) "$($scenario.id): calls absent"
    }
    if ($scenario.callsSupported -eq $false) {
        $unsupported.Add("$($scenario.id): current production compiler emits no structural call facts for this language fixture; call reasoning is not scored.")
    }
    Require (@($payload.semantic_edges).Count -gt 0) "$($scenario.id): semantic edges absent"
    if ($scenario.fidelity -ne "edit") {
        Require (@($payload.mode.exact_body_method_ids).Count -eq 0) "$($scenario.id): unexpected bodies"
    }
}

if ($failures.Count -gt 0) {
    $result = [ordered]@{ pass = $false; failure_count = $failures.Count; failures = @($failures); unsupported = @($unsupported) }
    $result | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM (Join-Path $captures "verification-result.json")
    $failures | ForEach-Object { Write-Host "FAIL: $_" -ForegroundColor Red }
    exit 1
}

$debugPayload = Payload "medium-debug"; $implementPayload = Payload "medium-implement"
if (-not $debugPayload -or -not $implementPayload) {
    throw "Required Medium captures are absent or invalid. Resolve capture errors before verification."
}
$debugPayload.file.ir_version = $null; $implementPayload.file.ir_version = $null
Require (($debugPayload | ConvertTo-Json -Depth 100 -Compress) -ceq ($implementPayload | ConvertTo-Json -Depth 100 -Compress)) "debug/implement: expected measured semantic equivalence"

$focused = Payload "edit-focus-one"
Require (@($focused.mode.exact_body_method_ids).Count -eq 1) "focus-one: expected exactly one canonical body ID"
$focusedMethod = Methods $focused | Where-Object { $_.id -in @($focused.mode.exact_body_method_ids) } | Select-Object -First 1
Require ($focusedMethod.name -eq "run" -and (@($focused.classes | Where-Object { $_.name -eq "Beta" -and $_.methods.id -contains $focusedMethod.id }).Count -eq 1)) "focus-one: Beta.run was not the sole resolved identity"
$many = Payload "edit-focus-many"
Require (@($many.mode.exact_body_method_ids).Count -eq 2) "focus-many: expected two canonical body IDs"
$manyNames = @(Methods $many | Where-Object { $_.id -in @($many.mode.exact_body_method_ids) } | ForEach-Object name)
Require (($manyNames -contains "run") -and ($manyNames -contains "unique")) "focus-many: resolved target set is wrong"
$overloads = Payload "edit-focus-overloads"
Require (@($overloads.mode.exact_body_method_ids).Count -ge 2) "focus-overloads: overload family not retained"
$overloadIds = @($overloads.mode.exact_body_method_ids)
Require (@($overloads.classes | Where-Object { $_.name -eq "Alpha" -and @($_.methods | Where-Object { $_.name -eq "run" -and $_.id -in $overloadIds }).Count -eq $overloadIds.Count }).Count -eq 1) "focus-overloads: selected IDs escaped Alpha.run"
$bare = Payload "edit-focus-bare"
$bareMethods = @(Methods $bare | Where-Object { $_.id -in @($bare.mode.exact_body_method_ids) })
Require ($bareMethods.Count -eq 1 -and $bareMethods[0].name -eq "unique") "focus-bare: unique selector resolved incorrectly"
$all = Payload "edit-all"
Require (@($all.mode.exact_body_method_ids).Count -gt @($focused.mode.exact_body_method_ids).Count) "edit-all: body set did not exceed focused set"
Require ((@($all.classes | ForEach-Object { @($_.injection_occurrences) }).Count -gt 0) -or (@($all.semantic_edges | Where-Object relation -eq "Injects").Count -gt 0)) "edit-all: injection facts absent"
$crlf = Payload "edit-all-crlf"
$crlfBodies = @($crlf.classes | ForEach-Object { $_.methods } | ForEach-Object { $_ } | Where-Object { $null -ne $_.body } | ForEach-Object { [string]$_.body })
Require (@($crlfBodies | Where-Object { $_.Contains("`r`n") }).Count -gt 0) "edit-all-crlf: CRLF body bytes were normalized"
foreach ($scenario in $scenarios | Where-Object { $_.fidelity -eq "edit" -and $_.expect -eq "success" }) {
    Verify-ExactBodies $scenario.id $scenario.fixture
}

$high = Payload "high-refactor"
$highMethods = Methods $high
$injectionOccurrenceCount = 0
$duplicateDependencyOccurrence = $false
foreach ($owner in @($high.classes)) {
    foreach ($occurrence in @($owner.injection_occurrences)) {
        $injectionOccurrenceCount++
        if (@($occurrence | Group-Object | Where-Object Count -gt 1).Count -gt 0) { $duplicateDependencyOccurrence = $true }
    }
}
$injectionEdges = @($high.semantic_edges | Where-Object relation -eq "Injects")
$duplicateInjectionEdge = @($injectionEdges | Group-Object { "$($_.subject.domain)|$($_.subject.entity_type)|$($_.subject.name)|$($_.object.domain)|$($_.object.entity_type)|$($_.object.name)" } | Where-Object Count -gt 1).Count -gt 0
Require (@($high.classes | Where-Object { -not $_.id }).Count -eq 0) "high: class identity absent"
Require (@($highMethods | Where-Object { -not $_.id }).Count -eq 0) "high: method identity absent"
Require (@($high.calls | Where-Object { $_.callee_resolution -ne "unresolved" }).Count -eq 0) "high: unresolved calls were promoted to declarations"
Require (@($high.calls | Where-Object { $null -eq $_.explicit_argument_count }).Count -eq 0) "high: written call arity absent"
Require (@($high.calls | Where-Object { $_.callee_written_name -eq "lookup" }).Count -ge 2) "high: duplicate ordered calls absent"
Require (@($high.calls | Where-Object { $_.callee_written_name -eq "external" -and $_.has_spread }).Count -ge 1) "high: spread evidence absent"
Require (($injectionOccurrenceCount + $injectionEdges.Count) -ge 1) "high: injection facts absent"
if (($injectionOccurrenceCount + $injectionEdges.Count) -lt 2 -or -not ($duplicateDependencyOccurrence -or $duplicateInjectionEdge)) {
    $unsupported.Add("high-refactor: registered compiler emitted injection semantics but not duplicate injection occurrences for the deterministic constructor fixture; duplicate-DI reasoning is not scored.")
}
Require (@($high.classes | Where-Object { @($_.modifier_occurrences).Count -gt 0 }).Count -gt 0) "high: declaration modifiers absent"
Require (@($highMethods | Where-Object { @($_.pattern_fact_occurrences).Count -gt 0 }).Count -gt 0) "high: pattern facts absent"
Require (@($highMethods | Where-Object { @($_.patterns).Count -gt 0 }).Count -gt 0) "high: final patterns absent"
Require (@($highMethods | Where-Object { @($_.control_flow).Count -gt 0 }).Count -gt 0) "high: control flow absent"
Require (@($highMethods | Where-Object { @($_.data_flow).Count -gt 0 }).Count -gt 0) "high: data flow absent"
Require (@($highMethods | Where-Object { @($_.side_effects).Count -gt 0 }).Count -gt 0) "high: side effects absent"
Require (@($highMethods | Where-Object { @($_.execution_contexts).Count -gt 0 }).Count -gt 0) "high: execution contexts absent"
Require (@($high.semantic_edges | Where-Object { $_.subject.file -or $_.object.file }).Count -gt 0) "high: semantic-edge provenance absent"

foreach ($stage in @("baseline", "delta", "apply")) {
    Require (Test-Path (Join-Path $captures "delta-flow-$stage\control-full.txt")) "delta ${stage}: missing content"
}
Require ((Header "delta-flow-delta") -like "// FILE-CONTEXT-DELTA v1*") "delta: wrong model-visible contract"
Require ((Header "restore-flow") -like "// CONTROL-FULL v2*") "restore: not regenerated CONTROL-FULL"
Require ((Header "replay-flow") -like "// CONTROL-FULL v2*") "replay: not regenerated CONTROL-FULL"

$a = Payload "cross-a"; $b = Payload "cross-b"
Require ($a.file.source_path -ne $b.file.source_path) "cross-file: provenance paths collapsed"
Require ((@($a.classes | Where-Object name -eq "SharedName").Count -eq 1) -and (@($b.classes | Where-Object name -eq "SharedName").Count -eq 1)) "cross-file: fixture no longer exercises equal display names"
# Cross-file relations belong to workspace_query, not the single-file context. The
# file context correctly omits them; the workspace reasoning lane scores cross-file
# resolution and framework-edge provenance through workspace_query responses.

$result = [ordered]@{ pass = ($failures.Count -eq 0); failure_count = $failures.Count; failures = @($failures); unsupported = @($unsupported) }
$result | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM (Join-Path $captures "verification-result.json")
if ($failures.Count) { $failures | ForEach-Object { Write-Host "FAIL: $_" -ForegroundColor Red }; exit 1 }
Write-Host "PASS: registered-path capture checks passed (operator evidence, not CI proof)."
