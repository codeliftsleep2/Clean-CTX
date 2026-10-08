#requires -Version 7.0
param(
    [string]$BinaryPath = '',
    [string]$DotnetPath = 'dotnet',
    [string]$NodePath = 'node',
    [Parameter(Mandatory)][string]$NodeToolsRoot,
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$extension = if ($IsWindows) { '.exe' } else { '' }
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target/debug/clean-ctx$extension" }
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$modules = (Resolve-Path -LiteralPath (Join-Path $NodeToolsRoot 'node_modules')).Path
$run = Join-Path $RepositoryRoot ('target/native-diagnostics-verification/' + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $run 'workspace'
New-Item -ItemType Directory -Force $workspace | Out-Null
Copy-Item (Join-Path $PSScriptRoot '../fixtures/*') $workspace -Recurse
$linkType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
New-Item -ItemType $linkType -Path (Join-Path $workspace 'node_modules') -Target $modules | Out-Null
$rows = [Collections.Generic.List[object]]::new()

function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Save-Text([string]$Path, [string]$Text) {
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}
function Save-Json([string]$Path, $Value) {
    Save-Text $Path ($Value | ConvertTo-Json -Depth 100)
}

# Same ProcessStartInfo/ArgumentList pattern as the existing MCP harnesses;
# drain both streams concurrently and keep each producer's actual exit code.
function Invoke-Captured([string]$Executable, [string[]]$Arguments, [string]$Directory, [string]$InputText = '') {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Executable
    $info.WorkingDirectory = $Directory
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $utf8 = [Text.UTF8Encoding]::new($false)
    $info.StandardInputEncoding = $utf8
    $info.StandardOutputEncoding = $utf8
    $info.StandardErrorEncoding = $utf8
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $info.Environment['DOTNET_CLI_HOME'] = $run
    $info.Environment['DOTNET_CLI_TELEMETRY_OPTOUT'] = '1'
    $info.Environment['DOTNET_NOLOGO'] = '1'
    $info.Environment['DOTNET_SKIP_FIRST_TIME_EXPERIENCE'] = '1'
    $info.Environment['NUGET_PACKAGES'] = Join-Path $run 'nuget'
    $info.Environment['NG_CLI_ANALYTICS'] = 'false'
    $info.Environment['NO_COLOR'] = '1'
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    try {
        Require ($process.Start()) "Could not start $Executable"
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if ($InputText) { $process.StandardInput.Write($InputText) }
        $process.StandardInput.Close()
        Require ($process.WaitForExit(120000)) "Operator deadline exceeded for $Executable"
        return @{ exit_code = $process.ExitCode; stdout = $stdout.GetAwaiter().GetResult(); stderr = $stderr.GetAwaiter().GetResult() }
    } finally {
        if ($process.Id -and -not $process.HasExited) { $process.Kill($true); $process.WaitForExit() }
        $process.Dispose()
    }
}

function Verify-Case($Case) {
    $directory = Join-Path $run $Case.name
    New-Item -ItemType Directory $directory | Out-Null
    $producer = Invoke-Captured $Case.exe $Case.args (Join-Path $workspace $Case.cwd)
    Save-Json (Join-Path $directory 'producer.json') $producer
    Require ($producer.exit_code -eq $Case.exit) "$($Case.name): unexpected producer exit $($producer.exit_code)"
    $eventName = if ($producer.exit_code -eq 0) { 'PostToolUse' } else { 'PostToolUseFailure' }
    $event = @{
        hook_event_name = $eventName; tool_name = 'Bash'
        tool_input = @{ command = $Case.command }
        tool_response = @{
            stdout = $producer.stdout; stderr = $producer.stderr
            interrupted = $false; isImage = $false
            operator_metadata = @{ ordered = @(1, 2, 2, 3) }
        }
    }
    Save-Json (Join-Path $directory 'event.json') $event
    $hook = Invoke-Captured $BinaryPath @('claude-hook', 'post-tool-use') $workspace ($event | ConvertTo-Json -Depth 100 -Compress)
    Save-Text (Join-Path $directory 'hook.stdout.json') $hook.stdout
    Save-Text (Join-Path $directory 'hook.facts.json') $hook.stderr
    Require ($hook.exit_code -eq 0) "$($Case.name): hook exited $($hook.exit_code)"
    $facts = $hook.stderr | ConvertFrom-Json -AsHashtable -Depth 100
    if ($Case.filter) {
        Require ($facts.outcome -eq 'replaced') "$($Case.name): expected real reduction"
        $updated = ($hook.stdout | ConvertFrom-Json -AsHashtable -Depth 100).hookSpecificOutput.updatedToolOutput
        Require ($updated.stderr -ceq $producer.stderr) "$($Case.name): stderr changed"
        Require (($updated.operator_metadata | ConvertTo-Json -Compress) -ceq ($event.tool_response.operator_metadata | ConvertTo-Json -Compress)) "$($Case.name): unrelated metadata changed"
        $filters = @($facts.fields | Where-Object { $_.ContainsKey('filter') })
        Require ($filters.Count -eq 1 -and $filters[0].field -eq 'stdout' -and $filters[0].filter.filter_id -eq $Case.filter) "$($Case.name): wrong filter/stream authority"
        Require ([regex]::Matches($updated.stdout, '§FILTERED ').Count -eq 1) "$($Case.name): expected exactly one disclosure"
        foreach ($needle in $Case.keep) {
            Require (($producer.stdout + $producer.stderr).Contains($needle)) "$($Case.name): real producer evidence missing $needle"
            Require (($updated.stdout + $updated.stderr).Contains($needle)) "$($Case.name): lost $needle"
        }
    } else {
        Require ($hook.stdout.Length -eq 0 -and $facts.outcome -eq 'passed_through') "$($Case.name): unexpected replacement"
        Require ($facts.pass_through_reason -eq $Case.reason) "$($Case.name): incorrect pass-through reason"
        foreach ($needle in $Case.keep) { Require (($producer.stdout + $producer.stderr).Contains($needle)) "$($Case.name): missing $needle" }
    }
    if ($Case.name -eq 'tsc-clean') { Require ($producer.stdout.Length -eq 0 -and $producer.stderr.Length -eq 0) 'Clean TSC emitted unexpected output' }
    $rows.Add(@{ name = $Case.name; result = 'PASS'; producer_exit = $producer.exit_code; outcome = $facts.outcome })
    Write-Host "PASS: $($Case.name)"
}

$cases = @(
    @{ name='dotnet-warning'; exe=$DotnetPath; args=@('build', '--disable-build-servers', '-v:minimal'); cwd='dotnet'; command='dotnet build'; exit=0; filter='dotnet-build-v1'; keep=@('CS1030', 'CTX_OPERATOR_WARNING', 'Build succeeded.', 'Warning(s)', 'Error(s)') },
    @{ name='dotnet-info'; exe=$DotnetPath; args=@('--version'); cwd='dotnet'; command='dotnet --version'; exit=0; filter=''; keep=@(); reason='unchanged' },
    @{ name='eslint-warning'; exe=$NodePath; args=@((Join-Path $modules 'eslint/bin/eslint.js'), 'warning.js'); cwd=''; command='eslint warning.js'; exit=0; filter='eslint-v1'; keep=@('CTX_OPERATOR_UNUSED', 'no-unused-vars', '1 problem', '1 warning'); },
    @{ name='tsc-clean'; exe=$NodePath; args=@((Join-Path $modules 'typescript/bin/tsc'), '--noEmit', '--skipLibCheck', 'valid.ts'); cwd=''; command='tsc --noEmit valid.ts'; exit=0; filter=''; keep=@(); reason='unchanged' },
    @{ name='tsc-failure'; exe=$NodePath; args=@((Join-Path $modules 'typescript/bin/tsc'), '--noEmit', '--skipLibCheck', 'invalid.ts'); cwd=''; command='tsc --noEmit invalid.ts'; exit=2; filter=''; keep=@('TS2322'); reason='unsupported_event' },
    @{ name='tsc-info'; exe=$NodePath; args=@((Join-Path $modules 'typescript/bin/tsc'), '--version'); cwd=''; command='tsc --version'; exit=0; filter=''; keep=@('Version'); reason='unchanged' },
    @{ name='angular-warning'; exe=$NodePath; args=@((Join-Path $modules '@angular/cli/bin/ng.js'), 'build', 'probe', '--progress=false'); cwd='angular'; command='ng build probe --progress=false'; exit=0; filter='angular-build-v1'; keep=@('Application bundle generation complete', 'exceeded maximum budget') }
)
$failures = 0
foreach ($case in $cases) {
    try { Verify-Case $case } catch {
        $failures++
        $rows.Add(@{ name=$case.name; result='FAIL'; evidence=$_.Exception.Message })
        Write-Warning "$($case.name): $($_.Exception.Message)"
    }
}
Save-Json (Join-Path $run 'results.json') @{
    category='operator adapter replay; not Claude consumption, tracked tests, or CI'
    binary_sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    rows=@($rows.ToArray()); failures=$failures
}
Write-Host "Evidence: $run"
if ($failures) { throw "$failures compiler-output observation(s) failed" }
Write-Host 'PASS: real C#/TypeScript/Angular/ESLint output through the built native hook adapter'
