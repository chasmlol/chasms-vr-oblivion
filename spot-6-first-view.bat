@echo off
title RustVR worldmenu: The first view: Lake Rumare from the prison sewer's exit, morning
rem A candidate spot for the main menu: The first view: Lake Rumare from the prison sewer's exit, morning.
rem Puts away a game another feature's .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
rem Boots Oblivion into rustvr-weye, then the game's own coc to ICPrisonSewerExit01, the player set on the spot facing the view,
rem then the hour and the weather (Clear), all with the game's own console lines.
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" worldmenu -Save rustvr-weye -At ICPrisonSewerExit01 -Commands "player.setpos x 47110|player.setpos y 83000|player.setpos z 327|player.setangle z 40|set gamehour to 9.0|fw 00038EEE"
echo.
echo The first view: Lake Rumare from the prison sewer's exit, morning:
echo where every game begins outside: at the sewer grate, facing Vilverin across the lake.
echo Put the headset on and look around (walk a little if you like). Press a key here when you're done.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
echo The game is closed and your settings are back.
pause
