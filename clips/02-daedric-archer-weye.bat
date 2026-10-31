@echo off
title Trailer 02: Daedric armour and a Daedric bow, in Weye
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main, <builds>\trailer) and boots rustvr-archery, then takes you to
rem Weye, the hamlet across the bridge from the Imperial City: outside the Wawnet Inn's door, where the
rem game puts you coming out of it, facing down into the hamlet (its houses, the stables, the Legion
rem guard, people about) at 1 pm in clear weather. The Daedric cuirass, greaves, boots and gauntlets
rem are put on (the iron taken off), the Daedric Bow goes on your back and 100 Daedric Arrows in your
rem quiver; Strength 100 so the armour never weighs you down. Your RustVR settings are left as they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-archery -At WawnetInnExterior -Give "0003635B 1,0003635A 1,00036359 1,00036358 1,00035E7C 1,0001EFD3 100" -Commands "player.setpos x -459|player.setpos y 63928|player.setpos z 1125|player.setangle z 179|set gamehour to 13|fw 00038EEE|player.setav strength 100|player.removeitem 0001C6D1 1|player.removeitem 0001C6D2 1|player.removeitem 0001C6D0 1|player.equipitem 0003635B|player.equipitem 0003635A|player.equipitem 00036359|player.equipitem 00036358|player.equipitem 00035E7C|EquipAll 0001EFD3"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're outside the Wawnet Inn in Weye in Daedric armour. Give it about fifteen
echo seconds after the load for the armour and the bow to go on.
echo.
echo For the shot: hold your hands up and look at the gauntlets, look down at the armour. Reach over
echo your left shoulder for the bow, over your right for an arrow, nock it and draw to your cheek:
echo the desktop view zooms in as you aim. Loose at something across the hamlet. Walk down among
echo the houses past the people.
echo.
echo Press a key here when you're done to close the game.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem Keeps this run's runtime log in the next clip-<shot>-<n> folder: the next start replaces it.
powershell -NoProfile -Command "$n = 1; while (Test-Path ('<builds>\trailer\clip-02-daedric-archer-weye-' + $n)) { $n++ }; $d = '<builds>\trailer\clip-02-daedric-archer-weye-' + $n; New-Item -ItemType Directory -Force $d | Out-Null; Copy-Item '%LOCALAPPDATA%\RustVR\rustvr.log' $d -ErrorAction SilentlyContinue"
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
