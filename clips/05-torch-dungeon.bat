@echo off
title Trailer 05: a torch in a dark Ayleid ruin
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-torches: Vilverin, at the
rem top of the stairs down into the dark Ayleid ruin, a burning torch equipped at your right hip (two
rem more in the pack). No spell is selected (SelectSpell none): nothing in your off hand, so the
rem torch is the only light you carry. Your RustVR settings are left as they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-torches -Commands "SelectSpell none"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're at the top of the stairs into Vilverin, the torch burning at your
echo right hip. Reach either hand to the hip and grip to draw it.
echo.
echo For the shot: hold the torch up and walk down into the dark. Wave it past the walls and pillars,
echo hold it close to your armour and hands: it lights them as the game lights everything, and the
echo shadows move with it.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-05-torch-dungeon-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-05-torch-dungeon-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
