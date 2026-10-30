@echo off
title RustVR worldmenu: Dive Rock, the highest lookout in Cyrodiil, early morning
rem A candidate spot for the main menu: Dive Rock, the highest lookout in Cyrodiil, early morning.
rem Puts away a game another feature's .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
rem Boots Oblivion into rustvr-weye, then the game's own coc to DiveRock, the player set on the spot facing the view,
rem then the hour and the weather (Clear), all with the game's own console lines.
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" worldmenu -Save rustvr-weye -At DiveRock -Commands "player.setpos x 138134|player.setpos y 142738|player.setpos z 34208|player.setangle z 234|set gamehour to 6.5|fw 00038EEE"
echo.
echo Dive Rock, the highest lookout in Cyrodiil, early morning:
echo on the summit, facing south-west over the Nibenay towards the Imperial City.
echo Put the headset on and look around (walk a little if you like). Press a key here when you're done.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
echo The game is closed and your settings are back.
pause
