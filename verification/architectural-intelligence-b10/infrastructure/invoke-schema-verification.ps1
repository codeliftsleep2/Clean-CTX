param(
    [Parameter(Mandatory)][string]$WorkspaceRoot,
    [Parameter(Mandatory)][string]$OutputRoot
)
# User-owned Cargo/format/build run. The agent must not launch this script.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output = [IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $output) { throw 'Use a fresh verification output directory.' }
New-Item -ItemType Directory -Path $output | Out-Null
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Push-Location $root
try {
    $binary = Join-Path $root 'target/debug/clean-ctx.exe'
    $baselineHash = '5dcd08d0929f5f6a10c6ad7d4392e15f23827da9a926b25bc2dc43c99b336fb1'
    if ((Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant() -ne $baselineHash) {
        throw 'Before-build executable differs from accepted B9. Report its identity before proceeding.'
    }
    Copy-Item -LiteralPath $binary -Destination (Join-Path $output 'accepted-b9-clean-ctx.exe')
    if ((Get-FileHash (Join-Path $output 'accepted-b9-clean-ctx.exe')).Hash.ToLowerInvariant() -ne $baselineHash) {
        throw 'Baseline executable preservation failed.'
    }
    $steps = @(
        @{name='format'; command='rustfmt'; arguments=@('--edition','2024','--config','skip_children=true','--check','src/mcp/tool_handlers/pcg_documents.rs','src/tests/mcp/pcg_knowledge.rs','src/tests/mcp/pcg_knowledge_schema.rs')},
        @{name='focused-green'; command='cargo'; arguments=@('test','--lib','--all-features','schema_interoperability','--','--nocapture')},
        @{name='registered-tools'; command='cargo'; arguments=@('test','--lib','--all-features','pcg_documents::tests','--','--nocapture')},
        @{name='clippy'; command='cargo'; arguments=@('clippy','-p','clean-ctx','--all-targets','--all-features','--','-D','warnings')},
        @{name='production-build'; command='cargo'; arguments=@('build','--bin','clean-ctx','--all-features')}
    )
    $results = [Collections.Generic.List[object]]::new()
    foreach ($step in $steps) {
        Write-Output "Running $($step.name): $($step.command) $($step.arguments -join ' ')"
        $stepArguments = $step.arguments
        & $step.command @stepArguments 2>&1 |
            Tee-Object -FilePath (Join-Path $output "$($step.name).log")
        $code = $LASTEXITCODE
        $results.Add([ordered]@{name=$step.name; command=$step.command; arguments=$step.arguments; exit_code=$code})
        [IO.File]::WriteAllText((Join-Path $output 'steps.json'), ($results.ToArray() | ConvertTo-Json -Depth 8), $utf8)
        if ($code -ne 0) { throw "Step $($step.name) failed ($code); later steps were not run." }
    }
    $identity = [ordered]@{
        head = (& git rev-parse HEAD)
        source_state = 'working tree; not claimed to be a committed B10 executable'
        binary = $binary; binary_sha256 = (Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant()
        profile_sha256 = (Get-FileHash '.clean-ctx/pcg-documents/profile.json').Hash.ToLowerInvariant()
        descriptor_source_sha256 = (Get-FileHash 'src/mcp/tool_handlers/pcg_documents.rs').Hash.ToLowerInvariant()
        scope = 'focused tracked tests, registered tools, scoped Clippy, production build; not full repository gate'
    }
    [IO.File]::WriteAllText((Join-Path $output 'build-identity.json'), ($identity | ConvertTo-Json -Depth 4), $utf8)
    Write-Output "Scoped verification finished. Build identity: $output/build-identity.json"
} finally {
    Pop-Location
}
