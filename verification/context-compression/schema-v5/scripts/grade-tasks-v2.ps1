param(
    [string]$BinaryPath = "",
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
)

# v2 grader: apply_edit round-trip. Feeds each model answer's operation to the
# real apply_edit tool and checks acceptance + the on-disk change. This is the
# byte-exact gate the v1 substring grader cannot reach: apply_edit verifies
# expectedOldText against the current bytes (EOL-preserving) before writing.

$ErrorActionPreference = "Stop"
. (Join-Path $RepositoryRoot "verification\context-compression\scripts\McpSession.ps1")
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$runtimeRoot = Join-Path $RepositoryRoot "target\context-compression-verification"
$editRuntime = Join-Path $runtimeRoot "edit-runtime"
$workspace = Join-Path $editRuntime "workspace"
$captures = Join-Path $runtimeRoot "captures"
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target\debug\clean-ctx.exe" }
if (-not (Test-Path -LiteralPath $BinaryPath)) { throw "Missing binary. Run: cargo build --all-features" }

$tasks = @(Get-Content -Raw (Join-Path $definitionRoot "schema-v5\tasks.json") | ConvertFrom-Json)
$answersPath = Join-Path $captures "task-answers.json"
if (-not (Test-Path -LiteralPath $answersPath)) { throw "Missing task-answers.json. Run run-tasks.ps1 first." }
$answers = @(Get-Content -Raw -LiteralPath $answersPath | ConvertFrom-Json)

# Fresh workspace: copy the large fixtures (apply_edit mutates them on disk).
New-Item -ItemType Directory -Force $workspace | Out-Null
foreach ($large in @("LargeService.ts", "UserManagementService.ts")) {
    Copy-Item -LiteralPath (Join-Path $RepositoryRoot "src\test_files\$large") -Destination $workspace -Force
}
$config = @{
    additional_roots = @($workspace)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $false }
} | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText((Join-Path $editRuntime ".clean-ctx.json"), $config, [Text.UTF8Encoding]::new($false))

$session = Start-CleanCtxSession $BinaryPath $editRuntime
try {
    $id = 1
    $results = foreach ($task in $tasks) {
        $ans = @($answers | Where-Object { $_.task_id -eq $task.id })
        if ($ans.Count -ne 1) { throw "Missing answer for $($task.id)" }
        $op = $ans[0].answer
        $path = Join-Path $workspace ($task.fixture | Split-Path -Leaf)

        # 1. Establish tracked state (apply_edit requires prior compile/read).
        $readArgs = @{ filePath = $path; workspaceRoot = $workspace; fidelity = "edit"; focusMethods = @($task.focus) }
        $read = Invoke-CleanCtxTool $session $id "provide_code_context" $readArgs
        $id++
        if ($null -ne $read.error) { throw "read failed for $($task.id): $($read.error.message)" }

        # 2. Apply the model's edit operation.
        $editArgs = @{
            filePath = $path
            workspaceRoot = $workspace
            operations = @(@{ type = "replace_body"; target = [string]$op.target; expectedOldText = [string]$op.expectedOldText; newText = [string]$op.newText })
            verify = $true
        }
        $edit = Invoke-CleanCtxTool $session $id "apply_edit" $editArgs
        $id++
        $accepted = ($null -eq $edit.error)
        $errorMsg = if ($edit.error) { [string]$edit.error.message } else { "" }

        # 3. Verify the on-disk change.
        $fileText = Get-Content -Raw -LiteralPath $path
        $changed = ($fileText.Contains($task.replace) -and -not $fileText.Contains($task.find))

        [ordered]@{ task_id = $task.id; accepted = $accepted; changed = $changed; pass = ($accepted -and $changed); error = $errorMsg }
    }
} finally { Stop-CleanCtxSession $session }

$output = Join-Path $captures "task-apply-grades.json"
$results | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $output
$results | ConvertTo-Json -Depth 10
$pass = @($results | Where-Object pass).Count
Write-Host "PASS: $pass/$($tasks.Count) apply_edit round-trips"
