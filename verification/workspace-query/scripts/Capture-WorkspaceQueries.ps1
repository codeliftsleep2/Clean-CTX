param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
)

$ErrorActionPreference = "Stop"
. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")
$packageRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outputRoot = Join-Path $RepositoryRoot "target\workspace-query-verification"
$workspace = Join-Path $outputRoot "workspace"
$captures = Join-Path $outputRoot "captures"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Missing production binary. Run: cargo build --all-features --bin clean-ctx"
}

New-Item -ItemType Directory -Force $workspace, $captures | Out-Null
Copy-Item (Join-Path $packageRoot "fixtures\graph.ts") (Join-Path $workspace "graph.ts") -Force
$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $outputRoot ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

function Save-Query([string]$name, $response) {
    if ($response.error) { throw "$name failed: $($response.error.message)" }
    $path = Join-Path $captures "$name.json"
    [IO.File]::WriteAllText($path, ($response | ConvertTo-Json -Depth 100), [Text.UTF8Encoding]::new($false))
    Write-Host "Captured $name"
}

$session = $null
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $outputRoot
    $file = Join-Path $workspace "graph.ts"
    $published = Invoke-CleanCtxTool $session 1 "provide_code_context" @{
        filePath = $file; workspaceRoot = $workspace; fidelity = "high"
    }
    if ($published.error) { throw "publication failed: $($published.error.message)" }
    Save-Query "publication" $published

    $queries = @(
        @{ id=2; name="find-entities"; args=@{ type="find_entities"; name="Alpha"; workspaceRoot=$workspace } },
        @{ id=3; name="forward-edges"; args=@{ type="forward_edges"; domain="angular"; entity_type="Service"; name="Alpha"; workspaceRoot=$workspace } },
        @{ id=4; name="reverse-edges"; args=@{ type="reverse_edges"; domain="angular"; entity_type="Service"; name="Repository"; workspaceRoot=$workspace } },
        @{ id=5; name="entities-in-file"; args=@{ type="entities_in_file"; file_path=$file; workspaceRoot=$workspace } },
        @{ id=6; name="transitive-dependencies"; args=@{ type="transitive_dependencies"; domain="angular"; entity_type="Service"; name="Alpha"; depth=3; workspaceRoot=$workspace } },
        @{ id=7; name="has-cycle"; args=@{ type="has_cycle"; workspaceRoot=$workspace } },
        @{ id=8; name="negative-reverse-edges"; args=@{ type="reverse_edges"; domain="angular"; entity_type="Service"; name="MissingService"; workspaceRoot=$workspace } }
    )
    foreach ($query in $queries) {
        Save-Query $query.name (Invoke-CleanCtxTool $session $query.id "workspace_query" $query.args)
    }
} finally {
    Stop-CleanCtxSession $session
}

Write-Host "Workspace-query captures written beneath $captures"
