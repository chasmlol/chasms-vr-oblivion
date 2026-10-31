@echo off
title Trailer 01: shaping your face by hand
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main) and starts a new game, its film skipped:
rem the prison cell, the race menu opening by itself, as a player starting out has it. Your RustVR
rem settings are left as they are.
rem Every spell and power is taken away (RemoveAllSpells): nothing in your hands but what you take.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -NewGame -Commands "RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
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
rem Closes the game and keeps this run's runtime log in the trailer build's next clip-01-face-sculpt-<n> folder
rem (tools\play.ps1 -KeepLogs), as the next start replaces it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Close -KeepLogs clip-01-face-sculpt
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
