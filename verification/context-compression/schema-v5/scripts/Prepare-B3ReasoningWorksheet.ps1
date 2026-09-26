# Prepare paired SCHEMA-v5/B3 reasoning cases for grouped field rows.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$recordsPath = Join-Path $captures "schema-vnext-b3-records.json"

if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-b3-records.json. Run Measure-B3GroupedFields.ps1"
}

$baselineHeader = "// SCHEMA v5  @=meta X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$candidateHeader = $baselineHeader.Replace('SCHEMA v5', 'SCHEMA vNext')
$fieldRunPattern = '(?m)(?:^F [^\r\n]+(?:\r?\n|$)){2,}'

function FieldPayloads([string]$text) {
    return @([regex]::Matches($text, '(?m)^F (?<payload>[^\r\n]+)$') |
        ForEach-Object { $_.Groups['payload'].Value })
}

function ConvertTo-B3([string]$capture, [string]$baseline) {
    if ([regex]::Matches($baseline, "(?m)^$([regex]::Escape($baselineHeader))$").Count -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header"
    }
    $candidate = $baseline
    $runs = [regex]::Matches($baseline, $fieldRunPattern)
    if (-not $runs.Count) { throw "${capture}: no groupable field runs" }
    for ($index = $runs.Count - 1; $index -ge 0; $index--) {
        $run = $runs[$index]
        $payloads = @(FieldPayloads $run.Value)
        $lineEnding = if ($run.Value.EndsWith("`r`n")) {
            "`r`n"
        } elseif ($run.Value.EndsWith("`n")) {
            "`n"
        } else {
            ""
        }
        $replacement = 'F ' + ($payloads -join ' ') + $lineEnding
        $candidate = $candidate.Remove($run.Index, $run.Length).
            Insert($run.Index, $replacement)
    }
    $candidate = $candidate.Replace($baselineHeader, $candidateHeader)
    return $candidate
}

function Add-Pair(
    [Collections.Generic.List[object]]$rows,
    [string]$id,
    [string]$capture,
    [string]$language,
    [string]$fidelity,
    [string]$question,
    [string]$oracle,
    [string[]]$zeroTolerance
) {
    $directory = Join-Path $captures $capture
    $baselinePath = Join-Path $directory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $directory "schema-vnext-b3-grouped-fields.txt"
    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = [IO.File]::ReadAllText($candidatePath, [Text.Encoding]::UTF8)
    if ((ConvertTo-B3 $capture $baseline) -cne $candidate) {
        throw "${capture}: candidate is not the exact audited B3 transform"
    }

    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $baselinePath },
        [pscustomobject]@{ name = 'candidate'; payload = $candidatePath }
    )) {
        $rows.Add([ordered]@{
            case_id = "b3-fields-$id-$($variant.name)"
            task_family = "b3-grouped-fields"
            lane = "file"
            fixture = $capture
            fidelity = $fidelity
            intent = "refactor"
            focus_mode = "none"
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "b3-candidate" }
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
if ($records.Count -ne 30) { throw "Expected 30 B3 token records, found $($records.Count)" }

$rows = [Collections.Generic.List[object]]::new()
Add-Pair $rows "angular-order" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" `
    "What are the first four fields of UserManagementService, in order?" `
    "userCreated, userUpdated, userDeleted, batchOperationComplete." `
    @("field order changed", "multi-word field fragment treated as a field")

Add-Pair $rows "angular-modifiers" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" `
    "List the five consecutive private readonly fields of UserManagementService that begin with apiBasePath." `
    "apiBasePath, defaultPageSize, maxPageSize, cacheDurationMs, and cache." `
    @("private or readonly treated as field names", "field boundary lost")

Add-Pair $rows "angular-interface-boundary" `
    "economics-angular-usermanagementservice-ts-medium-none" "angular" "medium" `
    "Which interface owns avatarUrl, and which interface owns theme?" `
    "UserProfile owns avatarUrl; UserPreferences owns theme." `
    @("adjacent interfaces conflated", "field assigned to wrong owner")

Add-Pair $rows "angular-negative-membership" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" `
    "Does ApiResponse own totalPages? Identify the actual owner and name one field ApiResponse does own." `
    "No. PaginatedResult owns totalPages; ApiResponse owns statusCode (among its fields)." `
    @("negative membership answered yes", "owner boundary crossed")

Add-Pair $rows "csharp-adjacent-classes" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" `
    "Between Order and OrderItem, which owns TrackingNumber and which owns UnitPrice?" `
    "Order owns TrackingNumber; OrderItem owns UnitPrice." `
    @("adjacent classes conflated", "field assigned to wrong class")

Add-Pair $rows "csharp-service-controller" `
    "economics-csharp-dotnet-ordermanagementservice-cs-medium-none" "csharp" "medium" `
    "List every rendered field owned by OrderService and every rendered field owned by OrdersController." `
    "OrderService owns _db, _logger, _pricing, SalesTaxRate, PromoDiscountRate, and MaxItemsPerOrder. OrdersController owns _service and _logger." `
    @("shared _logger causes owner conflation", "field omitted")

Add-Pair $rows "typescript-modifiers" `
    "economics-typescript-largeservice-ts-high-none" "typescript" "high" `
    "List the base identifiers of all six UserService fields in rendered order, omitting modifiers such as private and readonly." `
    "logger, CACHE_TTL_SECONDS, MAX_BATCH_SIZE, cachePrefix, index, error." `
    @("private or readonly treated as field names", "field order changed")

Add-Pair $rows "typescript-negative" `
    "economics-typescript-largeservice-ts-medium-none" "typescript" "medium" `
    "Are userRepository and dataSource fields in the rendered field list? Explain where they appear instead." `
    "No. They appear as constructor parameters/dependencies, not in the rendered F field list." `
    @("constructor parameter misclassified as field", "negative membership answered yes")

if ($rows.Count -ne 16) { throw "Expected 16 paired B3 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 16) {
    throw "B3 reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-b3-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b3-reasoning-template.json (16 paired cases; zero model calls)"
