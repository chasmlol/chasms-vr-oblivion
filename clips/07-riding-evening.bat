@echo off
title Trailer 07: riding your own horse out of Chorrol, evening
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-horse: Chorrol, just
rem outside its stables, your own horse (My Chestnut Horse, yours) about two metres behind you, a
rem little to the left; the paddock's two are the stable's. The hour set to 7 pm, the weather held
rem clear. Your RustVR settings are left as they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-horse -Commands "set gamehour to 19|fw 00038EEE"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're outside Chorrol's stables at 7 pm. Your horse stands about two metres
echo behind you, a little to the left.
echo.
echo For the shot: turn round, take hold of the saddle to get on, take the reins in your hands and
echo ride out along the road, walking, then galloping.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-07-riding-evening-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-07-riding-evening-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
