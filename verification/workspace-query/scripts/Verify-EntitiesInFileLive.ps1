param(
    [string]$BinaryPath,
    [switch]$KeepWorkspace
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $repoRoot 'target\debug\clean-ctx.exe'
}
$BinaryPath = (Resolve-Path $BinaryPath).Path

$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("clean-ctx-live-" + [guid]::NewGuid())
[System.IO.Directory]::CreateDirectory($workspace) | Out-Null

$typescriptPath = Join-Path $workspace 'service.ts'
$typescriptHardLink = Join-Path $workspace 'service-hard-link.ts'
$csharpPath = Join-Path $workspace 'UsersController.cs'
$process = $null
$nextId = 1

function Write-Utf8File {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$Content
    )

    [System.IO.File]::WriteAllText(
        $Path,
        $Content,
        [System.Text.UTF8Encoding]::new($false)
    )
}

function Invoke-McpRequest {
    param(
        [Parameter(Mandatory)] [string]$Method,
        [Parameter(Mandatory)] [hashtable]$Params
    )

    $script:nextId += 1
    $requestId = $script:nextId
    $request = @{
        jsonrpc = '2.0'
        id = $requestId
        method = $Method
        params = $Params
    } | ConvertTo-Json -Compress -Depth 20

    $script:process.StandardInput.WriteLine($request)
    $script:process.StandardInput.Flush()

    while (-not $script:process.HasExited) {
        $line = $script:process.StandardOutput.ReadLine()
        if ($null -eq $line) {
            break
        }

        try {
            $response = $line | ConvertFrom-Json
        }
        catch {
            continue
        }

        if ($response.id -eq $requestId) {
            return $response
        }
    }

    throw "MCP server exited before responding to request $requestId ($Method)."
}

function Invoke-WorkspaceQuery {
    param(
        [Parameter(Mandatory)] [string]$FilePath,
        [Parameter(Mandatory)] [string]$Fidelity
    )

    return Invoke-McpRequest -Method 'tools/call' -Params @{
        name = 'workspace_query'
        arguments = @{
            type = 'entities_in_file'
            file_path = $FilePath
            workspaceRoot = $workspace
            fidelity = $Fidelity
        }
    }
}

function Invoke-FindEntities {
    param([Parameter(Mandatory)] [string]$Name)

    return Invoke-McpRequest -Method 'tools/call' -Params @{
        name = 'workspace_query'
        arguments = @{
            type = 'find_entities'
            name = $Name
            workspaceRoot = $workspace
        }
    }
}

function Assert-NoError {
    param(
        [Parameter(Mandatory)] $Response,
        [Parameter(Mandatory)] [string]$Scenario
    )

    $errorProperty = $Response.PSObject.Properties['error']
    if ($null -ne $errorProperty -and $null -ne $errorProperty.Value) {
        throw "$Scenario returned an MCP error: $($Response.error | ConvertTo-Json -Compress -Depth 10)"
    }
}

function Get-Entities {
    param([Parameter(Mandatory)] $Response)

    $resultProperty = $Response.PSObject.Properties['result']
    if ($null -eq $resultProperty -or $null -eq $resultProperty.Value.structuredContent) {
        throw 'workspace_query response did not expose structuredContent.'
    }

    return @($Response.result.structuredContent.entities)
}

function Assert-Entity {
    param(
        [Parameter(Mandatory)] [object[]]$Entities,
        [Parameter(Mandatory)] [string]$Domain,
        [Parameter(Mandatory)] [string]$EntityType,
        [Parameter(Mandatory)] [string]$Name,
        [Parameter(Mandatory)] [bool]$Present
    )

    $matches = @($Entities | Where-Object {
        $_.domain -eq $Domain -and
        $_.entity_type -eq $EntityType -and
        $_.name -eq $Name
    })

    if ($Present -and $matches.Count -eq 0) {
        throw "Expected $Domain/$EntityType '$Name', but it was absent."
    }
    if (-not $Present -and $matches.Count -ne 0) {
        throw "Expected $Domain/$EntityType '$Name' to be absent, but it was present."
    }
}

try {
    Write-Utf8File -Path $typescriptPath -Content @'
export class LiveUserService {
  getUser(id: string): string { return id; }
}
'@

    Write-Utf8File -Path $csharpPath -Content @'
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase
{
    [HttpGet]
    public IActionResult GetAll() => Ok();
}
'@

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $BinaryPath
    $startInfo.WorkingDirectory = $workspace
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $false
    $startInfo.CreateNoWindow = $true

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw "Failed to start $BinaryPath"
    }

    $toolList = Invoke-McpRequest -Method 'tools/list' -Params @{}
    Assert-NoError -Response $toolList -Scenario 'tools/list'
    $workspaceTool = @($toolList.result.tools | Where-Object { $_.name -eq 'workspace_query' })
    if ($workspaceTool.Count -ne 1) {
        throw 'tools/list did not expose exactly one workspace_query tool.'
    }
    $fidelityValues = @($workspaceTool[0].inputSchema.properties.fidelity.enum)
    foreach ($expected in @('low', 'medium', 'high', 'edit', 'verbatim')) {
        if ($fidelityValues -notcontains $expected) {
            throw "workspace_query schema is missing fidelity '$expected'."
        }
    }
    Write-Host 'PASS: live tools/list exposes workspace_query fidelity.'

    $firstTouch = Invoke-WorkspaceQuery -FilePath $typescriptPath -Fidelity 'high'
    Assert-NoError -Response $firstTouch -Scenario 'first-touch TypeScript query'
    $firstEntities = @(Get-Entities -Response $firstTouch)
    Assert-Entity -Entities $firstEntities -Domain 'builtin' -EntityType 'Class' -Name 'LiveUserService' -Present $true
    Write-Host 'PASS: entities_in_file compiles an untracked TypeScript file on first touch.'

    if ($IsWindows -or $IsLinux -or $IsMacOS) {
        New-Item -ItemType HardLink -Path $typescriptHardLink -Target $typescriptPath | Out-Null
        $hardLinkTouch = Invoke-WorkspaceQuery -FilePath $typescriptHardLink -Fidelity 'high'
        Assert-NoError -Response $hardLinkTouch -Scenario 'hard-link alternate query'
        $hardLinkEntities = @(Get-Entities -Response $hardLinkTouch)
        Assert-Entity -Entities $hardLinkEntities -Domain 'builtin' -EntityType 'Class' -Name 'LiveUserService' -Present $true

        $hardLinkDiscovery = Invoke-FindEntities -Name 'LiveUserService'
        Assert-NoError -Response $hardLinkDiscovery -Scenario 'hard-link discovery query'
        $hardLinkOccurrences = @((Get-Entities -Response $hardLinkDiscovery) | Where-Object {
            $_.domain -eq 'builtin' -and $_.entity_type -eq 'Class' -and $_.name -eq 'LiveUserService'
        })
        if ($hardLinkOccurrences.Count -ne 1) {
            throw "Expected one physical LiveUserService occurrence, found $($hardLinkOccurrences.Count)."
        }
        Write-Host 'PASS: hard-link spellings publish one workspace occurrence.'
    }

    $lowerRequest = Invoke-WorkspaceQuery -FilePath $typescriptPath -Fidelity 'low'
    Assert-NoError -Response $lowerRequest -Scenario 'lower-fidelity reuse query'
    $lowerEntities = @(Get-Entities -Response $lowerRequest)
    Assert-Entity -Entities $lowerEntities -Domain 'builtin' -EntityType 'Class' -Name 'LiveUserService' -Present $true
    Write-Host 'PASS: a later lower-fidelity request remains semantically complete.'

    $invalid = Invoke-WorkspaceQuery -FilePath $typescriptPath -Fidelity 'maximum'
    $invalidError = $invalid.PSObject.Properties['error']
    if ($null -eq $invalidError -or $invalidError.Value.code -ne -32602) {
        throw "Invalid fidelity did not return JSON-RPC -32602: $($invalid | ConvertTo-Json -Compress -Depth 10)"
    }
    Write-Host 'PASS: invalid fidelity returns a clear invalid-params error.'

    $csharpLow = Invoke-WorkspaceQuery -FilePath $csharpPath -Fidelity 'low'
    Assert-NoError -Response $csharpLow -Scenario 'C# Low query'
    $lowEntities = @(Get-Entities -Response $csharpLow)
    Assert-Entity -Entities $lowEntities -Domain 'dotnet' -EntityType 'Action' -Name 'GetAll' -Present $false

    $csharpMedium = Invoke-WorkspaceQuery -FilePath $csharpPath -Fidelity 'medium'
    Assert-NoError -Response $csharpMedium -Scenario 'C# Medium upgrade query'
    $mediumEntities = @(Get-Entities -Response $csharpMedium)
    Assert-Entity -Entities $mediumEntities -Domain 'dotnet' -EntityType 'Action' -Name 'GetAll' -Present $true
    Write-Host 'PASS: a higher-fidelity request upgrades the semantic projection.'

    Write-Utf8File -Path $typescriptPath -Content @'
export class ReplacementService {
  replacement(): boolean { return true; }
}
'@
    $replacement = Invoke-WorkspaceQuery -FilePath $typescriptPath -Fidelity 'high'
    Assert-NoError -Response $replacement -Scenario 'changed-source query'
    $replacementEntities = @(Get-Entities -Response $replacement)
    Assert-Entity -Entities $replacementEntities -Domain 'builtin' -EntityType 'Class' -Name 'ReplacementService' -Present $true
    Assert-Entity -Entities $replacementEntities -Domain 'builtin' -EntityType 'Class' -Name 'LiveUserService' -Present $false
    Write-Host 'PASS: source changes replace stale semantic entities.'

    Write-Utf8File -Path $typescriptPath -Content '// intentionally empty semantic projection'
    $empty = Invoke-WorkspaceQuery -FilePath $typescriptPath -Fidelity 'high'
    Assert-NoError -Response $empty -Scenario 'empty-projection query'
    $emptyEntities = @(Get-Entities -Response $empty)
    if ($emptyEntities.Count -ne 0) {
        throw "Expected an empty replacement projection, found $($emptyEntities.Count) entities."
    }
    Write-Host 'PASS: an empty recompilation clears stale semantic entities.'

    Write-Host 'PASS: live MCP entities_in_file fidelity-aware auto-compile verification completed.'
}
finally {
    if ($null -ne $process) {
        try {
            $process.StandardInput.Close()
        }
        catch {
            # Best-effort shutdown after a failed assertion.
        }

        if (-not $process.HasExited) {
            $process.Kill()
            if (-not $process.WaitForExit(5000)) {
                Write-Warning "MCP server did not exit within five seconds; workspace cleanup may be deferred."
            }
        }
        $process.Dispose()
    }

    if ($KeepWorkspace) {
        Write-Host "Retained live verification workspace: $workspace"
    }
    elseif (Test-Path -LiteralPath $workspace) {
        $removed = $false
        foreach ($attempt in 1..5) {
            try {
                Remove-Item -LiteralPath $workspace -Recurse -Force
                $removed = $true
                break
            }
            catch {
                if ($attempt -lt 5) {
                    Start-Sleep -Milliseconds 200
                }
            }
        }

        if (-not $removed) {
            Write-Warning "Could not remove live verification workspace: $workspace"
        }
    }
}
