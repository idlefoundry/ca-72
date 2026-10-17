# The workers' scaling grid on Windows (decisions.md R-THREADS): scripts/scaling-sweep.sh's grid,
# one `scaling:` line a run appended to $Out. Runs whose host threads and workers together
# would pass the processors less two are left out, as on Linux.
param([string]$Out = "scaling.txt", [int]$Seconds = 10)
$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")
$json = cargo test --release -p ca72-plugin --test scaling_bench --no-run --message-format=json 2>$null
$bin = ($json | Select-String '"executable":"([^"]*scaling_bench[^"]*)"' | Select-Object -Last 1).Matches[0].Groups[1].Value -replace '\\\\', '\'
$cpus = [Environment]::ProcessorCount
$cpu = (Get-CimInstance Win32_Processor).Name
"# $((Get-Date).ToUniversalTime().ToString('s'))Z Windows cpus=$cpus $cpu" | Tee-Object -FilePath $Out -Append
function Run([hashtable]$vars) {
    foreach ($k in $vars.Keys) { Set-Item "env:$k" $vars[$k] }
    $env:CA72_SECONDS = "$Seconds"
    $line = & $bin --ignored --nocapture --test-threads 1 2>$null | Select-String 'scaling:.*' | ForEach-Object { $_.Matches[0].Value }
    if (-not $line) { $line = "# failed: $($vars | Out-String)" }
    $line | Tee-Object -FilePath $Out -Append
    foreach ($k in $vars.Keys) { Remove-Item "env:$k" }
}
foreach ($quality in "hi", "ultra") {
    foreach ($block in 256, 128) {
        foreach ($instances in 1, 2, 4, 8) {
            foreach ($workers in 0..12) {
                if ($instances + $workers -gt $cpus - 2) { continue }
                Run @{ CA72_QUALITY = $quality; CA72_BLOCK = "$block"; CA72_INSTANCES = "$instances"; CA72_WORKERS = "$workers" }
            }
            if ($instances -gt 1) {
                Run @{ CA72_QUALITY = $quality; CA72_BLOCK = "$block"; CA72_INSTANCES = "$instances"; CA72_WORKERS = "4"; CA72_SHARE = "first" }
            }
        }
    }
}
