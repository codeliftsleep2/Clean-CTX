param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")
$packageRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$runtime = Join-Path $runtimeRoot "edge-case-runtime"
$workspace = Join-Path $runtime "workspace"
$captures = Join-Path $runtimeRoot "captures"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Missing production binary. Run: cargo build --all-features --bin clean-ctx"
}

New-Item -ItemType Directory -Force $runtime, $workspace, $captures | Out-Null
Copy-Item (Join-Path $packageRoot "fixtures\*") $workspace -Force
$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $runtime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

$cases = @(
    @{ id="edge-angular-arrows-edit"; file="angular-arrows.ts" },
    @{ id="edge-csharp-lambdas-edit"; file="csharp-lambdas.cs" }
)
$session = $null
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $runtime
    $requestId = 700
    foreach ($case in $cases) {
        $requestId++
        $path = Join-Path $workspace $case.file
        $response = Invoke-CleanCtxTool $session $requestId "provide_code_context" @{
            filePath=$path; workspaceRoot=$workspace; fidelity="edit"; tokenizer="o200k"
        }
        if ($response.error) { throw "$($case.id) failed: $($response.error.message)" }
        $directory = Join-Path $captures $case.id
        New-Item -ItemType Directory -Force $directory | Out-Null
        $response | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM (Join-Path $directory "response.json")
        $text = [string]$response.result.content[0].text
        if (-not $text.StartsWith("// CONTROL-FULL v2")) { throw "$($case.id) did not return CONTROL-FULL v2" }
        [IO.File]::WriteAllText((Join-Path $directory "control-full.txt"), $text, [Text.UTF8Encoding]::new($false))
        Write-Host "Captured $($case.id)"
    }
} finally {
    Stop-CleanCtxSession $session
}
Write-Host "Edge-case captures written beneath $captures"
