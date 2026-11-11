param(
    [Parameter(Mandatory)][string]$Binary,
    [string]$OutputRoot = 'target/pcg-175/documents-live'
)
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$outputPath = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $outputPath) { throw 'Use a fresh output root; retained evidence is never overwritten' }
$null = New-Item -ItemType Directory -Path (Join-Path $outputPath 'workspace/docs') -Force
$workspace = Join-Path $outputPath 'workspace'
$profilePath = Join-Path $outputPath 'profile.json'
$hash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
function Write-Json([string]$Path, $Value) {
    [IO.File]::WriteAllText($Path, ($Value | ConvertTo-Json -Depth 64), $utf8)
}
Write-Json $profilePath @{
    schema_version=1; workspace_root=$workspace; database_path=(Join-Path $outputPath 'documents.sqlite')
    document_roots=@('docs'); max_files=128; max_nodes=64; max_edges=128; max_bytes=1048576; token_budget=32768
}
Write-Json (Join-Path $workspace '.clean-ctx.json') @{
    cbm=@{enabled=$false}; persistence=@{enabled=$false}; cache=@{enabled=$false}; proxy=@{auto_start=$false}
}
function Write-Document([string]$Id, [string]$Status, [string]$Relation = '', [string]$Target = '') {
    $text = "# Architectural evidence`n`n``````architecture-graph`nnode: $Id`nkind: MISSION`nstatus: $Status`ntitle: Contract $Id`ntrack: REPOSITORY`nartifact: docs/$Id.md`n"
    if ($Relation) { $text += "${Relation}:`n  - $Target`n" }
    $text += "```````n"
    [IO.File]::WriteAllText((Join-Path $workspace "docs/$Id.md"), $text, $utf8)
}
function Start-Session {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$binaryPath; $info.WorkingDirectory=$workspace; $info.UseShellExecute=$false
    $info.CreateNoWindow=$true; $info.RedirectStandardInput=$true; $info.RedirectStandardOutput=$true; $info.RedirectStandardError=$true
    $info.StandardInputEncoding=$utf8; $info.StandardOutputEncoding=$utf8; $info.StandardErrorEncoding=$utf8
    $info.Environment['CLEAN_CTX_PROJECT_ROOT']=$workspace
    $info.Environment['CLEAN_CTX_PCG_DOCUMENT_PROFILE']=$profilePath
    $null=$info.Environment.Remove('CLEAN_CTX_PCG_PROFILE')
    $p=[Diagnostics.Process]::new();$p.StartInfo=$info
    if (-not $p.Start()) {throw 'Server launch failed'}
    return @{process=$p;stderr=$p.StandardError.ReadToEndAsync()}
}
function Rpc($Session, [int]$Id, [string]$Method, $Params) {
    $p=$Session.process
    $request=@{jsonrpc='2.0';id=$Id;method=$Method;params=$Params}
    $p.StandardInput.WriteLine(($request | ConvertTo-Json -Depth 64 -Compress))
    $task=$p.StandardOutput.ReadLineAsync()
    if (-not $task.Wait(30000)) {throw 'Bounded MCP read timed out'}
    $line=$task.GetAwaiter().GetResult()
    [IO.File]::WriteAllText((Join-Path $outputPath "$Id.response.json"), $line, $utf8)
    Write-Json (Join-Path $outputPath "$Id.request.json") $request
    $response=$line | ConvertFrom-Json -AsHashtable
    if ($response.ContainsKey('error')) {throw "MCP error: $($response.error.message)"}
    return $response
}
function Query($Session,[int]$Id,[string]$Mode,[string]$Node='') {
    $a=@{workspaceRoot=$workspace;mode=$Mode;tokenizer='o200k';tokenBudget=32768}
    if ($Node) {$a.id=$Node}
    $r=Rpc $Session $Id 'tools/call' @{name='pcg_documents_v1';arguments=$a}
    $payload=$r.result.content[0].text | ConvertFrom-Json -AsHashtable
    Write-Json (Join-Path $outputPath "$Id.payload.json") $payload
    return $payload
}
function Initialize($Session,[int]$Id) {
    $null=Rpc $Session $Id 'initialize' @{protocolVersion='2025-11-25';capabilities=@{};clientInfo=@{name='pcg-documents-live';version='1'}}
    $Session.process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}')
}
function Stop-Session($Session,[string]$Label) {
    $p=$Session.process;$p.StandardInput.Close()
    if (-not $p.WaitForExit(10000)) {$p.Kill();$p.WaitForExit()}
    [IO.File]::WriteAllText((Join-Path $outputPath "$Label.stderr.txt"),$Session.stderr.GetAwaiter().GetResult(),$utf8)
    if ($p.ExitCode -ne 0) {throw "Server exited $($p.ExitCode)"}
    $p.Dispose()
}
$session=$null;$status='failed';$failure=$null
try {
    $session=Start-Session;Initialize $session 0
    $empty=Query $session 1 'search'
    if ($empty.result_state -cne 'ready' -or $empty.documents.Count -ne 0) {throw 'Initial discovery failed'}
    Write-Document 'DOC-042' 'FROZEN'
    Write-Document 'DOC-068' 'FROZEN' 'depends_on' 'DOC-042'
    Write-Document 'DOC-081' 'INVESTIGATING' 'implements' 'DOC-068'
    $context=Query $session 2 'context' 'DOC-081'
    if (-not $context.context.complete -or $context.documents.Count -ne 3) {throw 'Transitive context failed'}
    $path=@($context.context.paths | Where-Object {$_.node -ceq 'DOC-042'})[0]
    if ($path.steps.Count -ne 2) {throw 'Canonical constraint path missing'}
    Write-Document 'DOC-081' 'FROZEN' 'implements' 'DOC-068'
    $current=Query $session 3 'get' 'DOC-081'
    if ($current.documents[0].declaration.status -cne 'FROZEN') {throw 'Revision not reconciled'}
    Write-Document 'DOC-081' 'TYPO' 'implements' 'DOC-068'
    $invalid=Query $session 4 'get' 'DOC-081'
    if ($invalid.result_state -cne 'blocked' -or $invalid.documents.Count -ne 0) {throw 'Invalid revision leaked authority'}
    $history=Query $session 5 'history' 'DOC-081'
    if ($history.catalog_current -or $history.historical_documents.Count -ne 2) {throw 'Historical continuity missing'}
    Write-Document 'DOC-081' 'FROZEN' 'implements' 'DOC-068'
    Stop-Session $session 'first';$session=$null
    $session=Start-Session;Initialize $session 10
    $restart=Query $session 11 'context' 'DOC-081'
    if ($restart.generation -cne $current.generation -or $restart.documents.Count -ne 3) {throw 'Restart changed admitted graph'}
    Stop-Session $session 'restart';$session=$null
    if ((Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $hash) {throw 'Binary changed'}
    $status='succeeded'
} catch {$failure=$_.Exception.Message;throw}
finally {
    if ($session) {try {Stop-Session $session 'interrupted'} catch {}}
    Write-Json (Join-Path $outputPath 'receipt.json') @{
        status=$status;error=$failure;binary=$binaryPath;binary_sha256=$hash
        workflow='production stdio document creation/revision/invalid-current/history/transitive-context/restart'
    }
}
Write-Output "Operator receipt: $(Join-Path $outputPath 'receipt.json')"
