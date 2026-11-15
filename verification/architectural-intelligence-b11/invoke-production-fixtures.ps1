param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$OutputRoot
)
# User-owned production verification. Authorization is simulated ONLY in a fresh
# synthetic workspace under target. Never authorizes the actual repository.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
$allowed = Join-Path $root 'target/pcg-b11'
if (-not $output.StartsWith($allowed + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Synthetic authorization fixtures must remain under target/pcg-b11.'
}
if (Test-Path -LiteralPath $output) { throw 'Use a fresh fixture output directory.' }
New-Item -ItemType Directory -Path $output | Out-Null
$fixture = Join-Path $output 'synthetic-workspace'
$docs = Join-Path $fixture 'docs'
New-Item -ItemType Directory -Path $docs | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$profile = Join-Path $fixture 'profile.json'
$database = Join-Path $fixture 'documents.sqlite'
$helper = Join-Path $PSScriptRoot 'capture-production.py'
$driver = Join-Path $root 'verification/pcg-175/invoke-workspace-documents.ps1'
$launch = Join-Path $root '.clean-ctx/pcg-documents/authoring'
$observations = [Collections.Generic.List[object]]::new()
function Save([string]$Name, $Value) {
    [IO.File]::WriteAllText((Join-Path $output $Name), (ConvertTo-Json -InputObject $Value -Depth 100), $utf8)
}
function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
    $observations.Add(@{passed=$true;assertion=$Message})
    Save 'assertions.json' $observations.ToArray()
}
function Query([string]$Step, [hashtable]$Arguments, [string]$Tool = 'pcg_knowledge_v1') {
    $Arguments.workspaceRoot = $fixture
    if ($Tool -ceq 'pcg_documents_v1' -and -not $Arguments.ContainsKey('tokenizer')) {
        $Arguments.tokenizer = 'o200k'
    }
    Save "$Step.requests.json" @($Arguments)
    & $driver -WorkspaceRoot $fixture -Profile $profile -Binary $Binary `
        -Requests (Join-Path $output "$Step.requests.json") -OutputRoot (Join-Path $output "$Step-rpc") `
        -LaunchDirectory $launch -ToolName $Tool | Out-Null
    return (Get-Content -LiteralPath (Join-Path $output "$Step-rpc/2.payload.json") -Raw | ConvertFrom-Json -AsHashtable)
}
function Snapshot([string]$Step) {
    $path = Join-Path $output "$Step.catalog.json"
    & python $helper snapshot $database $path | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Fixture snapshot failed: $Step" }
    return (Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable)
}
function Operator([string]$Step, [string]$Action, [string]$Digest, [switch]$Reject) {
    # The profile is constructed here, not accepted from caller/candidate data.
    $settings = Get-Content -LiteralPath $profile -Raw | ConvertFrom-Json
    Require ($settings.workspace_root -ceq $fixture) 'Operator simulation profile is the synthetic workspace'
    $arguments = @('pcg','documents',$Action,'--profile',$profile)
    if ($Digest) { $arguments += @('--digest',$Digest) }
    $stderr = Join-Path $output "$Step.stderr.log"
    $lines = & $Binary @arguments 2> $stderr
    $code = $LASTEXITCODE
    [IO.File]::WriteAllText((Join-Path $output "$Step.stdout.json"), ($lines -join "`n"), $utf8)
    Save "$Step.exit.json" @{exit_code=$code;scope='synthetic procedural operator simulation; no real-workspace authorization'}
    if ($Reject) {
        Require ($code -ne 0 -and (Get-Content -LiteralPath $stderr -Raw) -match 'governance_authorization_descriptor_mismatch') 'Substituted or stale digest is explicitly rejected'
        return
    }
    Require ($code -eq 0) "Operator fixture command succeeds: $Step"
    return (($lines -join "`n") | ConvertFrom-Json -AsHashtable)
}
function ApproveFixture([string]$Step) {
    $before = Snapshot "$Step-before"
    $description = Operator "$Step-describe" 'describe' ''
    Require ($description.accepted_publication -eq $false -and $description.digest.Length -eq 64) 'Describe yields an exact nonpublishing descriptor'
    $grant = Operator "$Step-authorize" 'authorize' $description.digest
    Require ($grant.authorization_recorded -eq $true -and $grant.accepted_publication -eq $false) 'Synthetic grant records authorization without graph publication'
    Require ($grant.authorization_assurance -ceq 'procedural_operator_action_not_authenticated_identity') 'Authorization does not claim authenticated identity'
    $after = Snapshot "$Step-after"
    Require ($after.generation -ceq $before.generation) 'Describe and authorize preserve accepted graph head'
    return $description.digest
}
function Blocked([hashtable]$Payload, [string]$Generation, [string]$Code) {
    Require ($Payload.result_state -ceq 'blocked' -and -not $Payload.catalog_current) 'Invalid candidate is blocked and not current'
    Require ($Payload.generation -ceq $Generation) 'Rejection preserves accepted generation'
    Require (@($Payload.diagnostics | Where-Object code -CEQ $Code).Count -gt 0) "Expected rejection is exposed: $Code"
}

Save 'scope.json' @{workspace=$fixture;binary_sha256=(Get-FileHash -LiteralPath $Binary).Hash.ToLowerInvariant();real_workspace_authorization='never submitted';independent_evaluation='not executed'}
[IO.File]::WriteAllText($profile, (@{schema_version=1;workspace_root=$fixture;database_path=$database;document_roots=@('docs');max_files=128;max_nodes=128;max_edges=512;max_bytes=1048576;token_budget=1} | ConvertTo-Json), $utf8)
$rule = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'fixtures/rule.md'), $utf8)
$authority = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'fixtures/authority.md'), $utf8)
$rulePath = Join-Path $docs 'rule.md'
[IO.File]::WriteAllText($rulePath, $rule, $utf8)
[IO.File]::WriteAllText((Join-Path $docs 'authority.md'), $authority, $utf8)
$baseline = Query 'baseline' @{mode='get';id='DOC-001'} 'pcg_documents_v1'
Require ($baseline.result_state -ceq 'ready' -and $baseline.catalog_current) 'Legacy document retrieval is ready with retired token_budget=1'
$governance = Join-Path $docs 'architecture/knowledge-governance'
New-Item -ItemType Directory -Path $governance | Out-Null
[IO.File]::WriteAllText((Join-Path $governance 'corpus.csv'), "source_path,expected_ids`ndocs/rule.md,DOC-001`ndocs/authority.md,AUTH.PCG_DOCUMENTS`n", $utf8)
[IO.File]::WriteAllText((Join-Path $governance 'protected.csv'), "id,role,label,selector`nDOC-001,statement,,POLICY`n", $utf8)
$bootstrap = Query 'bootstrap-blocked' @{mode='get';id='DOC-001'}
Blocked $bootstrap $baseline.generation 'E_GOVERNANCE_BOOTSTRAP_REQUIRED'
$null = ApproveFixture 'bootstrap'
$active = Query 'bootstrap-ready' @{mode='get';id='DOC-001'}
Require ($active.result_state -ceq 'ready' -and $active.complete) 'Authorized synthetic bootstrap publishes a complete current unit'
$activeState = Snapshot 'active'
Require (@($activeState.governance_publications).Count -eq 1) 'Accepted governance snapshot is published with the graph'
$ordinaryText = "Ordinary prefix outside protected content.`n" + $rule
[IO.File]::WriteAllText($rulePath, $ordinaryText, $utf8)
$ordinary = Query 'ordinary-edit' @{mode='get';id='DOC-001'}
Require ($ordinary.result_state -ceq 'ready' -and $ordinary.generation -cne $active.generation) 'Ordinary outside edit publishes without another authorization'
$ordinaryState = Snapshot 'ordinary'
Require ($ordinaryState.governance_publications[-1][1] -ceq $activeState.governance_publications[-1][1]) 'Outside edit preserves exact protected state digest'
$changed = $ordinaryText.Replace('Exact controlling rule.','Exact controlling rule!')
[IO.File]::WriteAllText($rulePath, $changed, $utf8)
$protected = Query 'protected-blocked' @{mode='get';id='DOC-001'}
Blocked $protected $ordinary.generation 'E_GOVERNANCE_AUTHORIZATION_REQUIRED'
$historicalRefs = @($ordinary.units[0].components | ForEach-Object {
    $reference = (ConvertTo-Json -InputObject $_.reference -Depth 100) | ConvertFrom-Json -AsHashtable
    $reference.historical = $true
    $reference
})
$historical = Query 'accepted-history' @{mode='acquire';references=$historicalRefs}
$texts = @($historical.acquisitions | ForEach-Object { $_.components } | ForEach-Object { $_.text })
Require ($historical.freshness -ceq 'historical_snapshot' -and -not $historical.catalog_current) 'Historical acquisition never claims currentness for a rejected candidate'
Require (($texts -join "`n").Contains('Exact controlling rule.')) 'Exact accepted rule remains retrievable during rejection'
$digest = ApproveFixture 'protected-change'
$accepted = Query 'protected-ready' @{mode='get';id='DOC-001'}
Require ($accepted.result_state -ceq 'ready' -and $accepted.catalog_current) 'Reviewed protected change publishes current knowledge'
[IO.File]::WriteAllText($rulePath, $changed.Replace('Exact controlling rule!','Exact controlling rule?'), $utf8)
$null = Operator 'replay-refused' 'authorize' $digest -Reject
Blocked (Query 'replay-blocked' @{mode='get';id='DOC-001'}) $accepted.generation 'E_GOVERNANCE_AUTHORIZATION_REQUIRED'
[IO.File]::WriteAllText($rulePath, $changed, $utf8)
$restored = Query 'restored' @{mode='get';id='DOC-001'}
Require ($restored.result_state -ceq 'ready' -and $restored.generation -ceq $accepted.generation) 'Exact correction restores accepted state without a reusable future grant'
[IO.File]::WriteAllText($rulePath, 'Ordinary text cannot discharge registered membership.', $utf8)
Blocked (Query 'declaration-missing' @{mode='get';id='DOC-001'}) $accepted.generation 'E_GOVERNED_DECLARATION_MISSING'
[IO.File]::WriteAllText($rulePath, $changed.Replace('track: FOUNDATION', "title: Duplicate`ntrack: FOUNDATION"), $utf8)
$malformed = Query 'malformed' @{mode='get';id='DOC-001'} 'pcg_documents_v1'
Blocked $malformed $accepted.generation 'E_MALFORMED_DECLARATION'
$diagnostic = @($malformed.diagnostics | Where-Object code -CEQ 'E_MALFORMED_DECLARATION')[0]
Require ($diagnostic.location.field -ceq 'title') 'Serialized diagnostic identifies the offending field'
Require (@($diagnostic.governing_authority.acquire_request.references | Where-Object role -CEQ 'constraint').Count -gt 0) 'Production diagnostic includes its independently owned accepted constraint'
$ruleEvidence = Query 'governing-rule' $diagnostic.governing_authority.acquire_request
Require ($ruleEvidence.freshness -ceq 'historical_snapshot') 'Governing rule acquisition preserves accepted historical standing'
[IO.File]::WriteAllText($rulePath, $changed, $utf8)
$final = Query 'final-ready' @{mode='get';id='DOC-001'}
Require ($final.result_state -ceq 'ready' -and $final.generation -ceq $accepted.generation) 'All rejected fixture candidates leave accepted publication intact'
$finalState = Snapshot 'final'
Require (@($finalState.authorization_grants).Count -eq 2) 'Only bootstrap and the exact protected change recorded synthetic grants'
Save 'result.json' @{result='passed';assertions=$observations.Count;final_generation=$final.generation;scope='operator production CLI/MCP fixtures, not Rust regression or CI evidence';actual_repository_activation='not performed'}
Write-Output "Production governance fixtures complete: $output"
