@echo off
title Trailer 04: alchemy at a table
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-alchemy (Odiil Farm's
rem farmhouse, standing in its entrance), then sets out one table on your right (alchemy-table.txt):
rem the Journeyman mortar and pestle, alembic, calcinator and retort, Flax Seeds and Steel-Blue
rem Entoloma Cap for Restore Magicka on the left, Boar Meat and Venison for Restore Health on the
rem right. Your RustVR settings are left as they are.
rem Every spell and power is taken away (RemoveAllSpells): nothing in your hands but what you take.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-alchemy -Commands "RunFile %~dp0alchemy-table.txt|RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're in the farmhouse's entrance. A few seconds in, a table stands on your
echo right with the alchemy set at the back and the ingredients at the front.
echo.
echo For the shot: step up to the table, drop the Boar Meat and the Venison into the bowl, take the
echo pestle by its handle and grind round the bottom of the bowl: the Restore Health potion appears.
echo Then the same with the Flax Seeds and the Steel-Blue Entoloma Cap for Restore Magicka.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-04-alchemy-table-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-04-alchemy-table-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
