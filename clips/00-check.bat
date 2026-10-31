@echo off
title Trailer 00: check the trailer build and the desktop view
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem The shot's look in the RustVR menu's Studio tab (your own RustVR settings come back when it closes).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Look cinematic
rem Installs the trailer build (main, <builds>\trailer) and boots Oblivion into rustvr-weye: open
rem ground on Lake Rumare's shore, at 1 pm in clear weather.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-weye -Commands "set gamehour to 13|fw 00038EEE"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on and watch the monitor: the desktop view should be up, in the Cinematic look
echo (a 90 degree lens, warm glow, teal shadows, a little grain). Turn, walk, run. Other looks are in
echo the RustVR menu's Studio tab; whatever you change there goes back when this closes.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Restore
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-00-check-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-00-check-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed and your settings are back.
pause
exit /b 0
:failed
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Restore
pause
exit /b 1
