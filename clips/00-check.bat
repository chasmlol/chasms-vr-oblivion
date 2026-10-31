@echo off
title Trailer 00: check the trailer build and the desktop view
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main) and boots Oblivion into rustvr-weye: open
rem ground on Lake Rumare's shore, at 1 pm in clear weather. Your RustVR settings are left as they are.
rem Every spell and power is taken away (RemoveAllSpells): nothing in your hands but what you take.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-weye -Commands "set gamehour to 13|fw 00038EEE|RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on and watch the monitor: the desktop view should be up, in front of the game's
echo window. Click it, or OBS: the game should go on taking your controls, with its sound.
echo Press a key here when you're done to close the game.
pause >nul
rem Closes the game and keeps this run's runtime log in the trailer build's next clip-00-check-<n> folder
rem (tools\play.ps1 -KeepLogs), as the next start replaces it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Close -KeepLogs clip-00-check
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
