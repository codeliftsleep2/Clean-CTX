param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "McpSession.ps1")
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$fixtureSource = Join-Path $definitionRoot "fixtures"
$trackedEconomicsSource = Join-Path $RepositoryRoot "src\test_files"
$generatedFixtures = Join-Path $runtimeRoot "fixtures"
$runtime = Join-Path $runtimeRoot "runtime"
$workspace = Join-Path $runtime "workspace"
$economicsWorkspace = Join-Path $workspace "tracked-economics"
$captures = Join-Path $runtimeRoot "captures"
$measure = Join-Path $runtimeRoot "scripts\measure.exe"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) { throw "Missing binary. Run: cargo build --all-features" }
if (-not (Test-Path -LiteralPath $measure)) { throw "Missing measure.exe. Run scripts/Build-MeasureHelper.ps1" }

New-Item -ItemType Directory -Force $runtime, $workspace, $captures | Out-Null
Get-ChildItem -LiteralPath $fixtureSource -File | Copy-Item -Destination $workspace -Force
New-Item -ItemType Directory -Force $economicsWorkspace | Out-Null
foreach ($family in @("angular", "dotnet", "typescript")) {
    Copy-Item -LiteralPath (Join-Path $trackedEconomicsSource $family) `
        -Destination $economicsWorkspace -Recurse -Force
}
foreach ($largeFixture in @("LargeService.ts", "UserManagementService.ts")) {
    Copy-Item -LiteralPath (Join-Path $trackedEconomicsSource $largeFixture) `
        -Destination $economicsWorkspace -Force
}
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

function Save-Capture($scenario, $response, [string]$suffix = "", [string]$sourcePath = "") {
    $name = if ($suffix) { "$($scenario.id)-$suffix" } else { $scenario.id }
    $dir = Join-Path $captures $name
    New-Item -ItemType Directory -Force $dir | Out-Null
    $response | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM (Join-Path $dir "response.json")
    $content = @($response.result.content)
    if ($content.Count -gt 0 -and $null -ne $content[0].text) {
        $text = [string]$content[0].text
        # control-full.txt is overwritten with the codec CONTROL-FULL oracle for
        # non-verbatim, non-error scenarios; content.txt preserves the actual
        # model-visible SCHEMA-v5 presentation for the schema-v5 harness.
        [IO.File]::WriteAllText((Join-Path $dir "control-full.txt"), $text, [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $dir "content.txt"), $text, [Text.UTF8Encoding]::new($false))
    }
    if ($sourcePath -and (Test-Path -LiteralPath $sourcePath)) {
        Copy-Item -LiteralPath $sourcePath -Destination (Join-Path $dir "raw-source.txt") -Force
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

function Invoke-OracleRender($structuredResponse, $scenario, $dir, $path) {
    if ($null -eq $structuredResponse.result.ir -and
        $null -eq $structuredResponse.result.structuredContent.ir) {
        throw "CONTROL-FULL oracle input for $($scenario.id) is missing result.ir"
    }
    # Semantic edges are served on demand by workspace_query and are no longer
    # shipped on content responses, so the CONTROL-FULL oracle is rendered from
    # the reduced result.ir with an empty edge snapshot.
    $responsePath = Join-Path $dir "oracle-source-response.json"
    $structuredResponse | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $responsePath
    $focus = if ($scenario.focus) { ($scenario.focus -join ",") } else { "" }
    & $measure oracle $responsePath $path $scenario.fidelity $focus (Join-Path $dir "control-full.txt")
    if ($LASTEXITCODE -ne 0) { throw "CONTROL-FULL oracle render failed for $($scenario.id)" }
}

function Get-EconomicsLanguage([string]$relative) {
    switch -Regex ($relative.Replace('\', '/')) {
        '^LargeService\.ts$' { return "typescript" }
        '^UserManagementService\.ts$' { return "angular" }
        '^dotnet/' { return "csharp" }
        '^angular/' { return "angular" }
        '^typescript/' { return "typescript" }
        '^java/' { return "java" }
        default { return "unknown" }
    }
}

function Get-EconomicsFocus([string]$relative) {
    switch -Regex ($relative.Replace('\', '/')) {
        '^LargeService\.ts$' { return "UserService.createUser" }
        '^UserManagementService\.ts$' { return "UserManagementService.createUser" }
        '^dotnet/OrderManagementService\.cs$' { return "OrderService.CreateOrderAsync" }
        default { return "" }
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
        $dir = Save-Capture $scenario $response "" $path
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
        Invoke-OracleRender $compress $scenario $dir $path
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
            $dir = Save-Capture $synthetic $response "" $path
            $requestId++
            $compress = Invoke-CleanCtxTool $session $requestId "compress_code_context" @{
                filePath=$path; workspaceRoot=$workspace; fidelity=$fidelity; tokenizer="o200k"
            }
            Invoke-OracleRender $compress $synthetic $dir $path
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
        $dir = Save-Capture $scenario $response "" $path
        if ($scenario.fidelity -eq "verbatim") {
            foreach ($tokenizer in @("cl100k", "o200k")) {
                Copy-Item -LiteralPath $path -Destination (Join-Path $dir "control-prod-$tokenizer.txt") -Force
            }
        } else {
            Invoke-OracleRender $response $scenario $dir $path
            Invoke-ProdRender $response $scenario $dir $path
        }
    }

    $minimumEconomicsBytes = 8kb
    $supportedTrackedFiles = Get-ChildItem -LiteralPath $economicsWorkspace -Recurse -File |
        Where-Object { $_.Extension -in ".ts", ".cs" }
    $economicsFiles = @($supportedTrackedFiles | Where-Object { $_.Length -ge $minimumEconomicsBytes })
    $smallTrackedFiles = @($supportedTrackedFiles | Where-Object { $_.Length -lt $minimumEconomicsBytes })
    $rawOnlyTemplates = Get-ChildItem -LiteralPath $economicsWorkspace -Recurse -File |
        Where-Object { $_.Extension -eq ".html" }
    Write-Host "Skipping $($rawOnlyTemplates.Count) Angular HTML template(s): structured IR is unsupported, so production is raw-only."
    Write-Host "Excluding $($smallTrackedFiles.Count) sub-8 KiB file(s) from compression economics; they remain correctness fixtures and raw-fallback cases."
    if (-not $economicsFiles.Count) {
        throw "No tracked source file meets the 8 KiB production-economics minimum"
    }
    Get-ChildItem -LiteralPath $captures -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "economics-*" } |
        Remove-Item -Recurse -Force
    foreach ($economicsFile in $economicsFiles) {
        $relative = [IO.Path]::GetRelativePath($economicsWorkspace, $economicsFile.FullName)
        $language = Get-EconomicsLanguage $relative
        $slug = ($relative -replace '[^A-Za-z0-9]+', '-').Trim('-').ToLowerInvariant()
        $focusTarget = Get-EconomicsFocus $relative
        Write-Host "Capturing tracked economics scenario: $relative ($language)"
        $responses = @{}
        foreach ($captureFidelity in @("low", "medium", "high", "edit")) {
            $requestId++
            $responses[$captureFidelity] = Invoke-CleanCtxTool $session $requestId "compress_code_context" @{
                filePath = $economicsFile.FullName
                workspaceRoot = $workspace
                fidelity = $captureFidelity
                tokenizer = "o200k"
            }
            Require-ToolSuccess $responses[$captureFidelity] "tracked economics capture $relative @ $captureFidelity"
        }
        $matrix = @(
            @{ fidelity = "low";    focusCsv = "";           focusMode = "none";       capture = "low" }
            @{ fidelity = "medium"; focusCsv = "";           focusMode = "none";       capture = "medium" }
            @{ fidelity = "high";   focusCsv = "";           focusMode = "none";       capture = "high" }
            @{ fidelity = "edit";   focusCsv = "";           focusMode = "all-bodies"; capture = "edit" }
            @{ fidelity = "edit";   focusCsv = $focusTarget; focusMode = "focused";    capture = "edit" }
        )
        foreach ($entry in $matrix) {
            if ($entry.focusMode -eq "focused" -and -not $entry.focusCsv) {
                Write-Host "  skipping focused Edit for $relative (no focus target defined)"
                continue
            }
            $dirName = "economics-$language-$slug-$($entry.fidelity)-$($entry.focusMode)"
            $dir = Join-Path $captures $dirName
            New-Item -ItemType Directory -Force $dir | Out-Null
            $meta = @{
                fixture = $relative
                language = $language
                fidelity = $entry.fidelity
                focus_mode = $entry.focusMode
                focus_target = $entry.focusCsv
            } | ConvertTo-Json -Depth 10
            [IO.File]::WriteAllText((Join-Path $dir "capture-meta.json"), $meta, [Text.UTF8Encoding]::new($false))
            Copy-Item -LiteralPath $economicsFile.FullName -Destination (Join-Path $dir "raw-source.txt") -Force
            $oracleResponsePath = Join-Path $dir "oracle-source-response.json"
            $responses[$entry.capture] | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8NoBOM $oracleResponsePath
            & $measure oracle $oracleResponsePath $economicsFile.FullName $entry.fidelity $entry.focusCsv (Join-Path $dir "control-full.txt")
            if ($LASTEXITCODE -ne 0) { throw "CONTROL-FULL oracle render failed for $dirName" }
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
    Save-Capture $deltaScenario $baseline "baseline" $deltaPath | Out-Null
    $baselineDir = Join-Path $captures "delta-flow-baseline"
    Invoke-OracleRender $baseline ([pscustomobject]@{ id="delta-flow-baseline"; fidelity="high"; focus=$null }) $baselineDir $deltaPath
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
    Save-Capture $deltaScenario $delta "delta" $deltaPath | Out-Null
    $applied = Invoke-CleanCtxTool $session 1004 "apply_delta" @{
        delta = $delta.result.delta; currentVersion = $delta.result.from_version
    }
    Require-ToolSuccess $applied "delta application"
    $applyDir = Save-Capture $deltaScenario $applied "apply" $deltaPath
    Invoke-OracleRender $applied ([pscustomobject]@{ id="delta-flow-apply"; fidelity="high"; focus=$null }) $applyDir $deltaPath
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
    $restoreDir = Save-Capture $restoreScenario $restored "" (Join-Path $runtime "restore-latest-source.ts")
    Invoke-OracleRender $restored $restoreScenario $restoreDir $restorePath
    Invoke-ProdRender $restored $restoreScenario $restoreDir $restorePath
    $replayed = Invoke-CleanCtxTool $session 2006 "replay_history" @{ filePath = $restorePath; targetSequence = 0 }
    Require-ToolSuccess $replayed "registered replay"
    $replayDir = Save-Capture $replayScenario $replayed "" (Join-Path $runtime "restore-baseline-source.ts")
    Invoke-OracleRender $replayed $replayScenario $replayDir $restorePath
    Invoke-ProdRender $replayed $replayScenario $replayDir $restorePath
} finally { Stop-CleanCtxSession $session }

Write-Host "Captures written beneath $captures"
