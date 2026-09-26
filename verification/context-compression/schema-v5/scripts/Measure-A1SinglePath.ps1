# Measure SCHEMA-vNext A1 independently: remove the duplicated path from the
# decorative file footer while retaining the alias and authoritative PATHMAP.
# Uses existing Phase 0 candidates only; invokes neither a model nor Clean-CTX.

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

function ConvertTo-A1SinglePath([string]$capture, [string]$baseline) {
    $footerPattern = '(?m)^// ── (?<alias>α\d+) \([^\r\n]*\) ──(?=\r?$)'
    $footerMatches = [regex]::Matches($baseline, $footerPattern)
    if ($footerMatches.Count -ne 1) {
        throw "${capture}: expected exactly one decorative file footer, found $($footerMatches.Count)"
    }

    $footer = $footerMatches[0]
    $alias = $footer.Groups['alias'].Value
    $mappingPattern = "(?m)^  $([regex]::Escape($alias)) = [^\r\n]+(?=\r?$)"
    $mappingMatches = [regex]::Matches($baseline, $mappingPattern)
    if ($mappingMatches.Count -ne 1) {
        throw "${capture}: expected exactly one authoritative mapping for $alias, found $($mappingMatches.Count)"
    }

    $replacement = "// $alias"
    $candidate = $baseline.Remove($footer.Index, $footer.Length).Insert($footer.Index, $replacement)
    if ([regex]::Matches($candidate, $footerPattern).Count -ne 0) {
        throw "${capture}: A1 left a duplicated-path footer"
    }
    if ([regex]::Matches($candidate, $mappingPattern).Count -ne 1) {
        throw "${capture}: A1 changed the authoritative path mapping"
    }

    $restored = $candidate.Remove($footer.Index, $replacement.Length).Insert($footer.Index, $footer.Value)
    if ($restored -cne $baseline) {
        throw "${capture}: A1 changed content outside the decorative file footer"
    }
    return $candidate
}

$phase0 = @(Get-Content -Raw -LiteralPath $anatomyRecords | ConvertFrom-Json -Depth 20)
$rows = @($phase0 | Where-Object { $_.capture -like $CapturePattern })
if (-not $rows.Count) { throw "No Phase 0 rows matched $CapturePattern" }

$gitCommit = (& git -C $repositoryRoot rev-parse HEAD | Out-String).Trim()
$gitDirty = [bool]((& git -C $repositoryRoot status --porcelain --untracked-files=no | Out-String).Trim())
$records = @()

foreach ($captureGroup in $rows | Group-Object capture) {
    $capture = $captureGroup.Name
    $captureDirectory = Join-Path $captures $capture
    $baselinePath = Join-Path $captureDirectory "schema-v5-candidate.txt"
    $candidatePath = Join-Path $captureDirectory "schema-vnext-a1-single-path.txt"
    if (-not (Test-Path -LiteralPath $baselinePath)) {
        throw "${capture}: missing schema-v5-candidate.txt"
    }

    $baseline = [IO.File]::ReadAllText($baselinePath, [Text.Encoding]::UTF8)
    $candidate = ConvertTo-A1SinglePath $capture $baseline
    [IO.File]::WriteAllText($candidatePath, $candidate, [Text.UTF8Encoding]::new($false))
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
            raw_tokens = [int]$baselineRow.raw_tokens
            schema_v5_baseline_tokens = $baselineTokens
            a1_single_path_tokens = $candidateTokens
            saved_tokens = $savedTokens
            baseline_reduction_percent = [Math]::Round((100.0 * $savedTokens) / $baselineTokens, 2)
            raw_to_a1_savings_percent = [Math]::Round(
                (100.0 * ([int]$baselineRow.raw_tokens - $candidateTokens)) / [int]$baselineRow.raw_tokens,
                2
            )
            baseline_sha256 = [string]$baselineRow.candidate_sha256
            candidate_sha256 = $candidateHash
            measurement_git_commit = $gitCommit
            measurement_worktree_dirty = $gitDirty
        }
    }
}

$output = Join-Path $captures "schema-vnext-a1-records.json"
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote schema-vnext-a1-records.json ($($records.Count) records; zero model calls)"
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} baseline={3,5} -> A1={4,5} saved={5,3} ({6,5}%) raw-to-A1={7,6}%" -f
            $row.language, $row.fidelity, $row.focus_mode,
            $row.schema_v5_baseline_tokens, $row.a1_single_path_tokens,
            $row.saved_tokens, $row.baseline_reduction_percent,
            $row.raw_to_a1_savings_percent
        )
    }
}
