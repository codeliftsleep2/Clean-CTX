<#
.SYNOPSIS
    CI Guard: verify all tree-sitter crates share the same tree-sitter-language version.
.DESCRIPTION
    Parses Cargo.lock and checks that there is exactly ONE version of the
    tree-sitter-language crate. If multiple versions exist, one or more grammar
    crates has an ABI mismatch that could cause segfaults at runtime.
.EXAMPLE
    ./scripts/check-tree-sitter-versions.ps1
    PASS: all 6 tree-sitter crates depend on tree-sitter-language 0.1.7
.NOTES
    Exit code 0 = success, 1 = failure.
    Intended for CI (GitHub Actions) and local pre-commit use.
#>

$ErrorActionPreference = "Stop"

# Paths — support running from repo root or from scripts/ directory
$CargoLock = if (Test-Path "Cargo.lock") { "Cargo.lock" }
            elseif (Test-Path "../Cargo.lock") { "../Cargo.lock" }
            else {
                Write-Host "error: Cargo.lock not found" -ForegroundColor Red
                exit 1
            }

# Read the lock file and split into [[package]] blocks
$content = Get-Content $CargoLock -Raw
$blocks = $content -split '(?m)(?=^\[\[package\]\])'

# Find all tree-sitter-language versions
$langVersions = @()
$treeSitterPackages = @()

foreach ($block in $blocks) {
    if ($block -match 'name\s*=\s*"(?<pkg>tree-sitter[^"]*)"') {
        $pkgName = $matches['pkg']

        if ($block -match 'version\s*=\s*"(?<ver>[^"]+)"') {
            $ver = $matches['ver']
        } else {
            $ver = "<unknown>"
        }

        if ($pkgName -eq "tree-sitter-language") {
            $langVersions += $ver
        } else {
            $treeSitterPackages += "$pkgName $ver"
        }
    }
}

# Check that there is exactly one tree-sitter-language version
if ($langVersions.Count -eq 0) {
    Write-Host "FAIL: tree-sitter-language package not found in Cargo.lock!" -ForegroundColor Red
    exit 1
}

if ($langVersions.Count -gt 1) {
    Write-Host "FAIL: multiple tree-sitter-language versions detected!" -ForegroundColor Red
    Write-Host "  Versions found: $($langVersions -join ', ')" -ForegroundColor Red
    Write-Host ""
    Write-Host "This indicates an ABI mismatch between grammar crates. All grammar" -ForegroundColor Yellow
    Write-Host "crates must depend on the same tree-sitter-language version." -ForegroundColor Yellow
    exit 1
}

# Success
$langVer = $langVersions[0]
$pkgCount = $treeSitterPackages.Count
Write-Host "PASS: all $pkgCount tree-sitter crates depend on tree-sitter-language $langVer" -ForegroundColor Green
Write-Host "  $($treeSitterPackages -join "`n  ")"
exit 0