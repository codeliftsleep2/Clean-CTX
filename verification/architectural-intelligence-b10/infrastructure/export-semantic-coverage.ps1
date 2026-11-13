param([Parameter(Mandatory)][string]$WorkspaceRoot)
# Read-only evidence analysis. No source declarations or graph rows are written.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$reports = Join-Path $root 'verification/architectural-intelligence-b10/operator'
$baseline = Get-Content (Join-Path $root 'target/pcg-b10/baseline.json') -Raw | ConvertFrom-Json
$migration = @{}
Import-Csv (Join-Path $root 'docs/architecture/knowledge-normalization/B9_IDENTITY_MIGRATION_DISPOSITIONS.csv') | ForEach-Object { $migration[$_.identity] = $_ }
$probes = Import-Csv (Join-Path $reports 'B10_DISCOVERY_CALLS.csv')
$roles = 'statement','subject','scope','outcome','rationale','alternative','rejected_alternative','rejection_reason','consequence','constraint','verification_obligation','observation','counterexample','qualification','historical_applicability','evidence'
$tagged = foreach ($node in $baseline.nodes) {
    foreach ($component in $node.components | Where-Object role -ne statement) {
        [pscustomobject]@{id=$node.id;component=$component}
    }
}
$gateRows = [Collections.Generic.List[object]]::new()
$coverage = foreach ($node in $baseline.nodes) {
    $statement = @($node.components | Where-Object role -eq statement)[0]
    $row = [ordered]@{
        id=$node.id;scope=$node.scope;kind=$node.declaration.kind;lifecycle=$node.declaration.status
        title=$node.declaration.title;source=$node.path;revision=$node.revision
        statement_path=$statement.path;statement_start=$statement.start;statement_end=$statement.end
        statement_content_sha256=$statement.content_sha256
        migration_disposition=$migration[$node.id].disposition
        exact_components=$node.components.Count
        source_purpose_excerpt=((($statement.text -split '(?:\r?\n){2,}' | Select-Object -First 2) -join ' / ') -replace '\r?\n',' ')
        captured_in_probes=(@($probes | Where-Object { $_.mode -in @('get','context') -and $node.id -in ($_.matched_or_acquired_identities -split ';') } | ForEach-Object {$_.probe} | Sort-Object -Unique) -join ';')
        scope_assessment='exact declared subject retained; broader applicability is an engineering judgment'
        semantic_limit='role absence is not semantic absence; no exhaustive prose/approval re-adjudication'
    }
    foreach ($role in $roles) {
        $explicit = @($node.components | Where-Object role -eq $role)
        if ($explicit.Count) {
            $row["role_$role"] = '1 independent authored component: ' + (@($explicit | ForEach-Object {$_.ordinal}) -join ';')
        } else {
            $contained = @($tagged | Where-Object {
                $_.component.role -eq $role -and
                $_.component.path -ceq $statement.path -and
                $_.component.source_sha256 -eq $statement.source_sha256 -and
                $_.component.start -ge $statement.start -and $_.component.end -le $statement.end
            })
            if ($contained.Count) {
                $row["role_$role"] = '2 broader statement contains authored role interval(s) from: ' + (@($contained | ForEach-Object {$_.id} | Sort-Object -Unique) -join ';')
            } elseif ($role -eq 'evidence' -and @($node.evidence).Count) {
                $row["role_$role"] = '4 structured evidence associations; source acquisition required; not an evidence-role component'
            } else {
                $row["role_$role"] = 'UNCLASSIFIED broader/source meaning; not independently tagged; absence/not-applicability not inferred'
            }
        }
    }
    if ($node.scope -eq 'invariant' -and !@($node.components | Where-Object role -eq constraint).Count) {
        $row.role_constraint = '2 authored invariant rule within statement; independently identifiable by invariant node/kind'
    }
    $gate = [regex]::Match($statement.text, '(?m)^\| \*\*Gate\*\* \| (.*) \|\r?$')
    if ($gate.Success -and $node.scope -eq 'invariant') {
        $independent = @($node.components | Where-Object role -eq verification_obligation).Count -gt 0
        if (!$independent) { $row.role_verification_obligation = '2 explicit source-authored Gate row within broader statement' }
        $gateRows.Add([pscustomobject]@{id=$node.id;path=$statement.path;statement_start=$statement.start;statement_end=$statement.end;statement_content_sha256=$statement.content_sha256;exact_gate=$gate.Groups[1].Value;independently_tagged=$independent;meaning='source-described gate/witness; not execution or a replacement for current verification policy'})
    }
    if ($node.id -in @('AUTH.FINAL_VERIFICATION','PERF-005-BOUNDED')) {
        $row.role_verification_obligation = '2 controlling verification/work-count obligations within exact statement'
    }
    if ($node.id -eq 'PERF-002-STARTUP') {
        $row.role_rationale = '2 exact DEFER outcome explains exposure/benefit/complexity; broader statement retains measured comparison'
        $row.role_rejected_alternative = '2 broader statement and REJECT outcome distinguish count removal'
    }
    if ($node.id -in @('DOC-139','DOC-165','DOC-175','DOC-178','DOC-187','DOC-196','PERF-003-OVERLOAD','PERF-006-FIFO')) {
        $row.role_outcome = '2 historical disposition in broader statement; lifecycle is not that decision outcome'
    }
    [pscustomobject]$row
}
$coverage | Export-Csv (Join-Path $reports 'B10_COMPONENT_SEMANTIC_COVERAGE.csv') -NoTypeInformation -Encoding utf8NoBOM
$gateRows | Export-Csv (Join-Path $reports 'B10_EMBEDDED_VERIFICATION_OBLIGATIONS.csv') -NoTypeInformation -Encoding utf8NoBOM
[pscustomobject]@{identities=@($coverage).Count;explicit_invariant_gate_rows=$gateRows.Count;gate_rows_without_independent_role=@($gateRows|Where-Object independently_tagged -eq $false).Count;classification_limit='unclassified is not absent and is not completed semantic verification'} | ConvertTo-Json
