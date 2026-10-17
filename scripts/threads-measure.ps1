# scripts/threads-measure.sh on Windows (decisions.md R-THREADS): the same runs, one
# `scaling:` line each appended to $Out.
param([string]$Out = "threads.txt", [int]$Seconds = 10)
$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")
$json = cargo test --release -p ca72-plugin --test scaling_bench --no-run --message-format=json 2>$null
$bin = ($json | Select-String '"executable":"([^"]*scaling_bench[^"]*)"' | Select-Object -Last 1).Matches[0].Groups[1].Value -replace '\\\\', '\'
"# $((Get-Date).ToUniversalTime().ToString('s'))Z Windows cpus=$([Environment]::ProcessorCount)" | Out-File -Append -Encoding utf8 $Out
function Run([hashtable]$vars) {
    $env:CA72_AUTO = "1"; $env:CA72_SECONDS = "$Seconds"
    foreach ($k in $vars.Keys) { Set-Item "env:$k" $vars[$k] }
    $line = & $bin --ignored --nocapture --test-threads 1 2>$null | Select-String 'scaling:.*' | ForEach-Object { $_.Matches[0].Value }
    $args = ($vars.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ','
    if (-not $line) { $line = "# failed: $args" } else { $line = "$line args=$args" }
    $line | Out-File -Append -Encoding utf8 $Out
    foreach ($k in $vars.Keys) { Remove-Item "env:$k" }
}
foreach ($block in 256, 128) {
    foreach ($rep in 1, 2) {
        foreach ($split in "auto", "0") {
            $one = @{ CA72_QUALITY = "ultra"; CA72_BLOCK = "$block"; CA72_POLY = "0"; CA72_VOICES = "1" }
            $two = @{ CA72_QUALITY = "ultra"; CA72_BLOCK = "$block"; CA72_VOICES = "2" }
            if ($split -eq "0") { $one.CA72_SPLIT = "0"; $two.CA72_SPLIT = "0" }
            Run $one; Run $two
        }
    }
    Run @{ CA72_QUALITY = "ultra"; CA72_BLOCK = "$block"; CA72_VOICES = "4" }
    foreach ($instances in 1, 2, 4, 8) {
        Run @{ CA72_QUALITY = "hi"; CA72_BLOCK = "$block"; CA72_INSTANCES = "$instances" }
    }
}
