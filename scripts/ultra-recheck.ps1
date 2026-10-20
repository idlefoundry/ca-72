# scripts/ultra-recheck.sh on Windows (decisions.md R-THREADS): the same runs, one line each
# appended to $Out.
param([string]$Out = "ultra-recheck.txt", [int]$Seconds = 10)
$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")
function Exe($name) {
    $json = cargo test --release -p ca72-plugin --test $name --no-run --message-format=json 2>$null
    ($json | Select-String "`"executable`":`"([^`"]*$name[^`"]*)`"" | Select-Object -Last 1).Matches[0].Groups[1].Value -replace '\\\\', '\'
}
$split = Exe "ultra_split"
$bench = Exe "scaling_bench"
"# $((Get-Date).ToUniversalTime().ToString('s'))Z Windows cpus=$([Environment]::ProcessorCount)" | Out-File -Append -Encoding utf8 $Out
function Run($exe, $filter, $tag, [hashtable]$vars) {
    $env:CA72_SECONDS = "$Seconds"
    foreach ($k in $vars.Keys) { Set-Item "env:$k" $vars[$k] }
    $args = @("--ignored", "--nocapture", "--test-threads", "1")
    if ($filter) { $args = @($filter) + $args }
    $line = & $exe @args 2>$null | Select-String "$tag.*" | ForEach-Object { $_.Matches[0].Value }
    $a = ($vars.GetEnumerator() | Sort-Object Key | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ','
    if (-not $line) { $line = "# failed: $a" } else { $line = "$line args=$a" }
    $line | Out-File -Append -Encoding utf8 $Out
    foreach ($k in $vars.Keys) { Remove-Item "env:$k" }
}
foreach ($block in 256, 128) {
    foreach ($c in @(@(1,4,0), @(1,6,1), @(1,12,1), @(2,12,0), @(2,12,2))) {
        Run $split "split_ultra" "split:" @{ CA72_BLOCK = "$block"; CA72_VOICES = "$($c[0])"; CA72_BUDGET = "$($c[1])"; CA72_SPLIT = "$($c[2])" }
    }
    foreach ($rep in 1, 2) {
        foreach ($s in "auto", "0") {
            $one = @{ CA72_AUTO = "1"; CA72_QUALITY = "ultra"; CA72_BLOCK = "$block"; CA72_POLY = "0"; CA72_VOICES = "1"; REP = "$rep" }
            $two = @{ CA72_AUTO = "1"; CA72_QUALITY = "ultra"; CA72_BLOCK = "$block"; CA72_VOICES = "2"; REP = "$rep" }
            if ($s -eq "0") { $one.CA72_SPLIT = "0"; $two.CA72_SPLIT = "0" }
            Run $bench $null "scaling:" $one
            Run $bench $null "scaling:" $two
        }
    }
}
