@echo off
title Trailer 05: a torch in a dark Ayleid ruin
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main) and boots rustvr-torches: Vilverin, at the
rem top of the stairs down into the dark Ayleid ruin, a burning torch equipped at your right hip (two
rem more in the pack). Every spell and power is taken away (RemoveAllSpells): nothing in your hands
rem but what you take, so the torch is the only light you carry. Your RustVR settings are left as
rem they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-torches -Commands "RemoveAllSpells"
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
rem Closes the game and keeps this run's runtime log in the trailer build's next clip-05-torch-dungeon-<n> folder
rem (tools\play.ps1 -KeepLogs), as the next start replaces it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Close -KeepLogs clip-05-torch-dungeon
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
