<#
.SYNOPSIS
    Enable the repository's versioned Git hooks directory (core.hooksPath = .githooks).

.DESCRIPTION
    Points this clone at the versioned .githooks/ directory so Git runs the
    project's pre-commit UTF-8 guard automatically on every commit.

    It does NOT copy scripts into .git/hooks/ and does NOT touch any Git
    configuration other than the local core.hooksPath value. It is idempotent:
    safe to run any number of times.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-git-hooks.ps1

.NOTES
    Exit code 0 = configured, 1 = failure.
    Requires Git on PATH. PowerShell 7 (pwsh) or 5.1 both work; the script
    itself only uses Git config operations.
#>

$ErrorActionPreference = "Stop"

# Locate the repository root. Mirrors scripts/check-utf8.ps1: the script may be
# invoked from the repo root or from the scripts/ directory.
$RepoRoot =
    if (Test-Path ".git") { (Get-Location).Path }
    elseif (Test-Path "../.git") { (Resolve-Path "../").Path }
    else {
        Write-Host "error: run from the repository root or the scripts/ directory" -ForegroundColor Red
        exit 1
    }

Set-Location $RepoRoot

$HooksDir = ".githooks"
$HookName = "pre-commit"
$HookFile = Join-Path $HooksDir $HookName

if (-not (Test-Path $HookFile)) {
    Write-Host "error: expected hook not found: $HookFile" -ForegroundColor Red
    Write-Host "       (the versioned hooks directory is missing this hook)" -ForegroundColor Red
    exit 1
}

Write-Host "Installing Git hooks for clean-ctx (clone-local)..."
Write-Host "  repository root : $RepoRoot"
Write-Host "  hooks directory : $HooksDir"

# Configure core.hooksPath (local to this clone only; does not touch global/user
# config). Would overwrite a pre-existing local value.
git config core.hooksPath $HooksDir
if ($LASTEXITCODE -ne 0) {
    Write-Host "error: 'git config core.hooksPath $HooksDir' failed" -ForegroundColor Red
    exit 1
}

# Verify the value actually stuck.
$Configured = git config --local --get core.hooksPath
if ($Configured -ne $HooksDir) {
    Write-Host "error: verification failed - core.hooksPath is '$Configured', expected '$HooksDir'" -ForegroundColor Red
    exit 1
}

# Verify the hook file is present at the configured location.
if (-not (Test-Path $HookFile)) {
    Write-Host "error: hook missing after configuration: $HookFile" -ForegroundColor Red
    exit 1
}

# Best-effort: record the executable bit on POSIX hosts so a fresh checkout on
# Linux/macOS runs the hook. On Windows this is a no-op. Non-fatal.
try {
    git update-index --add --chmod=+x ".githooks/$HookName" 2>$null
} catch {
    # ignore: exec-bit recording is a convenience, not a requirement to install.
}

Write-Host ""
Write-Host "Done. Git will now run $HooksDir/$HookName automatically on every commit."
Write-Host "The hook invokes scripts/check-utf8.ps1 and aborts the commit if the"
Write-Host "UTF-8 / mojibake guard fails."
Write-Host ""
Write-Host "To disable:  git config --unset core.hooksPath"
exit 0