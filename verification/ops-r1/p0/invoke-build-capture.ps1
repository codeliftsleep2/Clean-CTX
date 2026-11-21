# PROPOSED OPERATOR COMMAND ONLY: requires separate exact build approval and applied P0 patch.
# Builds tests but executes none. No patch application, source formatting, CI or publication.
param([string]$OutputDirectory='target/ops-r1/p0-build-reviewed')
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$savedColor=$env:CARGO_TERM_COLOR
$env:CARGO_TERM_COLOR='never'
Push-Location $root
try {
    $launcher=Join-Path $PSScriptRoot 'run_bounded.py'
    $before=@(& python $launcher --verify-sources-only)
    if($LASTEXITCODE -ne 0 -or $before.Count -ne 1){throw 'Reviewed P0 source tree is not applied exactly; no build started.'}
    $out=[IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
    $boundary=[IO.Path]::GetFullPath((Join-Path $root 'target/ops-r1'))+[IO.Path]::DirectorySeparatorChar
    if(-not $out.StartsWith($boundary,[StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $out)){throw 'Build capture must be new and inside target/ops-r1; existing evidence is preserved.'}
    New-Item -ItemType Directory -Path $out|Out-Null
    $argumentsForBuild=@('test','--lib','--all-features','--no-run','--message-format=json')
    $started=[Diagnostics.Stopwatch]::StartNew()
    & cargo @argumentsForBuild 1> (Join-Path $out 'build.jsonl') 2> (Join-Path $out 'build-stderr.txt')
    $code=$LASTEXITCODE
    $started.Stop()
    $after=@(& python $launcher --verify-sources-only)
    $sourceCode=$LASTEXITCODE
    $artifacts=@(Get-Content (Join-Path $out 'build.jsonl')|ForEach-Object{
        $line=$_|ConvertFrom-Json
        if($line.reason -ceq 'compiler-artifact' -and $line.target.name -ceq 'clean_ctx' -and $line.target.kind -contains 'lib' -and $line.profile.test -and $line.executable){$line.executable}
    }|Sort-Object -Unique)
    $receipt=[ordered]@{
        category='Operator allfeature tracked-test build only; no execution'
        command=('cargo '+($argumentsForBuild -join ' '));exit_code=$code;elapsed_seconds=$started.Elapsed.TotalSeconds
        head=(git rev-parse HEAD);source_tree_sha256=$before[0]
        launcher_sha256=(Get-FileHash $launcher).Hash.ToLowerInvariant()
        capture_script_sha256=(Get-FileHash $PSCommandPath).Hash.ToLowerInvariant()
        build_log_sha256=(Get-FileHash (Join-Path $out 'build.jsonl')).Hash.ToLowerInvariant()
        build_stderr_sha256=(Get-FileHash (Join-Path $out 'build-stderr.txt')).Hash.ToLowerInvariant()
        rustflags=$env:RUSTFLAGS;encoded_rustflags=$env:CARGO_ENCODED_RUSTFLAGS
        executable=$null;executable_sha256=$null
    }
    if($artifacts.Count -eq 1 -and (Test-Path -LiteralPath $artifacts[0])){
        $receipt.executable=$artifacts[0]
        $receipt.executable_sha256=(Get-FileHash -LiteralPath $artifacts[0]).Hash.ToLowerInvariant()
    }
    $receipt|ConvertTo-Json -Depth 10|Set-Content (Join-Path $out 'build-receipt.json') -Encoding utf8NoBOM
    if($code -ne 0 -or $sourceCode -ne 0 -or $before[0] -cne $after[0] -or $artifacts.Count -ne 1){throw "Build/identity capture incomplete; preserve $out. No test started."}
    if(Select-String -LiteralPath (Join-Path $out 'build-stderr.txt') -Pattern '^warning:' -Quiet){throw "Build warnings; preserve $out. No test started."}
    Write-Output "Return build receipt for review before any execution: $out/build-receipt.json"
} finally {$env:CARGO_TERM_COLOR=$savedColor;Pop-Location}
