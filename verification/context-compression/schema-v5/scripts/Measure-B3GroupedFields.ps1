# Measure SCHEMA-vNext B3 independently: group owner-local field runs at
# Medium/High while leaving existing Low grouping and body-bearing Edit intact.
# Uses current production candidates; invokes neither a model nor Clean-CTX.

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

$baselineHeader = "// SCHEMA v5  @=meta X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$candidateHeader = $baselineHeader.Replace('SCHEMA v5', 'SCHEMA vNext')
$fieldRunPattern = '(?m)(?:^F [^\r\n]+(?:\r?\n|$)){2,}'

function FieldPayloads([string]$text) {
    return @([regex]::Matches($text, '(?m)^F (?<payload>[^\r\n]+)$') |
        ForEach-Object { $_.Groups['payload'].Value })
}

function ConvertTo-B3GroupedFields(
    [string]$capture,
    [string]$fidelity,
    [string]$baseline
) {
    if ([regex]::Matches($baseline, "(?m)^$([regex]::Escape($baselineHeader))$").Count -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header"
    }
    $baselinePayloads = @(FieldPayloads $baseline)
    if (-not $baselinePayloads.Count) { throw "${capture}: no visible field rows" }

    $candidate = $baseline
    $groupedRuns = 0
    $removedRows = 0
    if ($fidelity -in @('medium', 'high')) {
        $runs = [regex]::Matches($baseline, $fieldRunPattern)
        $groupedRuns = $runs.Count
        for ($index = $runs.Count - 1; $index -ge 0; $index--) {
            $run = $runs[$index]
            $payloads = @(FieldPayloads $run.Value)
            if ($payloads.Count -lt 2) {
                throw "${capture}: B3 matched a non-groupable field run"
            }
            $removedRows += $payloads.Count - 1
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
        if (-not $groupedRuns -or -not $removedRows) {
            throw "${capture}: body-free candidate had no grouping opportunity"
        }
    }
    $candidate = $candidate.Replace($baselineHeader, $candidateHeader)

    $candidatePayloadText = @(FieldPayloads $candidate) -join ' '
    $baselinePayloadText = $baselinePayloads -join ' '
    if ($candidatePayloadText -cne $baselinePayloadText) {
        throw "${capture}: B3 changed field payload bytes or ordering"
    }
    $baselineRows = $baselinePayloads.Count
    $candidateRows = @(FieldPayloads $candidate).Count
    if ($candidateRows -ne $baselineRows - $removedRows) {
        throw "${capture}: B3 changed an unexpected number of field rows"
    }
    if ($fidelity -in @('low', 'edit')) {
        $expected = $baseline.Replace($baselineHeader, $candidateHeader)
        if ($candidate -cne $expected) {
            throw "${capture}: B3 changed a Low or Edit payload"
        }
    }
    if (-not $candidate.StartsWith($candidateHeader, [StringComparison]::Ordinal)) {
        throw "${capture}: B3 candidate does not carry the complete vNext header"
    }

    return [pscustomobject]@{
        text = $candidate
        baseline_field_rows = $baselineRows
        candidate_field_rows = $candidateRows
        grouped_runs = $groupedRuns
        removed_rows = $removedRows
    }
}

$baselineRows = @(Get-Content -Raw -LiteralPath $anatomyRecords | ConvertFrom-Json -Depth 20 |
    Where-Object { $_.capture -like $CapturePattern })
if (-not $baselineRows.Count) { throw "No baseline rows matched $CapturePattern" }

$gitCommit = (& git -C $repositoryRoot rev-parse HEAD | Out-String).Trim()
$gitDirty = [bool]((& git -C $repositoryRoot status --porcelain --untracked-files=no |
    Out-String).Trim())
$records = @()

foreach ($captureGroup in $baselineRows | Group-Object capture) {
    $capture = $captureGroup.Name
    $fidelities = @($captureGroup.Group.fidelity | Sort-Object -Unique)
    if ($fidelities.Count -ne 1) {
        throw "${capture}: tokenizer records disagree on fidelity"
    }
    $fidelity = [string]$fidelities[0]
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-b3-grouped-fields.txt"
    if (-not (Test-Path -LiteralPath $baselinePath)) {
        throw "${capture}: missing schema-v5-candidate.txt"
    }

    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-B3GroupedFields $capture $fidelity $baseline
    [IO.File]::WriteAllText(
        $candidatePath,
        $candidate.text,
        [Text.UTF8Encoding]::new($false)
    )
    $candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidatePath).
        Hash.ToLowerInvariant()

    foreach ($baselineRow in $captureGroup.Group) {
        $tokenizer = [string]$baselineRow.tokenizer
        $candidateTokens = [int]((& $measure count $tokenizer $candidatePath |
            Out-String).Trim())
        $baselineTokens = [int]$baselineRow.complete_schema_v5_tokens
        $savedTokens = $baselineTokens - $candidateTokens
        $records += [ordered]@{
            capture = $capture
            language = [string]$baselineRow.language
            fidelity = $fidelity
            focus_mode = [string]$baselineRow.focus_mode
            tokenizer = $tokenizer
            tokenizer_implementation = [string]$baselineRow.tokenizer_implementation
            baseline_field_rows = $candidate.baseline_field_rows
            candidate_field_rows = $candidate.candidate_field_rows
            grouped_runs = $candidate.grouped_runs
            removed_rows = $candidate.removed_rows
            schema_v5_baseline_tokens = $baselineTokens
            b3_grouped_field_tokens = $candidateTokens
            saved_tokens = $savedTokens
            baseline_reduction_percent = [Math]::Round(
                (100.0 * $savedTokens) / $baselineTokens,
                2
            )
            baseline_sha256 = [string]$baselineRow.candidate_sha256
            candidate_sha256 = $candidateHash
            measurement_git_commit = $gitCommit
            measurement_worktree_dirty = $gitDirty
        }
    }
}

$output = Join-Path $captures "schema-vnext-b3-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b3-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} rows={3,3}->{4,2} baseline={5,5} -> B3={6,5} saved={7,4} ({8,6}%)" -f
            $row.language, $row.fidelity, $row.focus_mode,
            $row.baseline_field_rows, $row.candidate_field_rows,
            $row.schema_v5_baseline_tokens, $row.b3_grouped_field_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent
        )
    }
}
