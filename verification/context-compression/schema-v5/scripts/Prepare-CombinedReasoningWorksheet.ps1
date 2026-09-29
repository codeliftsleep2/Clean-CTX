# Prepare paired SCHEMA-v5/combined-vNext interaction reasoning cases.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$recordsPath = Join-Path $captures "schema-vnext-combined-records.json"

if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-combined-records.json. Run Measure-CombinedGrammar.ps1"
}

$records = @(Get-Content -Raw -LiteralPath $recordsPath | ConvertFrom-Json -Depth 20)
if ($records.Count -ne 30) {
    throw "Expected 30 combined token records, found $($records.Count)"
}

function Assert-CapturedPair([string]$capture) {
    $captureRecords = @($records | Where-Object capture -eq $capture)
    if ($captureRecords.Count -ne 2) {
        throw "${capture}: expected two tokenizer records"
    }
    $candidateHashes = @($captureRecords.candidate_sha256 | Sort-Object -Unique)
    $baselineHashes = @($captureRecords.baseline_sha256 | Sort-Object -Unique)
    if ($candidateHashes.Count -ne 1 -or $baselineHashes.Count -ne 1) {
        throw "${capture}: tokenizer records disagree on candidate identity"
    }
    $directory = Join-Path $captures $capture
    $baselinePath = Join-Path $directory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $directory "schema-vnext-combined.txt"
    foreach ($path in @($baselinePath, $candidatePath)) {
        if (-not (Test-Path -LiteralPath $path)) { throw "Missing $path" }
    }
    $actualBaseline = (Get-FileHash -Algorithm SHA256 -LiteralPath $baselinePath).
        Hash.ToLowerInvariant()
    $actualCandidate = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidatePath).
        Hash.ToLowerInvariant()
    if ($actualBaseline -cne [string]$baselineHashes[0] -or
        $actualCandidate -cne [string]$candidateHashes[0]) {
        throw "${capture}: payload hash does not match the asserted measurement record"
    }
    return [pscustomobject]@{
        baseline = $baselinePath
        candidate = $candidatePath
    }
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
    $paths = Assert-CapturedPair $capture
    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $paths.baseline },
        [pscustomobject]@{ name = 'candidate'; payload = $paths.candidate }
    )) {
        $rows.Add([ordered]@{
            case_id = "combined-$id-$($variant.name)"
            task_family = "schema-vnext-combined"
            lane = "file"
            fixture = $capture
            fidelity = $fidelity
            intent = if ($focusMode -eq 'focused') { "edit" } else { "refactor" }
            focus_mode = $focusMode
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "combined-candidate" }
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

$rows = [Collections.Generic.List[object]]::new()
Add-Pair $rows "angular-method" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" "none" `
    "What parameter and return type does getUserById declare, and which class owns it?" `
    "UserManagementService.getUserById takes userId: UserId and returns Promise<ApiResponse<UserProfile>>." `
    @("parameter/return boundary lost", "wrong class owner")

Add-Pair $rows "typescript-constructor" `
    "economics-typescript-largeservice-ts-high-none" "typescript" "high" "none" `
    "What are the types of constructor dependencies userRepository and dataSource, and does healthCheck take parameters?" `
    "userRepository is Repository<UserEntity>; dataSource is DataSource; healthCheck takes no parameters." `
    @("multiline constructor detached", "parameterless method misparsed")

Add-Pair $rows "csharp-class-interface-method" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "Compare CreateOrderAsync under OrderService and IOrderService: what return payload is shown for each?" `
    "OrderService.CreateOrderAsync shows Task<OrderResponse>; IOrderService.CreateOrderAsync shows a semicolon placeholder (;)." `
    @("class and interface conflated", "return assigned to wrong owner")

Add-Pair $rows "csharp-multiline-record" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "Which record owns Line1 and Country, and which class owns Orders and OrderItems?" `
    "Address owns Line1 and Country; OrderManagementDbContext owns Orders and OrderItems." `
    @("multiline owner lost", "adjacent class ownership crossed")

Add-Pair $rows "csharp-inheritance" `
    "economics-csharp-dotnet-ordermanagementservice-cs-medium-none" "csharp" "medium" "none" `
    "What does OrderManagementDbContext extend, and is IOrderService a class or interface?" `
    "OrderManagementDbContext extends DbContext; IOrderService is an interface." `
    @("inheritance assigned to wrong class", "interface treated as class")

Add-Pair $rows "angular-fields" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" "none" `
    "List the first four UserManagementService fields and then the five consecutive private readonly fields beginning with apiBasePath." `
    "The first four are userCreated, userUpdated, userDeleted, batchOperationComplete. The five private readonly fields are apiBasePath, defaultPageSize, maxPageSize, cacheDurationMs, cache." `
    @("field order changed", "modifier treated as field", "field boundary lost")

Add-Pair $rows "angular-interface-negative" `
    "economics-angular-usermanagementservice-ts-medium-none" "angular" "medium" "none" `
    "Does ApiResponse own totalPages? Identify its actual owner and state which interfaces own avatarUrl and theme." `
    "No. PaginatedResult owns totalPages; UserProfile owns avatarUrl; UserPreferences owns theme." `
    @("negative membership answered yes", "adjacent interfaces conflated")

Add-Pair $rows "csharp-repeated-field" `
    "economics-csharp-dotnet-ordermanagementservice-cs-medium-none" "csharp" "medium" "none" `
    "List every rendered field of OrderService and OrdersController, preserving the repeated _logger ownership." `
    "OrderService owns _db, _logger, _pricing, SalesTaxRate, PromoDiscountRate, MaxItemsPerOrder. OrdersController owns _service and _logger." `
    @("shared _logger owner lost", "field omitted", "field assigned to wrong class")

Add-Pair $rows "typescript-fields" `
    "economics-typescript-largeservice-ts-high-none" "typescript" "high" "none" `
    "List the base identifiers of all UserService fields in order, omitting modifiers. Are userRepository and dataSource in that field list?" `
    "logger, CACHE_TTL_SECONDS, MAX_BATCH_SIZE, cachePrefix, index, error. userRepository and dataSource are constructor dependencies, not rendered fields." `
    @("modifier treated as field name", "constructor dependency misclassified", "field order changed")

Add-Pair $rows "focused-edit" `
    "economics-angular-usermanagementservice-ts-edit-focused" "angular" "edit" "focused" `
    "Which class method has the exact body, and what parameter and return type does that method declare?" `
    "UserManagementService.createUser has the exact body; it takes userData: Partial<UserProfile> and returns Promise<ApiResponse<UserProfile>>." `
    @("body bound to wrong owner", "signature/body association lost")

if ($rows.Count -ne 20) { throw "Expected 20 paired combined cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 20) {
    throw "Combined reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-combined-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-combined-reasoning-template.json (20 paired cases; zero model calls)"
