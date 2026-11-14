<#
A stand-in for the world preview's scripts\preview.ps1, so that the app's run code is tested without the game.
It mimics the protocol of the real script: the same arguments, the files it deletes first, the progress.json the
mod writes every 0.5 s (atomically, by a rename), the files a finished run leaves, timing.json, run.json, and the
exit by throw when the run fails or is killed. It downloads nothing, starts no game and writes only under this
repository's target\tmp (it refuses any other Out or Dir before it creates anything).

The mode is $env:NUI_STUB: 'finish' (default) copies a finished out folder given by $env:NUI_STUB_FROM, 'hold'
stays in the map pass with a child process of its own, as the game server is, until it is killed, and 'fail'
throws after its first progress.
Windows PowerShell 5.1 compatible.
#>
param(
    [string]$Pack,
    [string]$Dir,
    [string]$Seed = 'cap-1',
    [hashtable]$Tectonic = @{},
    [int]$Samples = 512,
    [int]$Step = 16,
    [int]$Detail,
    [string]$Out,
    [string[]]$Mods
)

$ErrorActionPreference = 'Stop'

# The guard comes first and nothing is created before it: a test run must never write anywhere else.
$tmp = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\target\tmp'))
if (-not $Out) { throw 'stub: -Out is required.' }
foreach ($path in @($Out, $Dir)) {
    if ($path -and -not [IO.Path]::GetFullPath($path).StartsWith("$tmp\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "stub: refusing $path, it is not under $tmp"
    }
}

if (-not $Detail) { $Detail = $Step }
if ($Detail -notin $Step, (2 * $Step), (4 * $Step)) { throw "Detail must be $Step, $(2 * $Step) or $(4 * $Step) blocks." }
$Out = [IO.Path]::GetFullPath($Out)
if ($Dir) { New-Item -ItemType Directory -Force $Dir | Out-Null }
New-Item -ItemType Directory -Force $Out | Out-Null

Remove-Item (Join-Path $Out 'preview.json'), (Join-Path $Out 'coarse.json'), (Join-Path $Out 'coarse-*.bin'),
    (Join-Path $Out 'section.json'), (Join-Path $Out 'section-*.bin'), (Join-Path $Out 'ores.json'),
    (Join-Path $Out 'structures.json'), (Join-Path $Out 'timing.json'), (Join-Path $Out 'progress.json'),
    (Join-Path $Out 'partial.json'), (Join-Path $Out 'partial-*.bin') -ErrorAction SilentlyContinue

$mode = if ($env:NUI_STUB) { $env:NUI_STUB } else { 'finish' }
$written = 0

# What Progress.java does: write beside, then replace; a failed replace is tried again at the next write.
function Set-StubProgress([string]$Pass, [int]$At, [int]$Done, [int]$Total) {
    $tmpFile = Join-Path $Out 'progress.json.tmp'
    [IO.File]::WriteAllText($tmpFile, "{`"pass`":`"$Pass`",`"step`":$At,`"done`":$Done,`"total`":$Total}")
    try { Move-Item -LiteralPath $tmpFile -Destination (Join-Path $Out 'progress.json') -Force } catch { }
    $script:written++
    if ($mode -eq 'fail' -and $script:written -eq 1) { throw 'stub: the run failed on purpose.' }
}

# The game starts: no progress file yet.
Start-Sleep -Milliseconds 1200

foreach ($at in (@(4, 2, 1) | ForEach-Object { $_ * $Step } | Where-Object { $_ -ge $Detail })) {
    if ($at -gt $Detail) {
        for ($done = 0; $done -le 4; $done++) { Set-StubProgress 'sketch' $at $done 4; Start-Sleep -Milliseconds 200 }
    } else {
        for ($done = 0; $done -le 10; $done++) {
            Set-StubProgress 'map' $at $done 10
            if ($mode -eq 'hold' -and $done -eq 4) {
                # Stands for the game server Invoke-Java starts: a process of its own that outlives a polite stop.
                $p = Start-Process powershell.exe -ArgumentList '-NoProfile', '-Command', 'Start-Sleep -Seconds 600' -NoNewWindow -PassThru
                [IO.File]::WriteAllText((Join-Path $Out 'stub-child.pid'), "$($p.Id)")
                $p.WaitForExit()
                throw 'The preview server ended without a result.'
            }
            Start-Sleep -Milliseconds 200
        }
    }
}
Set-StubProgress 'extras' $Detail 0 0
Start-Sleep -Milliseconds 300

if ($mode -ne 'finish') { throw "stub: unknown mode $mode" }
$from = $env:NUI_STUB_FROM
if (-not $from -or -not (Test-Path (Join-Path $from 'preview.json'))) { throw 'stub: NUI_STUB_FROM must name a folder with a preview.json.' }
foreach ($name in 'preview.json', 'heights.bin', 'biomes.bin', 'section.json', 'section-*.bin', 'ores.json', 'structures.json') {
    Get-ChildItem -Path (Join-Path $from $name) -ErrorAction SilentlyContinue | Copy-Item -Destination $Out -Force
}
[IO.File]::WriteAllText((Join-Path $Out 'timing.json'), '{"workMillis":1500}')

# As preview.ps1's last lines: a switch is kept as 1 or 0.
$recorded = @{}
foreach ($key in $Tectonic.Keys) { $recorded[$key] = if ($Tectonic[$key] -is [bool]) { [int]$Tectonic[$key] } else { $Tectonic[$key] } }
[IO.File]::WriteAllText((Join-Path $Out 'run.json'), (@{ seed = $Seed; tectonic = $recorded } | ConvertTo-Json))
Write-Host "Preview ready in $Out"
