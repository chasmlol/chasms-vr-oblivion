@echo off
title Trailer 03: glass armour in the Imperial City, evening
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-base (the Nord), then takes
rem you to the Imperial City's Market District at 7 pm in clear weather. The glass cuirass, greaves,
rem boots and gauntlets are put on (the iron taken off), and no spell is selected (SelectSpell none):
rem nothing in your off hand, its trigger the game's use. Your RustVR settings are left as they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-base -At ICMarketDistrict -Give "00036343 1,00036342 1,00036341 1,00036340 1" -Commands "set gamehour to 19|fw 00038EEE|player.removeitem 0001C6D1 1|player.removeitem 0001C6D2 1|player.removeitem 0001C6D0 1|player.equipitem 00036343|player.equipitem 00036342|player.equipitem 00036341|player.equipitem 00036340|SelectSpell none"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're in the Imperial City's Market District at 7 pm, in glass armour, with
echo no spell in your hands. Give it about ten seconds after the load for the armour to go on.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-03-glass-imperial-city-evening-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-03-glass-imperial-city-evening-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
