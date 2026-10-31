@echo off
title Trailer 03: glass armour in the Imperial City, evening
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem Installs the trailer build (main) and boots rustvr-base (the Nord), then takes
rem you to the Imperial City's Market District (its street cell ICMarketDistrict03: the district is a
rem worldspace, which coc can't go to) at 7 pm in clear weather, where the game puts you coming out of
rem The Merchants Inn, facing up the street. The glass cuirass, greaves, boots and gauntlets are put on
rem (the iron taken off), and every spell and power is taken away (RemoveAllSpells): nothing in your
rem hands, nothing to pick. Sneak is 100 (master: crouch a little and you're hardly seen, pockets
rem picked at 85%). Your RustVR settings are left as they are.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-base -At ICMarketDistrict03 -Give "00036343 1,00036342 1,00036341 1,00036340 1" -Commands "player.setpos x 33298|player.setpos y 68771|player.setpos z 3536|player.setangle z 41|set gamehour to 19|fw 00038EEE|player.setav sneak 100|player.removeitem 0001C6D1 1|player.removeitem 0001C6D2 1|player.removeitem 0001C6D0 1|player.equipitem 00036343|player.equipitem 00036342|player.equipitem 00036341|player.equipitem 00036340|RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're in the Imperial City's Market District at 7 pm, outside The Merchants
echo Inn, in glass armour, Sneak 100, with no spells at all. Give it about fifteen seconds after the
echo load for the armour to go on and the spells to go. Crouch a little to sneak.
echo.
echo Press a key here when you're done to close the game.
pause >nul
rem Closes the game and keeps this run's runtime log in the trailer build's next clip-03-glass-imperial-city-evening-<n> folder
rem (tools\play.ps1 -KeepLogs), as the next start replaces it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Close -KeepLogs clip-03-glass-imperial-city-evening
echo The game is closed.
pause
exit /b 0
:failed
pause
exit /b 1
