# Prepare paired SCHEMA-v5/B2 reasoning cases for typed class records.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$recordsPath = Join-Path $captures "schema-vnext-b2-records.json"

if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-b2-records.json. Run Measure-B2ClassRecord.ps1"
}

$baselineHeader = "// SCHEMA v5  @=meta X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$candidateHeader = "// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$classPattern = '(?ms)^// ── (?<name>.*?) ──(?=\r?$)'

function ConvertTo-B2([string]$capture, [string]$baseline) {
    if ([regex]::Matches($baseline, "(?m)^$([regex]::Escape($baselineHeader))$").Count -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header"
    }
    $matches = [regex]::Matches($baseline, $classPattern)
    if (-not $matches.Count) { throw "${capture}: no class boundaries" }
    $candidate = $baseline
    for ($index = $matches.Count - 1; $index -ge 0; $index--) {
        $match = $matches[$index]
        $replacement = 'C ' + $match.Groups['name'].Value
        $candidate = $candidate.Remove($match.Index, $match.Length).
            Insert($match.Index, $replacement)
    }
    $candidate = $candidate.Replace($baselineHeader, $candidateHeader)
    if ([regex]::IsMatch($candidate, $classPattern)) {
        throw "${capture}: B2 left a decorative class boundary"
    }
    if ([regex]::Matches($candidate, '(?m)^C ').Count -ne $matches.Count) {
        throw "${capture}: B2 changed the class-boundary count"
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
    $candidatePath = Join-Path $directory "schema-vnext-b2-class-record.txt"
    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = [IO.File]::ReadAllText($candidatePath, [Text.Encoding]::UTF8)
    if ((ConvertTo-B2 $capture $baseline) -cne $candidate) {
        throw "${capture}: candidate is not the exact audited B2 transform"
    }

    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $baselinePath },
        [pscustomobject]@{ name = 'candidate'; payload = $candidatePath }
    )) {
        $rows.Add([ordered]@{
            case_id = "b2-class-$id-$($variant.name)"
            task_family = "b2-class-grammar"
            lane = "file"
            fixture = $capture
            fidelity = $fidelity
            intent = if ($focusMode -eq 'focused') { "edit" } else { "refactor" }
            focus_mode = $focusMode
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "b2-candidate" }
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
if ($records.Count -ne 30) { throw "Expected 30 B2 token records, found $($records.Count)" }

$rows = [Collections.Generic.List[object]]::new()
Add-Pair $rows "angular-owner" `
    "economics-angular-usermanagementservice-ts-high-none" "angular" "high" "none" `
    "Which class owns getUserById and the apiBasePath field?" `
    "UserManagementService owns both getUserById and apiBasePath." `
    @("method assigned to an interface", "field assigned to an interface")

Add-Pair $rows "typescript-owner" `
    "economics-typescript-largeservice-ts-high-none" "typescript" "high" "none" `
    "Which class owns createUser and healthCheck?" `
    "UserService owns both createUser and healthCheck." `
    @("method assigned to another owner", "class boundary missed")

Add-Pair $rows "multiclass-owner" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "Which class owns the Orders and OrderItems fields?" `
    "OrderManagementDbContext owns both Orders and OrderItems." `
    @("fields assigned to adjacent Order or OrderItem class", "wrong class owner")

Add-Pair $rows "multiline-record" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "Which record declaration contains the Line1 and Country parameters?" `
    "The Address record declaration contains Line1 and Country." `
    @("multiline record detached from its name", "wrong record owner")

Add-Pair $rows "inheritance-interface" `
    "economics-csharp-dotnet-ordermanagementservice-cs-high-none" "csharp" "high" "none" `
    "What does OrderManagementDbContext extend, and is IOrderService shown as a class or an interface?" `
    "OrderManagementDbContext extends DbContext. IOrderService is shown as an interface." `
    @("inheritance assigned to adjacent class", "interface treated as class")

Add-Pair $rows "focused-body-owner" `
    "economics-csharp-dotnet-ordermanagementservice-cs-edit-focused" "csharp" "edit" "focused" `
    "Which owner has the exact CreateOrderAsync body, and which owner has only its interface signature?" `
    "OrderService has the exact CreateOrderAsync body; IOrderService has only the interface signature." `
    @("body assigned to interface", "class and interface owners conflated")

if ($rows.Count -ne 12) { throw "Expected 12 paired B2 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 12) {
    throw "B2 reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-b2-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b2-reasoning-template.json (12 paired cases; zero model calls)"
