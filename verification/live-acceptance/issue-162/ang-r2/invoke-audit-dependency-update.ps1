param([switch]$AuditOnly, [string]$AuditExecutable)
# Operator-owned targeted lockfile updates and actual audit. No ignores or main push.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Use PowerShell 7 or later.' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $root
try {
    if ((& git branch --show-current) -cne '(fix)Performance-Regression-Issues') { throw 'Expected existing ANG-R2 feature branch.' }
    if (-not $AuditOnly -and @(& git status --porcelain -- Cargo.lock).Count) { throw 'Existing lockfile edits need review first.' }
    if ($AuditOnly -and (Get-FileHash Cargo.lock).Hash.ToLowerInvariant() -cne 'c1d18af45660ebb7dd3898ff567a856a18f85a2ba9344d15996c49b8a4e74a59') { throw 'Reviewed lockfile changed before audit-only retry.' }
    $out = Join-Path $root ('target/ang-r2/audit-update-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
    New-Item -ItemType Directory -Path $out | Out-Null
    [IO.File]::WriteAllBytes((Join-Path $out 'Cargo.lock.before'),[IO.File]::ReadAllBytes((Join-Path $root 'Cargo.lock')))
    $runs = @(
        @{name='quinn-proto';arguments=@('update','-p','quinn-proto','--precise','0.11.19')},
        @{name='quinn-udp';arguments=@('update','-p','quinn-udp','--precise','0.5.16')},
        @{name='rustls-webpki';arguments=@('update','-p','rustls-webpki','--precise','0.103.16')},
        @{name='webpki-roots';arguments=@('update','-p','webpki-roots','--precise','1.0.10')},
        @{name='rustix';arguments=@('update','-p','rustix','--precise','1.1.6')},
        @{name='tokio';arguments=@('update','-p','tokio','--precise','1.53.3')},
        @{name='audit';arguments=@('audit','--json')}
    )
    foreach ($run in $runs) {
        if ($AuditOnly -and $run.name -ne 'audit') { continue }
        $argumentsForRun = $run.arguments
        $stdout = Join-Path $out ($run.name+'-stdout.txt')
        $stderr = Join-Path $out ($run.name+'-stderr.txt')
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $executable = if ($run.name -eq 'audit' -and $AuditExecutable) { (Resolve-Path -LiteralPath $AuditExecutable).Path } else { 'cargo' }
        & $executable @argumentsForRun 1> $stdout 2> $stderr
        $code = $LASTEXITCODE
        $watch.Stop()
        [ordered]@{
            category='Targeted operator dependency resolution/audit; not compilation or runtime verification'
            command=($executable+' '+($argumentsForRun -join ' '));exit_code=$code;elapsed_seconds=$watch.Elapsed.TotalSeconds
            head=(& git rev-parse HEAD);lockfile_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant()
            stdout_sha256=(Get-FileHash $stdout).Hash.ToLowerInvariant();stderr_sha256=(Get-FileHash $stderr).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out ($run.name+'-receipt.json')) -Encoding utf8NoBOM
        if ($code -ne 0) { throw "$($run.name) failed; preserve partial lockfile and $out for review" }
    }
    [IO.File]::WriteAllBytes((Join-Path $out 'Cargo.lock.after'),[IO.File]::ReadAllBytes((Join-Path $root 'Cargo.lock')))
    Write-Host "Return receipts and audit JSON from $out. Lockfile diff requires review before compilation, commit or CI."
} finally { Pop-Location }
