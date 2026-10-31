@echo off
title Trailer 01: shaping your face by hand
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Cinematic: a 90 degree lens, the focus pulled to the head you're pulling at, the cell behind it
rem soft (your own RustVR settings come back when this closes).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Look cinematic
rem Installs the trailer build (main, <builds>\trailer) and starts a new game, its film skipped:
rem the prison cell, the race menu opening by itself, as a player starting out has it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -NewGame
if errorlevel 1 goto failed
echo.
echo Put the headset on: a new game is starting. In the prison cell the race menu opens by itself,
echo the mirror in front of you, the menu at its left and a big copy of your head to your right.
echo.
echo For the shot: grab the copy's nose and pull it out long, then the ears out, the eyes apart,
echo the lips fuller. Hold the trigger while pulling the nose sideways to bend it crooked. Reach up
echo to your own chin and ears and pull them, watching the mirror. Move your head slowly: the
echo monitor shows your right eye's view.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Restore
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-01-face-sculpt-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-01-face-sculpt-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed and your settings are back.
pause
exit /b 0
:failed
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0studio.ps1" -Restore
pause
exit /b 1
