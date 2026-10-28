# Playtest Oblivion on the emulated headset (framework/crates/xrsim, docs/XRSIM_OBLIVION.md): builds
# the adapter (32-bit) and the runtime and emulator (from framework/), installs the adapter, runs
# rustvr-play on the emulated headset, and starts Oblivion on a hidden desktop of its own (it never
# shows or takes the focus). Blocks until the script's duration is up, then closes Oblivion. Usage:
#   powershell -File tools\xrsim_oblivion.ps1 -Script tools\xrsim\oblivion_boot.toml [-Out captures\xrsim] [-NoBuild] [-Console <file>]
# -Console: a `rustvr-console-test` file (timed console commands, crates/oblivion/src/console.rs),
# e.g. to load a save.
param([Parameter(Mandatory)][string]$Script, [string]$Out = "", [switch]$NoBuild, [string]$Console = "")
$ErrorActionPreference = "Continue"
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
if (-not $env:RUSTVR_DESKTOP_VIEW) { $env:RUSTVR_DESKTOP_VIEW = "0" }
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo
if (-not (Test-Path "$repo\framework\Cargo.toml")) { "the framework isn't checked out: git submodule update --init"; exit 1 }
if (-not $Out) { $Out = Join-Path $repo "captures\xrsim" }
$Script = (Resolve-Path $Script).Path
$game = "${env:ProgramFiles(x86)}\Steam\steamapps\common\Oblivion"
# Only runtimes started for testing (and any earlier emulator run) are closed.
Get-CimInstance Win32_Process -Filter "Name = 'rustvr-xrsim.exe' OR Name = 'rustvr-play.exe' OR Name = 'rustvr.exe'" |
    Where-Object { $_.Name -eq 'rustvr-xrsim.exe' -or $_.CommandLine -match '--attach|--game-pid|--exit-with-game|--no-range' } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Get-Process Oblivion -ErrorAction SilentlyContinue | Stop-Process -Force
if (-not $NoBuild) {
    $build = cmd /c "cargo build -q --release -p rustvr-oblivion --target i686-pc-windows-msvc -j 8 2>&1 && cargo build -q --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-runtime -p rustvr-xrsim -j 8 2>&1"
    if ($LASTEXITCODE -ne 0) { "build failed:"; $build; exit 1 }
}
python tools\install_oblivion.py | Select-Object -First 1
# Tests run in the background; the setting goes back afterwards.
Set-Content "$game\rustvr-oblivion.toml" "background = true`n"
if ($Console) { Copy-Item $Console "$game\rustvr-console-test" -Force }
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force $Out | Out-Null
$sim = Start-Process -FilePath "$repo\target\release\rustvr-xrsim.exe" `
    -ArgumentList "run", "--dir", "`"$Out`"", "--script", "`"$Script`"", "--", "`"$repo\target\release\rustvr-play.exe`"", "--no-range" `
    -RedirectStandardError "$Out\xrsim_err.txt" -RedirectStandardOutput "$Out\xrsim_out.txt" -WindowStyle Hidden -PassThru
Start-Sleep 2
# Straight from its exe (Steam running): SteamAppId keeps Steam's DRM from restarting it through Steam.
# That fails when this script runs in a sandbox (the DRM can't reach Steam): Steam then opens
# Oblivion's launcher on the desktop. Run it from an ordinary shell; the launcher is closed at once
# if it shows anyway.
$env:SteamAppId = "22330"
$env:SteamGameId = "22330"
$gamePid = (python tools\start_hidden.py "$game\Oblivion.exe")[0]
"emulated headset $($sim.Id), Oblivion $gamePid starting; captures in $Out\captures"
while (-not $sim.HasExited) {
    $launcher = Get-Process OblivionLauncher -ErrorAction SilentlyContinue
    if ($launcher) {
        $launcher | Stop-Process -Force
        "Steam restarted Oblivion through its launcher (closed): the game didn't start; stopping"
        $sim | Stop-Process -Force
        break
    }
    Start-Sleep -Milliseconds 100
}
Get-Process Oblivion -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1
# The player's settings back (a killed game doesn't restore them itself until its next start).
Set-Content "$game\rustvr-oblivion.toml" "background = false`n"
python tools\install_oblivion.py --restore-ini
Remove-Item "$game\rustvr-console-test" -ErrorAction SilentlyContinue
Copy-Item "$env:LOCALAPPDATA\RustVR\rustvr.log" "$Out\runtime.log" -Force -ErrorAction SilentlyContinue
Copy-Item "$game\rustvr-oblivion.log" "$Out\adapter.log" -Force -ErrorAction SilentlyContinue
Get-ChildItem "$Out\captures" -Filter *.png -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch '_raw' } | ForEach-Object { $_.FullName }
