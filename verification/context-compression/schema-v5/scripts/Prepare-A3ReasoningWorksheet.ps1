# Prepare paired baseline/A3 reasoning cases for generated import-handle removal.
# The runner invokes a fresh model for each row; this script makes no model calls.

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$measure = Join-Path $repositoryRoot "target\context-compression-verification\scripts\measure.exe"
$recordsPath = Join-Path $captures "schema-vnext-a3-records.json"

if (-not (Test-Path -LiteralPath $measure)) {
    throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1"
}
if (-not (Test-Path -LiteralPath $recordsPath)) {
    throw "Missing schema-vnext-a3-records.json. Run Measure-A3ImportHandles.ps1"
}

function ConvertTo-A3([string]$capture, [string]$baseline) {
    $imports = @([regex]::Matches(
        $baseline,
        '(?m)^\$ (?<handle>IM\d+) (?<payload>[^\r\n]*)(?=\r?$)'
    ))
    if (-not $imports.Count) { throw "${capture}: no generated import handles" }

    $candidate = $baseline
    foreach ($import in $imports | Sort-Object Index -Descending) {
        $handle = $import.Groups['handle'].Value
        $references = [regex]::Matches(
            $baseline,
            "(?<![A-Za-z0-9_])$([regex]::Escape($handle))(?![A-Za-z0-9_])"
        )
        if ($references.Count -ne 1) {
            throw "${capture}: $handle is not presentation-local"
        }
        $payload = $import.Groups['payload'].Value
        if ($payload.StartsWith(' ')) { $payload = $payload.Substring(1) }
        $replacement = '$ ' + $payload
        $candidate = $candidate.Remove($import.Index, $import.Length).Insert($import.Index, $replacement)
    }
    return $candidate
}

function Add-PairedCases(
    [Collections.Generic.List[object]]$rows,
    [string]$id,
    [string]$fixture,
    [string]$language,
    [string]$baselinePath,
    [string]$candidatePath,
    [string]$question,
    [string]$oracle,
    [string[]]$zeroTolerance
) {
    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = [IO.File]::ReadAllText($candidatePath, [Text.Encoding]::UTF8)
    if ((ConvertTo-A3 $fixture $baseline) -cne $candidate) {
        throw "${fixture}: candidate is not the exact audited A3 transform"
    }

    foreach ($variant in @(
        [pscustomobject]@{ name = 'baseline'; payload = $baselinePath },
        [pscustomobject]@{ name = 'candidate'; payload = $candidatePath }
    )) {
        $rows.Add([ordered]@{
            case_id = "a3-import-$id-$($variant.name)"
            task_family = "a3-import-meaning"
            lane = "file"
            fixture = $fixture
            fidelity = "low"
            intent = "overview"
            focus_mode = "none"
            production_operation = if ($variant.name -eq 'baseline') { "captured-baseline" } else { "a3-candidate" }
            capture = "$fixture-$($variant.name)"
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
$lowRows = @($records |
    Where-Object { $_.tokenizer -eq 'o200k' -and $_.fidelity -eq 'low' -and $_.focus_mode -eq 'none' } |
    Sort-Object language)
if ($lowRows.Count -ne 3) {
    throw "Expected one Low A3 record for each of three languages, found $($lowRows.Count)"
}

$questions = @{
    angular = @{
        question = "Which modules provide HttpClient and firstValueFrom?"
        oracle = "HttpClient is imported from @angular/common/http. firstValueFrom is imported from rxjs."
    }
    csharp = @{
        question = "Which imported namespaces provide Task and Entity Framework's DbContext API?"
        oracle = "Task is provided by System.Threading.Tasks. DbContext is provided by Microsoft.EntityFrameworkCore."
    }
    typescript = @{
        question = "Which modules provide Repository, UserEntity, and EncryptionService?"
        oracle = "Repository is imported from typeorm; UserEntity from ./entities/user.entity; EncryptionService from ../security/encryption.service."
    }
}

$rows = [Collections.Generic.List[object]]::new()
foreach ($record in $lowRows) {
    $capture = [string]$record.capture
    $language = [string]$record.language
    $captureDirectory = Join-Path $captures $capture
    Add-PairedCases $rows $language $capture $language `
        (Join-Path $captureDirectory "schema-v5-candidate.txt") `
        (Join-Path $captureDirectory "schema-vnext-a3-import-handles.txt") `
        $questions[$language].question $questions[$language].oracle `
        @("wrong module-to-symbol association", "generated IMn handle treated as a source alias")
}

# Add a source-written alias case. This fixture normally selects economical raw
# passthrough, so render its canonical IR solely for this paired reasoning gate.
$aliasCapture = "cross-b"
$aliasDirectory = Join-Path $captures $aliasCapture
$controlFullPath = Join-Path $aliasDirectory "control-full.txt"
$responsePath = Join-Path $aliasDirectory "oracle-source-response.json"
if (-not (Test-Path -LiteralPath $controlFullPath) -or
    -not (Test-Path -LiteralPath $responsePath)) {
    throw "Missing cross-b alias capture; run the production capture harness"
}
$controlText = [IO.File]::ReadAllText($controlFullPath, [Text.Encoding]::UTF8)
$jsonStart = $controlText.IndexOf('{')
if ($jsonStart -lt 0) { throw "cross-b control-full has no JSON payload" }
$footerStart = $controlText.IndexOf("`n// ", $jsonStart, [StringComparison]::Ordinal)
if ($footerStart -lt 0) { throw "cross-b control-full has no presentation footer boundary" }
$semantic = $controlText.Substring($jsonStart, $footerStart - $jsonStart) |
    ConvertFrom-Json -Depth 100
$sourcePath = [string]$semantic.file.source_path
$aliasBaselinePath = Join-Path $aliasDirectory "schema-vnext-a3-alias-baseline.txt"
$aliasCandidatePath = Join-Path $aliasDirectory "schema-vnext-a3-alias-candidate.txt"
& $measure prod $responsePath $sourcePath low "" o200k renderer $aliasBaselinePath
if ($LASTEXITCODE -ne 0) { throw "Could not render cross-b alias baseline" }
$aliasBaseline = [IO.File]::ReadAllText($aliasBaselinePath, [Text.Encoding]::UTF8)
if ($aliasBaseline -notmatch '(?m)^\$ IM\d+ \./cross-a \[SharedName as ImportedShared\]$') {
    throw "Source-written import alias was not preserved by the production renderer"
}
$aliasCandidate = ConvertTo-A3 $aliasCapture $aliasBaseline
[IO.File]::WriteAllText($aliasCandidatePath, $aliasCandidate, [Text.UTF8Encoding]::new($false))
Add-PairedCases $rows "source-alias" $aliasCapture "typescript" `
    $aliasBaselinePath $aliasCandidatePath `
    "For the import from ./cross-a, what is the exported name and what local name is it imported as?" `
    "The module ./cross-a exports SharedName, which is imported locally as ImportedShared." `
    @("source-written alias lost or reversed", "generated IMn handle treated as the local source alias")

if ($rows.Count -ne 8) { throw "Expected eight paired A3 reasoning cases, found $($rows.Count)" }
if (($rows.case_id | Sort-Object -Unique).Count -ne 8) {
    throw "A3 reasoning case IDs are not unique"
}

$output = Join-Path $captures "schema-vnext-a3-reasoning-template.json"
$rows | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-a3-reasoning-template.json (8 paired cases; zero model calls)"
