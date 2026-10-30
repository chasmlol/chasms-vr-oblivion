@echo off
title RustVR worldmenu: The Imperial City across Lake Rumare at sunset, from the slope above Weye
rem A candidate spot for the main menu: The Imperial City across Lake Rumare at sunset, from the slope above Weye.
rem Puts away a game another feature's .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
rem Boots Oblivion into rustvr-weye, then the game's own coc to WeyeExterior, the player set on the spot facing the view,
rem then the hour and the weather (Clear), all with the game's own console lines.
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" worldmenu -Save rustvr-weye -At WeyeExterior -Commands "player.setpos x -2000|player.setpos y 66000|player.setpos z 1197|player.setangle z 95|set gamehour to 18.5|fw 00038EEE"
echo.
echo The Imperial City across Lake Rumare at sunset, from the slope above Weye:
echo facing east down the great bridge to the White-Gold Tower, the sun setting behind you.
echo Put the headset on and look around (walk a little if you like). Press a key here when you're done.
pause >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "<checkout>\tools\play.ps1" -Close
echo The game is closed and your settings are back.
pause
