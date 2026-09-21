param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "McpSession.ps1")
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$fixtureSource = Join-Path $definitionRoot "fixtures"
$generatedFixtures = Join-Path $runtimeRoot "fixtures"
$runtime = Join-Path $runtimeRoot "runtime"
$workspace = Join-Path $runtime "workspace"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) { throw "Missing binary. Run: cargo build --all-features" }
if (-not (Test-Path -LiteralPath $measure)) { throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1" }

New-Item -ItemType Directory -Force $runtime, $workspace, $captures | Out-Null
Get-ChildItem -LiteralPath $fixtureSource -File | Copy-Item -Destination $workspace -Force
if (Test-Path -LiteralPath $generatedFixtures) {
    Get-ChildItem -LiteralPath $generatedFixtures -File | Copy-Item -Destination $workspace -Force
}
$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $true; auto_save = $true; db_path = (Join-Path $runtime "baseline.db") }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $runtime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

function Save-Capture($scenario, $response, [string]$suffix = "") {
    $name = if ($suffix) { "$($scenario.id)-$suffix" } else { $scenario.id }
    $dir = Join-Path $captures $name
    New-Item -ItemType Directory -Force $dir | Out-Null
    $response | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM (Join-Path $dir "response.json")
    $content = @($response.result.content)
    if ($content.Count -gt 0 -and $null -ne $content[0].text) {
        [IO.File]::WriteAllText((Join-Path $dir "control-full.txt"), [string]$content[0].text, [Text.UTF8Encoding]::new($false))
    }
    return $dir
}

function Require-ToolSuccess($response, [string]$label) {
    if ($null -ne $response.error) {
        throw "$label failed: $($response.error.message)"
    }
}

function Invoke-ProdRender($compressResponse, $scenario, $dir, $path) {
    $responsePath = Join-Path $dir "control-prod-source-response.json"
    $compressResponse | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $responsePath
    $focus = if ($scenario.focus) { ($scenario.focus -join ",") } else { "" }
    $policy = if ($scenario.operation -eq "provide_code_context") { "provide-fallback" } else { "renderer" }
    foreach ($tokenizer in @("cl100k", "o200k")) {
        & $measure prod $responsePath $path $scenario.fidelity $focus $tokenizer $policy (Join-Path $dir "control-prod-$tokenizer.txt")
        if ($LASTEXITCODE -ne 0) { throw "CONTROL-PROD render failed for $($scenario.id)/$tokenizer" }
    }
}

$scenarios = Get-Content -Raw (Join-Path $definitionRoot "expected\scenarios.json") | ConvertFrom-Json
$session = Start-CleanCtxSession $BinaryPath $runtime
$requestId = 10
try {
    foreach ($scenario in $scenarios | Where-Object { $_.operation -eq "provide_code_context" }) {
        Write-Host "Capturing provide scenario: $($scenario.id)"
        $requestId++
        $path = Join-Path $workspace $scenario.fixture
        $args = @{ filePath = $path; workspaceRoot = $workspace; tokenizer = "o200k" }
        if ($scenario.intent) { $args.intent = $scenario.intent }
        if ($scenario.explicitFidelity) { $args.fidelity = $scenario.explicitFidelity }
        if ($scenario.focus) { $args.focusMethods = @($scenario.focus) }
        $response = Invoke-CleanCtxTool $session $requestId "provide_code_context" $args
        $dir = Save-Capture $scenario $response
        if ($scenario.expect -eq "error") { continue }
        if ($scenario.fidelity -eq "verbatim") {
            foreach ($tokenizer in @("cl100k", "o200k")) {
                Copy-Item -LiteralPath $path -Destination (Join-Path $dir "control-prod-$tokenizer.txt") -Force
            }
            continue
        }
        $requestId++
        $compress = Invoke-CleanCtxTool $session $requestId "compress_code_context" @{
            filePath = $path; workspaceRoot = $workspace; fidelity = $scenario.fidelity; tokenizer = "o200k"
        }
        Invoke-ProdRender $compress $scenario $dir $path
    }

    foreach ($definition in Get-Content -Raw (Join-Path $definitionRoot "expected\marginal-fixtures.json") | ConvertFrom-Json) {
        foreach ($variant in @("base", "plus")) {
            $id = "marginal-$($definition.id)-$variant"
            Write-Host "Capturing marginal scenario: $id"
            $path = Join-Path $workspace "$id.ts"
            $fidelity = if ($definition.id -eq "exact-body") { "edit" } else { "high" }
            $synthetic = [pscustomobject]@{ id=$id; fidelity=$fidelity; focus=$null; operation="provide_code_context" }
            $requestId++
            $response = Invoke-CleanCtxTool $session $requestId "provide_code_context" @{
                filePath=$path; workspaceRoot=$workspace; fidelity=$fidelity; tokenizer="o200k"
            }
            $dir = Save-Capture $synthetic $response
            $requestId++
            $compress = Invoke-CleanCtxTool $session $requestId "compress_code_context" @{
                filePath=$path; workspaceRoot=$workspace; fidelity=$fidelity; tokenizer="o200k"
            }
            Invoke-ProdRender $compress $synthetic $dir $path
        }
    }

    foreach ($scenario in $scenarios | Where-Object { $_.operation -eq "compress_code_context" }) {
        Write-Host "Capturing compress scenario: $($scenario.id)"
        $requestId++
        $path = Join-Path $workspace $scenario.fixture
        $response = Invoke-CleanCtxTool $session $requestId "compress_code_context" @{
            filePath=$path; workspaceRoot=$workspace; fidelity=$scenario.fidelity; tokenizer="o200k"
        }
        $dir = Save-Capture $scenario $response
        if ($scenario.fidelity -eq "verbatim") {
            foreach ($tokenizer in @("cl100k", "o200k")) {
                Copy-Item -LiteralPath $path -Destination (Join-Path $dir "control-prod-$tokenizer.txt") -Force
            }
        } else {
            Invoke-ProdRender $response $scenario $dir $path
        }
    }
} finally { Stop-CleanCtxSession $session }

# Acknowledged baseline -> delta -> apply, through registered tools.
$deltaScenario = $scenarios | Where-Object id -eq "delta-flow"
$deltaPath = Join-Path $workspace "delta-work.ts"
Copy-Item (Join-Path $fixtureSource "semantic-rich.ts") $deltaPath -Force
$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    Write-Host "Capturing delta baseline/generation/application"
    $baseline = Invoke-CleanCtxTool $session 1001 "delta_code_context" @{ filePath = $deltaPath; workspaceRoot = $workspace; fidelity = "high" }
    Require-ToolSuccess $baseline "delta baseline"
    Save-Capture $deltaScenario $baseline "baseline" | Out-Null
} finally { Stop-CleanCtxSession $session }
$deltaReplacement = "audit(`"delta-change`");`n    return `"none`";"
$updated = (Get-Content -Raw $deltaPath).Replace('return "none";', $deltaReplacement)
[IO.File]::WriteAllText($deltaPath, $updated, [Text.UTF8Encoding]::new($false))
$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    $hydrated = Invoke-CleanCtxTool $session 1002 "restore_context" @{ filePath = $deltaPath; workspaceRoot = $workspace }
    Require-ToolSuccess $hydrated "delta baseline hydration"
    $delta = Invoke-CleanCtxTool $session 1003 "delta_code_context" @{ filePath = $deltaPath; workspaceRoot = $workspace; fidelity = "high" }
    Require-ToolSuccess $delta "delta generation"
    Save-Capture $deltaScenario $delta "delta" | Out-Null
    $applied = Invoke-CleanCtxTool $session 1004 "apply_delta" @{
        delta = $delta.result.delta; currentVersion = $delta.result.from_version
    }
    Require-ToolSuccess $applied "delta application"
    $applyDir = Save-Capture $deltaScenario $applied "apply"
    $after = Invoke-CleanCtxTool $session 1005 "compress_code_context" @{ filePath = $deltaPath; workspaceRoot = $workspace; fidelity = "low" }
    Require-ToolSuccess $after "post-apply historical projection"
    Invoke-ProdRender $after ([pscustomobject]@{ id="delta-flow-apply"; fidelity="low"; focus=$null; operation="apply_delta" }) $applyDir $deltaPath
} finally { Stop-CleanCtxSession $session }

# Persist Edit, restart, restore from durable IR/edges, then replay baseline history.
$restoreScenario = $scenarios | Where-Object id -eq "restore-flow"
$replayScenario = $scenarios | Where-Object id -eq "replay-flow"
$restorePath = Join-Path $workspace "restore-work.ts"
Copy-Item (Join-Path $fixtureSource "semantic-rich.ts") $restorePath -Force
Copy-Item $restorePath (Join-Path $runtime "restore-baseline-source.ts") -Force
$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    Write-Host "Persisting restore/replay history"
    $baseline = Invoke-CleanCtxTool $session 2001 "compress_code_context" @{ filePath = $restorePath; workspaceRoot = $workspace; fidelity = "edit" }
    Require-ToolSuccess $baseline "restore/replay baseline"
} finally { Stop-CleanCtxSession $session }
$updated = (Get-Content -Raw $restorePath).Replace('return "none";', 'return "historical-latest";')
[IO.File]::WriteAllText($restorePath, $updated, [Text.UTF8Encoding]::new($false))
Copy-Item $restorePath (Join-Path $runtime "restore-latest-source.ts") -Force
$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    $hydrated = Invoke-CleanCtxTool $session 2002 "restore_context" @{ filePath = $restorePath; workspaceRoot = $workspace }
    Require-ToolSuccess $hydrated "restore/replay baseline hydration"
    $transition = Invoke-CleanCtxTool $session 2003 "delta_code_context" @{ filePath = $restorePath; workspaceRoot = $workspace; fidelity = "edit" }
    Require-ToolSuccess $transition "restore/replay delta generation"
    $applied = Invoke-CleanCtxTool $session 2004 "apply_delta" @{
        delta = $transition.result.delta; currentVersion = $transition.result.from_version
    }
    Require-ToolSuccess $applied "restore/replay delta application"
} finally { Stop-CleanCtxSession $session }
$session = Start-CleanCtxSession $BinaryPath $runtime
try {
    Write-Host "Capturing registered restore and replay"
    $restored = Invoke-CleanCtxTool $session 2005 "restore_context" @{ filePath = $restorePath; workspaceRoot = $workspace }
    Require-ToolSuccess $restored "registered restore"
    $restoreDir = Save-Capture $restoreScenario $restored
    Invoke-ProdRender $restored $restoreScenario $restoreDir $restorePath
    $replayed = Invoke-CleanCtxTool $session 2006 "replay_history" @{ filePath = $restorePath; targetSequence = 0 }
    Require-ToolSuccess $replayed "registered replay"
    $replayDir = Save-Capture $replayScenario $replayed
    Invoke-ProdRender $replayed $replayScenario $replayDir $restorePath
} finally { Stop-CleanCtxSession $session }

Write-Host "Captures written beneath $captures"
