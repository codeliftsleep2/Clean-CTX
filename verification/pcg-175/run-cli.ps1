param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [string]$OutputRoot = (Join-Path $PSScriptRoot '../../target/pcg-175'),
    [ValidateRange(1, 10000)][int]$BaseRecords = 8,
    [switch]$SingleCase
)
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$gitBinaryPath = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$runRoot = Join-Path ([IO.Path]::GetFullPath($OutputRoot)) ('m2a1-' + [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-ffff'))
[IO.Directory]::CreateDirectory($runRoot) | Out-Null

function Write-Json([string]$Path, $Value) {
    [IO.File]::WriteAllText($Path, (ConvertTo-Json -Depth 40 -Compress -InputObject $Value), $utf8)
}
function Hash-Bytes([byte[]]$Bytes) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}
function Invoke-RecordedNative([string]$Executable, [string[]]$Arguments, [string]$Phase, [switch]$JsonResult) {
    [IO.Directory]::CreateDirectory($phaseRoot) | Out-Null
    $stdoutPath = Join-Path $phaseRoot "$Phase.stdout.txt"
    $stderrPath = Join-Path $phaseRoot "$Phase.stderr.txt"
    $receiptPath = Join-Path $phaseRoot "$Phase.receipt.json"
    $receipt = @{
        phase = $Phase; command = @{ executable = $Executable; arguments = @($Arguments) }
        working_directory = $repoRoot; started = $false; exit_code = $null
        status = 'pending'; result = $null; error = $null
        stdout = $stdoutPath; stderr = $stderrPath
        pcg_binary_sha256 = $binaryHash
    }
    $stdoutText = ''; $stderrText = ''
    [IO.File]::WriteAllText($stdoutPath, '', $utf8)
    [IO.File]::WriteAllText($stderrPath, '', $utf8)
    Write-Json $receiptPath $receipt
    $process = [Diagnostics.Process]::new()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    try {
        $process.StartInfo = [Diagnostics.ProcessStartInfo]::new()
        $process.StartInfo.FileName = $Executable
        $process.StartInfo.WorkingDirectory = $repoRoot
        $process.StartInfo.UseShellExecute = $false
        $process.StartInfo.CreateNoWindow = $true
        $process.StartInfo.RedirectStandardOutput = $true
        $process.StartInfo.RedirectStandardError = $true
        $process.StartInfo.StandardOutputEncoding = $utf8
        $process.StartInfo.StandardErrorEncoding = $utf8
        foreach ($argument in $Arguments) { $process.StartInfo.ArgumentList.Add($argument) }
        if (-not $process.Start()) { throw 'Native process did not start' }
        $receipt.started = $true
        $receipt.status = 'running'
        Write-Json $receiptPath $receipt
        # Drain both pipes concurrently so stderr cannot deadlock stdout capture.
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $receipt.exit_code = $process.ExitCode
        $stdoutText = $stdoutTask.GetAwaiter().GetResult()
        $stderrText = $stderrTask.GetAwaiter().GetResult()
        if ($receipt.exit_code -ne 0) { throw "Native phase failed: $Phase" }
        $receipt.result = if ($JsonResult) { $stdoutText | ConvertFrom-Json -AsHashtable -ErrorAction Stop } else { $stdoutText.Trim() }
        if ($JsonResult -and $receipt.result -isnot [Collections.IDictionary]) { throw "Invalid CLI result: $Phase" }
        $receipt.status = 'succeeded'
    } catch {
        $receipt.status = 'failed'
        $receipt.error = $_.Exception.Message
        if ($receipt.started -and -not $process.HasExited) {
            $process.Kill($true)
            $process.WaitForExit()
            $receipt.exit_code = $process.ExitCode
        }
        throw
    } finally {
        $clock.Stop()
        $receipt['operator_process_milliseconds'] = $clock.Elapsed.TotalMilliseconds
        [IO.File]::WriteAllText($stdoutPath, $stdoutText, $utf8)
        [IO.File]::WriteAllText($stderrPath, $stderrText, $utf8)
        Write-Json $receiptPath $receipt
        $process.Dispose()
    }
    $receipt
}
function Invoke-FixtureGit([string]$Root, [string]$Phase, [string[]]$Arguments) {
    $receipt = Invoke-RecordedNative -Executable $gitBinaryPath -Arguments (@('-C', $Root) + $Arguments) -Phase $Phase
    $receipt.result
}
function Invoke-Pcg([string]$ProfilePath, [string]$Phase, [string[]]$Extra) {
    $nativeArguments = @('pcg', 'import', '--profile', $ProfilePath)
    if ($Extra) { $nativeArguments += $Extra }
    $receipt = Invoke-RecordedNative -Executable $binaryPath -Arguments $nativeArguments -Phase $Phase -JsonResult
    $report = $receipt.result
    $report['operator_process_milliseconds'] = $receipt.operator_process_milliseconds
    $report
}

$binaryHash = Hash-Bytes ([IO.File]::ReadAllBytes($binaryPath))
$receipts = @()
$caseSizes = if ($SingleCase) { @($BaseRecords) } else { @($BaseRecords, (2 * $BaseRecords), (4 * $BaseRecords)) }
$runStatus = 'failed'; $runError = $null
try {
    foreach ($n in $caseSizes) {
        $caseRoot = Join-Path $runRoot "n-$n"
        $repoRoot = Join-Path $caseRoot 'repo'
        $phaseRoot = Join-Path $caseRoot 'phases'
        [IO.Directory]::CreateDirectory($repoRoot) | Out-Null
        $document = [IO.File]::ReadAllBytes((Join-Path $PSScriptRoot 'fixtures/decision.md'))
        [IO.File]::WriteAllBytes((Join-Path $repoRoot 'decision.md'), $document)
        Invoke-FixtureGit $repoRoot 'git-init' @('init', '--quiet') | Out-Null
        Invoke-FixtureGit $repoRoot 'git-config' @('config', 'core.autocrlf', 'false') | Out-Null
        Invoke-FixtureGit $repoRoot 'git-add' @('add', '--', 'decision.md') | Out-Null
        Invoke-FixtureGit $repoRoot 'git-commit' @('-c', 'user.name=PCG Fixture', '-c', 'user.email=pcg@example.invalid', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=', 'commit', '--quiet', '-m', 'fixture') | Out-Null
        $commit = Invoke-FixtureGit $repoRoot 'git-head' @('rev-parse', 'HEAD')
        $blob = Invoke-FixtureGit $repoRoot 'git-blob' @('rev-parse', 'HEAD:decision.md')
        $records = @()
        foreach ($i in 1..$n) {
            $records += @{
                id = "decision-$i"; role = 'decision'; required = $true; primary = 'quote'
                authority = @{ class = 'explicitly_approved_decision'; attributed_to = 'fixture source' }
                verification = @{ outcome = 'supported'; subject = 'retain_reads'; execution = 'report_of_execution' }
                lifecycle = 'active'; subject = "fixture-$i"; disposition = 'PROTECTED'; evidence_refs = @()
            }
        }
        $manifestPath = Join-Path $caseRoot 'manifest.json'
        $profilePath = Join-Path $caseRoot 'profile.json'
        $databasePath = Join-Path $caseRoot 'projection.db'
        $manifest = @{
            schema_version = 1; parser_version = 'pcg-json-v1'; corpus_id = 'fixture'; source_namespace = 'fixture:git'
            enrollment = @{ owner_label = 'fixture-owner'; policy_revision = 'fixture-v1'; complete = $true }
            sources = @(@{ id = 'source'; path = 'decision.md'; commit = $commit; blob = $blob; sha256 = (Hash-Bytes $document); check = 'current' })
            selectors = @(@{ id = 'coverage'; purpose = 'coverage'; workspace = 'fixture'; from = $null; until = $null })
            evidence = @(@{ id = 'quote'; source = 'source'; start = 0; end = $document.Length; sha256 = (Hash-Bytes $document); heading = '# Fixture decision' })
            records = $records; edges = @()
        }
        Write-Json $manifestPath $manifest
        $profile = @{
            schema_version = 1; partition_id = 'fixture'; owner_label = 'fixture-owner'; workspace_id = 'fixture'
            workspace_root = $repoRoot; database_path = $databasePath; manifest_path = $manifestPath
            approved_manifest_sha256 = (Hash-Bytes ([IO.File]::ReadAllBytes($manifestPath)))
            policy_revision = 'fixture-v1'; history_allowed = $true
            limits = @{ records = (8 * $n + 32); edges = (8 * $n + 32); evidence_bytes = (16777216 + 4096 * $n); token_budget = 8192 }
        }
        Write-Json $profilePath $profile
        $import = Invoke-Pcg $profilePath 'cli-import' @()
        $repeat = Invoke-Pcg $profilePath 'cli-reimport' @()
        $rebuild = Invoke-Pcg $profilePath 'cli-rebuild' @('--rebuild')
        $beforeAudit = Hash-Bytes ([IO.File]::ReadAllBytes($databasePath))
        $audit = Invoke-Pcg $profilePath 'cli-audit' @('--verify-only')
        $afterAudit = Hash-Bytes ([IO.File]::ReadAllBytes($databasePath))
        if ($import.generation -ne $repeat.generation -or $import.generation -ne $rebuild.generation -or $import.generation -ne $audit.generation) {
            throw 'Generation identity changed across identical import/rebuild/audit'
        }
        if ($beforeAudit -ne $afterAudit) { throw 'Audit changed the main database bytes' }
        if ($repeat.operation -ne 'idempotent_import' -or $import.work.current_sources -ne 1 -or $import.work.current_bytes -ne (2 * $document.Length)) {
            throw 'CLI operation/source accounting mismatch'
        }
        $receipts += @{ records = $n; document_bytes = $document.Length; import = $import; repeat = $repeat; rebuild = $rebuild; audit = $audit }
    }
    if ($binaryHash -ne (Hash-Bytes ([IO.File]::ReadAllBytes($binaryPath)))) { throw 'Binary changed during operator run' }
    $runStatus = 'succeeded'
} catch {
    $runError = $_.Exception.Message
    throw
} finally {
    Write-Json (Join-Path $runRoot 'operator-receipt.json') @{
        category = 'operator_cli_verification'; tracked_tests = $false
        status = $runStatus; error = $runError; single_case = [bool]$SingleCase; planned_records = @($caseSizes)
        binary_sha256 = $binaryHash
        utc = [DateTime]::UtcNow.ToString('o'); cases = $receipts
        limits = 'Synthetic local correctness/work fixtures only; no enterprise efficiency claim.'
    }
}
Write-Output "Operator CLI receipt: $(Join-Path $runRoot 'operator-receipt.json')"
