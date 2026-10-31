# Playtest: the main menu in Cyrodiil, and loading without leaving the world

Save: none

The research and the design: `docs/research/worldmenu.md`. The code: `crates/oblivion/src/worldmenu.rs`
(the place), `render.rs` (the held view while loading) and `hud.rs` (the menu's floating words).

## Playing it

From the repository's root:

    powershell -File tools\oblivion.ps1 -Save none

It builds main, installs it beside the game and boots it for the headset with no save loaded, so
the game stays at its main menu, and the main menu takes you to its place.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and your settings back. No test save
is needed. (With `-Save <a save>` the boot loads that save from the main menu before the place comes,
so there's no place to see.)

Another of the places: `-Settings "menu_spot = 'cloud-ruler'"` (a line for this run's
`rustvr-oblivion.toml`, which `-Close` puts back), or `RUSTVR_OBLIVION_MENU_SPOT=cloud-ruler` in the
environment the game starts from, which wins over `menu_spot`.

## What it does

**The main menu in a place.** At a main menu with no game (the boot's, or after quitting to it), the
adapter runs the game's own console lines on the default player the boot made: `set gamehour`, then
`coc` to the spot's cell (or `cow Tamriel x y` to its cell of the world), and once the player is in
it `player.setpos` x, y and z, `player.setangle z` and `fw` (the weather forced, so it's the spot's
and not the region's). The main menu stays open over the place: its question boxes, music, sounds
and what each button does are the game's.

- **Drawn.** The frame's draw renders the world only when the top menu isn't the main menu; while
  the place is up it's told there's none, so the world is drawn under it in stereo, as under any
  other menu.
- **Alive.** A menu freezes the world; while the place is up the sky, water and wind are updated
  with the time passing, as in play. The clouds drift, the water moves, the grass and trees sway;
  the hour and everyone in the place stay still.
- **Where you are.** The view's eye is on the spot at a standing eye's height (about 1.7 m),
  turned once to face the view whichever way you stand in the room. The hands are there, but no body:
  the game has no character for you yet (`GS_HANDS_ONLY`).
- **The words.** The logo and the six words (Continue, New, Load, Options, Credits, Exit) float as a
  column 5 m out over the view, beside what it's of, on its side (so they never cover it), 6 degrees
  under the horizon, fixed in the game's world while you look round (`ANCHOR_VISTA`). They're in the
  HUD's cream, in the game's shadowed Kingthings (the main menu's own brown is for the film behind it
  in the flat game). The laser points at each: the game's own hover (the brush-stroke under it, its
  focus sound) and click do the rest. A question over them moves the menu clear of it.
- **The runtime's own settings** aren't beside the main menu over its place, nor in the runtime's
  room while the game goes there: they're beside the pause menu in a game.
- **New** starts from the game's clean state: the answer is held back over the place, the game's own
  exit to the main menu resets the world first, and on the fresh main menu New runs with the same
  answer (given up after 45 s). Continue and Load load a save, which replaces the world as it does
  from a game. The new game's intro film shows as it is (flat, in the runtime's room), not over a
  held place.
- **If the place doesn't come** in 45 s (or the game refuses a line), that main menu stays the flat
  one.

**Loading doesn't leave the world.** While the game loads another place (Continue, Load, a door, fast
travel, `coc`, quitting to the main menu) it can't draw one:

- The runtime is given the last picture of the world again (both eyes, their depth and the poses
  they were drawn from, `SLOT_HELD`) and holds that place round you, still moving with your head.
- Only the game's loading bar floats in it, a little bigger than a line of text: the loading
  screen's black, its location's picture and its title page are cleared.
- The game's own fade after a load (its fader 2) is the handover: the held place fades out as the new
  one fades in, as fast as the game's own fade from black; the fader's quad isn't drawn.
- While the loading screen is open a thread beats for the adapter (up to 60 s), so a load that blocks
  the game for seconds doesn't make the runtime count it gone and drop the place.
- The game's frozen picture of the world, which it renders into a texture for a cell's load and for
  menus, is never taken for the eyes in VR: a door's load holds the place you were in, not black.
- The first load, with no place to hold yet (the boot's), floats its bar in the runtime's room.

## The places

| `menu_spot` | The place | Hour | Goes there by |
|---|---|---|---|
| `imperial-city` (the default) | The Imperial City across Lake Rumare at sunset, from the slope above Weye: east down the great bridge to the White-Gold Tower, the sun behind you | 18:30 | `coc WeyeExterior` |
| `cloud-ruler` | Cloud Ruler Temple in the Jerall Mountains: south over Bruma and the Great Forest | 8:30 | `coc CloudRulerTempleExterior02` |
| `dive-rock` | Dive Rock's summit at dawn: south-west over the Nibenay to the Imperial City | 6:30 | `coc DiveRock` |
| `anvil` | Anvil's lighthouse on the Gold Coast at sunset, over the Abecean Sea | 19:00 | `cow Tamriel -48 -10` |
| `rumare-night` | Lake Rumare by moonlight from Vilverin's ruin, the city across the water | 22:00 | `cow` to its cell |
| `first-view` | The first view: Lake Rumare from the prison sewer's exit, facing Vilverin (the menu on its right) | 9:00 | `coc ICPrisonSewerExit01` |

All in clear weather. Positions, heights and headings are from `Oblivion.esm`'s own map markers and
land.

## What to try

1. **The start.** Boot it and put the headset on. As the game starts you're in the runtime's room.
   When the main menu comes up it takes you to the place at once: with no place to hold yet, the
   game's loading bar floats alone in the room, then the place fades in from black.
   - Expect the Imperial City at sunset, the logo and the words in a column floating over the lake,
     5 m away, to the left of the White-Gold Tower, a little under the horizon.
   - You face the view whichever way you stand in the room. Your hands are there, no body.
   - Listen for the main menu's music.
2. **The place lives.** Look round, lean, crouch: the clouds drift, the water moves, the grass and
   trees sway. The menu stays where it floats.
3. **The words.** Point the laser at each: the brush-stroke comes up under it with the game's focus
   sound. Credits, Options and Load open the game's own menus, floating ahead as from the pause
   menu; B (or their own way out) brings you back.
4. **Continue** (or Load a save): answer the game's question. The view stays round you, moving with
   your head, and only the loading bar floats ahead, filling. When the save's place is loaded it fades
   in over the held view.
5. **A door, fast travel, `coc`.** In the game, go through a door and out again, and fast travel
   once: each time the place you were in stays round you with the bar, then the new place fades in.
6. **Quit to the main menu** (the pause menu's Quit, then Main Menu): your last view of the game stays
   while the game resets, then you're back over the Imperial City with the menu.
7. **New**: answer Yes. The view stays while the game resets itself, then the intro film plays flat
   in the runtime's room, then the prison and character creation.
8. **Exit**: the game's question, then it quits.

Tell me about anything that drops you into the room or the headset's own grid, the bar's size and
place, the held view's edges when you turn far during a long load, the fade at the end of a load,
and whether the words are easy to read and to point at.

## Settings

| Where | Setting | Default | What it does |
|---|---|---|---|
| `rustvr-oblivion.toml` | `menu_world` | `true` | The main menu in its place; `false`: the flat main menu |
| `rustvr-oblivion.toml` | `menu_spot` | `""` (the Imperial City) | The place by name (the table above), or several separated by commas: one of them each time the game starts |
| Environment | `RUSTVR_OBLIVION_MENU_SPOT` | unset | Names the place, over `menu_spot` |

`rustvr-oblivion.toml` is beside the game's exe (`crates/oblivion/src/settings.rs`); a string value
needs its quotes (`menu_spot = 'anvil, first-view'`).

## Decisions

- **The game's own main menu, in the game's own world**: the place is reached with the console lines
  a player would type, and every button stays the game's. The adapter only lets the world be drawn
  under it and cuts the menu into words floating over the view.
- **The Imperial City across Lake Rumare at sunset** is the default, picked by looking at the six in
  the headset; the others stay as `menu_spot` names.
- **The menu floats beside its subject**, fixed in the world, a little under the horizon, so it never
  covers the view it's there for.
- **The hour and the people stay still**: only the sky, water and wind are given time, since the
  clock, actors and scripts belong to the frame, which skips them in a menu.
- **New goes through the game's own reset**, so the new game is a clean one, as from the flat main
  menu.
- **A load holds the last place** instead of the loading screen's picture: nothing in VR should drop
  you out of the world, and the game's own fade from black becomes the handover.
- **Films aren't held over**: the intro plays as it is.
- **A boot's own console lines come first**: with a save to load (or New answered by the boot's
  `-NewGame`), the main menu isn't taken to its place, whose `coc` would undo it.

## Not done / known issues

- The held view is the eyes' last picture: turn far round during a long load and its edges show.
- Not confirmed in the headset (as far as this record goes): the main menu's music over the place;
  the words' change to cream Kingthings taking (if not, they're cream in the plain font); New from
  the place's own menu reaching character creation (the boot's `-NewGame` does).
- The adapter's log lines to look for: `worldmenu:` (the place, its lines, New), `render: the game
  can't draw its world` (holding) and `render: the game is busy loading` (the heartbeat).

## History

- **After merging into main**: the boot's `-NewGame` ended at the main menu at Weye, because the
  place's `coc` undid the new game in the same frame. A main menu that's closing is now left alone.
- **A door's load held a black place**: a cell's load renders the game's frozen picture of the world
  into a texture, and that was taken for the eyes. Every such render is skipped in VR now. A question
  over the main menu, drawn across its words, now moves the menu clear of it.
- **The words were moved into the world**: fixed in the game's world 5 m out, and RustVR's settings
  no longer float beside them or come up in the room while the game goes to the place.
- **The place never came** and the main menu stayed flat: the game makes its console's manager only
  when asked, and nothing had before a game was loaded, so every line failed. A line now asks for
  it, and the main menu is made again if the way there closed it.
- **The six places**, each booted on its own to be looked at in the headset: the Imperial City at
  sunset was picked.
