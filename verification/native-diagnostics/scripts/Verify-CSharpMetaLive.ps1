#requires -Version 7.0
param(
    [string]$BinaryPath = '',
    [string]$HelperPath = '',
    [string]$PythonPath = '',
    [string]$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $RepositoryRoot 'verification/context-compression/scripts/McpSession.ps1')
$extension = if ($IsWindows) { '.exe' } else { '' }
if (-not $BinaryPath) { $BinaryPath = Join-Path $RepositoryRoot "target/debug/clean-ctx$extension" }
if (-not $HelperPath) { $HelperPath = Join-Path $RepositoryRoot "target/context-compression-verification/scripts/measure$extension" }
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$HelperPath = (Resolve-Path -LiteralPath $HelperPath).Path
$pythonPrefix = @()
if (-not $PythonPath) {
    foreach ($name in @('python3', 'python', 'py')) {
        $command = Get-Command $name -ErrorAction SilentlyContinue
        if ($command) { $PythonPath = $command.Source; if ($name -eq 'py') { $pythonPrefix = @('-3') }; break }
    }
}
if (-not $PythonPath) { throw 'Python 3 is required for the existing read-only baseline exporter' }
$run = Join-Path $RepositoryRoot ('target/csharp-meta-verification/' + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $run 'workspace'
New-Item -ItemType Directory $workspace -Force | Out-Null
foreach ($file in @('UserController.cs','AppDbContext.cs','ChatHub.cs','UserProfile.cs')) {
    Copy-Item (Join-Path $RepositoryRoot "src/test_files/dotnet/$file") $workspace
}
$database = Join-Path $run 'persistence.db'
function Save-Text([string]$Path, [string]$Text) { [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false)) }
function Save-Json([string]$Path, $Value) { Save-Text $Path ($Value | ConvertTo-Json -Depth 100) }
function Require([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
Save-Json (Join-Path $run '.clean-ctx.json') @{
    additional_roots=@($workspace); auto_delta=$false
    cbm=@{ enabled=$false; auto_launch=$false }
    persistence=@{ enabled=$true; auto_save=$true; db_path=$database }
}
Save-Text (Join-Path $workspace 'Ordinary.cs') 'public class Ordinary { public int Id { get; set; } }'
$cases = @(
    @{ file='UserController.cs'; owner='UserController'; type='Controller'; expected=@{ HasRoute=@('api/[controller]'); ControllerAction=@('GetAll','GetById','Create','Update','Delete') }; model=@('UserController','GetById') },
    @{ file='AppDbContext.cs'; owner='AppDbContext'; type='DbContext'; expected=@{ HasEntity=@('User','Order','Product') }; model=@('AppDbContext','OnModelCreating') },
    @{ file='ChatHub.cs'; owner='ChatHub'; type='Hub'; expected=@{ HubMethodTargets=@('SendMessage','SendToUser') }; model=@('ChatHub','SendMessage') },
    @{ file='UserProfile.cs'; owner='UserProfile'; type='MapperProfile'; expected=@{ MapsFrom=@('User','CreateUserRequest','UpdateUserRequest','Order'); MapsTo=@('UserDto','User','OrderDto','UserSummary') }; model=@('UserProfile','CreateMap') },
    @{ file='Ordinary.cs'; owner='Ordinary'; type=''; expected=@{}; model=@('Ordinary') }
)
function Assert-Edges($Edges, $Case, [string]$Stage) {
    $framework = @($Edges | Where-Object { $_.subject.domain -eq 'dotnet' })
    if (-not $Case.type) { Require ($framework.Count -eq 0) "$Stage fabricated a .NET framework relation"; return }
    foreach ($relation in $Case.expected.Keys) {
        foreach ($target in $Case.expected[$relation]) {
            $matching = @($framework | Where-Object { $_.relation -eq $relation -and $_.subject.name -eq $Case.owner -and $_.subject.entity_type -eq $Case.type -and $_.object.name -eq $target })
            Require ($matching.Count -gt 0) "$Stage missing $($Case.owner) --$relation--> $target"
        }
    }
}
$session = $null
$id = 900
try {
    $session = Start-CleanCtxSession $BinaryPath $run
    foreach ($case in $cases) {
        $id++
        $path = Join-Path $workspace $case.file
        $response = Invoke-CleanCtxTool $session $id 'provide_code_context' @{ filePath=$path; workspaceRoot=$workspace; fidelity='edit'; tokenizer='o200k' }
        Require ($null -eq $response.PSObject.Properties['error']) "$($case.file): provide failed"
        $directory = Join-Path $run $case.file
        New-Item -ItemType Directory $directory | Out-Null
        Save-Json (Join-Path $directory 'provide.json') $response
        $text = [string]$response.result.content[0].text
        Save-Text (Join-Path $directory 'model.txt') $text
        Require (-not $text.StartsWith('// CONTROL-FULL')) 'Codec leaked into model content'
        if ($response.result._meta.content_kind -eq 'raw_passthrough') {
            Require ($text -ceq [IO.File]::ReadAllText($path)) "$($case.file): raw source changed"
        }
        foreach ($needle in $case.model) { Require ($text.Contains($needle)) "$($case.file): model content missing $needle" }
    }
} finally { Stop-CleanCtxSession $session; $session=$null }

foreach ($case in $cases) {
    $directory = Join-Path $run $case.file
    & $PythonPath @pythonPrefix (Join-Path $RepositoryRoot 'verification/context-compression/edge-cases/scripts/Export-BinaryBaseline.py') $database (Join-Path $workspace $case.file) $directory
    Require ($LASTEXITCODE -eq 0) "$($case.file): durable export failed"
    $oracle = Join-Path $directory 'control-full.txt'
    & $HelperPath baseline-oracle (Join-Path $directory 'baseline.bin') (Join-Path $directory 'baseline.json') $oracle
    Require ($LASTEXITCODE -eq 0) "$($case.file): production Binary0x04 decoding failed"
    $payload = (([IO.File]::ReadAllText($oracle)) -replace '^[^\r\n]*\r?\n','') | ConvertFrom-Json -Depth 100
    Assert-Edges $payload.semantic_edges $case "$($case.file) durable oracle"
}

try {
    $session = Start-CleanCtxSession $BinaryPath $run
    foreach ($case in $cases | Where-Object type) {
        $id++
        $response = Invoke-CleanCtxTool $session $id 'restore_context' @{ filePath=(Join-Path $workspace $case.file); workspaceRoot=$workspace; tokenizer='o200k' }
        Save-Json (Join-Path $run "$($case.file)/restore.json") $response
        Require ($null -eq $response.PSObject.Properties['error'] -and $response.result._meta.restored -eq $true) "$($case.file): fresh restore failed"
        $id++
        $query = Invoke-CleanCtxTool $session $id 'workspace_query' @{ type='forward_edges'; domain='dotnet'; entity_type=$case.type; name=$case.owner; workspaceRoot=$workspace }
        Save-Json (Join-Path $run "$($case.file)/query.json") $query
        Require ($null -eq $query.PSObject.Properties['error']) "$($case.file): query failed"
        Assert-Edges $query.result.structuredContent.edges $case "$($case.file) restored query"
        Write-Host "PASS: $($case.file) model content, Binary0x04, and restored framework edges"
    }
} finally { Stop-CleanCtxSession $session }
Save-Json (Join-Path $run 'results.json') @{
    result='PASS'; frameworks=@('ASP.NET','EF Core','SignalR','AutoMapper'); negative_control='PASS'
    binary_sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    category='operator MCP observation; not runner execution, tracked tests, or host consumption'
}
Write-Host "PASS: C# meta-layer observations; evidence: $run"
