param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot,
    [string]$ExpectedBinarySha256 = 'd91702b1c2a5b8e875b2c937e54b9ed4d5ad6fcbc304d79c9001967aff78fa38'
)
# User-owned one-session host graph-to-explicit-acquisition witness. No build.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
& (Join-Path $root 'verification/architectural-intelligence-b10/infrastructure/invoke-host-smoke.ps1') `
    -WorkspaceRoot $root -OutputRoot $OutputRoot `
    -ExpectedBinarySha256 $ExpectedBinarySha256 `
    -TaskPromptFile (Join-Path $PSScriptRoot 'host-graph-acquisition-prompt.txt') `
    -Purpose 'B9-R1 bounded actual-host graph-to-explicit-acquisition verification; not B10 discovery rerun or independent evaluation'
