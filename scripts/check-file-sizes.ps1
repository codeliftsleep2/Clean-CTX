<#
.SYNOPSIS
    Enforces the active-file line-count policy.
.DESCRIPTION
    New and modified text files must not exceed 615 lines. Tracked files that
    were already oversized and are not active are reported as legacy debt but
    do not fail validation. The preferred target remains 600 lines.

    Without -BaseRef, active files are taken from the working tree, index, and
    untracked files. With -BaseRef, committed changes from the merge base to
    HEAD are included as active for CI and branch verification.
.PARAMETER BaseRef
    Optional Git revision used as the comparison base for committed changes.
.PARAMETER RepositoryRoot
    Optional repository root. Intended primarily for validator self-tests.
.NOTES
    Exit code 0 = policy satisfied, 1 = active oversized files or guard error.
#>

[CmdletBinding()]
param(
    [string]$BaseRef,
    [string]$RepositoryRoot,
    [switch]$PassThru
)

$ErrorActionPreference = 'Stop'
$TargetLines = 600
$MaximumLines = 615
$TextPattern = '\.(rs|toml|json|jsonc|ya?ml|md|markdown|txt|html|css|scss|mjs|cjs|js|jsx|ts|tsx|cs|java|sql|xml|csv|tsv|ps1|psm1|lock|gitattributes|dotsettings|sh)$'
$DotFiles = '^\.(gitignore|gitattributes|editorconfig)$'

if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
    $RepositoryRoot = Join-Path $PSScriptRoot '..'
}

try {
    $RepoRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
}
catch {
    Write-Host "FILE-SIZE-ERROR: repository root not found: $RepositoryRoot" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot '.git'))) {
    Write-Host "FILE-SIZE-ERROR: not a Git repository: $RepoRoot" -ForegroundColor Red
    exit 1
}

$GitSafeRoot = $RepoRoot.Replace('\', '/')

function Invoke-GitLines {
    param([string[]]$Arguments)

    $output = @(& git -c "safe.directory=$GitSafeRoot" -C $RepoRoot @Arguments)
    if ($LASTEXITCODE -ne 0) {
        $message = ($output -join [Environment]::NewLine)
        throw "git $($Arguments -join ' ') failed: $message"
    }
    return @($output | ForEach-Object { "$_" } | Where-Object { $_.Length -gt 0 })
}

function Test-NormalTextFile {
    param([string]$RelativePath)
    return $RelativePath -match $TextPattern -or $RelativePath -match $DotFiles
}

function Get-LineCount {
    param([string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -eq 0) { return 0 }

    $lineFeeds = 0
    foreach ($byte in $bytes) {
        if ($byte -eq 10) { $lineFeeds++ }
    }
    if ($bytes[$bytes.Length - 1] -ne 10) { $lineFeeds++ }
    return $lineFeeds
}

$active = @{}

function Add-ActivePaths {
    param(
        [string[]]$Paths,
        [string]$Kind
    )

    foreach ($path in $Paths) {
        $normalized = $path.Replace('\', '/')
        if (-not (Test-NormalTextFile $normalized)) { continue }
        if (-not $active.ContainsKey($normalized) -or $Kind -eq 'new') {
            $active[$normalized] = $Kind
        }
    }
}

try {
    if (-not [string]::IsNullOrWhiteSpace($BaseRef)) {
        [void](Invoke-GitLines @('rev-parse', '--verify', "$BaseRef^{commit}"))
        Add-ActivePaths (Invoke-GitLines @('diff', '--name-only', '--diff-filter=A', "$BaseRef...HEAD")) 'new'
        Add-ActivePaths (Invoke-GitLines @('diff', '--name-only', '--diff-filter=CMRTUXB', "$BaseRef...HEAD")) 'modified'
    }

    Add-ActivePaths (Invoke-GitLines @('diff', '--cached', '--name-only', '--diff-filter=A')) 'new'
    Add-ActivePaths (Invoke-GitLines @('diff', '--cached', '--name-only', '--diff-filter=CMRTUXB')) 'modified'
    Add-ActivePaths (Invoke-GitLines @('diff', '--name-only', '--diff-filter=CMRTUXB')) 'modified'
    Add-ActivePaths (Invoke-GitLines @('ls-files', '--others', '--exclude-standard')) 'new'

    $tracked = @(Invoke-GitLines @('ls-files', '--cached') | Where-Object {
        Test-NormalTextFile $_
    })
}
catch {
    Write-Host "FILE-SIZE-ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

$failures = New-Object System.Collections.Generic.List[string]
$targetNotices = New-Object System.Collections.Generic.List[string]
$legacyDebt = New-Object System.Collections.Generic.List[string]

foreach ($relativePath in @($active.Keys | Sort-Object)) {
    $fullPath = Join-Path $RepoRoot $relativePath
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) { continue }

    $lines = Get-LineCount $fullPath
    if ($lines -gt $MaximumLines) {
        $failures.Add("ACTIVE-OVERSIZE`t$($active[$relativePath])`t$relativePath`t$lines lines")
    }
    elseif ($lines -gt $TargetLines) {
        $targetNotices.Add("TARGET-NOTICE`t$($active[$relativePath])`t$relativePath`t$lines lines")
    }
}

foreach ($relativePath in @($tracked | Sort-Object -Unique)) {
    $normalized = $relativePath.Replace('\', '/')
    if ($active.ContainsKey($normalized)) { continue }

    $fullPath = Join-Path $RepoRoot $relativePath
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) { continue }
    $lines = Get-LineCount $fullPath
    if ($lines -gt $MaximumLines) {
        $legacyDebt.Add("LEGACY-DEBT`t$normalized`t$lines lines")
    }
}

foreach ($entry in $legacyDebt) { Write-Host $entry -ForegroundColor DarkYellow }
foreach ($entry in $targetNotices) { Write-Host $entry -ForegroundColor Yellow }

if ($failures.Count -gt 0) {
    foreach ($entry in $failures) { Write-Host $entry -ForegroundColor Red }
    Write-Host ("FAIL: {0} active file(s) exceed the {1}-line ceiling. Untouched legacy debt is non-blocking." -f $failures.Count, $MaximumLines) -ForegroundColor Red
    if ($PassThru) {
        return [pscustomobject]@{
            ExitCode = 1
            Failures = @($failures)
            TargetNotices = @($targetNotices)
            LegacyDebt = @($legacyDebt)
        }
    }
    exit 1
}

Write-Host ("PASS: {0} active text file(s) checked; {1} legacy oversized file(s) reported; target={2}, ceiling={3}." -f $active.Count, $legacyDebt.Count, $TargetLines, $MaximumLines)
if ($PassThru) {
    return [pscustomobject]@{
        ExitCode = 0
        Failures = @($failures)
        TargetNotices = @($targetNotices)
        LegacyDebt = @($legacyDebt)
    }
}
exit 0
