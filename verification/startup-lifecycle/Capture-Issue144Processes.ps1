[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
$outputDirectory = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null

$processes = @(
    Get-CimInstance Win32_Process |
        Where-Object { $_.Name -ieq 'clean-ctx.exe' } |
        Sort-Object CreationDate, ProcessId |
        ForEach-Object {
            [ordered]@{
                pid = [int]$_.ProcessId
                parent_pid = [int]$_.ParentProcessId
                creation_date = $_.CreationDate
                executable_path = $_.ExecutablePath
                command_line = $_.CommandLine
            }
        }
)

$capture = [ordered]@{
    captured_at = [DateTimeOffset]::Now.ToString('o')
    machine = $env:COMPUTERNAME
    count = $processes.Count
    processes = $processes
}

$capture | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $resolvedOutput -Encoding utf8NoBOM
Write-Output "Captured $($processes.Count) clean-ctx.exe process(es): $resolvedOutput"
