# Developing

How the Oblivion adapter is worked on: the rules, the builds, booting the game straight into a
test for a check or a playtest, the emulated headset, and feature worktrees. How the adapter itself
works is [OBLIVION.md](OBLIVION.md); the framework's side is
[framework/docs/DESIGN.md](../framework/docs/DESIGN.md).

## The rules

- **The game is the source of truth.** Its data, its models, its sounds and its effects: nothing
  hand-tuned where the game says, and no game effect dropped. Each feature goes through the game's
  own systems (its hit, its casting, its activation, its menus), however much work that is, rather
  than a workaround that would do for any game.
- **The adapter only translates.** If the adapter re-implements a VR mechanic, or the runtime a
  game rule, the design has gone wrong ([framework/docs/DESIGN.md](../framework/docs/DESIGN.md)).
- **Hands.** A hand touching nothing follows its controller exactly. Every weapon snaps to its
  rigged grip when grabbed near it. The gravity gloves pull things to the hand.
- **Research first.** A feature starts with research in `docs/research/<feature>.md`: what players
  like most in the best VR games that do it, how those games do it, what people complain about,
  and the game's own systems for it. Inspiration only: nothing copied.
- **None of the game's code is committed.** The adapter hooks and calls the game's own functions,
  and the comments say what each does; the game's code itself, copied or rewritten, never goes in
  the repositories.
- **Nothing of the game, and nothing of this machine, in the repositories.** Game files are
  extracted into the git-ignored `local_assets/`. Paths of your machine go in
  `%LOCALAPPDATA%\RustVR\dev.toml` or environment variables (below), keys in the voice plugin's
  `voice.toml` or an environment variable. `.gitignore` covers the usual suspects (logs, saves,
  game files, `.env`, `voice.toml`, `dev.toml`).

## Building

```bat
cargo build --release -p rustvr-oblivion --target i686-pc-windows-msvc
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-runtime -p rustvr-range -p rustvr-xrsim
```

The first is the 32-bit adapter (`target\i686-pc-windows-msvc\release\dinput8.dll`), the second
the runtime (`rustvr-play.exe`, `rustvr.exe`), the test range and the emulated headset, all into
the same target folder. `CARGO_TARGET_DIR` moves both (a worktree's own builds,
`tools/install_oblivion.py` reads it too). Add `-p rustvr-voice` for the voice plugin, which
nothing builds by default ([README.md](../README.md), "The voice plugin"). Unit tests:

```bat
cargo test --release -p rustvr-oblivion --target i686-pc-windows-msvc
cargo test --manifest-path framework\Cargo.toml --workspace
```

Tests that need the game's files (the fog test reads the game's shader package) find the game as
the tools do, and skip without it. The framework's C header,
`framework/crates/protocol/include/rustvr_protocol.h`, is checked against the Rust layout by a
test; after changing the protocol, run that test with `RUSTVR_BLESS=1` to rewrite it, and bump
`VERSION` (the mapping's name follows it). New regions chain after the last one in `LAYOUT`.

## The fast boot: `tools/oblivion.ps1`

Boots Oblivion for a check or a playtest in one command, about five seconds to in the game, nothing
to click:

```bat
powershell -ExecutionPolicy Bypass -File tools\oblivion.ps1 -Save rustvr-weye -Give "0000000A 5" -Hold 20 -Out captures\run
powershell -ExecutionPolicy Bypass -File tools\oblivion.ps1 -Close
```

It runs in this order:

1. takes the game lock (one game on the machine, whichever checkout asks; it waits for another's
   run to finish, and takes over a lock whose owner is gone);
2. builds and installs this checkout's adapter and runtime (`-NoBuild` skips the build);
3. starts the game from its exe (`SteamAppId` keeps Steam from restarting it through its launcher),
   skips the intro films and the logo, and loads `-Save` the moment the main menu is up
   (`-NewGame`: the main menu's New, its film skipped, the prison cell and the race menu;
   `-Save none`: the main menu, in its place in Cyrodiil);
4. once in the game: `-At <cell>` (`coc`), `-Give "<form id> <count>,..."`, `-Hour`, `-Weather
   <id>`, `-Commands "a|b"` (console lines in order), `-ConsoleFile <file>` (timed lines:
   `game+<s> <command>`, `menu+<s> ...`, `<s> ...`), and `-Settings "a = 1; b = 2"` (lines for the
   adapter's `rustvr-oblivion.toml`, this run only);
5. then one of: `-Sim <script.toml>` (an emulated-headset script, then the game is closed), `-Hold
   <seconds>` (the emulated headset held still that long, then closed), or neither (the real
   headset: a playtest; the game keeps the lock until `-Close`).

Closing puts back everything it changed (`Oblivion.ini`, `rustvr-oblivion.toml`) and gives the lock
back. An emulated run's results go to `-Out` (`captures\run` by default): `adapter.log` (this run's
part), `runtime.log`, and the captures. `tools/oblivion.ps1` turns the desktop view off
(`RUSTVR_DESKTOP_VIEW=0`) unless the caller sets `RUSTVR_DESKTOP_VIEW=1` first.

Run it from an ordinary shell: in a sandbox, Steam's DRM can't reach Steam and the game comes back
through its launcher. Steam must be signed in, or the game stops at an "Application load error" box.

The console can't be typed into in VR, so everything a test needs goes in its command line. Besides
the game's own console commands, the adapter's test console (`crates/oblivion/src/console.rs`) has
its own lines (ids in hex): `PlaceAt <base> <ahead> <right> <up> [<turn>]`, `Place <base> <x> <y>
<z> [<angle>]`, `PlaceInFront <ref> <metres> [<up>]`, `ActivateRef <ref>`, `SelectSpell <name or
id | none>`, `RemoveAllSpells`, `EquipAll <id>`, `PlayerHorse <base> [<metres>]`, `GoreCut
<bone>`, `RunFile <path>`, `LoadGame <save>` and `NewGame`; anything else runs as a line of
script.

### Test saves

The test saves are a developer's own, in the game's saves folder, never the player's: `rustvr-base`
(a Nord in iron armour, Danwood), `rustvr-city` (the Imperial City's Market District, noon),
`rustvr-weye` (Weye, open ground on Lake Rumare's shore), `rustvr-ruin` (Vilverin, an Ayleid
ruin), `rustvr-chorrol` (Chorrol, near its stables), and a feature's own (`rustvr-torches`,
`rustvr-horse`, ...). A save for another place is made from one of these: boot it with
`-At <cell>` (and `-Give` and `-Commands` for what the test needs), save from the game's pause
menu in the headset, and rename the new save in `Documents\My Games\Oblivion\Saves` to
`rustvr-<place>.ess` (its `.obse` too, with xOBSE), which `-Save rustvr-<place>` then loads. (A
`SaveGame <name>` test line, saving by name from the command line, comes with the `interact`
branch.) `coc` takes cells, not worldspaces: the Imperial
City's districts are worldspaces, so `-At` takes one of their cells (`ICMarketDistrict01` to `04`)
and `player.setpos` the spot.

## The emulated headset

`framework/crates/xrsim` is an OpenXR runtime with no headset: a script moves the head and the
controllers and presses buttons, and it saves what each eye would have seen
([framework/docs/XRSIM.md](../framework/docs/XRSIM.md)). `tools/oblivion.ps1 -Sim
tools\xrsim\<script>.toml` runs one in the game; `tools/xrsim/` has the scripts, and their console
files (`oblivion_load_*.txt`) set the scene. `tools/xrsim_oblivion.ps1` is an older wrapper of the
same.

Quick checks only: one direct check that a change does what it should (a log line, one emulated
run, one capture). The real headset is the player's: a playtest.

## Feature worktrees and playtests

A feature is built in its own worktree, on a branch of its name in both repositories (the
framework submodule checked out on the same-named branch), and pushed there; nothing is merged
into `main` until it has been played.

- Each worktree builds into its own folder, so they don't fight over one `target`: dev.toml's
  `builds` folder (`<builds>\<worktree's name>`), else the worktree's own `target`.
- A finished feature leaves `PLAYTEST.md` at its worktree's root, with a line `Save: rustvr-...`:
  what to try, in what order, what was checked and what wasn't. A `playtest.bat` beside it (not
  committed: root `.bat`s are git-ignored) boots it; at the merge, `PLAYTEST.md` moves to
  `docs/playtest/<feature>.md`.
- `tools/play.ps1 <feature>` installs that worktree's build and boots the save its `PLAYTEST.md`
  names (`-Save`, `-NewGame`, `-At`, `-Give`, `-Commands` and `-Settings` as for
  `oblivion.ps1`). `-Range` starts the framework's test range from that build instead. `-Close`
  puts everything away, whichever worktree has the game; `-Close -KeepLogs <name>` also keeps the
  run's logs (`-Logs "rustvr.log,voice.log"`) in the next `<name>-<n>` folder of the feature's
  build folder. Run with no feature, it lists the worktrees.
- The test range shares the runtime's link with the game's runtime: a test run of the range gets a
  link and a settings folder of its own with `RUSTVR_LINK` and `RUSTVR_DIR`
  ([framework/README.md](../framework/README.md)), so it never meets a game another worktree has
  up. `play.ps1 -Range` stops every waiting runtime and range on the machine first: check nothing of
  another worktree's is running.

## This machine's places

The scripts in `tools/` and `clips/` never name a folder of one machine. `tools/paths.ps1` (for the
scripts) and `tools/local_paths.py` (for the Python tools; `oblivion_assets::install` for
`oblivion-extract` and the tests) find them:

| What | Environment variable | `dev.toml` | Otherwise |
|---|---|---|---|
| Oblivion's folder | `RUSTVR_OBLIVION_DIR` | `game` | the game installer's registry key, then Steam's libraries |
| The feature worktrees | `RUSTVR_WORKTREES` | `worktrees` | the folder this checkout is in |
| Their builds | `RUSTVR_BUILDS` | `builds` | each worktree's own `target` |
| The voice plugin's servers | | `[voice]` | (`clips\voice-servers.ps1` asks) |

`dev.toml` is `%LOCALAPPDATA%\RustVR\dev.toml`, yours alone; `tools/dev.example.toml` shows every
line. The game lock is `game.lock` in the builds folder, else in `%LOCALAPPDATA%\RustVR`.

## Debugging

Logs: the adapter's `rustvr-oblivion.log` in the game's folder (it grows across runs), the
runtime's `%LOCALAPPDATA%\RustVR\rustvr.log` (`RUSTVR_LOG` sets its filter), the range's
`range.log` and the voice plugin's `voice.log` beside it.

The adapter's testing switches are environment variables read as the game starts, each documented
where it's read (`git grep RUSTVR_OBLIVION_ crates`). Among them: `RUSTVR_OBLIVION_PROFILE=1` (what
the game's thread does in a long frame), `RUSTVR_OBLIVION_TRACE_MOVE=1` (the stick and the
player's walking), `RUSTVR_OBLIVION_TRACE_UI=1` and `RUSTVR_OBLIVION_DUMP_TILES=1` (the menus),
`RUSTVR_OBLIVION_LEAVE_OUT=1` with `RUSTVR_OBLIVION_EYE_PNG` (which kind of draw causes something
in the eyes), `RUSTVR_OBLIVION_NO_XOBSE=1`, `RUSTVR_OBLIVION_FLAT_INI=1` (the player's
`Oblivion.ini` left as it is), `RUSTVR_OBLIVION_INI_TEST="Section.key=value;..."`,
`RUSTVR_OBLIVION_MENU_SPOT`, `RUSTVR_OBLIVION_ANISOTROPY`, and the `RUSTVR_OBLIVION_GAME_*` ones
that put one of the game's own behaviours back in place of the adapter's (its fog, grass cone,
leaves, distant trees, arms).
