@echo off
title RustVR worldmenu: Cloud Ruler Temple in the Jerall Mountains, morning
rem A candidate spot for the main menu: Cloud Ruler Temple in the Jerall Mountains, morning.
rem Puts away a game another feature's .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
rem Boots Oblivion into rustvr-weye, then the game's own coc to CloudRulerTempleExterior02, the player set on the spot facing the view,
rem then the hour and the weather (Clear), all with the game's own console lines.
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" worldmenu -Save rustvr-weye -At CloudRulerTempleExterior02 -Commands "player.setpos x 13391|player.setpos y 160255|player.setpos z 31624|player.setangle z 171|set gamehour to 8.5|fw 00038EEE"
echo.
echo Cloud Ruler Temple in the Jerall Mountains, morning:
echo on the snow outside the Blades' temple, facing south over Bruma and the Great Forest towards the Imperial City.
echo Put the headset on and look around (walk a little if you like). Press a key here when you're done.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
echo The game is closed and your settings are back.
pause
