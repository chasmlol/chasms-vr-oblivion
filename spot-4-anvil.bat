@echo off
title RustVR worldmenu: Anvil's lighthouse on the Gold Coast at sunset
rem A candidate spot for the main menu: Anvil's lighthouse on the Gold Coast at sunset.
rem Puts away a game another feature's .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
rem Boots Oblivion into rustvr-weye, then the game's own CenterOnWorld to the spot's cell, the player set on the spot facing the view,
rem then the hour and the weather (Clear), all with the game's own console lines.
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" worldmenu -Save rustvr-weye -Commands "cow Tamriel -48 -10|player.setpos x -196600|player.setpos y -39000|player.setpos z 770|player.setangle z 300|set gamehour to 19.0|fw 00038EEE"
echo.
echo Anvil's lighthouse on the Gold Coast at sunset:
echo on the rocks south of the lighthouse, facing north-west over the Abecean Sea.
echo Put the headset on and look around (walk a little if you like). Press a key here when you're done.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
echo The game is closed and your settings are back.
pause
