$ErrorActionPreference = "Stop"
$definitionRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$fixtures = Join-Path $repositoryRoot "target\context-compression-verification\fixtures"
New-Item -ItemType Directory -Force $fixtures | Out-Null
$definitions = Get-Content -Raw (Join-Path $definitionRoot "expected\marginal-fixtures.json") | ConvertFrom-Json

foreach ($definition in $definitions) {
    $base = Join-Path $fixtures ("marginal-{0}-base.ts" -f $definition.id)
    $plus = Join-Path $fixtures ("marginal-{0}-plus.ts" -f $definition.id)
    [IO.File]::WriteAllText($base, $definition.base, [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($plus, $definition.plus, [Text.UTF8Encoding]::new($false))
}

$lf = Get-Content -Raw (Join-Path $definitionRoot "fixtures\semantic-rich.ts")
$crlf = $lf -replace "`r?`n", "`r`n"
[IO.File]::WriteAllText((Join-Path $fixtures "semantic-rich-crlf.ts"), $crlf, [Text.UTF8Encoding]::new($false))
Write-Host "Prepared marginal pairs and semantic-rich-crlf.ts"
