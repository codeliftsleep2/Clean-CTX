param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
)

# Captures the task-evaluation scenarios (schema-v5/task-scenarios.json) into
# captures/<scenario>/content.txt (the SCHEMA-v5 Edit presentation). Kept fully
# separate from the codec harness: it uses its own scenario file and does not
# touch the shared expected/scenarios.json.

$ErrorActionPreference = "Stop"
. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$runtime = Join-Path $runtimeRoot "runtime"
$workspace = Join-Path $runtime "workspace"
$captures = Join-Path $runtimeRoot "captures"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) { throw "Missing binary. Run: cargo build --all-features" }

New-Item -ItemType Directory -Force $runtime, $workspace, $captures | Out-Null

# Copy the large fixtures into the workspace (flat), alongside the correctness
# fixtures already present.
foreach ($large in @("LargeService.ts", "UserManagementService.ts")) {
    $src = Join-Path $RepositoryRoot "src\test_files\$large"
    if (-not (Test-Path -LiteralPath $src)) { throw "Missing large fixture: $src" }
    Copy-Item -LiteralPath $src -Destination $workspace -Force
}

$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $runtime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

$scenarios = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\task-scenarios.json") | ConvertFrom-Json)

function Save-TaskCapture($scenario, $response) {
    $dir = Join-Path $captures $scenario.id
    New-Item -ItemType Directory -Force $dir | Out-Null
    $response | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM (Join-Path $dir "response.json")
    $content = @($response.result.content)
    if ($content.Count -gt 0 -and $null -ne $content[0].text) {
        [IO.File]::WriteAllText((Join-Path $dir "content.txt"), [string]$content[0].text, [Text.UTF8Encoding]::new($false))
    }
    return $dir
}

$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    $id = 1
    foreach ($scenario in $scenarios) {
        Write-Host "Capturing task scenario: $($scenario.id)"
        $path = Join-Path $workspace $scenario.fixture
        $args = @{ filePath = $path; workspaceRoot = $workspace; tokenizer = "o200k" }
        if ($scenario.explicitFidelity) { $args.fidelity = $scenario.explicitFidelity }
        if ($scenario.focus) { $args.focusMethods = @($scenario.focus) }
        $response = Invoke-CleanCtxTool $session $id "provide_code_context" $args
        $id++
        if ($null -ne $response.error) { throw "provide_code_context failed for $($scenario.id): $($response.error.message)" }
        $dir = Save-TaskCapture $scenario $response
        Write-Host "  wrote $dir"
    }
} finally { Stop-CleanCtxSession $session }

Write-Host "Task captures written beneath $captures"
