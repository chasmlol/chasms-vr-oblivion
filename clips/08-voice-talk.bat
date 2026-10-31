@echo off
title Trailer 08: talking to people out loud
rem Puts away a game another .bat may have left running (else this boot waits for its lock).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" -Close
rem The trailer is recorded from the desktop view: tools\oblivion.ps1 turns it off unless it's asked for.
set RUSTVR_DESKTOP_VIEW=1
rem The voice servers (voice-servers.ps1): speech to text (Parakeet) and Breeze (the cloned voices),
rem started if they aren't up and waited for, then RustVR's Voice setting turned on (it stays on: the
rem one setting this .bat changes). The DeepSeek key is your own, already in
rem %LOCALAPPDATA%\RustVR\voice.toml.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0voice-servers.ps1"
if errorlevel 1 goto failed
rem Installs the trailer build (main with voice) and boots rustvr-voice: The
rem Merchants Inn in the Imperial City's Market District, people about, with the Adoring Fan put a
rem couple of steps in front of you.
rem Every spell and power is taken away (RemoveAllSpells): nothing in your hands but what you take.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Save rustvr-voice -Commands "player.placeatme 000AED40 1 150 0|RemoveAllSpells"
if errorlevel 1 goto failed
echo.
echo Connect the headset in Virtual Desktop if it isn't: RustVR waits for it, and the desktop view
echo opens on the monitor once it's there.
echo.
echo Put the headset on: you're in The Merchants Inn, the Adoring Fan a couple of steps in front of
echo you. Look at someone's face from a few steps away and talk: they turn to you and answer in their
echo own voice, their lips moving with it. Their prompts and lines show on http://127.0.0.1:8090
echo.
echo Press a key here when you're done to close the game.
pause >nul
rem Closes the game and keeps this run's runtime and voice logs in the trailer build's next clip-08-voice-talk-<n> folder
rem (tools\play.ps1 -KeepLogs), as the next start replaces them.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\tools\play.ps1" trailer -Close -KeepLogs clip-08-voice-talk -Logs "rustvr.log,voice.log"
echo The game is closed. The voice servers are still running.
pause
exit /b 0
:failed
pause
exit /b 1
