<#
.SYNOPSIS
    CI Guard: strict UTF-8 validation and mojibake detection across tracked text files.
.DESCRIPTION
    Enforces the project encoding invariant (.clinerules/encoding.md):
      1. Every tracked text file must be valid UTF-8 (strict decode, no BOM).
      2. Files must not contain known mojibake signature sequences produced by
         UTF-8 text misinterpreted as Windows-1252/Latin-1 and re-encoded,
         nor U+FFFD replacement characters.
    Intentional occurrences (documentation that QUOTES mojibake signatures)
    must be registered in $MojibakeAllowedPaths below with justification.
    Silent expansion of the allowlist is forbidden.
.EXAMPLE
    ./scripts/check-utf8.ps1
    PASS: 712 text files, 0 invalid encodings, 0 BOMs, 0 unexplained mojibake hits.
.NOTES
    Exit code 0 = success, 1 = failure.
    Intended for CI (GitHub Actions) and local pre-commit use.
    Source is kept pure ASCII; signature characters are constructed from
    Unicode codepoints at runtime so this file itself cannot be corrupted.
#>

$ErrorActionPreference = "Stop"

# Locate the repository root (script may run from root or scripts/)
$RepoRoot = if (Test-Path ".git") { (Get-Location).Path }
            elseif (Test-Path "../.git") { (Resolve-Path "../").Path }
            else {
                Write-Host "error: run from repository root or scripts/ directory" -ForegroundColor Red
                exit 1
            }

Set-Location $RepoRoot

# --- File selection ---------------------------------------------------------
# Tracked files only; text-ish extensions plus root dotfiles. Binary formats
# (.png, .rlib, .db...) are excluded by design.
$TextPattern = '\.(rs|toml|json|jsonc|ya?ml|md|markdown|txt|html|css|scss|mjs|cjs|js|jsx|ts|tsx|cs|java|sql|xml|csv|tsv|ps1|psm1|lock|gitattributes|dotsettings|sh)$'
$DotFiles    = '^\.(gitignore|gitattributes|editorconfig)$'

$tracked = @(git ls-files --cached --others --exclude-standard | Where-Object {
    $_ -match $TextPattern -or $_ -match $DotFiles
})

if ($tracked.Count -eq 0) {
    Write-Host "error: no tracked text files found (is git available?)" -ForegroundColor Red
    exit 1
}

# --- Mojibake signatures (constructed from codepoints; source stays ASCII) --
function New-Sig([int[]]$Points) { return (($Points | ForEach-Object { [char]$_ }) -join '') }

# Common smart punctuation/arrows whose UTF-8 bytes were decoded through
# Latin-1/cp1252-style tables and written back out.
$Signatures = @(
    # A full mojibake sequence is: byte stream of X misread per-byte through
    # cp1252 (or Latin-1) then re-encoded as UTF-8. Both decode tables are
    # covered below. Example: U+2019 = E2 80 99 -> cp1252: a-circumflex,
    # euro-sign, trademark-sign.
    @{ Name = "smart-apostrophe/cp1252"; Sig = (New-Sig @(0xE2, 0x20AC, 0x2122)) }  # from U+2019
    @{ Name = "smart-apostrophe/latin1"; Sig = (New-Sig @(0xE2, 0x20AC, 0x0099)) }
    @{ Name = "left-quote/cp1252";       Sig = (New-Sig @(0xE2, 0x20AC, 0x0153)) }  # from U+201C
    @{ Name = "left-quote/latin1";       Sig = (New-Sig @(0xE2, 0x20AC, 0x009C)) }
    @{ Name = "right-quote/cp1252";      Sig = (New-Sig @(0xE2, 0x20AC, 0x017E)) }  # from U+201D
    @{ Name = "right-quote/latin1";      Sig = (New-Sig @(0xE2, 0x20AC, 0x009D)) }
    @{ Name = "em-dash/cp1252";          Sig = (New-Sig @(0xE2, 0x20AC, 0x201D)) }  # from U+2014 (0x94 -> smart right quote)
    @{ Name = "em-dash/latin1";          Sig = (New-Sig @(0xE2, 0x20AC, 0x0094)) }
    @{ Name = "incomplete-smart-punct";  Sig = (New-Sig @(0xE2, 0x20AC)) }          # bare E2 80 prefix fragment
    @{ Name = "wide-A-tilde";            Sig = ([string][char]0xC3) }               # lone cp1252-mapped lead byte C3
    @{ Name = "wide-A-circumflex";       Sig = ([string][char]0xC2) }               # lone lead byte C2
    @{ Name = "replacement-char";        Sig = ([string][char]0xFFFD) }             # U+FFFD substituted bytes
    @{ Name = "emoji-mangled-pair";      Sig = (([string][char]0xF0) + ([string][char]0x17F)) }
)

# --- Intentional-documentation allowlist -------------------------------------
# Files where mojibake-signature characters legitimately occur because they
# QUOTE historical corruption (bug reports, changelogs, forensic notes).
# Each entry requires a stated reason. Adding entries to silence real
# corruption is forbidden; every addition must be reviewed like code.
$MojibakeAllowedPaths = @{
    'docs/CHANGELOG.md' = 'Historical bug-fix record quoting mojibake sequences'
}

# --- Scan --------------------------------------------------------------------
$violations      = New-Object System.Collections.Generic.List[string]
$invalidUtf8Count = 0
$bomCount         = 0
$mojiCount        = 0
$scanOk           = 0

foreach ($rel in $tracked) {
    $full = Join-Path $RepoRoot $rel

    # Raw bytes -> strict UTF-8 decode.
    try { $bytes = [System.IO.File]::ReadAllBytes($full) }
    catch {
        $violations.Add("READ-FAIL`t$rel`t$($_.Exception.Message)")
        continue
    }

    $bom = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
    if ($bom) {
        $bomCount++
        $violations.Add("BOM`t$rel`tUTF-8 BOM present; project standard is BOM-less UTF-8")
    }

    $strict = New-Object System.Text.UTF8Encoding($false, $true)  # throwOnInvalidBytes
    try { $text = $strict.GetString($bytes) }
    catch [System.Text.DecoderFallbackException] {
        $invalidUtf8Count++
        $badByteIdx = $_.Exception.Index
        $contextHex = ''
        if ($null -ne $bytes -and $bytes.Length -gt 0) {
            $start = [Math]::Max(0, $badByteIdx - 4)
            $len   = [Math]::Min(12, $bytes.Length - $start)
            $contextHex = (($bytes[$start..($start + $len - 1)] |
                ForEach-Object { $_.ToString('X2') }) -join ' ')
        }
        $violations.Add("INVALID-UTF8`t${rel}:$badByteIdx`tnear bytes [$contextHex]")
        continue  # signature scan on decoded text is meaningless for corrupt bytes
    }

    # Mojibake signature scan over decoded text (line-aware for diagnostics).
    $allowed = $MojibakeAllowedPaths.ContainsKey($rel)
    $lines = $text -split "`r?`n"
    for ($i = 0; $i -lt $lines.Count; $i++) {
        foreach ($sigEntry in $Signatures) {
            $idx = $lines[$i].IndexOf($sigEntry.Sig, [System.StringComparison]::Ordinal)
            if ($idx -lt 0) { continue }
            if (-not $allowed) {
                $mojiCount++
                $violations.Add(
                    "MOJIBAKE`t${rel}:$($i + 1)`t{0}" -f $sigEntry.Name)
            }
        }
    }
}

# --- Report ------------------------------------------------------------------
$failed = $false
if ($violations.Count -gt 0) {
    Write-Host ''
    Write-Host 'UTF-8 / mojibake validation FAILED:' -ForegroundColor Red
    foreach ($v in $violations) { Write-Host "  $v" -ForegroundColor Yellow }
    Write-Host ''
    Write-Host ('Totals: invalid-utf8={0}  bom={1}  mojibake={2}  read-failures={3}' -f `
        $invalidUtf8Count, $bomCount, $mojiCount, `
        @($violations | Where-Object { $_ -like 'READ-FAIL*' }).Count)
    Write-Host ''
    Write-Host 'A MOJIBAKE hit means text crossed an encoding boundary incorrectly.'
    Write-Host 'Do NOT blind-replace characters. See docs/ENCODING_POLICY.md:'
    Write-Host 'identify the boundary (shell pipe? editor default? writer?), fix the'
    Write-Host 'boundary, then restore the intended characters. Quoting signatures in'
    Write-Host 'documentation requires an entry in $MojibakeAllowedPaths with justification.'
    $failed = $true
}
else {
    Write-Host ('PASS: {0} text files valid UTF-8 (strict), 0 BOMs, 0 unexplained mojibake signatures.' -f $tracked.Count)
}

exit $(if ($failed) { 1 } else { 0 })

