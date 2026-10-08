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
$run = Join-Path $RepositoryRoot ('target/angular-testing-meta-verification/' + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $run 'workspace'
New-Item -ItemType Directory $workspace -Force | Out-Null
Copy-Item (Join-Path $PSScriptRoot '../fixtures/angular/*.ts') $workspace
$database = Join-Path $run 'persistence.db'
function Save-Text([string]$Path, [string]$Text) { [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false)) }
function Save-Json([string]$Path, $Value) { Save-Text $Path ($Value | ConvertTo-Json -Depth 100) }
function Require([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
Save-Json (Join-Path $run '.clean-ctx.json') @{
    additional_roots=@($workspace); auto_delta=$false
    cbm=@{ enabled=$false; auto_launch=$false }
    persistence=@{ enabled=$true; auto_save=$true; db_path=$database }
}
Save-Text (Join-Path $workspace 'utility.spec.ts') @'
// TestBed.createComponent(IncorrectComponent) is comment evidence only.
const text = 'TestBed.createComponent(IncorrectComponent)';
describe('generic utility', () => { it('works', () => {}); });
'@
$cases = @(
    @{ file='karma-probe.spec.ts'; suite='Karma Jasmine TestBed'; spy='spyOn'; target='DiagnosticProbe' },
    @{ file='vitest-probe.spec.ts'; suite='Vitest TestBed'; spy='vi.spyOn'; target='DiagnosticProbe' },
    @{ file='utility.spec.ts'; suite='generic utility'; spy=''; target='' }
)
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
        Require ($text.Contains($case.suite)) "$($case.file): suite missing in model content"
        if ($case.spy) {
            if ($response.result._meta.content_kind -eq 'raw_passthrough') {
                Require ($text.Contains($case.spy)) "$($case.file): spy source missing in raw model content"
            } else {
                Require ($response.result._meta.content_kind -eq 'skeleton_with_verbatim_bodies') "$($case.file): unexpected model content kind"
                Require (@($text -split '\r?\n' | Where-Object { $_ -ceq 'T @spy = fixture.componentInstance.renderLabel' }).Count -eq 1) "$($case.file): exact spy target missing or duplicated in compressed model content"
            }
        }
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
    if ($case.spy) {
        $spies = @($payload.type_aliases | Where-Object { $_.alias -eq '@spy' -and $_.original_type -eq 'fixture.componentInstance.renderLabel' })
        Require ($spies.Count -eq 1) "$($case.file): durable spy marker missing or duplicated"
    }
    $tests = @($payload.semantic_edges | Where-Object relation -eq 'Tests')
    if ($case.target) {
        Require ($tests.Count -eq 1 -and $tests[0].subject.name -eq $case.file -and $tests[0].object.name -eq $case.target) "$($case.file): expected one exact Tests edge"
    } else { Require ($tests.Count -eq 0) 'Generic/comment-only spec fabricated an Angular Tests edge' }
}

try {
    $session = Start-CleanCtxSession $BinaryPath $run
    foreach ($case in $cases | Where-Object target) {
        $id++
        $response = Invoke-CleanCtxTool $session $id 'restore_context' @{ filePath=(Join-Path $workspace $case.file); workspaceRoot=$workspace; tokenizer='o200k' }
        Save-Json (Join-Path $run "$($case.file)/restore.json") $response
        Require ($null -eq $response.PSObject.Properties['error'] -and $response.result._meta.restored -eq $true) "$($case.file): fresh restore failed"
        $id++
        $query = Invoke-CleanCtxTool $session $id 'workspace_query' @{ type='forward_edges'; domain='angular'; entity_type='TestArtifact'; name=$case.file; workspaceRoot=$workspace }
        Save-Json (Join-Path $run "$($case.file)/query.json") $query
        Require ($null -eq $query.PSObject.Properties['error']) "$($case.file): query failed"
        $tests = @($query.result.structuredContent.edges | Where-Object relation -eq 'Tests')
        Require ($tests.Count -eq 1 -and $tests[0].object.name -eq $case.target) "$($case.file): restored workspace lost Tests edge"
        Write-Host "PASS: $($case.file) model content, Binary0x04, and restored Tests edge"
    }
} finally { Stop-CleanCtxSession $session }
Save-Json (Join-Path $run 'results.json') @{
    result='PASS'; suites=@('Karma/Jasmine TestBed','Vitest TestBed'); negative_control='PASS'
    binary_sha256=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash
    category='operator MCP observation; not runner execution, tracked tests, or host consumption'
}
Write-Host "PASS: Angular testing meta-layer observations; evidence: $run"
