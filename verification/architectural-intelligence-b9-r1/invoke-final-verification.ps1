param([Parameter(Mandatory)][string]$WorkspaceRoot,[Parameter(Mandatory)][string]$OutputRoot)
# User-owned orchestration of the existing authoritative gate, not a new gate.
# Commands are read verbatim from docs/agent/verification.md, never redefined here.
$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath $WorkspaceRoot).Path
$output=[IO.Path]::GetFullPath($OutputRoot)
if(Test-Path -LiteralPath $output){throw 'Use a fresh final-verification output directory.'}
$authority=Join-Path $root 'docs/agent/verification.md'
$utf8=[Text.UTF8Encoding]::new($false,$true)
$text=[IO.File]::ReadAllText($authority,$utf8)
$section=[regex]::Match($text,'(?s)## Final Verification Gate\r?\n(.*?)(?=\r?\n## |\z)').Groups[1].Value
$block=[regex]::Match($section,'(?s)```bash\r?\n(.*?)\r?\n```')
if(!$block.Success){throw 'Authoritative final gate command block not found; do not guess commands.'}
$commands=@($block.Groups[1].Value -split '\r?\n'|Where-Object {$_ -match '\S'})
New-Item -ItemType Directory -Path $output|Out-Null
$manifest=[ordered]@{authority=$authority;authority_sha256=(Get-FileHash $authority).Hash.ToLowerInvariant();commands=$commands;scope='single existing final repository gate; user-run';head=(& git -C $root rev-parse HEAD)}
[IO.File]::WriteAllText((Join-Path $output 'manifest.json'),($manifest|ConvertTo-Json -Depth 6),$utf8)
$results=[Collections.Generic.List[object]]::new()
Push-Location $root
try {
    for($index=0;$index -lt $commands.Count;$index++){
        $command=$commands[$index]
        Write-Output "Final gate $($index+1)/$($commands.Count): $command"
        & pwsh -NoProfile -Command $command 2>&1|Tee-Object (Join-Path $output ('{0:d2}.log' -f ($index+1)))
        $code=$LASTEXITCODE
        $results.Add([ordered]@{step=$index+1;command=$command;exit_code=$code})
        [IO.File]::WriteAllText((Join-Path $output 'results.json'),(ConvertTo-Json -InputObject $results.ToArray() -Depth 6),$utf8)
        # Preserve every result; failure does not silently become a later success.
    }
} finally {Pop-Location}
$failed=@($results|Where-Object exit_code -ne 0)
Write-Output "Final gate finished: $($results.Count) commands; $($failed.Count) failed. Evidence: $output"
if($failed.Count){throw 'Final gate has failures. Preserve logs; do not claim completion or change unrelated code.'}
