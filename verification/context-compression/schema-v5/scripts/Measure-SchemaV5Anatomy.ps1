# Measure independent token anatomy for the model-visible SCHEMA-v5 candidate.
#
# Uses existing economics captures only. Low/Medium/High use the exact captured
# content. Edit captures selected raw source at the economics boundary, so the
# helper regenerates the complete SCHEMA-v5 candidate from captured canonical
# IR solely for measurement. No model or Clean-CTX server is invoked.

param([string]$CapturePattern = "economics-*")

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$runtimeRoot = Join-Path $repositoryRoot "target\context-compression-verification"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not (Test-Path -LiteralPath $measure)) {
    throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1"
}

function Count-TextTokens([string]$tokenizer, [string]$text) {
    if (-not $text) { return 0 }
    $temp = Join-Path $env:TEMP ("schema-v5-anatomy-$([guid]::NewGuid().ToString('N')).txt")
    try {
        [IO.File]::WriteAllText($temp, $text, [Text.UTF8Encoding]::new($false))
        return [int]((& $measure count $tokenizer $temp | Out-String).Trim())
    } finally {
        Remove-Item -LiteralPath $temp -Force -ErrorAction SilentlyContinue
    }
}

function Read-ControlFull([string]$path) {
    $text = [IO.File]::ReadAllText($path, [Text.Encoding]::UTF8)
    if (-not $text.StartsWith("// CONTROL-FULL v2")) {
        throw "$path is not CONTROL-FULL v2"
    }
    $headerEnd = $text.IndexOf("`n", [StringComparison]::Ordinal)
    $remainder = $text.Substring($headerEnd + 1)
    $footerStart = $remainder.LastIndexOf("`n// ── ", [StringComparison]::Ordinal)
    if ($footerStart -lt 0) { throw "$path has no file footer" }
    return $remainder.Substring(0, $footerStart) | ConvertFrom-Json -Depth 100
}

function Get-Methods($semantic) {
    return @(@($semantic.classes) + @($semantic.interfaces) |
        ForEach-Object { @($_.methods) })
}

function Get-TiktokenVersion {
    $lock = [IO.File]::ReadAllText((Join-Path $repositoryRoot "Cargo.lock"), [Text.Encoding]::UTF8)
    $match = [regex]::Match(
        $lock,
        '(?ms)^name = "tiktoken-rs"\r?\nversion = "([^"]+)"'
    )
    if (-not $match.Success) { return "unknown" }
    return $match.Groups[1].Value
}

function Remove-ExactBodies([string]$candidate, [object[]]$methods) {
    $withoutBodies = $candidate
    foreach ($method in $methods | Where-Object { $null -ne $_.body }) {
        $body = [string]$method.body
        $index = $withoutBodies.IndexOf($body, [StringComparison]::Ordinal)
        if ($index -lt 0) {
            throw "Rendered candidate is missing exact body for $($method.id)/$($method.name)"
        }
        $withoutBodies = $withoutBodies.Remove($index, $body.Length)
    }
    return $withoutBodies
}

function Add-Fragment($families, [string]$family, [string]$text) {
    if ($text) { [void]$families[$family].Add($text) }
}

function Split-SchemaAnatomy([string]$capture, [string]$candidate, [object[]]$methods) {
    $names = @(
        "fixed_legend", "file_path", "declaration_signature",
        "behavior_fact", "imports_type_alias", "exact_body"
    )
    $families = @{}
    foreach ($name in $names) {
        $families[$name] = [Collections.Generic.List[string]]::new()
    }
    foreach ($method in $methods | Where-Object { $null -ne $_.body }) {
        Add-Fragment $families "exact_body" ([string]$method.body)
    }

    $text = Remove-ExactBodies $candidate $methods
    $inMethodSignature = $false
    $inFileFooter = $false
    $lineMatches = [regex]::Matches($text, '[^\r\n]*(?:\r\n|\r|\n|$)')
    foreach ($lineMatch in $lineMatches) {
        $line = $lineMatch.Value
        if (-not $line) { continue }
        $content = $line.TrimEnd("`r", "`n")

        if ($inFileFooter -or $content -match '^// ── α') {
            $inFileFooter = $true
            Add-Fragment $families "file_path" $line
            continue
        }
        if ($content.StartsWith("// SCHEMA v5")) {
            Add-Fragment $families "fixed_legend" $line
            continue
        }
        if ($content -match '^\$ ' -or $content -match '^T ') {
            $inMethodSignature = $false
            Add-Fragment $families "imports_type_alias" $line
            continue
        }
        if ($content -match '^(?:cmod|imod|cl):' -or $content -match '^P ') {
            $inMethodSignature = $false
            Add-Fragment $families "behavior_fact" $line
            continue
        }

        $isMethodStart = $content -match '^M '
        $isContinuation = $inMethodSignature -and $content -match '^\s+'
        if ($isMethodStart -or $isContinuation) {
            $inMethodSignature = $true
            $annotation = [regex]::Match(
                $content,
                ' (?=(?:mod|ctl|pf|fl|cl|cf|df|se|ec):)'
            )
            if ($annotation.Success) {
                Add-Fragment $families "declaration_signature" $content.Substring(0, $annotation.Index)
                $suffix = $content.Substring($annotation.Index)
                $ending = $line.Substring($content.Length)
                Add-Fragment $families "behavior_fact" ($suffix + $ending)
            } else {
                Add-Fragment $families "declaration_signature" $line
            }
            continue
        }

        # C# record/type names can span the decorative owner heading itself:
        # `// ── public sealed record Address(` followed by indented
        # parameter lines and a final `); ──`. These are declaration
        # continuations, not method bodies (exact bodies were removed above).
        if ($content -match '^\s+') {
            Add-Fragment $families "declaration_signature" $line
            continue
        }

        if (-not $content -or $content -match '^// ' -or
            $content -match '^(?:Q|X|I|F) ' -or $content -eq '// Q=interface') {
            $inMethodSignature = $false
            Add-Fragment $families "declaration_signature" $line
            continue
        }

        throw "${capture}: unclassified SCHEMA-v5 line: $content"
    }
    return $families
}

function Join-Fragments($fragments) {
    # Families are diagnostic independent measurements. Joining with a newline
    # preserves readable record boundaries but does not create an additive
    # partition: BPE merges at cross-family boundaries remain tokenizer-local.
    return (@($fragments) -join "`n")
}

$gitCommit = (& git -C $repositoryRoot rev-parse HEAD | Out-String).Trim()
$gitDirty = [bool]((& git -C $repositoryRoot status --porcelain --untracked-files=no | Out-String).Trim())
$tiktokenVersion = Get-TiktokenVersion
$records = @()

foreach ($dir in Get-ChildItem -LiteralPath $captures -Directory |
    Where-Object { $_.Name -like $CapturePattern }) {
    $responsePath = Join-Path $dir.FullName "oracle-source-response.json"
    $rawPath = Join-Path $dir.FullName "raw-source.txt"
    $metaPath = Join-Path $dir.FullName "capture-meta.json"
    $controlFullPath = Join-Path $dir.FullName "control-full.txt"
    if (-not (Test-Path -LiteralPath $responsePath) -or
        -not (Test-Path -LiteralPath $rawPath) -or
        -not (Test-Path -LiteralPath $metaPath) -or
        -not (Test-Path -LiteralPath $controlFullPath)) { continue }

    $meta = Get-Content -Raw -LiteralPath $metaPath | ConvertFrom-Json
    if ($meta.fidelity -notin @("low", "medium", "high", "edit")) { continue }
    $response = Get-Content -Raw -LiteralPath $responsePath | ConvertFrom-Json -Depth 100
    $selected = [string]$response.result.content[0].text
    $semantic = Read-ControlFull $controlFullPath
    $methods = @(Get-Methods $semantic)
    $candidatePath = Join-Path $dir.FullName "schema-v5-candidate.txt"

    if ($selected.StartsWith("// SCHEMA v5")) {
        $candidate = $selected
        [IO.File]::WriteAllText($candidatePath, $candidate, [Text.UTF8Encoding]::new($false))
    } elseif ($meta.fidelity -eq "edit") {
        $focus = ""
        if ($meta.focus_mode -eq "focused") {
            $focus = ([string]$meta.focus_target -split '\.')[-1]
        }
        & $measure prod $responsePath ([string]$semantic.file.source_path) edit $focus o200k renderer $candidatePath
        if ($LASTEXITCODE -ne 0) { throw "SCHEMA-v5 Edit render failed for $($dir.Name)" }
        $candidate = [IO.File]::ReadAllText($candidatePath, [Text.Encoding]::UTF8)
    } else {
        continue
    }

    if (-not $candidate.StartsWith("// SCHEMA v5")) {
        throw "$($dir.Name): complete candidate is not SCHEMA-v5"
    }
    $families = Split-SchemaAnatomy $dir.Name $candidate $methods
    $rawHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $rawPath).Hash.ToLowerInvariant()
    $candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidatePath).Hash.ToLowerInvariant()

    foreach ($tokenizer in @("cl100k", "o200k")) {
        $rawTokens = [int]((& $measure count $tokenizer $rawPath | Out-String).Trim())
        $candidateTokens = [int]((& $measure count $tokenizer $candidatePath | Out-String).Trim())
        $selectedTokens = Count-TextTokens $tokenizer $selected
        $record = [ordered]@{
            capture = $dir.Name
            language = [string]$meta.language
            fidelity = [string]$meta.fidelity
            focus_mode = [string]$meta.focus_mode
            tokenizer = $tokenizer
            tokenizer_implementation = "tiktoken-rs $tiktokenVersion"
            raw_tokens = $rawTokens
            complete_schema_v5_tokens = $candidateTokens
            selected_representation = if ($selected.StartsWith("// SCHEMA v5")) { "schema_v5" } else { [string]$response.result.content_kind }
            selected_tokens = $selectedTokens
            fixed_legend_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.fixed_legend)
            file_path_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.file_path)
            declaration_signature_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.declaration_signature)
            behavior_fact_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.behavior_fact)
            imports_type_alias_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.imports_type_alias)
            exact_body_tokens = Count-TextTokens $tokenizer (Join-Fragments $families.exact_body)
            exact_body_count = @($methods | Where-Object { $null -ne $_.body }).Count
            raw_sha256 = $rawHash
            candidate_sha256 = $candidateHash
            capture_git_commit = if ($meta.PSObject.Properties.Name -contains "git_commit") { [string]$meta.git_commit } else { $null }
            measurement_git_commit = $gitCommit
            measurement_worktree_dirty = $gitDirty
        }
        $records += $record
    }
}

if (-not $records.Count) { throw "No SCHEMA-v5 economics captures were measured" }
$outputName = if ($CapturePattern -eq "economics-*") {
    "schema-v5-anatomy-records.json"
} else {
    $safePattern = $CapturePattern -replace '[^A-Za-z0-9._-]', '_'
    "schema-v5-anatomy-records.$safePattern.json"
}
$output = Join-Path $captures $outputName
$records | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
Write-Host "Wrote $outputName ($($records.Count) records; zero model calls)"
Write-Host "Independent family counts are diagnostic and are not additive across BPE boundaries."
foreach ($tokenizer in @("cl100k", "o200k")) {
    Write-Host ""
    Write-Host "=== $tokenizer ==="
    foreach ($row in $records | Where-Object tokenizer -eq $tokenizer |
        Sort-Object language, fidelity, focus_mode) {
        Write-Host (
            "  {0,-10} {1,-6}/{2,-10} total={3,5} legend={4,3} path={5,3} decl={6,5} facts={7,4} imports={8,4} bodies={9,5} selected={10}:{11}" -f
            $row.language, $row.fidelity, $row.focus_mode,
            $row.complete_schema_v5_tokens, $row.fixed_legend_tokens,
            $row.file_path_tokens, $row.declaration_signature_tokens,
            $row.behavior_fact_tokens, $row.imports_type_alias_tokens,
            $row.exact_body_tokens, $row.selected_representation,
            $row.selected_tokens
        )
    }
}
