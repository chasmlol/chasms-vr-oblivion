# Playtest Oblivion on the emulated headset: tools\oblivion.ps1 -Sim does it (the game lock, this
# checkout's builds, the player's settings put back), with no save loaded unless the console file
# loads one. Kept for older notes:
#   powershell -File tools\xrsim_oblivion.ps1 -Script tools\xrsim\oblivion_boot.toml [-Out captures\xrsim] [-NoBuild] [-Console <file>] [-Settings "a = 1; b = 2"]
param([Parameter(Mandatory)][string]$Script, [string]$Out = "", [switch]$NoBuild, [string]$Console = "", [string]$Settings = "")
if (-not $Out) { $Out = Join-Path (Split-Path -Parent $PSScriptRoot) "captures\xrsim" }
& "$PSScriptRoot\oblivion.ps1" -Save "" -Sim $Script -Out $Out -NoBuild:$NoBuild -ConsoleFile $Console -Settings $Settings
