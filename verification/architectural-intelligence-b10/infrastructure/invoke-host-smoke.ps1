param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot,
    [ValidatePattern('^[0-9a-f]{64}$')]
    [string]$ExpectedBinarySha256 = '5dcd08d0929f5f6a10c6ad7d4392e15f23827da9a926b25bc2dc43c99b336fb1',
    [string]$TaskPromptFile,
    [string]$Purpose = 'operator-informed actual-host smoke; not isolated A/B or discovery evaluation'
)
# Operator-run host reachability only. Not an independent reasoning evaluation.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh output directory.' }
$binary = Join-Path $root 'target/debug/clean-ctx.exe'
$profile = Join-Path $root '.clean-ctx/pcg-documents/profile.json'
$launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
$expectedBinary = $ExpectedBinarySha256
$expectedProfile = '0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9'
if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -ne $expectedBinary) {
    throw 'Executable differs from the explicitly pinned identity. Stop and report it; do not rebuild implicitly.'
}
if ((Get-FileHash -LiteralPath $profile).Hash.ToLowerInvariant() -ne $expectedProfile) {
    throw 'Accepted profile changed. Stop and report the identity.'
}
$codex = (Get-Command codex -ErrorAction Stop).Source
$launcher = $codex
$prefixArguments = @()
if ([IO.Path]::GetExtension($codex) -in @('.ps1', '.cmd')) {
    # Same installed npm entry point as the shim, without PowerShell text decoding.
    $entry = Join-Path (Split-Path $codex -Parent) 'node_modules/@openai/codex/bin/codex.js'
    if (!(Test-Path -LiteralPath $entry)) { throw 'Installed Codex npm entry point not found.' }
    $localNode = Join-Path (Split-Path $codex -Parent) 'node.exe'
    $launcher = if (Test-Path -LiteralPath $localNode) { $localNode } else { (Get-Command node -ErrorAction Stop).Source }
    $prefixArguments = @($entry)
}
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
function TomlString([string]$value) { ConvertTo-Json -InputObject $value -Compress }
$prompt = @'
This is an operator-controlled MCP host reachability check, not an independent
engineering evaluation. Do not edit files, start shell processes, run tests,
build, reconcile manually, or change configuration. Normal source reconciliation
performed by the authorized knowledge MCP tool is permitted.

Determine whether the host advertises pcg_knowledge_v1. If it does not, report
the exact missing capability and stop; do not substitute shell or manual stdio.
Use that host-mediated MCP tool with the explicit workspace root below.
Normal production request syntax: every request requires workspaceRoot and mode.
Search uses text, not query. Exact acquisition uses id. Optional role/label are
component predicates, edgeKind/direction apply to related, maxNodes/maxEdges to
context, and path/sourceSha256 to source. Do not invent argument names. These
are generic interface instructions, not supplied identities or search answers.
Report which argument fields the host actually makes visible to you. If they
differ from these instructions, disclose the difference. If the host permits
the complete request, use workspaceRoot, mode and text for your search. If it
prevents those fields, stop and report the host limitation; do not bypass it.
Search for architectural pipeline ordering knowledge using a query you choose.
Follow returned references to acquire one exact unit, its typed relationships,
its complete declared context, and its original source: exercise search, get,
related, context and source. Use returned identities and reference fields.
If the first selected unit has no typed edges, discover an applicable related
authority unit and inspect its declared relationships; do not invent an edge.
Inspect currentness, completeness, diagnostics and match origins. Report each
actual call and result, source provenance and any refusal. Do not claim general
adoption, semantic sufficiency or improved reasoning from this smoke check.
Workspace root:
'@ + "`n$root"
if ($TaskPromptFile) {
    $prompt = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $TaskPromptFile).Path, $utf8) + "`nWorkspace root:`n$root"
}
$arguments = @(
    '--no-daemon', 'exec', '--ignore-user-config', '--ephemeral',
    '--sandbox', 'read-only', '-C', $root, '--model', 'gpt-6.1-sol',
    '--disable', 'memories',
    '-c', 'memories.use_memories=false',
    '-c', 'memories.generate_memories=false',
    '-c', "mcp_servers.clean_ctx.command=$(TomlString $binary)",
    '-c', 'mcp_servers.clean_ctx.args=[]',
    '-c', "mcp_servers.clean_ctx.cwd=$(TomlString $launch)",
    '-c', 'mcp_servers.clean_ctx.enabled=true',
    '-c', 'mcp_servers.clean_ctx.required=true',
    '-c', 'mcp_servers.clean_ctx.default_tools_approval_mode="approve"',
    '-c', "mcp_servers.clean_ctx.env.CLEAN_CTX_PROJECT_ROOT=$(TomlString $root)",
    '-c', "mcp_servers.clean_ctx.env.CLEAN_CTX_PCG_DOCUMENT_PROFILE=$(TomlString $profile)",
    '--json', '-o', (Join-Path $output 'agent-response.md'), $prompt
)
$manifest = [ordered]@{
    purpose = $Purpose
    executable = $binary; executable_sha256 = $expectedBinary
    profile = $profile; profile_sha256 = $expectedProfile
    host_command = $codex; arguments = $arguments
    native_launcher = $launcher; native_prefix_arguments = $prefixArguments
    capture = 'native stdout/stderr bytes; no PowerShell string round trip'
    security = 'read-only agent shell; existing MCP may write its derived catalog'
    normal_configuration_modified = $false
    status = 'STARTING; inspect actual trace before claiming advertisement or invocation'
}
[IO.File]::WriteAllText((Join-Path $output 'manifest.json'), ($manifest | ConvertTo-Json -Depth 6), $utf8)
[IO.File]::WriteAllText((Join-Path $output 'prompt.txt'), $prompt, $utf8)
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = $launcher
$start.WorkingDirectory = $root
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
foreach ($argument in ($prefixArguments + $arguments)) { $start.ArgumentList.Add($argument) }
$process = [Diagnostics.Process]::new()
$process.StartInfo = $start
$stdout = [IO.File]::Open((Join-Path $output 'host-events.jsonl'), [IO.FileMode]::CreateNew)
$stderr = [IO.File]::Open((Join-Path $output 'stderr.txt'), [IO.FileMode]::CreateNew)
try {
    if (!$process.Start()) { throw 'Codex launch failed.' }
    $stdoutCopy = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
    $stderrCopy = $process.StandardError.BaseStream.CopyToAsync($stderr)
    $process.WaitForExit()
    $stdoutCopy.GetAwaiter().GetResult()
    $stderrCopy.GetAwaiter().GetResult()
    $code = $process.ExitCode
} finally {
    $stdout.Dispose()
    $stderr.Dispose()
    $process.Dispose()
}
[IO.File]::WriteAllText((Join-Path $output 'exit-code.txt'), [string]$code, $utf8)
Write-Output "Host smoke exit code: $code; evidence: $output"
if ($code -ne 0) { throw 'Host smoke did not complete. Preserve all output; do not retry silently.' }
