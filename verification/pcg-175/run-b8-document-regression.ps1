param(
    [string]$WorkspaceRoot = (Resolve-Path '.').Path,
    [string]$Binary = 'target/debug/clean-ctx.exe',
    [string]$OutputRoot = 'target/pcg-b8/after'
)
# Operator/production verification only, never a tracked Rust regression or CI gate.
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$workspace = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh output root' }
[IO.Directory]::CreateDirectory($output) | Out-Null
$profile = Join-Path $workspace '.clean-ctx/pcg-documents/profile.json'
$driver = Join-Path $PSScriptRoot 'invoke-workspace-documents.ps1'
foreach ($name in @('regression', 'context-pairs', 'architecture')) {
    $inputFile = Join-Path $PSScriptRoot "b8-$name-queries.json"
    if ($name -eq 'context-pairs') { $inputFile = Join-Path $PSScriptRoot 'b8-context-pairs.json' }
    $requests = @([IO.File]::ReadAllText($inputFile, $utf8) | ConvertFrom-Json -AsHashtable)
    foreach ($request in $requests) { $request.workspaceRoot = $workspace }
    $requestFile = Join-Path $output "$name-queries.json"
    [IO.File]::WriteAllText($requestFile, ($requests | ConvertTo-Json -Depth 30), $utf8)
    & $driver -WorkspaceRoot $workspace -Profile $profile -Binary $Binary `
        -Requests $requestFile -OutputRoot (Join-Path $output $name) `
        -LaunchDirectory (Join-Path $workspace '.clean-ctx/pcg-documents/authoring') `
        -TokenCounter (Join-Path $workspace 'target/debug/clean-ctx-measure-helper.exe')
    $receipt = [IO.File]::ReadAllText((Join-Path $output "$name/receipt.json"), $utf8) |
        ConvertFrom-Json -AsHashtable
    foreach ($query in $receipt.queries) {
        if ($query.result_state -ne 'ready' -or -not $query.catalog_current -or $query.diagnostics.Count -ne 0) {
            throw "Incomplete production result for $($query.mode):$($query.node); preserve exact diagnostics"
        }
        if ($query.mode -eq 'context' -and -not $query.context.complete) {
            throw 'Source-only retrieval is not a complete authority closure'
        }
    }
}
& python (Join-Path $PSScriptRoot 'compare-document-context-formats.py') `
    (Join-Path $output 'context-pairs') (Join-Path $output 'reconstruction.csv')
if ($LASTEXITCODE -ne 0) { throw 'Exact independent reconstruction failed' }
Write-Output "B8 production verification and reconstruction: $output"
