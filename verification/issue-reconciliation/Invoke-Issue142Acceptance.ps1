param(
    [Parameter(Mandatory)][string]$BinaryPath,
    [ValidateRange(30, 600)][int]$CaseTimeoutSeconds = 240
)

# Consolidated Issue #142 live MCP acceptance. Builds nothing and starts no CBM provider.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinaryPath = (Resolve-Path $BinaryPath).Path
$runRoot = Join-Path $repo ('target/issue-142-live-verification/' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$primary = Join-Path $runRoot 'primary'
$additional = Join-Path $runRoot 'additional'
$database = Join-Path $runRoot 'issue-142.db'
$additionalTs = Join-Path $additional 'Additional.ts'
$consumerCs = Join-Path $additional 'Consumer.cs'
$sessions = [Collections.Generic.List[object]]::new()
$verdicts = [Collections.Generic.List[object]]::new()
$rpcId = 0
$deadline = [DateTime]::UtcNow.AddSeconds($CaseTimeoutSeconds)
. (Join-Path $PSScriptRoot '../cbm/scripts/PhaseBSession.ps1')

New-Item -ItemType Directory -Force $primary, $additional | Out-Null
& git -C $primary init --quiet
if ($LASTEXITCODE -ne 0) { throw 'Primary fixture Git root failed.' }
& git -C $additional init --quiet
if ($LASTEXITCODE -ne 0) { throw 'Additional fixture Git root failed.' }
$config = @{
    additional_roots = @($additional)
    auto_delta = $false
    cbm = @{ enabled = $false; auto_launch = $false }
    persistence = @{ enabled = $true; auto_save = $true; db_path = $database }
} | ConvertTo-Json -Depth 20
Write-BText (Join-Path $primary '.clean-ctx.json') $config
Write-BText (Join-Path $primary 'Primary.ts') "export class PrimaryEntity {}`n"
Write-BText (Join-Path $primary 'Permutation.ts') "export class PermutationEntity {}`n"
Write-BText (Join-Path $primary 'Target.cs') @'
public class Target {
    public void Run() { }
}
'@
Write-BText $additionalTs "export class AdditionalEntity {}`n"
Write-BText $consumerCs @'
public class Consumer {
    public void Go(Target target) { target.Run(); }
}
'@
Save-B 'provenance' @{
    binary = $BinaryPath
    sha256 = (Get-FileHash $BinaryPath).Hash
    provider_enabled = $false
    primary_root = $primary
    additional_root = $additional
    contract = 'Issue 142 public acceptance'
}

function Tool($Session, [string]$Name, [hashtable]$Arguments, [string]$Evidence) {
    $result = Get-BResult (Invoke-BTool $Session $Name $Arguments)
    Save-B $Evidence $result
    return $result
}

function Query($Session, [hashtable]$Arguments, [string]$Evidence) {
    $result = Tool $Session 'workspace_query' $Arguments $Evidence
    Assert-B ([bool]$result.PSObject.Properties['structuredContent']) 'workspace_query omitted structuredContent.'
    return $result.structuredContent
}

function Entity-Signature($Result) {
    return @($Result.entities | ForEach-Object {
        "$($_.domain)|$($_.entity_type)|$($_.name)|$($_.file)"
    } | Sort-Object)
}

function Assert-ExactFileCoverage($Result, [string]$Case) {
    Assert-B ($Result.coverage.status -eq 'authoritative_complete') "$Case did not earn authoritative completeness."
    Assert-B ($Result.coverage.source_complete -eq $true) "$Case did not project source_complete=true."
    Assert-B ($Result.coverage.result_semantics -eq 'exact') "$Case did not project exact result semantics."
    Assert-B ($Result.coverage.omitted_possible -eq $false) "$Case still permits omitted results."
    Assert-B (@($Result.coverage.evidence_requirements).Count -gt 0) "$Case omitted its satisfied evidence requirements."
}

function Result-ById($Batch, [string]$Id) {
    $matches = @($Batch.results | Where-Object id -eq $Id)
    Assert-B ($matches.Count -eq 1) "Batch did not contain exactly one '$Id' result."
    Assert-B ($matches[0].status -eq 'ok') "Batch item '$Id' failed."
    return $matches[0].result
}

function Stable-QueryProjection($Result) {
    return [ordered]@{
        count = $Result.count
        entities = @(Entity-Signature $Result)
        coverage = [ordered]@{
            status = $Result.coverage.status
            source_complete = $Result.coverage.source_complete
            result_semantics = $Result.coverage.result_semantics
            omitted_possible = $Result.coverage.omitted_possible
            evidence_requirements = @($Result.coverage.evidence_requirements | Sort-Object)
        }
    }
}

function Assert-SameProjection($Left, $Right, [string]$Message) {
    $leftJson = Stable-QueryProjection $Left | ConvertTo-Json -Depth 30 -Compress
    $rightJson = Stable-QueryProjection $Right | ConvertTo-Json -Depth 30 -Compress
    Assert-B ($leftJson -ceq $rightJson) "$Message Left=$leftJson Right=$rightJson"
}

function Stop-OwnedSession($Session) {
    Stop-BSession $Session
    $null = $sessions.Remove($Session)
}

try {
    $session = Start-BSession 'issue-142-cold' $BinaryPath $primary
    $registry = Get-BResult (Invoke-BRpc $session 'tools/list' @{})
    foreach ($tool in @('provide_code_context', 'workspace_query', 'refresh_workspace', 'context_stats')) {
        Assert-B ($tool -in $registry.tools.name) "Registered tool absent: $tool"
    }

    Invoke-BCase '142-A' 'cold-repeat-and-unrelated-warm-equivalence' {
        $args = @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = 'Primary.ts'; fidelity = 'high' }
        $cold = Query $session $args 'cold-primary'
        Assert-ExactFileCoverage $cold 'cold entities_in_file'
        $null = Query $session @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = $additionalTs; fidelity = 'high' } 'unrelated-warm'
        $warm = Query $session $args 'warm-primary'
        Assert-ExactFileCoverage $warm 'warm entities_in_file'
        Assert-SameProjection $cold $warm 'Unrelated warming changed authoritative file truth.'
    }

    Invoke-BCase '142-A' 'batch-permutation-shares-one-authoritative-snapshot' {
        $first = Query $session @{
            workspaceRoot = $primary
            queries = @(
                @{ id = 'find'; type = 'find_entities'; name = 'PermutationEntity' },
                @{ id = 'file'; type = 'entities_in_file'; file_path = 'Permutation.ts'; fidelity = 'high' }
            )
        } 'batch-find-first'
        $second = Query $session @{
            workspaceRoot = $primary
            queries = @(
                @{ id = 'file'; type = 'entities_in_file'; file_path = 'Permutation.ts'; fidelity = 'high' },
                @{ id = 'find'; type = 'find_entities'; name = 'PermutationEntity' }
            )
        } 'batch-file-first'
        $firstFile = Result-ById $first 'file'
        $secondFile = Result-ById $second 'file'
        Assert-ExactFileCoverage $firstFile 'find-first file item'
        Assert-ExactFileCoverage $secondFile 'file-first file item'
        Assert-SameProjection $firstFile $secondFile 'Batch order changed file-item truth.'
        $firstFind = Result-ById $first 'find'
        $secondFind = Result-ById $second 'find'
        Assert-B (($firstFind.entities | ConvertTo-Json -Depth 30 -Compress) -ceq ($secondFind.entities | ConvertTo-Json -Depth 30 -Compress)) 'Batch order changed find_entities truth.'
    }

    Invoke-BCase '142-A' 'lower-bound-family-remains-lower-bound' {
        $lower = Query $session @{
            workspaceRoot = $primary
            type = 'reverse_edges'
            domain = 'builtin'
            entity_type = 'DefinitelyUnsupported'
            name = 'DefinitelyMissing'
        } 'lower-bound-control'
        Assert-B ($lower.coverage.source_complete -eq $false) 'Lower-bound family falsely claimed source completeness.'
        Assert-B ($lower.coverage.result_semantics -eq 'lower_bound') 'Lower-bound family was promoted to exact.'
        Assert-B ($lower.coverage.omitted_possible -eq $true) 'Lower-bound family denied possible omissions.'
    }

    Invoke-BCase '142-B' 'requested-root-isolation-and-dependent-reconciliation' {
        $null = Query $session @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = 'Target.cs'; fidelity = 'high' } 'target-authority'
        $null = Query $session @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = $consumerCs; fidelity = 'high' } 'consumer-authority'
        $additionalBefore = Query $session @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = $additionalTs; fidelity = 'high' } 'additional-before-refresh'
        Write-BText (Join-Path $primary 'Target.cs') "public class Target { public void Replacement() { } }`n"
        $refresh = Tool $session 'refresh_workspace' @{ workspaceRoot = $primary; scope = 'requested_root' } 'requested-root-refresh'
        $summary = $refresh.structuredContent
        Assert-B ($summary.scope -eq 'requested_root') 'Refresh did not preserve requested-root scope.'
        Assert-B ($summary.roots_refreshed -eq 1) 'Requested-root refresh expanded to configured roots.'
        Assert-B (@($summary.workspace_roots).Count -eq 1) 'Requested-root response reported unrelated roots.'
        Assert-B ($summary.dependent_owners_retracted -gt 0) 'Proven cross-root C# dependent was not retracted.'
        $additionalAfter = Query $session @{ workspaceRoot = $primary; type = 'entities_in_file'; file_path = $additionalTs; fidelity = 'high' } 'additional-after-refresh'
        Assert-SameProjection $additionalBefore $additionalAfter 'Requested-root refresh changed unrelated additional-root truth.'
    }

    Invoke-BCase '142-C' 'restart-separates-session-persisted-and-combined-accounting' {
        $null = Tool $session 'provide_code_context' @{ workspaceRoot = $primary; filePath = 'Primary.ts'; fidelity = 'high' } 'persisted-context'
        $before = Tool $session 'context_stats' @{ format = 'json' } 'stats-before-restart'
        $beforeJson = $before.content[0].text | ConvertFrom-Json
        Assert-B ($beforeJson.accounting.session.session.total_files -gt 0) 'Active session did not record compression work.'
        Stop-OwnedSession $session
        $script:session = Start-BSession 'issue-142-restart' $BinaryPath $primary
        $after = Tool $session 'context_stats' @{ format = 'json' } 'stats-after-restart'
        $afterJson = $after.content[0].text | ConvertFrom-Json
        Assert-B ($afterJson.accounting.session.session.total_files -eq 0) 'Persisted totals leaked into the new session scope.'
        Assert-B ($afterJson.accounting.persisted.session.total_files -gt 0) 'Restart omitted persisted history.'
        Assert-B ($afterJson.accounting.combined.session.total_files -ge $afterJson.accounting.persisted.session.total_files) 'Combined accounting is smaller than persisted accounting.'
        Assert-B ($afterJson.accounting.compatibility_scope -eq 'combined') 'Compatibility aggregate scope is ambiguous.'
    }
} finally {
    foreach ($openSession in @($sessions)) {
        try { Stop-BSession $openSession }
        catch { $verdicts.Add(@{ phase = 'cleanup'; case = $openSession.Name; result = 'FAIL'; evidence = $_.Exception.Message }) }
    }
    Save-B 'verdicts' @($verdicts.ToArray())
    Write-Host "Issue #142 live evidence: $runRoot"
}
if (@($verdicts | Where-Object result -eq 'FAIL').Count) { exit 1 }
