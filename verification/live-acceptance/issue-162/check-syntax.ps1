# Syntax only: does not launch Clean-CTX, MCP sessions, or acceptance scenarios.
$ErrorActionPreference = 'Stop'
$suiteDirectory = $PSScriptRoot
$launcherPath = Join-Path (Split-Path -Parent $suiteDirectory) 'issue_162_live_acceptance.mjs'
$syntaxInputs = @($launcherPath) + @(Get-ChildItem -LiteralPath $suiteDirectory -Filter '*.mjs' -Recurse -File | Select-Object -ExpandProperty FullName)
foreach ($syntaxInput in $syntaxInputs) {
    & node --check $syntaxInput
    if ($LASTEXITCODE -ne 0) {
        throw "JavaScript syntax failed: $syntaxInput"
    }
}
Write-Output "Syntax PASS: $($syntaxInputs.Count) JavaScript files; no acceptance runtime started."
