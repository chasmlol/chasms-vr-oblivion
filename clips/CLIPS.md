# Trailer clips

Each `.bat` here boots Oblivion straight into one shot, set up and ready: put the headset on and
record the desktop view (the window on the monitor). Press a key in the bat's window when you're
done: it closes the game and puts your settings back.

Every bat:

- closes any game another `.bat` left running;
- turns the desktop view on (`RUSTVR_DESKTOP_VIEW=1`: `tools\oblivion.ps1` turns it off otherwise);
- sets the shot's look in the RustVR menu's Studio tab, and anything else the shot needs, with
  `studio.ps1` (your `settings.toml` is copied aside first and put back when the bat closes, so
  anything you change in the menu during the shot goes back too);
- installs the build it names and boots the save, moves you, gives you things and runs the console
  lines the shot needs (`tools\play.ps1`);
- keeps the run's `rustvr.log` in `<builds>\trailer\clip-<shot>-<n>`.

The trailer build is main as of `d2a3a62` (protocol v15), built in `<builds>\trailer`. Shots that
need a feature not in main yet boot that feature's own build (`play.ps1 <feature>`).

If a bat's window was closed without pressing a key, run `powershell -File clips\studio.ps1 -Restore`
(or the next bat's close does it): your settings sit in `%LOCALAPPDATA%\RustVR\settings.before-clip.toml`
until then.

## Shots

| Bat | Shot | Build | Look |
|---|---|---|---|
| `00-check.bat` | Check the build and the desktop view: Lake Rumare's shore at 1 pm | trailer | Cinematic |

## The looks (Studio tab)

| Look | For |
|---|---|
| Clean | the game's picture as it is; your own field of view (128 degrees now) |
| Cinematic | establishing shots, the menu, books, alchemy, faces: 90 degree lens, 24 fps blur, focus pulled to what you look at or hold up |
| Hyper-real | body-cam: fights, dungeon crawls, gore; chest-worn camera, fisheye, hard sharpening |
| Action | fights, archery, riding: shake on every blow, the view widens at a run |
| Vintage | a flashback or a joke beat: heavy grain, warm faded print |

`studio.ps1 -Look <look> -Set "spectator.fov=100;studio.grain=0"` sets a look and anything else
(`section.key=value`; a bare key is the studio's). Setting a studio value by hand makes the look
"custom", as the Studio tab does.

## Handy console lines for `-Commands`

- `set gamehour to 13`: the hour. `fw 00038EEE`: clear weather; `fw 00038EF2`: rain.
- `tgm`: god mode. `tai`: AI off (people freeze). `tcl`: no clipping.
- `player.placeatme <base id> 1 <distance> <direction>`: someone or something placed by you.
- The adapter's own: `PlaceAt <base> <ahead> <right> <up> [<turn>]`, `PlaceInFront <ref> <metres>`,
  `SelectSpell <name>`, `EquipAll <id>`, `PlayerHorse <base> [<metres>]`, `GoreCut <bone>`
  (`crates/oblivion/src/console.rs`).
