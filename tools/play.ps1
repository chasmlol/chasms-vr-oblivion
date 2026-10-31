# Playtest a feature (docs/DEVELOPING.md): its worktree's adapter and runtime, as built in that
# worktree's build folder, installed, and Oblivion booted straight into a save for the real headset;
# or (-Range) the framework's test range from that build. -Close puts the game away (whichever
# worktree has it) and the player's settings back. Where the worktrees and their builds are is this
# machine's own (tools\paths.ps1).
#
#   powershell -File tools\play.ps1                      the features there are
#   powershell -File tools\play.ps1 archery [-Save rustvr-weye | -NewGame] [-At <cell>] [-Give "<id> <n>,..."] [-Commands "a|b"]
#   powershell -File tools\play.ps1 archery -Range
#   powershell -File tools\play.ps1 -Close
#   powershell -File tools\play.ps1 trailer -Close -KeepLogs clip-02 [-Logs "rustvr.log,voice.log"]
#
# -Commands runs console lines once the save is loaded ("showracemenu", "player.setav sneak 100"):
# in VR the real keyboard doesn't reach the game, so its console can't be typed into. -NewGame starts
# a new game instead of loading a save (the prison cell and the race menu). -Settings adds lines to
# the game's rustvr-oblivion.toml for this run ("menu_world = false; menu_spot = cloud-ruler").
# -KeepLogs, with -Close, copies the run's logs (%LOCALAPPDATA%\RustVR's rustvr.log, or -Logs) into
# the next free <name>-<n> folder of the feature's build folder: the next start replaces them.
param([string]$Feature = "", [string]$Save = "", [switch]$NewGame, [string]$At = "", [string]$Give = "", [string]$Commands = "", [string]$Settings = "", [switch]$Range, [switch]$Close, [string]$KeepLogs = "", [string]$Logs = "rustvr.log")
$ErrorActionPreference = "Continue"
. "$PSScriptRoot\paths.ps1"

function Stop-Ranges {
    Get-CimInstance Win32_Process -Filter "Name = 'rustvr-play.exe' OR Name = 'rustvr.exe' OR Name = 'rustvr-range.exe'" |
        Where-Object { $_.CommandLine -match '--wait' -or $_.Name -eq 'rustvr-range.exe' } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}

if ($Close) {
    Stop-Ranges
    $owner = (Get-Content $LockFile -ErrorAction SilentlyContinue | Select-Object -First 1) -split '\|' | Select-Object -First 1
    $tool = if ($owner -and (Test-Path "$owner\tools\oblivion.ps1")) { "$owner\tools\oblivion.ps1" } else { "$PSScriptRoot\oblivion.ps1" }
    & $tool -Close
    if ($KeepLogs) {
        $base = if ($Feature) { Build-Dir $Feature } else { Join-Path (Split-Path -Parent $PSScriptRoot) "captures" }
        $n = 1
        while (Test-Path (Join-Path $base "$KeepLogs-$n")) { $n++ }
        $d = Join-Path $base "$KeepLogs-$n"
        New-Item -ItemType Directory -Force $d | Out-Null
        foreach ($log in ($Logs -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
            Copy-Item (Join-Path $env:LOCALAPPDATA "RustVR\$log") $d -ErrorAction SilentlyContinue
        }
        "the logs are in $d"
    }
    exit 0
}
if (-not $Feature -or -not (Test-Path "$Worktrees\$Feature")) {
    "features to play:"
    Get-ChildItem $Worktrees -Directory -ErrorAction SilentlyContinue | ForEach-Object {
        $ready = Test-Path "$($_.FullName)\PLAYTEST.md"
        "  $($_.Name)$(if ($ready) { '' } else { '  (no PLAYTEST.md yet)' })"
    }
    exit 1
}
$wt = "$Worktrees\$Feature"
$env:CARGO_TARGET_DIR = Build-Dir $Feature
if ($Range) {
    # (The game's runtime and the range share the runtime's link: the game goes first.)
    & "$PSScriptRoot\play.ps1" -Close | Out-Null
    Start-Process -FilePath "$env:CARGO_TARGET_DIR\release\rustvr-play.exe" -ArgumentList "--wait"
    "the test range from $Feature's build is up (rustvr-play --wait)"
    exit 0
}
# Steam's DRM starts the game only for a signed-in Steam: signed out, the game stops at an
# "Application load error" box on the desktop, out of the headset's sight.
if (-not (Get-ItemProperty "HKCU:\Software\Valve\Steam\ActiveProcess" -ErrorAction SilentlyContinue).ActiveUser) {
    "Steam isn't signed in: sign in to Steam, then run this again"
    exit 1
}
Stop-Ranges
if ($NewGame) {
    "$Feature : a new game"
    & "$wt\tools\oblivion.ps1" -NoBuild -NewGame -At $At -Give $Give -Commands $Commands -Settings $Settings
    exit 0
}
if (-not $Save) {
    # The save the feature's PLAYTEST.md names first (a line "Save: rustvr-..."), else the city.
    $named = Select-String -Path "$wt\PLAYTEST.md" -Pattern 'Save:\s*(\S+)' -ErrorAction SilentlyContinue | Select-Object -First 1
    $Save = if ($named) { $named.Matches[0].Groups[1].Value.Trim('`', '"') } else { "rustvr-city" }
}
"$Feature : $Save$(if ($At) { " at $At" })"
& "$wt\tools\oblivion.ps1" -NoBuild -Save $Save -At $At -Give $Give -Commands $Commands -Settings $Settings
