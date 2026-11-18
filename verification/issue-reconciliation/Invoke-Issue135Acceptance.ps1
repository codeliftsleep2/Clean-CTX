param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,
    [switch]$KeepWorkspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")

$outputRoot = Join-Path $RepositoryRoot "target\issue-135-live-verification"
$captures = Join-Path $outputRoot "captures"
$workspace = Join-Path $outputRoot "workspace"
$databasePath = Join-Path $workspace "issue-135-live.db"
$outside = Join-Path $outputRoot "outside\secret.cs"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Missing production binary. Run: cargo build --locked --all-features --bin clean-ctx"
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path

$changeMarkers = @(
    "src\ir\csharp_exact_calls.rs",
    "src\ir\compiler.rs",
    "src\ir\pipeline\core.rs",
    "src\mcp\csharp_call_reconciliation.rs",
    "src\mcp\tool_handlers\query\entities.rs",
    "src\mcp\tool_handlers\core\provide\batch.rs",
    "src\mcp\tool_handlers\core\provide\evaluate.rs",
    "src\mcp\tool_schemas.rs",
    "src\queries.rs"
) | ForEach-Object { Join-Path $RepositoryRoot $_ }
$binaryTime = (Get-Item -LiteralPath $BinaryPath).LastWriteTimeUtc
$newerMarker = $changeMarkers | Where-Object {
    (Get-Item -LiteralPath $_).LastWriteTimeUtc -gt $binaryTime
} | Select-Object -First 1
if ($null -ne $newerMarker) {
    throw "Production binary is stale relative to $newerMarker. Run: cargo build --locked --all-features --bin clean-ctx"
}

if (Test-Path -LiteralPath $workspace) { Remove-Item -LiteralPath $workspace -Recurse -Force }
New-Item -ItemType Directory -Force -Path $workspace, $captures, (Split-Path $outside) | Out-Null
$config = @{
    additional_roots = @()
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $true; auto_save = $true; db_path = $databasePath }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText(
    (Join-Path $workspace ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false)
)

$callerPath = Join-Path $workspace "Caller.cs"
$targetPath = Join-Path $workspace "Target.cs"
$duplicatePath = Join-Path $workspace "Duplicate.cs"
$shapePath = Join-Path $workspace "Shape.cs"
$emptyPath = Join-Path $workspace "included\empty.cs"
$excludedPath = Join-Path $workspace "excluded\Excluded.cs"
$unsupportedPath = Join-Path $workspace "included\notes.txt"
$targetIdentity = "Target::Save()->void"
$callerIdentity = "Caller::Run(Target)->void"

function Write-Utf8([string]$Path, [string]$Content) {
    $parent = Split-Path $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false))
}

function Save-Capture([string]$Name, $Value) {
    [IO.File]::WriteAllText(
        (Join-Path $captures "$Name.json"),
        ($Value | ConvertTo-Json -Depth 100),
        [Text.UTF8Encoding]::new($false)
    )
}

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-NoError($Response, [string]$Scenario) {
    if ($null -ne $Response.PSObject.Properties["error"]) {
        throw "$Scenario returned JSON-RPC error: $($Response.error | ConvertTo-Json -Compress -Depth 30)"
    }
}

$script:rpcId = 0
function Call-Tool($Session, [string]$Tool, [hashtable]$Arguments, [string]$Scenario) {
    $script:rpcId++
    $response = Invoke-CleanCtxTool $Session $script:rpcId $Tool $Arguments
    Save-Capture ("{0:D2}-{1}" -f $script:rpcId, $Scenario) $response
    return $response
}

function Publish-CSharp($Session, [string]$Path, [string]$Scenario) {
    $response = Call-Tool $Session "provide_code_context" @{
        workspaceRoot = $workspace; filePath = $Path; fidelity = "high"; tokenizer = "o200k"
    } $Scenario
    Assert-NoError $response $Scenario
}

function Exact-Callers($Session, [string]$Scenario) {
    $response = Call-Tool $Session "workspace_query" @{
        type = "reverse_edges"
        workspaceRoot = $workspace
        domain = "csharp"
        entity_type = "Method"
        name = $targetIdentity
    } $Scenario
    Assert-NoError $response $Scenario
    return @($response.result.structuredContent.edges | Where-Object {
        $_.relation -eq "Calls" -and
        $_.subject.domain -eq "csharp" -and
        $_.subject.entity_type -eq "Method" -and
        $_.object.domain -eq "csharp" -and
        $_.object.entity_type -eq "Method" -and
        $_.object.name -eq $targetIdentity
    })
}

function Assert-ExactState($Session, [bool]$Present, [string]$Scenario) {
    $edges = @(Exact-Callers $Session $Scenario)
    if ($Present) {
        Assert-True ($edges.Count -eq 1) "$Scenario expected one exact Calls edge, found $($edges.Count)."
        Assert-True ($edges[0].subject.name -eq $callerIdentity) "$Scenario returned the wrong caller identity."
    } else {
        Assert-True ($edges.Count -eq 0) "$Scenario retained an unproven exact Calls edge."
    }
}

function Delete-Semantics($Session, [string]$Path, [string]$Scenario) {
    $response = Call-Tool $Session "delete_context" @{ filePath = $Path } $Scenario
    Assert-NoError $response $Scenario
}

Write-Utf8 $callerPath "class Caller { void Run(Target target) { target.Save(); } }`n"
Write-Utf8 $targetPath "class Target { public void Save() {} }`n"
Write-Utf8 $emptyPath "// intentionally empty`n"
Write-Utf8 $excludedPath "class Excluded { }`n"
Write-Utf8 $unsupportedPath "not source`n"
Write-Utf8 $outside "class Secret { }`n"
$padding = "// fidelity-corpus-padding-0123456789abcdef`n" * 80
Write-Utf8 $shapePath ($padding + @'
/// <summary>Shape corpus</summary>
[Audit]
public partial record Shape<T>(T Value = default!) where T : class, new()
{
    [Authorize(Roles = "Admin")]
    public TResult Map<TResult>(T input = default!) where TResult : T => default!;
}
'@)

$session = $null
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace

    Publish-CSharp $session $callerPath "caller-first"
    Assert-ExactState $session $false "caller-before-declaration"
    Publish-CSharp $session $targetPath "target-second"
    Assert-ExactState $session $true "caller-first-established"
    Assert-ExactState $session $true "warm-repeat"
    Write-Host "PASS: caller-first admission and warm repetition preserve one exact cross-file call."

    Write-Utf8 $targetPath "class Target { public void Save(string value) {} }`n"
    Publish-CSharp $session $targetPath "signature-change"
    Assert-ExactState $session $false "signature-change-revokes"
    Write-Utf8 $targetPath "class Target { public void Save() {} }`n"
    Publish-CSharp $session $targetPath "signature-restore"
    Assert-ExactState $session $true "signature-restore-establishes"

    Write-Utf8 $duplicatePath "class Target { public void Save() {} }`n"
    Publish-CSharp $session $duplicatePath "ambiguity-add"
    Assert-ExactState $session $false "ambiguity-revokes"
    Delete-Semantics $session $duplicatePath "ambiguity-delete"
    Assert-ExactState $session $true "ambiguity-removal-restores"
    Delete-Semantics $session $targetPath "target-delete"
    Assert-ExactState $session $false "declaration-removal-revokes"
    Publish-CSharp $session $targetPath "target-republish"
    Assert-ExactState $session $true "declaration-republish-restores"
    Write-Host "PASS: signature, ambiguity, removal, and replacement transitions establish and revoke exact calls."

    $batch = Call-Tool $session "provide_code_context" @{
        workspaceRoot = $workspace
        fidelity = "verbatim"
        files = @(@{ id = "shape"; filePath = $shapePath; fidelity = "high" })
    } "batch-top-level-fidelity"
    Assert-True ($null -ne $batch.PSObject.Properties["error"]) "Top-level batch fidelity was accepted."
    Assert-True ($batch.error.code -eq -32602) "Top-level batch fidelity used the wrong error code."
    Assert-True ([string]$batch.error.message -match "each file item") "Batch fidelity error omitted item-owned guidance."

    $shape = Call-Tool $session "provide_code_context" @{
        workspaceRoot = $workspace; filePath = $shapePath; fidelity = "high"
    } "csharp-degradation"
    Assert-NoError $shape "C# degradation"
    $degradation = $shape.result._meta.degradation
    Assert-True ($degradation.language -eq "csharp") "C# degradation omitted its language scope."
    Assert-True ($degradation.semantic_completeness -eq "declared_loss") "C# degradation omitted declared-loss truth."
    Assert-True (@($degradation.omits) -contains "declaration_attributes") "C# degradation omitted attribute loss."
    Write-Host "PASS: batch validation and C# degradation metadata survive live JSON-RPC serialization."

    $empty = Call-Tool $session "workspace_query" @{
        type = "entities_in_file"; workspaceRoot = $workspace; file_path = $emptyPath
    } "empty-file"
    Assert-NoError $empty "empty admitted file"
    Assert-True ($empty.result.structuredContent.count -eq 0) "Admitted empty file did not return a valid zero result."

    $excluded = Call-Tool $session "workspace_query" @{
        type = "entities_in_file"; workspaceRoot = $workspace
        withinPath = "included"; file_path = $excludedPath
    } "excluded-file"
    Assert-True ($excluded.error.data.reason -eq "excluded_by_within_path") "Wrong withinPath exclusion reason."
    $unsupported = Call-Tool $session "workspace_query" @{
        type = "entities_in_file"; workspaceRoot = $workspace; file_path = $unsupportedPath
    } "unsupported-file"
    Assert-True ($unsupported.error.data.reason -eq "unsupported_file_type") "Wrong unsupported-file reason."
    $missingPath = Join-Path $workspace "included\missing.cs"
    $missing = Call-Tool $session "workspace_query" @{
        type = "entities_in_file"; workspaceRoot = $workspace; file_path = $missingPath
    } "missing-file"
    Assert-True ($missing.error.data.reason -eq "file_not_found") "Wrong missing-file reason."
    $unauthorized = Call-Tool $session "workspace_query" @{
        type = "entities_in_file"; workspaceRoot = $workspace; file_path = $outside
    } "unauthorized-file"
    Assert-True ($unauthorized.error.data.reason -eq "outside_authorized_roots") "Wrong unauthorized-file reason."
    $unauthorizedJson = $unauthorized | ConvertTo-Json -Depth 100 -Compress
    Assert-True (-not $unauthorizedJson.Contains($outside)) "Unauthorized response leaked the requested path."
    Write-Host "PASS: live file diagnostics distinguish empty, excluded, unsupported, missing, and unauthorized paths."
}
finally {
    Stop-CleanCtxSession $session
}

# The second process is a genuinely cold order-independence check. Remove only
# this harness-owned durable store after the first process has fully exited.
foreach ($databaseArtifact in @($databasePath, "$databasePath-wal", "$databasePath-shm")) {
    if (Test-Path -LiteralPath $databaseArtifact) {
        Remove-Item -LiteralPath $databaseArtifact -Force
    }
}

$session = $null
try {
    $session = Start-CleanCtxSession -BinaryPath $BinaryPath -WorkingDirectory $workspace
    Publish-CSharp $session $targetPath "cold-target-first"
    Publish-CSharp $session $callerPath "cold-caller-second"
    Assert-ExactState $session $true "target-first-established"
    Write-Host "PASS: target-first cold admission matches caller-first exact-call truth."
}
finally {
    Stop-CleanCtxSession $session
    if ($KeepWorkspace) {
        Write-Host "Retained Issue #135 live workspace: $workspace"
    } elseif (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
}

Write-Host "PASS: Issue #135 consolidated live acceptance completed."
Write-Host "Captures: $captures"
