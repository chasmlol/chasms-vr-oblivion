# Oblivion in one command, for checks and playtests (docs/OVERNIGHT.md, "Fast boot"): boot, test, close.
#
# Takes the game lock (one game at a time, whichever worktree asks), builds and installs this
# checkout's adapter and runtime, and starts the game from its exe (`SteamAppId` keeps Steam's DRM
# from restarting it through the launcher). It skips the intro films and the logo, and loads a save
# the moment the main menu is up. Then, if asked, it moves the player (`coc`), gives them things (by
# form id), sets the hour and the weather, and runs more console lines. After that it either:
#   -Sim <script.toml>   runs an emulated-headset script (framework/crates/xrsim), then closes the game;
#   -Hold <seconds>      the emulated headset, held still that long, then closes (a look at the logs);
#   (neither)            leaves the game for the real headset (a playtest); -Close puts it away later.
# Closing puts back everything it changed (Oblivion.ini, rustvr-oblivion.toml) and gives the lock back.
#
#   powershell -File tools\oblivion.ps1 [-Save rustvr-city | -NewGame] [-At ICMarketDistrict] [-Give "0000000A 5,0000000F 100"]
#       [-Hour 14] [-Weather 00038EEE] [-Commands "tgm|player.setav marksman 100"]
#       [-Sim tools\xrsim\x.toml | -Hold 30] [-Out captures\run] [-Settings "eye_width = 2048; eye_height = 2048"] [-NoBuild]
#   powershell -File tools\oblivion.ps1 -Close
#
# Run it from the Bash tool, not the PowerShell tool: a sandboxed start can't reach Steam and brings
# up Oblivion's launcher. Builds go to $env:CARGO_TARGET_DIR (a worktree's own), else target\.
# Test saves: rustvr-base (a Nord in iron armour), rustvr-city (Imperial City market), rustvr-weye
# (open ground on Lake Rumare's shore), rustvr-ruin (Vilverin, an Ayleid ruin), rustvr-chorrol
# (Chorrol, its stables); `-Save "Skip Tutorial"` and the player's own saves load too. -NewGame
# starts a new game instead (the main menu's New, answered yes, its film skipped): the prison cell
# and the race menu, as a player starting out has them.
param(
    [string]$Save = "rustvr-city",
    [switch]$NewGame,
    [string]$At = "",
    [string[]]$Give = @(),
    [int]$Hour = -1,
    [string]$Weather = "",
    [string[]]$Commands = @(),
    [string]$ConsoleFile = "",
    [string]$Sim = "",
    [int]$Hold = 0,
    [string]$Out = "",
    [string]$Settings = "",
    [switch]$NoBuild,
    [switch]$Close,
    [int]$Jobs = 4,
    [int]$LockWait = 5400
)
$ErrorActionPreference = "Continue"
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
if (-not $env:RUSTVR_DESKTOP_VIEW) { $env:RUSTVR_DESKTOP_VIEW = "0" }
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo
$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repo "target" }
$game = "${env:ProgramFiles(x86)}\Steam\steamapps\common\Oblivion"
$lockDir = "<builds>"
$lock = Join-Path $lockDir "game.lock"
New-Item -ItemType Directory -Force $lockDir | Out-Null

function Alive([int]$id) { $id -gt 0 -and (Get-Process -Id $id -ErrorAction SilentlyContinue) }

# The lock: "<owner>|<script pid>|<game pid>|<since>". Held while a script runs or a game it left
# is open; a lock whose script and game are both gone is stale and taken over.
function Read-Lock {
    $t = Get-Content $lock -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $t) { return $null }
    $p = $t -split '\|'
    [pscustomobject]@{ Owner = $p[0]; Script = [int]$p[1]; Game = [int]$p[2]; Since = $p[3] }
}
function Write-Lock([int]$gamePid) {
    Set-Content $lock "$repo|$PID|$gamePid|$(Get-Date -Format s)"
}
function Take-Lock {
    $deadline = (Get-Date).AddSeconds($LockWait)
    $said = $false
    while ($true) {
        try {
            $fs = [System.IO.File]::Open($lock, 'CreateNew', 'Write', 'None')
            $fs.Close()
            Write-Lock 0
            return
        } catch {
            $l = Read-Lock
            # Ours (a game this checkout left for the headset), or nobody's any more: taken over.
            if ($null -eq $l -or $l.Owner -eq $repo -or (-not (Alive $l.Script) -and -not (Alive $l.Game))) {
                Remove-Item $lock -Force -ErrorAction SilentlyContinue
                continue
            }
            if ((Get-Date) -gt $deadline) { "the game is still held by $($l.Owner) (since $($l.Since)): giving up"; exit 2 }
            if (-not $said) { "waiting for the game: $($l.Owner) has it (since $($l.Since))"; $said = $true }
            Start-Sleep 5
        }
    }
}
function Give-Lock {
    $l = Read-Lock
    if ($l -and $l.Owner -eq $repo) { Remove-Item $lock -Force -ErrorAction SilentlyContinue }
}

# Closes the game and what was started for it, and puts the player's settings back.
function Put-Away {
    Get-Process OblivionLauncher -ErrorAction SilentlyContinue | Stop-Process -Force
    Get-Process Oblivion -ErrorAction SilentlyContinue | Stop-Process -Force
    Get-Process Oblivion -ErrorAction SilentlyContinue | Wait-Process -Timeout 20 -ErrorAction SilentlyContinue
    # Only runtimes started for the game or a test (never the framework's own range, --wait).
    Get-CimInstance Win32_Process -Filter "Name = 'rustvr-xrsim.exe' OR Name = 'rustvr-play.exe' OR Name = 'rustvr.exe'" |
        Where-Object { $_.Name -eq 'rustvr-xrsim.exe' -or $_.CommandLine -match '--attach|--game-pid|--exit-with-game|--no-range' } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Start-Sleep 1
    Set-Content "$game\rustvr-oblivion.toml" "background = false`n"
    python tools\install_oblivion.py --restore-ini | Out-Null
    Remove-Item "$game\rustvr-console-test" -ErrorAction SilentlyContinue
}

if ($Close) {
    $l = Read-Lock
    if ($l -and $l.Owner -ne $repo -and ((Alive $l.Script) -or (Alive $l.Game))) { "the game is $($l.Owner)'s, not this checkout's: left alone"; exit 2 }
    Put-Away
    Give-Lock
    "closed; the player's settings are back"
    exit 0
}

# Steam's DRM starts the game only for a signed-in Steam: signed out, the game stops at an
# "Application load error" box (or, with Steam just starting, the first process dies in the adapter).
if (-not (Get-ItemProperty "HKCU:\Software\Valve\Steam\ActiveProcess" -ErrorAction SilentlyContinue).ActiveUser) {
    "Steam isn't signed in: sign in to Steam, then run this again"
    exit 1
}
Take-Lock
# (A game left for the headset keeps the lock until -Close; anything else gives it back.)
$keepLock = $false
try {
    # Nothing of anyone's is running now that the lock's ours: a game left over goes.
    Put-Away
    if (-not $NoBuild) {
        $b = cmd /c "cargo build -q --release -p rustvr-oblivion --target i686-pc-windows-msvc -j $Jobs 2>&1 && cargo build -q --release --manifest-path framework\Cargo.toml --target-dir `"$target`" -p rustvr-runtime -p rustvr-range -p rustvr-xrsim -j $Jobs 2>&1"
        if ($LASTEXITCODE -ne 0) { "build failed:"; $b; Give-Lock; exit 1 }
    }
    python tools\install_oblivion.py | Select-Object -First 1

    # The console lines: the save as soon as the main menu is up, then the rest once in the game.
    $lines = @()
    if ($NewGame) { $lines += "menu NewGame"; $Save = "a new game" } elseif ($Save) { $lines += "menu LoadGame `"$Save`"" }
    if ($At) { $lines += "game+0.5 coc $At" }
    $t = if ($At) { 4 } else { 1 }
    # (From the Bash tool a list arrives as one string: items split on commas, commands on "|".)
    $Give = @($Give | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    $Commands = @($Commands | ForEach-Object { $_ -split '\|' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    foreach ($g in $Give) { $lines += "game+$t player.additem $g" }
    if ($Hour -ge 0) { $lines += "game+$t set gamehour to $Hour" }
    if ($Weather) { $lines += "game+$t fw $Weather" }
    foreach ($c in $Commands) { $t += 0.5; $lines += "game+$t $c" }
    if ($ConsoleFile) { $lines += Get-Content $ConsoleFile }
    Set-Content "$game\rustvr-console-test" ($lines -join "`n")

    $emulated = $Sim -or $Hold -gt 0
    $toml = @("background = $($emulated.ToString().ToLower())") + (($Settings -split ";" | ForEach-Object { $_.Trim() }) | Where-Object { $_ })
    Set-Content "$game\rustvr-oblivion.toml" (($toml -join "`n") + "`n")

    $env:SteamAppId = "22330"
    $env:SteamGameId = "22330"
    if (-not $emulated) {
        # The real headset: the game as the player plays it; its runtime starts with it.
        $g = Start-Process -FilePath "$game\Oblivion.exe" -WorkingDirectory $game -PassThru
        Start-Sleep 3
        Get-Process OblivionLauncher -ErrorAction SilentlyContinue | Stop-Process -Force
        $og = Get-Process Oblivion -ErrorAction SilentlyContinue | Select-Object -First 1
        Write-Lock $(if ($og) { $og.Id } else { $g.Id })
        $keepLock = $true
        "Oblivion $($og.Id) is up for the headset, loading $Save; close it with: powershell -File tools\oblivion.ps1 -Close"
        exit 0
    }

    if (-not $Out) { $Out = Join-Path $repo "captures\run" }
    if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
    New-Item -ItemType Directory -Force $Out | Out-Null
    if (-not $Sim) {
        # Held still: the head at standing height, the hands at rest.
        $Sim = Join-Path $Out "hold.toml"
        Set-Content $Sim "duration = $Hold`n`n[[key]]`nt = 0`nhead = { pos = [0, 1.7, 0], yaw = 0, pitch = 0 }`n"
    }
    $Sim = (Resolve-Path $Sim).Path
    # (The adapter's log goes on growing across runs: this run's part is copied.)
    $adapterLog = "$game\rustvr-oblivion.log"
    $logFrom = if (Test-Path $adapterLog) { (Get-Item $adapterLog).Length } else { 0 }
    $emu = Start-Process -FilePath "$target\release\rustvr-xrsim.exe" `
        -ArgumentList "run", "--dir", "`"$Out`"", "--script", "`"$Sim`"", "--", "`"$target\release\rustvr-play.exe`"", "--no-range" `
        -RedirectStandardError "$Out\xrsim_err.txt" -RedirectStandardOutput "$Out\xrsim_out.txt" -WindowStyle Hidden -PassThru
    Start-Sleep 2
    $gamePid = (@(python tools\start_hidden.py "$game\Oblivion.exe")[0]).Trim()
    Write-Lock ([int]$gamePid)
    "emulated headset $($emu.Id), Oblivion $gamePid loading $Save; results in $Out"
    # (By its id: a Start-Process object whose handle wasn't held from the start never sees its exit.)
    while (Get-Process -Id $emu.Id -ErrorAction SilentlyContinue) {
        if (Get-Process OblivionLauncher -ErrorAction SilentlyContinue) {
            "Steam restarted Oblivion through its launcher: the game didn't start (run this from the Bash tool)"
            $emu | Stop-Process -Force
            break
        }
        Start-Sleep -Milliseconds 200
    }
    Put-Away
    Copy-Item "$env:LOCALAPPDATA\RustVR\rustvr.log" "$Out\runtime.log" -Force -ErrorAction SilentlyContinue
    if (Test-Path $adapterLog) {
        $fs = [System.IO.File]::Open($adapterLog, 'Open', 'Read', 'ReadWrite')
        $len = $fs.Length
        if ($len -lt $logFrom) { $logFrom = 0 }
        $null = $fs.Seek($logFrom, 'Begin')
        $buf = New-Object byte[] ($len - $logFrom)
        $null = $fs.Read($buf, 0, $buf.Length)
        $fs.Close()
        [System.IO.File]::WriteAllBytes("$Out\adapter.log", $buf)
    }
    Get-Content "$Out\adapter.log" -ErrorAction SilentlyContinue | Select-String "console: " | Select-Object -Last 12 | ForEach-Object { $_.Line }
    Get-ChildItem "$Out\captures" -Filter *.png -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch '_raw' } | ForEach-Object { $_.FullName }
} finally {
    if (-not $keepLock) { Give-Lock }
}
