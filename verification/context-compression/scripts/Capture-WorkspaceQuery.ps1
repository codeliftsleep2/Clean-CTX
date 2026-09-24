param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "McpSession.ps1")
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$fixtureSource = Join-Path $definitionRoot "fixtures"
$runtime = Join-Path $runtimeRoot "runtime"
$workspace = Join-Path $runtime "workspace"
$captures = Join-Path $runtimeRoot "captures"
$workspaceQueryDir = Join-Path $captures "workspace-query"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) { throw "Missing binary. Run: cargo build --all-features" }

New-Item -ItemType Directory -Force $runtime, $workspace, $workspaceQueryDir | Out-Null
Get-ChildItem -LiteralPath $fixtureSource -File | Copy-Item -Destination $workspace -Force

$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $runtime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

$mapping = Get-Content -Raw (Join-Path $definitionRoot "reasoning\workspace-query-oracles.json") | ConvertFrom-Json

$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    # Compile every fixture so the WorkspaceIndex has the cross-file facts.
    $fixtures = @("cross-a.ts", "cross-b.ts", "semantic-rich.ts", "large-dense.ts")
    $id = 1
    foreach ($fixture in $fixtures) {
        $path = Join-Path $workspace $fixture
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $response = Invoke-CleanCtxTool $session $id "compress_code_context" @{ filePath = $path; workspaceRoot = $workspace; fidelity = "high" }
        if ($null -ne $response.error) { throw "compile $fixture failed: $($response.error.message)" }
        Write-Host "Compiled $fixture"
        $id++
    }
    # Run each workspace_query and save its model-visible content text.
    foreach ($entry in $mapping) {
        $args = @{ type = $entry.type; workspaceRoot = $workspace }
        foreach ($property in $entry.arguments.PSObject.Properties) {
            $args[$property.Name] = $property.Value
        }
        $response = Invoke-CleanCtxTool $session $id "workspace_query" $args
        if ($null -ne $response.error) { throw "workspace_query $($entry.id) failed: $($response.error.message)" }
        $text = ""
        $content = @($response.result.content)
        if ($content.Count -gt 0 -and $null -ne $content[0].text) { $text = [string]$content[0].text }
        $outputPath = Join-Path $workspaceQueryDir "$($entry.id).txt"
        [IO.File]::WriteAllText($outputPath, $text, [Text.UTF8Encoding]::new($false))
        Write-Host "Captured workspace_query $($entry.id) ($($entry.type)) -> $outputPath"
        $id++
    }
} finally { Stop-CleanCtxSession $session }

Write-Host "Workspace-query captures written beneath $workspaceQueryDir"
