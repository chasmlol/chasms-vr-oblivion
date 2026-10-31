@echo off
title Trailer 06: swimming in Lake Rumare
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-swim: floating in Lake
rem Rumare just south of the Imperial Bridge, east of Weye, at 1 pm, the weather held clear (the save's
rem can turn to rain). The rocks and the bridge are to the north, open water east and south. Your
rem RustVR settings are left as they are.
rem Every spell and power is taken away (RemoveAllSpells): nothing in your hands but what you take.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-swim -Commands "set gamehour to 13|fw 00038EEE|RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're floating in the lake.
echo.
echo For the shot: put a hand under the water, squeeze and pull to swim (breaststroke or crawl).
echo Pull the water up to dive, push it down to come up. Duck your face under for the underwater fog,
echo slap the surface for a splash. Swim north to the rocks under the bridge and stand up.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-06-swimming-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-06-swimming-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
