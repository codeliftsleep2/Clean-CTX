# Prepare paired SCHEMA-v5/B1 reasoning cases for the method grammar change.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$recordsPath = Join-Path $captures "schema-vnext-b1-records.json"

if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-b1-records.json. Run Measure-B1MethodArrow.ps1"
}

$baselineHeader = "// SCHEMA v5  @=meta X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$candidateHeader = "// SCHEMA vNext  @=meta X=extends I=implements F=field M=method `$=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$scopeArrowPattern = '(?m)^(M [^\r\n]*?)  → '

function ConvertTo-B1([string]$capture, [string]$baseline) {
    if ([regex]::Matches($baseline, "(?m)^$([regex]::Escape($baselineHeader))$").Count -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header"
    }
    $methodCount = [regex]::Matches($baseline, '(?m)^M ').Count
    if ([regex]::Matches($baseline, $scopeArrowPattern).Count -ne $methodCount) {
        throw "${capture}: not every method owns one scope arrow"
    }
    $candidate = $baseline.Replace($baselineHeader, $candidateHeader)
    $candidate = [regex]::Replace($candidate, $scopeArrowPattern, '$1 ')
    if ([regex]::IsMatch($candidate, $scopeArrowPattern)) {
        throw "${capture}: B1 left a method scope arrow"
    }
    return $candidate
}

function Add-Pair(
    [Collections.Generic.List[object]]$rows,
    [string]$id,
    [string]$capture,
    [string]$language,
    [string]$fidelity,
    [string]$focusMode,
    [string]$question,
    [string]$oracle,
    [string[]]$zeroTolerance
) {
    $directory = Join-Path $captures $capture
    $baselinePath = Join-Path $directory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $directory "schema-vnext-b1-method-arrow.txt"
    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = [IO.File]::ReadAllText($candidatePath, [Text.Encoding]::UTF8)
    if ((ConvertTo-B1 $capture $baseline) -cne $candidate) {
        throw "${capture}: candidate is not the exact audited B1 transform"
    }

    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $baselinePath },
        [pscustomobject]@{ name = 'candidate'; payload = $candidatePath }
    )) {
        $rows.Add([ordered]@{
            case_id = "b1-method-$id-$($variant.name)"
            task_family = "b1-method-grammar"
            lane = "file"
            fixture = $capture
            fidelity = $fidelity
            intent = if ($focusMode -eq 'focused') { "edit" } else { "refactor" }
            focus_mode = $focusMode
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "b1-candidate" }
            capture = "$capture-$($variant.name)"
            control_full_capture = $variant.payload
            question = $question
            exact_expected_oracle = $oracle
            zero_tolerance = $zeroTolerance
            model = $null
            model_version = $null
            sampling_settings = $null
            actual_model_answer = $null
            pass = $null
            failure_categories = @()
            notes = $null
        })
    }
}

$records = @(Get-Content -Raw -LiteralPath $recordsPath | ConvertFrom-Json -Depth 20)
if ($records.Count -ne 30) { throw "Expected 30 B1 token records, found $($records.Count)" }

$rows = [Collections.Generic.List[object]]::new()
Add-Pair $rows "parameter-return" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" "none" `
    "What is the parameter and return type of getUserById?" `
    "getUserById takes userId: UserId and returns Promise<ApiResponse<UserProfile>>." `
    @("parameter attributed to another method", "return type attributed to another method")

Add-Pair $rows "parameterless" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" "none" `
    "Does isAuthenticated take any parameters, and what does it return?" `
    "isAuthenticated takes no parameters and returns boolean." `
    @("invented parameter", "wrong return type")

Add-Pair $rows "multiline-constructor" `
    "economics-typescript-largeservice-ts-high-none" "typescript" "high" "none" `
    "In the constructor, what are the types of userRepository and dataSource?" `
    "userRepository has type Repository<UserEntity>; dataSource has type DataSource." `
    @("multiline continuation detached from constructor", "wrong dependency type")

Add-Pair $rows "owner-distinction" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "Compare CreateOrderAsync under OrderService and IOrderService: what return payload is shown for each?" `
    "OrderService.CreateOrderAsync shows Task<OrderResponse>; IOrderService.CreateOrderAsync shows a semicolon placeholder (;)." `
    @("class and interface method conflated", "return payload assigned to the wrong owner")

Add-Pair $rows "focused-edit" `
    "economics-angular-usermanagementservice-ts-edit-focused" "angular" "edit" "focused" `
    "Which method has the exact body, and what parameter and return type does its signature declare?" `
    "createUser has the exact body. It takes userData: Partial<UserProfile> and returns Promise<ApiResponse<UserProfile>>." `
    @("body bound to another method", "signature bound to another method")

if ($rows.Count -ne 10) { throw "Expected 10 paired B1 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 10) {
    throw "B1 reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-b1-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b1-reasoning-template.json (10 paired cases; zero model calls)"
