# Measure SCHEMA-vNext B2 independently: replace decorative class comments
# with typed C records and update the cold legend to define C=class.
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
$candidateHeader = "// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method `$=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$classPattern = '(?ms)^// ── (?<name>.*?) ──(?=\r?$)'

function ConvertTo-B2ClassGrammar([string]$capture, [string]$baseline) {
    $headerCount = [regex]::Matches(
        $baseline,
        "(?m)^$([regex]::Escape($baselineHeader))$"
    ).Count
    if ($headerCount -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header, found $headerCount"
    }

    $classMatches = [regex]::Matches($baseline, $classPattern)
    $classCount = $classMatches.Count
    if (-not $classCount) { throw "${capture}: no class boundaries" }
    $classNames = @(
        for ($index = 0; $index -lt $classCount; $index++) {
            $classMatches[$index].Groups['name'].Value
        }
    )
    if (@($classNames | Where-Object { -not $_ }).Count) {
        throw "${capture}: empty class-boundary payload"
    }
    $interfaceRows = @([regex]::Matches($baseline, '(?m)^Q [^\r\n]+$') |
        ForEach-Object Value)
    $interfaceLegendCount = [regex]::Matches($baseline, '(?m)^// Q=interface$').Count

    # Apply positional replacements before changing the header length; the
    # match indices belong to the original baseline string.
    $candidate = $baseline
    for ($index = $classCount - 1; $index -ge 0; $index--) {
        $classMatch = $classMatches[$index]
        $replacement = 'C ' + $classMatch.Groups['name'].Value
        $candidate = $candidate.Remove($classMatch.Index, $classMatch.Length).
            Insert($classMatch.Index, $replacement)
    }
    $candidate = $candidate.Replace($baselineHeader, $candidateHeader)

    if ([regex]::IsMatch($candidate, $classPattern)) {
        throw "${capture}: B2 left a decorative class boundary"
    }
    $candidateClassCount = [regex]::Matches($candidate, '(?m)^C ').Count
    if ($candidateClassCount -ne $classCount) {
        throw "${capture}: B2 changed class count ($classCount -> $candidateClassCount)"
    }
    foreach ($className in $classNames) {
        if (-not $candidate.Contains('C ' + $className, [StringComparison]::Ordinal)) {
            throw "${capture}: B2 changed or lost a class payload"
        }
    }
    $candidateInterfaces = @([regex]::Matches($candidate, '(?m)^Q [^\r\n]+$') |
        ForEach-Object Value)
    if (($candidateInterfaces -join "`n") -cne ($interfaceRows -join "`n")) {
        throw "${capture}: B2 changed interface rows"
    }
    if ([regex]::Matches($candidate, '(?m)^// Q=interface$').Count -ne $interfaceLegendCount) {
        throw "${capture}: B2 changed the conditional interface legend"
    }
    if (-not $candidate.StartsWith($candidateHeader, [StringComparison]::Ordinal)) {
        throw "${capture}: B2 candidate does not carry the complete vNext legend"
    }

    return [pscustomobject]@{
        text = $candidate
        class_count = $classCount
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
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-b2-class-record.txt"
    if (-not (Test-Path -LiteralPath $baselinePath)) {
        throw "${capture}: missing schema-v5-candidate.txt"
    }

    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-B2ClassGrammar $capture $baseline
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
            fidelity = [string]$baselineRow.fidelity
            focus_mode = [string]$baselineRow.focus_mode
            tokenizer = $tokenizer
            tokenizer_implementation = [string]$baselineRow.tokenizer_implementation
            class_count = $candidate.class_count
            schema_v5_baseline_tokens = $baselineTokens
            b2_class_grammar_tokens = $candidateTokens
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

$output = Join-Path $captures "schema-vnext-b2-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b2-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} classes={3,2} baseline={4,5} -> B2={5,5} saved={6,4} ({7,6}%)" -f
            $row.language, $row.fidelity, $row.focus_mode, $row.class_count,
            $row.schema_v5_baseline_tokens, $row.b2_class_grammar_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent
        )
    }
}
