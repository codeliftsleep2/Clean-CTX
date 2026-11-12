param(
    [string]$WorkspaceRoot = (Resolve-Path '.').Path,
    [string]$Binary = 'target/debug/clean-ctx.exe',
    [string]$OutputRoot = 'target/pcg-b9/production-new-run'
)
# Operator verification only; this script does not build, run Cargo or evaluate agents.
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$profile = Join-Path $workspace '.clean-ctx/pcg-documents/profile.json'
$launch = Join-Path $workspace '.clean-ctx/pcg-documents/authoring'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh output directory' }
[IO.Directory]::CreateDirectory($output) | Out-Null
$queries = @([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'b9-knowledge-queries.json'), $utf8) | ConvertFrom-Json -AsHashtable)
foreach ($query in $queries) { $query.workspaceRoot = $workspace }
$requestFile = Join-Path $output 'requests.json'
[IO.File]::WriteAllText($requestFile, ($queries | ConvertTo-Json -Depth 30), $utf8)
& (Join-Path $PSScriptRoot 'invoke-workspace-knowledge.ps1') `
    -WorkspaceRoot $workspace -Profile $profile -Binary $Binary -Requests $requestFile `
    -OutputRoot (Join-Path $output 'knowledge') -LaunchDirectory $launch
& python (Join-Path $PSScriptRoot 'verify-b9-knowledge.py') (Join-Path $output 'knowledge') `
    --reports (Join-Path $output 'reports')
if ($LASTEXITCODE -ne 0) { throw 'Independent production capture verification failed' }
Write-Output "B9 production knowledge evidence: $output"
