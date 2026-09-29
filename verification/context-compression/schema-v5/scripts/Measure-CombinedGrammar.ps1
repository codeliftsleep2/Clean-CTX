# Measure the selected combined SCHEMA-vNext grammar: B1 method arrows,
# B2 typed class records, and B3 Medium/High grouped fields.
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
$candidateHeader = "// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method `$=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$classPattern = '(?ms)^// ── (?<name>.*?) ──(?=\r?$)'
$fieldRunPattern = '(?m)(?:^F [^\r\n]+(?:\r?\n|$)){2,}'
$scopeArrowPattern = '(?m)^(M [^\r\n]*?)  → '

function FieldPayloads([string]$text) {
    return @([regex]::Matches($text, '(?m)^F (?<payload>[^\r\n]+)$') |
        ForEach-Object { $_.Groups['payload'].Value })
}

function ConvertTo-CombinedGrammar(
    [string]$capture,
    [string]$fidelity,
    [string]$baseline
) {
    if ([regex]::Matches($baseline, "(?m)^$([regex]::Escape($baselineHeader))$").Count -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header"
    }

    $classMatches = [regex]::Matches($baseline, $classPattern)
    $methodCount = [regex]::Matches($baseline, '(?m)^M ').Count
    $scopeArrowCount = [regex]::Matches($baseline, $scopeArrowPattern).Count
    $baselineArrowCount = [regex]::Matches($baseline, '→').Count
    $baselineFields = @(FieldPayloads $baseline)
    if (-not $classMatches.Count -or -not $methodCount -or -not $baselineFields.Count) {
        throw "${capture}: combined candidate requires classes, methods, and fields"
    }
    if ($scopeArrowCount -ne $methodCount) {
        throw "${capture}: not every method owns one removable scope arrow"
    }

    $interfaceRows = @([regex]::Matches($baseline, '(?m)^Q [^\r\n]+$') |
        ForEach-Object Value)
    $interfaceLegendCount = [regex]::Matches($baseline, '(?m)^// Q=interface$').Count
    $replacements = [Collections.Generic.List[object]]::new()
    foreach ($classMatch in $classMatches) {
        $replacements.Add([pscustomobject]@{
            index = $classMatch.Index
            length = $classMatch.Length
            text = 'C ' + $classMatch.Groups['name'].Value
        })
    }

    $removedFieldRows = 0
    $groupedRuns = 0
    if ($fidelity -in @('medium', 'high')) {
        $fieldRuns = [regex]::Matches($baseline, $fieldRunPattern)
        $groupedRuns = $fieldRuns.Count
        foreach ($fieldRun in $fieldRuns) {
            $payloads = @(FieldPayloads $fieldRun.Value)
            if ($payloads.Count -lt 2) {
                throw "${capture}: combined candidate matched a non-groupable field run"
            }
            $removedFieldRows += $payloads.Count - 1
            $lineEnding = if ($fieldRun.Value.EndsWith("`r`n")) {
                "`r`n"
            } elseif ($fieldRun.Value.EndsWith("`n")) {
                "`n"
            } else {
                ""
            }
            $replacements.Add([pscustomobject]@{
                index = $fieldRun.Index
                length = $fieldRun.Length
                text = 'F ' + ($payloads -join ' ') + $lineEnding
            })
        }
        if (-not $groupedRuns -or -not $removedFieldRows) {
            throw "${capture}: Medium/High candidate had no B3 opportunity"
        }
    }

    $candidate = $baseline
    foreach ($replacement in $replacements | Sort-Object index -Descending) {
        $candidate = $candidate.Remove($replacement.index, $replacement.length).
            Insert($replacement.index, $replacement.text)
    }
    $candidate = [regex]::Replace($candidate, $scopeArrowPattern, '$1 ')
    $candidate = $candidate.Replace($baselineHeader, $candidateHeader)

    if ([regex]::Matches($candidate, '(?m)^C ').Count -ne $classMatches.Count) {
        throw "${capture}: combined candidate changed the class count"
    }
    if ([regex]::IsMatch($candidate, $classPattern)) {
        throw "${capture}: combined candidate left a decorative class boundary"
    }
    if ([regex]::Matches($candidate, '(?m)^M ').Count -ne $methodCount -or
        [regex]::IsMatch($candidate, $scopeArrowPattern)) {
        throw "${capture}: combined candidate changed method rows incorrectly"
    }
    if ([regex]::Matches($candidate, '→').Count -ne $baselineArrowCount - $methodCount) {
        throw "${capture}: combined candidate changed an arrow outside B1"
    }
    $candidateFields = @(FieldPayloads $candidate)
    if (($candidateFields -join ' ') -cne ($baselineFields -join ' ')) {
        throw "${capture}: combined candidate changed field payloads or ordering"
    }
    if ($candidateFields.Count -ne $baselineFields.Count - $removedFieldRows) {
        throw "${capture}: combined candidate changed an unexpected field-row count"
    }
    $candidateInterfaces = @([regex]::Matches($candidate, '(?m)^Q [^\r\n]+$') |
        ForEach-Object Value)
    if (($candidateInterfaces -join "`n") -cne ($interfaceRows -join "`n") -or
        [regex]::Matches($candidate, '(?m)^// Q=interface$').Count -ne $interfaceLegendCount) {
        throw "${capture}: combined candidate changed interface presentation"
    }
    if (-not $candidate.StartsWith($candidateHeader, [StringComparison]::Ordinal)) {
        throw "${capture}: combined candidate does not carry the complete legend"
    }

    return [pscustomobject]@{
        text = $candidate
        class_count = $classMatches.Count
        method_count = $methodCount
        baseline_field_rows = $baselineFields.Count
        candidate_field_rows = $candidateFields.Count
        grouped_runs = $groupedRuns
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
    if ($fidelities.Count -ne 1) { throw "${capture}: tokenizer fidelity mismatch" }
    $fidelity = [string]$fidelities[0]
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-combined.txt"
    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-CombinedGrammar $capture $fidelity $baseline
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
            class_count = $candidate.class_count
            method_count = $candidate.method_count
            baseline_field_rows = $candidate.baseline_field_rows
            candidate_field_rows = $candidate.candidate_field_rows
            grouped_runs = $candidate.grouped_runs
            schema_v5_baseline_tokens = $baselineTokens
            combined_tokens = $candidateTokens
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

$output = Join-Path $captures "schema-vnext-combined-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-combined-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} C={3,2} M={4,2} F={5,3}->{6,2} baseline={7,5} -> combined={8,5} saved={9,4} ({10,6}%)" -f
            $row.language, $row.fidelity, $row.focus_mode, $row.class_count,
            $row.method_count, $row.baseline_field_rows, $row.candidate_field_rows,
            $row.schema_v5_baseline_tokens, $row.combined_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent
        )
    }
}
