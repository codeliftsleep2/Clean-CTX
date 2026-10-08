#requires -Version 7.0
param(
    [string]$BinaryPath = "",
    [string]$HelperPath = "",
    [string]$PythonPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../../..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $RepositoryRoot "verification/context-compression/scripts/McpSession.ps1")
$packageRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target/context-compression-verification"
$runtime = Join-Path $runtimeRoot ("edge-case-runtime-" + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $runtime "workspace"
$captures = Join-Path $runtimeRoot "captures"
$manifestPath = Join-Path $captures "edge-case-verification.json"
$extension = if ($IsWindows) { ".exe" } else { "" }
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target/debug/clean-ctx$extension" }
if (-not $HelperPath) { $HelperPath = Join-Path $runtimeRoot "scripts/measure$extension" }
foreach ($executable in @($BinaryPath, $HelperPath)) {
    if (-not (Test-Path -LiteralPath $executable)) {
        throw "Missing executable $executable. Build the production binary and run Build-MeasureHelper.ps1."
    }
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$HelperPath = (Resolve-Path -LiteralPath $HelperPath).Path
$pythonPrefix = @()
if (-not $PythonPath) {
    foreach ($name in @('python3', 'python', 'py')) {
        $command = Get-Command $name -ErrorAction SilentlyContinue
        if ($command) { $PythonPath = $command.Source; if ($name -eq 'py') { $pythonPrefix = @('-3') }; break }
    }
}
if (-not $PythonPath) { throw "Python 3 is required for read-only SQLite capture; pass -PythonPath." }

New-Item -ItemType Directory -Force $runtime, $workspace, $captures | Out-Null
Remove-Item -LiteralPath $manifestPath -Force -ErrorAction SilentlyContinue
Copy-Item (Join-Path $packageRoot "fixtures/*") $workspace -Force
$database = Join-Path $runtime "persistence.db"
$config = @{
    additional_roots = @($workspace); auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $true; auto_save = $true; db_path = $database }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $runtime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))
$cases = @(
    @{ id = "edge-angular-arrows-edit"; file = "angular-arrows.ts" },
    @{ id = "edge-csharp-lambdas-edit"; file = "csharp-lambdas.cs" }
)
function Save-Response([string]$Directory, [string]$Name, $Response) {
    if ($Response.PSObject.Properties['error']) { throw "$Name failed: $($Response.error.message)" }
    $Response | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM (Join-Path $Directory "$Name.json")
    $text = [string]$Response.result.content[0].text
    if (-not $text -or $text.StartsWith('// CONTROL-FULL') -or $text.StartsWith('// COMPACT-A')) {
        throw "$Name violated the model-facing presentation boundary."
    }
    if ($Response.result._meta.content_kind -eq 'raw_passthrough') {
        $source = [IO.File]::ReadAllText((Join-Path $workspace $case.file))
        if ($text -cne $source) { throw "$Name raw passthrough changed the source document." }
    }
    [IO.File]::WriteAllText((Join-Path $Directory "$Name-model.txt"), $text, [Text.UTF8Encoding]::new($false))
}
$session = $null
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $runtime
    $requestId = 700
    foreach ($case in $cases) {
        $requestId++
        $path = Join-Path $workspace $case.file
        $response = Invoke-CleanCtxTool $session $requestId "provide_code_context" @{
            filePath = $path; workspaceRoot = $workspace; fidelity = "edit"; tokenizer = "o200k"
        }
        $directory = Join-Path $captures $case.id
        New-Item -ItemType Directory -Force $directory | Out-Null
        Save-Response $directory "response" $response
    }
} finally { Stop-CleanCtxSession $session; $session = $null }

foreach ($case in $cases) {
    $directory = Join-Path $captures $case.id
    & $PythonPath @pythonPrefix (Join-Path $PSScriptRoot "Export-BinaryBaseline.py") $database (Join-Path $workspace $case.file) $directory
    if ($LASTEXITCODE -ne 0) { throw "$($case.id) durable capture failed." }
    & $HelperPath baseline-oracle (Join-Path $directory "baseline.bin") (Join-Path $directory "baseline.json") (Join-Path $directory "control-full.txt")
    if ($LASTEXITCODE -ne 0) { throw "$($case.id) Binary0x04 decoding/oracle rendering failed." }
}

# A fresh production session must consume the same durable baseline and aligned
# semantic snapshot. Reduced restore result.ir is not the semantic oracle.
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $runtime
    foreach ($case in $cases) {
        $requestId++
        $response = Invoke-CleanCtxTool $session $requestId "restore_context" @{
            filePath = (Join-Path $workspace $case.file); workspaceRoot = $workspace; tokenizer = "o200k"
        }
        $directory = Join-Path $captures $case.id
        Save-Response $directory "restore" $response
        if ($response.result._meta.restored -ne $true -or $response.result._meta.semantic_edge_count -le 0) {
            throw "$($case.id) did not restore durable semantic state."
        }
    }
} finally { Stop-CleanCtxSession $session; $session = $null }

$hashes = @{}
foreach ($case in $cases) {
    foreach ($name in @('response.json', 'restore.json', 'baseline.bin', 'baseline.json', 'control-full.txt')) {
        $key = "$($case.id)/$name"
        $hashes[$key] = (Get-FileHash -LiteralPath (Join-Path $captures $key) -Algorithm SHA256).Hash
    }
}
@{
    physical_version = 4; runtime = $runtime; hashes = $hashes
    binary_sha256 = (Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    verified_at_utc = [DateTime]::UtcNow.ToString('o')
    category = 'operator evidence; not a tracked regression test or CI gate'
} | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $manifestPath
Write-Host "Captured Binary0x04, aligned semantic snapshots, separate model output, and fresh-session restore beneath $captures"
