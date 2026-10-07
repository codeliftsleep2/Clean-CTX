[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Get-Utf8Bytes([string] $Text) {
    $utf8.GetByteCount($Text)
}

function New-Span([int] $Index, [int] $SourceChars = 80) {
    [ordered]@{
        file_name = "src/module_$Index.rs"
        byte_start = 100 + $Index
        byte_end = 110 + $Index
        line_start = 10 + $Index
        line_end = 10 + $Index
        column_start = 5
        column_end = 15
        is_primary = $true
        text = @([ordered]@{
            text = ('x' * $SourceChars)
            highlight_start = 5
            highlight_end = 15
        })
        label = "synthetic span $Index"
        suggested_replacement = "replacement_$Index"
        suggestion_applicability = 'MachineApplicable'
        expansion = $null
    }
}

function New-CompilerMessage(
    [int] $Index,
    [int] $SpanCount = 1,
    [int] $ChildCount = 1,
    [int] $RenderedChars = 320,
    [string] $Level = 'error'
) {
    $spans = @()
    for ($i = 0; $i -lt $SpanCount; $i++) {
        $spans += New-Span -Index ($Index + $i)
    }

    $children = @()
    for ($i = 0; $i -lt $ChildCount; $i++) {
        $children += [ordered]@{
            message = "synthetic child $i for diagnostic $Index"
            code = $null
            level = if ($i % 2 -eq 0) { 'help' } else { 'note' }
            spans = @()
            children = @()
            rendered = $null
        }
    }

    [ordered]@{
        reason = 'compiler-message'
        package_id = "path+file:///fixture#fixture@$Index.0.0"
        manifest_path = 'C:\fixture\Cargo.toml'
        target = [ordered]@{
            kind = @('lib')
            crate_types = @('lib')
            name = 'fixture'
            src_path = 'C:\fixture\src\lib.rs'
            edition = '2021'
            doc = $true
            doctest = $true
            test = $true
        }
        message = [ordered]@{
            message = "synthetic diagnostic $Index"
            code = [ordered]@{ code = "E$('{0:D4}' -f ($Index % 10000))"; explanation = $null }
            level = $Level
            spans = $spans
            children = $children
            rendered = ('R' * $RenderedChars)
        }
    }
}

function Convert-Line($Value) {
    ($Value | ConvertTo-Json -Compress -Depth 30) + "`n"
}

function New-RepeatedLines([string] $Line, [int] $Count) {
    $builder = [System.Text.StringBuilder]::new($Line.Length * $Count)
    for ($i = 0; $i -lt $Count; $i++) {
        [void]$builder.Append($Line)
    }
    $builder.ToString()
}

$smallFinished = Convert-Line ([ordered]@{ reason = 'build-finished'; success = $true })
$baseDiagnostic = Convert-Line (New-CompilerMessage -Index 1)
$fiveDiagnostics = New-RepeatedLines -Line $baseDiagnostic -Count 5
$manyDiagnostics = New-RepeatedLines -Line $baseDiagnostic -Count 500
$longDiagnostic = Convert-Line (New-CompilerMessage -Index 2 -SpanCount 16 -ChildCount 16 -RenderedChars (1024 * 1024))
$stderrTail = (New-RepeatedLines -Line "warning: synthetic build-script noise`n" -Count 30000) + "fatal: decisive terminal cause`n"
$nonJson = New-RepeatedLines -Line "build-script arbitrary stdout with bounded synthetic payload`n" -Count 100000
$malformed = New-RepeatedLines -Line "{`"reason`":`"compiler-message`",`"message`":BROKEN}`n" -Count 100000
$unknownLine = Convert-Line ([ordered]@{ reason = 'future-cargo-record'; payload = ('u' * 512) })
$unknown = New-RepeatedLines -Line $unknownLine -Count 10000
$buildScriptLine = Convert-Line ([ordered]@{
    reason = 'build-script-executed'
    package_id = 'path+file:///fixture#fixture@0.1.0'
    linked_libs = @()
    linked_paths = @()
    cfgs = @('synthetic')
    env = @(@('SYNTHETIC', 'value'))
    out_dir = 'C:\fixture\target\debug\build\fixture\out'
})
$buildScriptNoise = New-RepeatedLines -Line $buildScriptLine -Count 1000
$repeated = New-RepeatedLines -Line $baseDiagnostic -Count 1000
$extremeLine = Convert-Line (New-CompilerMessage -Index 3 -SpanCount 64 -ChildCount 64 -RenderedChars (20 * 1024 * 1024))

$cases = [ordered]@{
    small_clean_check = $smallFinished
    few_compiler_diagnostics = $fiveDiagnostics + $smallFinished
    many_compiler_diagnostics = $manyDiagnostics + $smallFinished
    very_long_rustc_diagnostic = $longDiagnostic
    large_stderr_tail = $stderrTail
    large_non_json_stdout = $nonJson
    malformed_json_flood = $malformed
    unknown_cargo_record_flood = $unknown
    build_script_noise = $buildScriptNoise
    repeated_diagnostics = $repeated
    extreme_single_line_record = $extremeLine
}

$caseFacts = @()
foreach ($entry in $cases.GetEnumerator()) {
    $lines = ($entry.Value -split "`n").Count - 1
    $maxLine = 0
    foreach ($line in ($entry.Value -split "`n")) {
        $lineBytes = Get-Utf8Bytes $line
        if ($lineBytes -gt $maxLine) { $maxLine = $lineBytes }
    }
    $caseFacts += [ordered]@{
        case = $entry.Key
        bytes = Get-Utf8Bytes $entry.Value
        lines = $lines
        maximum_frame_bytes_excluding_newline = $maxLine
    }
}

$captureCandidates = @(2MB, 8MB, 16MB)
$captureSensitivity = @()
foreach ($candidate in $captureCandidates) {
    foreach ($fact in $caseFacts) {
        $captureSensitivity += [ordered]@{
            candidate_bytes = $candidate
            case = $fact.case
            complete = $fact.bytes -le $candidate
            retained_percent = [math]::Round(100 * [math]::Min(1.0, $candidate / [double]$fact.bytes), 2)
        }
    }
}

$frameCandidates = @(512KB, 2MB, 8MB)
$frameSensitivity = @()
foreach ($candidate in $frameCandidates) {
    foreach ($fact in $caseFacts) {
        $frameSensitivity += [ordered]@{
            candidate_bytes = $candidate
            case = $fact.case
            accepts_largest_frame = $fact.maximum_frame_bytes_excluding_newline -le $candidate
        }
    }
}

$diagnosticCandidates = @(32, 64, 128)
$diagnosticSensitivity = @()
foreach ($candidate in $diagnosticCandidates) {
    $diagnosticSensitivity += [ordered]@{
        candidate = $candidate
        unique_many_case_retained_percent = [math]::Round(100 * $candidate / 500, 2)
        repeated_case_semantic_instances_retained = $candidate
    }
}

$representativeDiagnosticBytes = Get-Utf8Bytes $baseDiagnostic
$projection = foreach ($count in @(32, 64, 128)) {
    [ordered]@{
        diagnostic_count = $count
        approximate_uncompacted_json_bytes = $representativeDiagnosticBytes * $count
    }
}

$recommendedFrameBytes = 2MB
$boundaryCases = @(
    [ordered]@{ case = 'below_limit'; bytes_before_newline = $recommendedFrameBytes - 1; expected = 'parse_candidate' },
    [ordered]@{ case = 'exactly_limit'; bytes_before_newline = $recommendedFrameBytes; expected = 'parse_candidate' },
    [ordered]@{ case = 'one_byte_over'; bytes_before_newline = $recommendedFrameBytes + 1; expected = 'reject_and_drain_to_delimiter' },
    [ordered]@{ case = 'far_over'; bytes_before_newline = 20MB; expected = 'reject_and_drain_to_delimiter' },
    [ordered]@{ case = 'unterminated_at_eof_below_limit'; bytes_before_newline = $recommendedFrameBytes - 1; expected = 'truncated_frame_evidence' },
    [ordered]@{ case = 'unterminated_at_eof_over_limit'; bytes_before_newline = $recommendedFrameBytes + 1; expected = 'over_limit_truncated_frame_evidence' },
    [ordered]@{ case = 'invalid_utf8_below_limit'; bytes_before_newline = 1024; expected = 'decode_fact_and_sanitized_evidence' },
    [ordered]@{ case = 'invalid_utf8_at_boundary'; bytes_before_newline = $recommendedFrameBytes; expected = 'decode_fact_and_sanitized_evidence' },
    [ordered]@{ case = 'valid_huge_json'; bytes_before_newline = $longDiagnostic.Length - 1; expected = 'parse_candidate' },
    [ordered]@{ case = 'malformed_huge_json'; bytes_before_newline = 20MB; expected = 'reject_and_drain_to_delimiter' }
)

[ordered]@{
    generated_at = 'deterministic-no-clock'
    invokes_cargo = $false
    representative_diagnostic_bytes = $representativeDiagnosticBytes
    cases = $caseFacts
    capture_sensitivity = $captureSensitivity
    frame_sensitivity = $frameSensitivity
    diagnostic_sensitivity = $diagnosticSensitivity
    structured_projection = $projection
    recommended_frame_boundary_cases = $boundaryCases
} | ConvertTo-Json -Depth 20
