# Where this machine keeps the game, the feature worktrees and their builds, for the scripts in
# tools\ and clips\ (dot-sourced: . "$PSScriptRoot\paths.ps1"). Each is an environment variable,
# else a line in %LOCALAPPDATA%\RustVR\dev.toml (tools\dev.example.toml), else a default:
#
#   $Game       Oblivion's folder: RUSTVR_OBLIVION_DIR, dev.toml's game, else where the game's
#               installer (its registry key) or Steam's libraries have it; $null if neither does.
#   $Worktrees  the folder of feature worktrees tools\play.ps1 plays: RUSTVR_WORKTREES, dev.toml's
#               worktrees, else the folder this checkout is in.
#   $Builds     where each worktree builds (<Builds>\<its name>): RUSTVR_BUILDS, dev.toml's builds,
#               else "" (each worktree builds into its own target\).
#   $LockFile   the game lock, one for the machine: in $Builds, else in %LOCALAPPDATA%\RustVR.
#
# Build-Dir <worktree> is that worktree's build folder; Get-DevSetting reads dev.toml.

$DevConfig = Join-Path $env:LOCALAPPDATA "RustVR\dev.toml"

# A string value from dev.toml (`key = 'value'` or `key = "value"`, under `[section]` if given).
function Get-DevSetting([string]$Key, [string]$Section = "") {
    if (-not (Test-Path $DevConfig)) { return "" }
    $in = ""
    foreach ($line in Get-Content $DevConfig) {
        $t = $line.Trim()
        if ($t -match '^\[([^\]]+)\]') { $in = $Matches[1].Trim(); continue }
        if ($in -ne $Section) { continue }
        if ($t -match "^$([regex]::Escape($Key))\s*=\s*'([^']*)'") { return $Matches[1] }
        if ($t -match "^$([regex]::Escape($Key))\s*=\s*`"((?:[^`"\\]|\\.)*)`"") { return ($Matches[1] -replace '\\(.)', '$1') }
    }
    ""
}

# Oblivion's folder as the game's installer left it in the registry, else in one of Steam's libraries.
function Find-Oblivion {
    $p = (Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Bethesda Softworks\Oblivion" -ErrorAction SilentlyContinue).'Installed Path'
    if ($p -and (Test-Path (Join-Path $p "Oblivion.exe"))) { return $p.TrimEnd('\') }
    $steam = (Get-ItemProperty "HKCU:\Software\Valve\Steam" -ErrorAction SilentlyContinue).SteamPath
    if (-not $steam) { return $null }
    $libraries = @($steam)
    $vdf = Join-Path $steam "steamapps\libraryfolders.vdf"
    if (Test-Path $vdf) {
        $libraries += Select-String -Path $vdf -Pattern '"path"\s+"([^"]+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value -replace '\\\\', '\' }
    }
    foreach ($l in $libraries) {
        $g = Join-Path $l "steamapps\common\Oblivion"
        if (Test-Path (Join-Path $g "Oblivion.exe")) { return (Resolve-Path $g).Path }
    }
    $null
}

$Game = if ($env:RUSTVR_OBLIVION_DIR) { $env:RUSTVR_OBLIVION_DIR } else { Get-DevSetting "game" }
if (-not $Game) { $Game = Find-Oblivion }
$Worktrees = if ($env:RUSTVR_WORKTREES) { $env:RUSTVR_WORKTREES } else { Get-DevSetting "worktrees" }
if (-not $Worktrees) { $Worktrees = Split-Path -Parent (Split-Path -Parent $PSScriptRoot) }
$Builds = if ($env:RUSTVR_BUILDS) { $env:RUSTVR_BUILDS } else { Get-DevSetting "builds" }
$LockFile = if ($Builds) { Join-Path $Builds "game.lock" } else { Join-Path $env:LOCALAPPDATA "RustVR\game.lock" }

function Build-Dir([string]$Worktree) {
    if ($Builds) { Join-Path $Builds $Worktree } else { Join-Path (Join-Path $Worktrees $Worktree) "target" }
}
