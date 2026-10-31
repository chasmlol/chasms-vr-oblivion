# Trailer clips

Each `.bat` here boots Oblivion straight into one shot, set up and ready: put the headset on and
record the desktop view (the window on the monitor). Press a key in the bat's window when you're
done: it closes the game.

Every bat:

- closes any game another `.bat` left running;
- turns the desktop view on for that run (`RUSTVR_DESKTOP_VIEW=1`: `tools\oblivion.ps1` turns it
  off otherwise);
- installs the build it names and boots the save, moves you, gives you things and runs the console
  lines the shot needs (`tools\play.ps1`);
- keeps the run's `rustvr.log` in `<builds>\trailer\clip-<shot>-<n>`.

The bats never touch your RustVR settings (`%LOCALAPPDATA%\RustVR\settings.toml`): the desktop
view's look, its field of view and the overscan are whatever you set in the RustVR menu, and stay
so. The looks are in the menu's Studio tab.

The trailer build is main as of `d2a3a62` (protocol v15), built in `<builds>\trailer`, with the
trailer branch's fixes: the game plays on (controls, clock, sound) with another window in front, the
desktop view opens in front of the game's window, the boot's New Game reaches the prison cell with
the main menu in Cyrodiil, a changed overscan takes at the next start, and the overscan goes up to
20. Shots that need a feature not in main yet boot that feature's own build (`play.ps1 <feature>`).

## Shots

| Bat | Shot | Build |
|---|---|---|
| `00-check.bat` | Check the build and the desktop view: Lake Rumare's shore at 1 pm | trailer |
| `01-face-sculpt.bat` | Shaping your face by hand: a new game, the prison cell's race menu, the big copy of your head and the mirror; pull the nose long, the ears out | trailer |
| `02-daedric-archer-weye.bat` | Daedric armour, the Daedric Bow on your back and Daedric Arrows, outside the Wawnet Inn in Weye at 1 pm (the hamlet across the bridge from the Imperial City, its houses and people) | trailer |
| `03-glass-imperial-city-evening.bat` | Glass armour in the Imperial City's Market District at 7 pm, no spell selected | trailer |
| `04-alchemy-table.bat` | Alchemy at one table in Odiil Farm's farmhouse: the Journeyman set, and Boar Meat and Venison (Restore Health), Flax Seeds and Steel-Blue Entoloma Cap (Restore Magicka) to grind (set out by `alchemy-table.txt`) | trailer |
| `05-torch-dungeon.bat` | A burning torch at your hip at the top of the stairs into Vilverin's dark Ayleid ruin, no spell selected | trailer |
| `06-swimming.bat` | Floating in Lake Rumare by the Imperial Bridge at 1 pm, clear | trailer |
| `07-riding-evening.bat` | Your own horse behind you outside Chorrol's stables at 7 pm, clear | trailer |

## The overscan

The game draws each eye wider than the headset shows by the overscan, and bigger to stay as sharp.
It can't change its size while it runs: a new overscan takes when the game starts again (the menu
says how much of it shows until then). With this headset (Virtual Desktop: about 54 degrees out, 40
in, 44 up and 55 down an eye):

| Overscan | Each eye | Pixels |
|---|---|---|
| 11 | 4669x5026 | 1x |
| 15 | 5591x6005 | 1.4x |
| 20 (the most) | 7249x7898 | 2.4x |

A 128 degree desktop view needs about 10 degrees to have picture at its edges with the head still,
about 18 for its camera's lag behind a turning head.

## Handy console lines for `-Commands`

- `set gamehour to 13`: the hour. `fw 00038EEE`: clear weather; `fw 00038EF2`: rain.
- `tgm`: god mode. `tai`: AI off (people freeze). `tcl`: no clipping.
- `player.placeatme <base id> 1 <distance> <direction>`: someone or something placed by you.
- The adapter's own: `PlaceAt <base> <ahead> <right> <up> [<turn>]`, `PlaceInFront <ref> <metres>`,
  `SelectSpell <name>` (`SelectSpell none`: no spell, nothing in the off hand), `EquipAll <id>`, `PlayerHorse <base> [<metres>]`, `GoreCut <bone>`
  (`crates/oblivion/src/console.rs`).
- Where a door leads (a spot to stand at): its teleport in `Oblivion.esm`, as clip 02's place was found.
