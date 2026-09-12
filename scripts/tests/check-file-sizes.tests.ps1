<#
.SYNOPSIS
    Focused behavior tests for scripts/check-file-sizes.ps1.
#>

$ErrorActionPreference = 'Stop'
$Validator = (Resolve-Path (Join-Path $PSScriptRoot '..\check-file-sizes.ps1')).Path
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$TestRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-file-size-tests-" + [Guid]::NewGuid().ToString('N'))
$Passed = 0

function Write-Lines {
    param(
        [string]$Path,
        [int]$Count
    )
    [System.IO.File]::WriteAllText($Path, ("line`n" * $Count), $Utf8NoBom)
}

function Invoke-Git {
    param(
        [string]$Repository,
        [string[]]$Arguments
    )
    & git -C $Repository @Arguments | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed in $Repository"
    }
}

function New-TestRepository {
    $repository = Join-Path $TestRoot 'repository'
    [void][System.IO.Directory]::CreateDirectory($repository)
    Invoke-Git $repository @('init', '--quiet')
    Invoke-Git $repository @('config', 'user.email', 'file-size-tests@example.invalid')
    Invoke-Git $repository @('config', 'user.name', 'File Size Tests')
    Invoke-Git $repository @('config', 'core.autocrlf', 'false')
    Write-Lines (Join-Path $repository 'normal.md') 10
    Write-Lines (Join-Path $repository 'legacy.md') 616
    Invoke-Git $repository @('add', '.')
    Invoke-Git $repository @('commit', '--quiet', '-m', 'baseline')
    return $repository
}

function Invoke-Validator {
    param(
        [string]$Repository,
        [string]$BaseRef
    )

    if ([string]::IsNullOrWhiteSpace($BaseRef)) {
        return & $Validator -RepositoryRoot $Repository -PassThru
    }
    return & $Validator -RepositoryRoot $Repository -BaseRef $BaseRef -PassThru
}

function Assert-Case {
    param(
        [string]$Name,
        [bool]$Condition,
        [string]$Details
    )
    if (-not $Condition) {
        throw "FAIL: $Name`n$Details"
    }
    $script:Passed++
    Write-Host "PASS: $Name"
}

try {
    [void][System.IO.Directory]::CreateDirectory($TestRoot)
    $repository = New-TestRepository
    $base = (& git -C $repository rev-parse HEAD).Trim()

    $result = Invoke-Validator $repository
    Assert-Case 'untouched oversized legacy file is non-blocking' (
        $result.ExitCode -eq 0 -and ($result.LegacyDebt -join "`n") -match 'LEGACY-DEBT.*legacy.md.*616 lines'
    ) ($result | Out-String)

    [System.IO.File]::AppendAllText((Join-Path $repository 'legacy.md'), "line`n", $Utf8NoBom)
    $result = Invoke-Validator $repository
    Assert-Case 'modified oversized legacy file fails' (
        $result.ExitCode -eq 1 -and ($result.Failures -join "`n") -match 'ACTIVE-OVERSIZE.*modified.*legacy.md.*617 lines'
    ) ($result | Out-String)
    Write-Lines (Join-Path $repository 'legacy.md') 616

    Write-Lines (Join-Path $repository 'new.md') 616
    $result = Invoke-Validator $repository
    Assert-Case 'new oversized file fails' (
        $result.ExitCode -eq 1 -and ($result.Failures -join "`n") -match 'ACTIVE-OVERSIZE.*new.*new.md.*616 lines'
    ) ($result | Out-String)
    [System.IO.File]::Delete((Join-Path $repository 'new.md'))

    Write-Lines (Join-Path $repository 'allowed.md') 615
    $result = Invoke-Validator $repository
    Assert-Case 'new file at 615-line ceiling passes' ($result.ExitCode -eq 0) ($result | Out-String)
    [System.IO.File]::Delete((Join-Path $repository 'allowed.md'))

    Write-Lines (Join-Path $repository 'Cargo.lock') 700
    Write-Lines (Join-Path $repository 'package-lock.json') 700
    $result = Invoke-Validator $repository
    Assert-Case 'generated dependency lockfiles are exempt' (
        $result.ExitCode -eq 0 -and
        ($result.Failures -join "`n") -notmatch 'Cargo.lock|package-lock.json'
    ) ($result | Out-String)
    [System.IO.File]::Delete((Join-Path $repository 'Cargo.lock'))
    [System.IO.File]::Delete((Join-Path $repository 'package-lock.json'))

    Write-Lines (Join-Path $repository 'manual.lock') 616
    $result = Invoke-Validator $repository
    Assert-Case 'ordinary lock-suffixed text file remains enforced' (
        $result.ExitCode -eq 1 -and ($result.Failures -join "`n") -match 'ACTIVE-OVERSIZE.*new.*manual.lock.*616 lines'
    ) ($result | Out-String)
    [System.IO.File]::Delete((Join-Path $repository 'manual.lock'))

    Write-Lines (Join-Path $repository 'committed.md') 616
    Invoke-Git $repository @('add', 'committed.md')
    Invoke-Git $repository @('commit', '--quiet', '-m', 'oversized active file')
    $result = Invoke-Validator $repository $base
    Assert-Case 'base-relative committed new file fails' (
        $result.ExitCode -eq 1 -and ($result.Failures -join "`n") -match 'ACTIVE-OVERSIZE.*new.*committed.md.*616 lines'
    ) ($result | Out-String)

    Write-Host "PASS: $Passed file-size validator behavior tests."
}
finally {
    if (Test-Path -LiteralPath $TestRoot) {
        Remove-Item -LiteralPath $TestRoot -Recurse -Force
    }
}
