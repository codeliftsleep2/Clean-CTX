param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$Profile,
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$Requests,
    [Parameter(Mandatory)][string]$OutputRoot,
    [Parameter(Mandatory)][string]$LaunchDirectory
)
# Operator verification, not a Rust regression or independent agent evaluation.
# Reuses the existing process isolation, transport limits, shutdown and receipts.
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'invoke-workspace-documents.ps1') `
    -WorkspaceRoot $WorkspaceRoot -Profile $Profile -Binary $Binary -Requests $Requests `
    -OutputRoot $OutputRoot -LaunchDirectory $LaunchDirectory -ToolName pcg_knowledge_v1
