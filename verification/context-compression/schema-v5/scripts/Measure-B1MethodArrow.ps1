# Measure SCHEMA-vNext B1 independently: remove the leading method scope arrow
# and update the cold legend so `p:` means parameters and `→` means return.
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
$candidateHeader = "// SCHEMA vNext  @=meta X=extends I=implements F=field M=method `$=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias"
$scopeArrowPattern = '(?m)^(M [^\r\n]*?)  → '

function ConvertTo-B1MethodGrammar([string]$capture, [string]$baseline) {
    $headerCount = [regex]::Matches(
        $baseline,
        "(?m)^$([regex]::Escape($baselineHeader))$"
    ).Count
    if ($headerCount -ne 1) {
        throw "${capture}: expected one exact SCHEMA-v5 header, found $headerCount"
    }

    $methodCount = [regex]::Matches($baseline, '(?m)^M ').Count
    $scopeArrowCount = [regex]::Matches($baseline, $scopeArrowPattern).Count
    if (-not $methodCount -or $scopeArrowCount -ne $methodCount) {
        throw "${capture}: $methodCount method rows but $scopeArrowCount removable scope arrows"
    }

    $baselineArrowCount = [regex]::Matches($baseline, '→').Count
    $candidate = $baseline.Replace($baselineHeader, $candidateHeader)
    $candidate = [regex]::Replace($candidate, $scopeArrowPattern, '$1 ')

    if ([regex]::Matches($candidate, '(?m)^M ').Count -ne $methodCount) {
        throw "${capture}: B1 changed the method-row count"
    }
    if ([regex]::IsMatch($candidate, $scopeArrowPattern)) {
        throw "${capture}: B1 left a leading method scope arrow"
    }
    $candidateArrowCount = [regex]::Matches($candidate, '→').Count
    if ($candidateArrowCount -ne $baselineArrowCount - $methodCount) {
        throw "${capture}: B1 changed an arrow outside the one-per-method scope slot"
    }
    if (-not $candidate.StartsWith($candidateHeader, [StringComparison]::Ordinal)) {
        throw "${capture}: B1 candidate does not carry the complete vNext legend"
    }

    return [pscustomobject]@{
        text = $candidate
        method_count = $methodCount
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
    $candidatePath = Join-Path $captureDirectory "schema-vnext-b1-method-arrow.txt"
    if (-not (Test-Path -LiteralPath $baselinePath)) {
        throw "${capture}: missing schema-v5-candidate.txt"
    }

    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-B1MethodGrammar $capture $baseline
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
            method_count = $candidate.method_count
            schema_v5_baseline_tokens = $baselineTokens
            b1_method_grammar_tokens = $candidateTokens
            saved_tokens = $savedTokens
            baseline_reduction_percent = [Math]::Round((100.0 * $savedTokens) / $baselineTokens, 2)
            baseline_sha256 = [string]$baselineRow.candidate_sha256
            candidate_sha256 = $candidateHash
            measurement_git_commit = $gitCommit
            measurement_worktree_dirty = $gitDirty
        }
    }
}

$output = Join-Path $captures "schema-vnext-b1-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-b1-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} methods={3,3} baseline={4,5} -> B1={5,5} saved={6,4} ({7,6}%)" -f
            $row.language, $row.fidelity, $row.focus_mode, $row.method_count,
            $row.schema_v5_baseline_tokens, $row.b1_method_grammar_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent
        )
    }
}
