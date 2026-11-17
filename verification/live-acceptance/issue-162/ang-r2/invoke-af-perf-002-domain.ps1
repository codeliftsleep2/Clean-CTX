# Agent MaxHeadRoom: operator-owned execution of the complete named test domain.
# Source discovery is reconciled against the compiler's actual test registration.
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$out = Join-Path $repo ('target/issue-162/af-perf-002-domain-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $out | Out-Null
Write-Output "AF-PERF-002 domain evidence: $out"
$oldBinary = $env:AF002_BINARY_PUBLIC_ORACLE
$oldSqlite = $env:AF002_SQLITE_TEXT_ORACLE
Push-Location $repo
try {
    $env:AF002_BINARY_PUBLIC_ORACLE = Join-Path $repo 'src/tests/mcp/fixtures/binary_clone_public_identities.json'
    $env:AF002_SQLITE_TEXT_ORACLE = Join-Path $repo 'src/tests/mcp/fixtures/sqlite_text_public_identities.json'
    $sources = @(Get-ChildItem -LiteralPath 'src/tests' -Recurse -Filter '*af_perf_002.rs' -File)
    $inventory = @()
    $names = @{root=@();proxy=@()}
    foreach ($source in $sources) {
        $relative = [IO.Path]::GetRelativePath($repo, $source.FullName).Replace('\','/')
        $text = [IO.File]::ReadAllText($source.FullName)
        $tests = @([regex]::Matches($text, '(?s)#\[test\](?:(?!fn ).)*?fn (\w+)') | ForEach-Object { $_.Groups[1].Value })
        $package = if ($relative.StartsWith('src/tests/proxy/')) { 'proxy' } else { 'root' }
        $names[$package] += $tests
        $inventory += @{path=$relative;sha256=(Get-FileHash -LiteralPath $source.FullName).Hash;tests=$tests}
    }
    $inventory | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $out 'source-inventory.json') -Encoding utf8NoBOM
    $results = @()
    $failures = @()
    foreach ($package in @('root','proxy')) {
        $cargoArgs = @('test','--lib','--all-features','--quiet')
        if ($package -eq 'proxy') { $cargoArgs += @('-p','clean-ctx-proxy') }
        $listLog = Join-Path $out ($package + '-registration.log')
        & cargo @cargoArgs -- --list *> $listLog
        if ($LASTEXITCODE -ne 0) { throw "Registration/compilation failed; preserve $listLog" }
        $registered = @([regex]::Matches([IO.File]::ReadAllText($listLog),'(?m)^(.+): test\r?$') | ForEach-Object { $_.Groups[1].Value })
        $selected = @()
        foreach ($name in ($names[$package] | Sort-Object -Unique)) {
            $matching = @($registered | Where-Object { ($_ -split '::')[-1] -ceq $name })
            if ($matching.Count -ne 1) { throw "Expected one registered $package test named $name; found $($matching.Count); preserve $out" }
            $selected += $matching[0]
        }
        if (-not $selected.Count) { throw "Empty $package domain selection" }
        # Bounded argument lengths on Windows; every registered test runs once.
        for ($offset=0; $offset -lt $selected.Count; $offset+=40) {
            $last = [Math]::Min($offset+39,$selected.Count-1)
            $batch = @($selected[$offset..$last])
            $log = Join-Path $out ("$package-$offset.log")
            & cargo @cargoArgs -- --exact --test-threads=1 --nocapture @batch *> $log
            $code = $LASTEXITCODE
            $summary = [regex]::Match([IO.File]::ReadAllText($log),'(?m)^test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
            $passed = $code -eq 0 -and $summary.Success -and [int]$summary.Groups[1].Value -eq $batch.Count
            $results += @{
                package=$package;tests=$batch
                arguments=@($cargoArgs)+@('--','--exact','--test-threads=1','--nocapture')+$batch
                exitCode=$code;passed=$passed;log=$log;sha256=(Get-FileHash -LiteralPath $log).Hash
            }
            if (-not $passed) {
                $failures += "$package-$offset"
                Get-Content -LiteralPath $log -Tail 120 | Write-Output
            }
        }
    }
    [pscustomobject]@{
        agent='Agent MaxHeadRoom';category='tracked AF-PERF-002 Rust tests'
        head=(git rev-parse HEAD);status=(git status --porcelain)
        results=$results;failedBatches=$failures
    } | ConvertTo-Json -Depth 7 | Set-Content (Join-Path $out 'results.json') -Encoding utf8NoBOM
    if ($failures.Count) { throw "Domain failed: $($failures -join ', '); preserve $out" }
    Write-Output "PASS: all $($names.root.Count + $names.proxy.Count) source-declared domain tests registered and passed. Receipts: $out"
} finally {
    $env:AF002_BINARY_PUBLIC_ORACLE=$oldBinary
    $env:AF002_SQLITE_TEXT_ORACLE=$oldSqlite
    Pop-Location
}
