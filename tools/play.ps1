# Playtest an overnight feature (docs/OVERNIGHT.md): its worktree's (<worktrees>\<feature>) adapter and
# runtime, as built in <builds>\<feature>, installed, and Oblivion booted straight into a save for
# the real headset; or (-Range) the framework's test range from that build. -Close puts the game away
# (whichever worktree has it) and the player's settings back.
#
#   powershell -File tools\play.ps1                      the features there are
#   powershell -File tools\play.ps1 archery [-Save rustvr-weye] [-At <cell>] [-Give "<id> <n>,..."]
#   powershell -File tools\play.ps1 archery -Range
#   powershell -File tools\play.ps1 -Close
param([string]$Feature = "", [string]$Save = "", [string]$At = "", [string]$Give = "", [switch]$Range, [switch]$Close)
$ErrorActionPreference = "Continue"
$worktrees = "<worktrees>"
$builds = "<builds>"

function Stop-Ranges {
    Get-CimInstance Win32_Process -Filter "Name = 'rustvr-play.exe' OR Name = 'rustvr.exe' OR Name = 'rustvr-range.exe'" |
        Where-Object { $_.CommandLine -match '--wait' -or $_.Name -eq 'rustvr-range.exe' } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}

if ($Close) {
    Stop-Ranges
    $owner = (Get-Content "$builds\game.lock" -ErrorAction SilentlyContinue | Select-Object -First 1) -split '\|' | Select-Object -First 1
    $tool = if ($owner -and (Test-Path "$owner\tools\oblivion.ps1")) { "$owner\tools\oblivion.ps1" } else { "$PSScriptRoot\oblivion.ps1" }
    & $tool -Close
    exit 0
}
if (-not $Feature -or -not (Test-Path "$worktrees\$Feature")) {
    "features to play:"
    Get-ChildItem $worktrees -Directory -ErrorAction SilentlyContinue | ForEach-Object {
        $ready = Test-Path "$($_.FullName)\PLAYTEST.md"
        "  $($_.Name)$(if ($ready) { '' } else { '  (no PLAYTEST.md yet)' })"
    }
    exit 1
}
$wt = "$worktrees\$Feature"
$env:CARGO_TARGET_DIR = "$builds\$Feature"
if ($Range) {
    # (The game's runtime and the range share the runtime's link: the game goes first.)
    & "$PSScriptRoot\play.ps1" -Close | Out-Null
    Start-Process -FilePath "$builds\$Feature\release\rustvr-play.exe" -ArgumentList "--wait"
    "the test range from $Feature's build is up (rustvr-play --wait)"
    exit 0
}
Stop-Ranges
if (-not $Save) {
    # The save the feature's PLAYTEST.md names first (a line "Save: rustvr-..."), else the city.
    $named = Select-String -Path "$wt\PLAYTEST.md" -Pattern 'Save:\s*(\S+)' -ErrorAction SilentlyContinue | Select-Object -First 1
    $Save = if ($named) { $named.Matches[0].Groups[1].Value.Trim('`', '"') } else { "rustvr-city" }
}
"$Feature : $Save$(if ($At) { " at $At" })"
& "$wt\tools\oblivion.ps1" -NoBuild -Save $Save -At $At -Give $Give
