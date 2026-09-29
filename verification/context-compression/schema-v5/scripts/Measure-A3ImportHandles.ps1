# Measure SCHEMA-vNext A3 independently: remove presentation-only generated
# import handles while preserving canonical IR and the visible import payload.
# Uses the current baseline candidates; invokes neither a model nor Clean-CTX.

param([string]$CapturePattern = "economics-*")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$captures = Join-Path $repositoryRoot "target\context-compression-verification\captures"
$measure = Join-Path $repositoryRoot "target\context-compression-verification\scripts\measure.exe"
$anatomyRecords = Join-Path $captures "schema-v5-anatomy-records.json"

if (-not (Test-Path -LiteralPath $measure)) {
    throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1"
}
if (-not (Test-Path -LiteralPath $anatomyRecords)) {
    throw "Missing schema-v5-anatomy-records.json. Run Measure-SchemaV5Anatomy.ps1"
}

function ConvertTo-A3WithoutImportHandles([string]$capture, [string]$baseline) {
    $importPattern = '(?m)^\$ (?<handle>IM\d+) (?<payload>[^\r\n]*)(?=\r?$)'
    $imports = @([regex]::Matches($baseline, $importPattern))
    if (-not $imports.Count) { throw "${capture}: no generated import handles to measure" }

    $handles = @($imports | ForEach-Object { $_.Groups['handle'].Value })
    if (($handles | Sort-Object -Unique).Count -ne $handles.Count) {
        throw "${capture}: generated import handles are not unique"
    }
    foreach ($handle in $handles) {
        $referencePattern = "(?<![A-Za-z0-9_])$([regex]::Escape($handle))(?![A-Za-z0-9_])"
        $references = [regex]::Matches($baseline, $referencePattern)
        if ($references.Count -ne 1) {
            throw "${capture}: $handle has $($references.Count) visible references; A3 requires exactly its defining import row"
        }
    }

    $candidate = $baseline
    $removedCharacters = 0
    $normalizedEmptyModules = 0
    foreach ($import in $imports | Sort-Object Index -Descending) {
        $payload = $import.Groups['payload'].Value
        # C# imports encode an empty module field as the second separator in
        # `$ IM1  [using ...]`. Once the handle field is hidden, that empty
        # field must not leave a phantom column in the visible grammar.
        if ($payload.StartsWith(' ')) {
            $payload = $payload.Substring(1)
            $removedCharacters += 1
            $normalizedEmptyModules += 1
        }
        $replacement = '$ ' + $payload
        $candidate = $candidate.Remove($import.Index, $import.Length).Insert($import.Index, $replacement)
        $removedCharacters += $import.Groups['handle'].Length + 1
    }
    if ($candidate.Length -ne $baseline.Length - $removedCharacters) {
        throw "${capture}: A3 changed content outside generated handle spelling"
    }
    foreach ($handle in $handles) {
        $referencePattern = "(?<![A-Za-z0-9_])$([regex]::Escape($handle))(?![A-Za-z0-9_])"
        if ([regex]::IsMatch($candidate, $referencePattern)) {
            throw "${capture}: A3 left visible reference $handle"
        }
    }

    return [pscustomobject]@{
        text = $candidate
        import_count = $imports.Count
        normalized_empty_module_count = $normalizedEmptyModules
        removed_characters = $removedCharacters
    }
}

$baselineRows = @(Get-Content -Raw -LiteralPath $anatomyRecords | ConvertFrom-Json -Depth 20 |
    Where-Object { $_.capture -like $CapturePattern })
if (-not $baselineRows.Count) { throw "No baseline rows matched $CapturePattern" }

$gitCommit = (& git -C $repositoryRoot rev-parse HEAD | Out-String).Trim()
$gitDirty = [bool]((& git -C $repositoryRoot status --porcelain --untracked-files=no | Out-String).Trim())
$records = @()

foreach ($captureGroup in $baselineRows | Group-Object capture) {
    $capture = $captureGroup.Name
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-a3-import-handles.txt"
    if (-not (Test-Path -LiteralPath $baselinePath)) {
        throw "${capture}: missing schema-v5-candidate.txt"
    }

    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-A3WithoutImportHandles $capture $baseline
    [IO.File]::WriteAllText($candidatePath, $candidate.text, [Text.UTF8Encoding]::new($false))
    $candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidatePath).Hash.ToLowerInvariant()

    foreach ($baselineRow in $captureGroup.Group) {
        $tokenizer = [string]$baselineRow.tokenizer
        $candidateTokens = [int]((& $measure count $tokenizer $candidatePath | Out-String).Trim())
        $baselineTokens = [int]$baselineRow.complete_schema_v5_tokens
        $savedTokens = $baselineTokens - $candidateTokens
        $records += [ordered]@{
            capture = $capture
            language = [string]$baselineRow.language
            fidelity = [string]$baselineRow.fidelity
            focus_mode = [string]$baselineRow.focus_mode
            tokenizer = $tokenizer
            tokenizer_implementation = [string]$baselineRow.tokenizer_implementation
            import_count = $candidate.import_count
            normalized_empty_module_count = $candidate.normalized_empty_module_count
            schema_v5_baseline_tokens = $baselineTokens
            a3_without_import_handles_tokens = $candidateTokens
            saved_tokens = $savedTokens
            baseline_reduction_percent = [Math]::Round((100.0 * $savedTokens) / $baselineTokens, 2)
            baseline_sha256 = [string]$baselineRow.candidate_sha256
            candidate_sha256 = $candidateHash
            measurement_git_commit = $gitCommit
            measurement_worktree_dirty = $gitDirty
        }
    }
}

$output = Join-Path $captures "schema-vnext-a3-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-a3-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} imports={3,2} baseline={4,5} -> A3={5,5} saved={6,3} ({7,5}%)" -f
            $row.language, $row.fidelity, $row.focus_mode, $row.import_count,
            $row.schema_v5_baseline_tokens, $row.a3_without_import_handles_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent
        )
    }
}
