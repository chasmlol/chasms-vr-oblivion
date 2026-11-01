# The world as the front end: the main menu in Cyrodiil, loading without leaving it

The user's words: *"I want all the in game menus stuff to kind of be in world. So I want the main menu
to like be in a really beautiful spot in oblivion with the normal menu options as floating text and
then when traveling between places I just want the like load screen bar to appear on screen but dont
unload the room/world youre in so that it never takes you out the experience."*

Two parts:

1. **The main menu in a beautiful spot.** When the game starts, the player stands in a real place in
   Cyrodiil, the game's own cell drawn by the game with its own sky, weather, water and hour. The main
   menu's entries float in front of the view as the game's own text, worked with the laser, and each
   does the game's own thing.
2. **Loading without leaving the world.** Through a door, fast travel, `coc` or a save loading, the
   place the player was in stays round them, still tracked by the head, with only the game's own
   loading bar floating in it. The new place takes over when it's ready.

This builds on the floating menus (`docs/design/menus.md`): the game draws its UI into the HUD
texture with real alpha, the adapter says which rectangles of it are what (`HudLayout` groups and
pieces cut from the game's own tiles), and the runtime floats them in the world under the laser.

Legend: **(snippet)** a search engine's summary, not a page read in full.

## 1. What other VR games do, and what players say

Researched for this note on 2026-10-30.

### Loading

- **When a game stops sending frames, the headset's compositor takes over**: SteamVR shows its own
  white room or grid, and can send the player back to the lobby; developers call it jarring. Epic's
  advice is to mask a load with "a loading screen that pumps frames to the compositor"
  ([Unreal forum](https://forums.unrealengine.com/t/steam-vr-and-loading-screens/381359)).
- **Valve's own loader** (`SteamVR_LoadLevel.cs` in the SteamVR Unity plugin) fades out over 0.5 s,
  sets a six-face skybox in the compositor, puts a loading overlay and a progress bar in front of the
  player, stops submitting, loads, and fades back in over 0.5 s
  ([source](https://raw.githubusercontent.com/ValveSoftware/steamvr_unity_plugin/master/Assets/SteamVR/Scripts/SteamVR_LoadLevel.cs)).
  That's this design's shape, with a prepared skybox where this has the place the player was in.
- **Meta** names loading screens as a use of its compositor layers, which "always appear to update at
  full frame rate" even when the app is stalled
  ([Meta](https://developers.meta.com/horizon/documentation/unity/os-compositor-layers)); its older
  advice is a still image the compositor keeps head-tracked while the game loads
  ([tech note](https://developers.meta.com/horizon/blog/tech-note-animated-loading-screen)).
- **The complaints are about held images that stop following the head or judder**: flight-sim
  players report flicker and nausea on loading screens and close their eyes (DCS, IL-2; snippet).
  A picture reprojected smoothly every headset frame is what they ask for. The risk of a single held
  frame is black at its edges when the head turns past it (NVIDIA forum; snippet).
- **Bethesda's own VR ports** are the cautionary tale: Fallout 4 VR's loading screen is "a
  straightforward copy of the pancake version" (Nexus; snippet), and UploadVR said staring at "deep
  blackness and a loading bar" won't do for VR
  ([UploadVR](https://uploadvr.com/bethesda-needs-to-do-fallout-4-vr/)). Skyrim VR on PSVR drops to a
  flat splash image on every load (snippet).
- **The flat equivalent of this design exists**: Skyrim SE's *Clean Loading Transitions* keeps the
  previous cell visible until the load finishes (Nexus; snippet).
- **Hidden loads** are the native games' answer: Batman: Arkham Shadow and Red Matter load in lifts and
  slow doors, Asgard's Wrath 2 streams its open world. A December 2025 article on killing the VR
  loading screen adds: when a load can't be hidden, keep it short and keep the bar fixed in the world,
  not stuck to the face ([Road to VR](https://roadtovr.com/vr-loading-screen-design-charlie-cochrane/)).

### Front ends

- **Bonelab's menu is a level** you're driven to; Saints & Sinners picks destinations from a map on
  a church bus
  ([Wikipedia](https://en.wikipedia.org/wiki/Bonelab); others snippet). The native games stage the
  front end in the world.
- **Half-Life: Alyx's developer commentary on its opening Citadel view** is the best guidance on a
  distant vista in VR: distant things get little stereo depth, so give the eye scale cues between it
  and the landmark (buildings, converging lines, fog, light), and keep clutter away from the view
  ([commentary](https://combineoverwiki.net/wiki/Developer_commentary/Half-Life:_Alyx)). The spots
  below were picked with that in mind: a bridge, a shore or a slope leading the eye to the subject.

## 2. The spots

Players' most-mentioned views (UESP, a UESP photo album of 39 shots with places and times, GameRant
2021 and 2025, CBR, Steam threads): the Imperial City across Lake Rumare (by far the most, and the
prison sewer's exit is remembered as the game's signature moment), Anvil and its lighthouse at
sunset, Frostcrag Spire and Gnoll Mountain (Frostcrag is a DLC, not in every install), Bruma and
Cloud Ruler Temple at sunrise, Dive Rock on a clear day ("one of the most spectacular views in the
province", [UESP](https://en.uesp.net/wiki/Oblivion:Dive_Rock)), then the Great Forest, the West
Weald, Niben Bay at dawn, Cheydinhal.

Each spot below was placed from `Oblivion.esm` itself: the map markers' positions, the land's height
records (`LAND`/`VHGT`, which match the markers' heights to a few units), and the references within a
few metres of the spot (nothing stands where the player does). Headings are the game's (degrees from
north, clockwise), towards the subject. Tamriel's climate has sunrise 6-10 and sunset 16-20.

| # | Spot (`menu_spot`) | Cell (coc) | Position (game units) | Facing | Hour | Weather |
|---|---|---|---|---|---|---|
| 1 | `imperial-city`: the Imperial City across Lake Rumare at sunset, from the slope above Weye: the great bridge leading the eye east to the White-Gold Tower (28672, 63424), the sun behind | `WeyeExterior` | (-2000, 66000, 1167) | 95 | 18.5 | Clear |
| 2 | `cloud-ruler`: Cloud Ruler Temple in the Jerall Mountains, morning: the snow outside the Blades' temple, south over Bruma and the Great Forest | `CloudRulerTempleExterior02` | (13391, 160255, 31594) | 171 | 8.5 | Clear |
| 3 | `dive-rock`: Dive Rock at dawn: the summit, south-west over the Nibenay to the Imperial City | `DiveRock` | (138134, 142738, 34178) | 234 | 6.5 | Clear |
| 4 | `anvil`: Anvil's lighthouse on the Gold Coast at sunset, over the Abecean Sea | (cell -48, -10) | (-196600, -39000, 740) | 300 | 19 | Clear |
| 5 | `rumare-night`: Lake Rumare by moonlight from Vilverin's ruin, the city across the water | (cell 12, 21) | (50804, 87399, 405) | 223 | 22 | Clear |
| 6 | `first-view`: the first view: Lake Rumare from the prison sewer's exit, facing Vilverin | `ICPrisonSewerExit01` | (47110, 83000, 297) | 40 | 9 | Clear |

Clear is `00038EEE`; the others in Tamriel's regions are Cloudy `00038EF0`, Overcast `00038EEC`, Fog
`00038EEF`, Rain `00038EF2`, Thunderstorm `00038EF1`, Snow `00038EED`.

**Picked: 1**, the Imperial City across Lake Rumare at sunset, by looking at them in the headset
(each booted into a test save and moved there with the game's own `coc`/`cow` and `setpos`, the
hour and the weather set). The adapter keeps the spots as a table (`worldmenu.rs`, `SPOTS`; the
default `imperial-city`); `menu_spot` in `rustvr-oblivion.toml` names one, or several separated by
commas to take turns each time the game starts.

## 3. The game's main menu and loading

### How the main menu comes up today

- **Boot**: with no game loaded the game closes its loading screen, makes the main menu, waits for
  the intro films, starts the main menu's music (`Data\Music\` + the ini's file) and its background
  film, then idles under the menu.
- **`MainMenu`** (`Data\Menus\Options\main_menu.xml`): a `backdrop` image (hidden), the Oblivion logo
  (`oblivion_logo`, `tes_oblivion_logo_final.dds` with the Bink logo over it), a row of six text
  buttons in `button_layout` (each `button_floating.xml`: its text, and under it while the cursor is
  on it the dialogue's brush-stroke selection, `dialog_selection_full.dds`), and `version`. Buttons:
  `main_continue`, `main_new`, `main_load`, `main_options`, `main_credits`, `main_exit`. There's no
  world: the background is the film.
- **What each does**:
  - Continue: a question ("Continue from your last saved game?"), then the newest save loads and the
    menu closes.
  - New: a question, then the world is set to Tamriel, the menu closes, its music stops and the
    character-generation quest starts.
  - Load: the load menu. Options: the options menu. Credits: the credits.
  - Exit: a question, then the game quits.
  - Hovering a button plays the game's focus sound.
- **Quitting to the main menu from a game** resets the whole game (the grid, the player, the save
  state, actors, the loaded data) and makes the main menu again. A new game from the main menu
  always starts from that clean state, or from the boot's.
- **Under the main menu the world isn't drawn**: the game renders the world under every menu but
  the main menu. Everything else about drawing the world under a menu already works in VR
  (`docs/design/menus.md`).
- **A menu freezes the world**: in menu mode the game skips the actors, the scripts and the clock,
  and updates the sky, water and wind with no time (the dialogue menu alone gets real time for its
  sky). That update advances the water's and the grass's clock, the clouds by the time passed and
  the sun from the hour (which it reads and never advances), and the scene's controllers. Given the
  frame's own time under the main menu, the place lives (clouds, water, grass, trees) while nothing
  in it acts and the hour stays put.

### Having the world behind the main menu

- **A save made at the spot** would carry a character, show in the Load menu and could become what
  Continue loads; making it needs the game run once. Rejected.
- **The player's own last save, moved to the spot**: their game in memory, their followers, the
  autosaves of travel. Rejected.
- **The game's own `coc` from the main menu (chosen).** The console's `CenterOnCell` and
  `CenterOnWorld` move the player to a cell as a door does (its loading screen, its fade). At the
  main menu the player is the default one the boot made; nothing of any save is touched. Then the
  game's own `setpos`, `setangle`, `set gamehour` and `fw` (ForceWeather) place them and set the
  scene. The main menu stays open the whole time (the loading screen doesn't close it), so it's still
  the game's main menu, with its question boxes, music and buttons, over a world. One catch: the game
  makes its console's manager only when asked to, and before a game is loaded nothing has asked, so
  every line run at the main menu failed until the adapter asked as the game's own console does
  (`console.rs`). If the way there closes the main menu, the game makes it again.
- **New** must start from the game's clean state. Clicked over the menu world, New's answer is held
  back: the game's own exit to the main menu resets the world, and on the fresh main menu the same
  answer runs again. Continue and Load load a save, which replaces the world wholesale as it does
  from inside a game. A New answered before the place came leaves the main menu closing, and the
  menu world leaves a closing main menu alone (else its `coc` undid the new game).

### How loading works

- **The loading screen** (`LoadingMenu`, `Data\Menus\loading_menu.xml`): a black rectangle over the
  screen (`black`), a picture (`load_main`, the location's `LSCR` or the title page), its hint text
  (`load_text`), and the bar `load_progress`: `stat_skill_level_ribbon_empty.dds` with the ribbon
  `load_progress_ribbon` (full, its width from `user3`/`user4`) and its slider, and under it the word
  `load_loading`. Its text is the parchment's brown (117, 59, 33), meant for the picture behind it.
- **Bethesda made an in-game bar**: `menus\loading_bar_ingame_menu.xml` (unused by the game) is just
  `load_progress` and `load_loading`, the word in the HUD's own cream (254, 244, 183) and font 2
  (Kingthings shadowed, the HUD's). That's what floats here: the game's bar, its word recoloured to
  Bethesda's own in-game design so it reads over the world.
- **Loading runs on the game's main thread.** The loading code updates the bar as it goes and draws
  the menus again (their own frame, presented) whenever the bar or the slideshow moves. So Present
  comes in bursts while loading, with gaps; the game's frame doesn't run. Opening, the loading menu
  presents twice at once.
- **A cell's load takes the game's frozen picture first** (as the menus do, and a question box over
  the load's fade): the world rendered once into a texture of the game's, drawn instead of the world
  while it's there. In VR every use of it is skipped, as the menus' already were. (In the first
  playtest it wasn't: the eyes were read from the back buffer as it went into the texture, and the
  black back buffer was the place held through a door.)
- **The old place can't stay drawn by the game**: the cell change detaches the old cells before the
  new load begins and the background loader shares the scene with the render through its lock. Only
  a picture of it can stay.
- **The fade after a load** is the game's fader 2 (it has three full-screen faders: 0 is
  `letterbox.dds`, 1 and 2 `black.dds`). A cell or save load starts fader 2. Once it's fully black
  the game closes the loading screen on the first frame, holds black for ten frames with the new
  world drawn under it, then lets the fader go, and it fades back over its time. The faders are
  screen-space quads drawn with the UI, so in VR they go into the HUD texture, and VR so far never
  showed them.
- **Films** (the new game's intro) are drawn by the menus' render.

## 4. The design

### Part 1: the main menu in the world

- **The place.** At the main menu with no game (the boot's, or after quitting to it), the adapter runs
  the game's own lines: `set gamehour to <h>`, then `coc <cell>` (or `cow Tamriel <x> <y>`), then
  once the player is in it `player.setpos` x, y and z, `player.setangle z`, `fw <weather>`. The view
  is recentred so the player faces the spot's way. Not when a test asks for a save at the main menu
  (`rustvr-console-test`), and not with `menu_world = false`.
- **The eye.** Under the main menu every frame is a menu's, so the game never lifts its camera from
  the feet its teleport put it at (its player's update, which does, is skipped in menus). The view's
  eye is the spot's own, on its ground at a standing eye's height (120 units), and the view is
  recentred once to face the spot's way: the eyes are drawn from there whatever the game's camera
  says.
- **Drawn.** While the menu world is up, the render's test for the main menu answers "no main
  menu", so the frame renders the world under it, in stereo, as under any other menu. Each frame the
  adapter also calls the game's sky, water and wind update with the frame's time, as the game does in play: clouds drift, water moves, grass and trees sway, the hour
  and everyone in the place stay still.
- **The options.** The main menu is cut from its own tiles as every menu is: the logo, and each
  button as a piece of its own (its box, so the brush-stroke under a hovered button shows), stacked
  in a column under the logo (the game's row would be 40 degrees wide). The words are the game's
  Kingthings in the HUD's shadowed cut and cream, as everything over the world is, set through the
  game's own `Tile::UpdateFloat` on each button's text. They float as a group in a new place for a
  group, **over the view** (`ANCHOR_VISTA`): 5 m out (first 2.5 m: near enough that a lean slid it
  across the far view, as if it came with the head), beside the spot's subject (the White-Gold Tower
  for most; its side is the spot's) so it never covers it, a little under the horizon, larger than a
  menu (a title's size). Fixed in the game's world: placed in the tracking space of its moment and
  carried with the world from then on (a turn of the stick, a recentre), never placed again where
  the head looks (first it was, whenever its far-off subject moved a metre in tracking space: a
  tenth of a degree at 450 m), and where it was again after a menu over it. The laser points at each
  word; the game's cursor goes there, so the game's own hover (the stroke, its focus sound) and
  click (its click sound, its question boxes, its loads) do everything. Questions pop up where the
  eyes look over the words; the load, options and credits menus float ahead as before. The runtime's
  own settings aren't beside the words (they're beside the pause menu in a game), nor in its room
  while the game starts.
- **The player** has no character yet: the runtime draws the hands only (`GS_HANDS_ONLY`), not the
  default body the boot made.
- **Its music** is the game's main menu music, as ever (unless the cell's load changes it: to check).
- **The spot**: one of the table's, `menu_spot` (or `RUSTVR_OBLIVION_MENU_SPOT` for a playtest: the
  test tools write the settings afresh at each boot). Its subject point (where the menu goes beside)
  and the side it goes on are part of the table.
- **If the place doesn't come** (45 s), the main menu stays the flat one, as before.

### Part 2: loading without leaving the world

Options:

1. **Keep the old cells drawn by the game while the new ones load.** Not possible: the load runs on
   the main thread after the old cells are detached, and the scene is the loader's while it works.
2. **A picture of the old place, held by the runtime, reprojected every headset frame with its depth
   (chosen).** The runtime already moves the game's picture to the head's pose of each headset frame by
   its depth (`warp.rs`: a grid pushed out to the depth, the other eye's picture filling what the move
   uncovers, the edges drawn on). Held through a load, the place stays round the player with full
   parallax: leaning and turning look right. The runtime keeps submitting frames whatever the game
   does, so the headset never falls to the void or the compositor's grid.
3. **A cube of the old place captured as the load starts** (Valve's skybox, made from the game):
   no edges when turning far. More renders at the moment of the load; kept as the next step if the
   held picture's edges show.

How it goes:

- **The adapter** keeps the last world picture's description (the eyes' poses, fields of view, world
  mappings, depth range). A Present without the world rendered, while the game is loading (its
  loading screen open, or its fader 2 there) or between quitting to the main menu and the menu world,
  gives the runtime that picture again: the bridge's last eye and depth copies, delivered into the
  slot with the old poses, flagged `SLOT_HELD`, and the HUD of the moment with its layout: the
  loading bar as a piece (`load_progress`, with its word under it), floating ahead (`ANCHOR_PLAYER`).
  The game says it's loading (`GS_LOADING`): the rig pauses. A film (the new game's intro) isn't held
  over: it plays as before.
- **A heartbeat while loading.** The runtime counts the adapter gone after 2 s without a beat; a load
  can block that long between two Presents. While the loading screen is open a thread of the
  adapter's beats for it (`Adapter::heartbeat`), for a minute at most.
- **The runtime** (`held.rs`), seeing a slot flagged `SLOT_HELD`, copies its eyes and depth once into
  textures of its own, and draws that picture through the warp every headset frame, fixed in
  tracking space (the walk isn't carried on, the world doesn't jump with the game's teleport), with
  the body, the hands and the floating bar over it.
- **The handover is the game's own fade.** After the load the game holds black ten frames with the new
  place drawn under its fader, then fades it in. Each live picture says how much of its fader there
  is (`SlotInfo::held_over`, fader 2's alpha); the runtime draws its held picture over the live one
  that much, so where the flat game fades in from black, the VR game fades in from the place the
  player was in. Once the fader's gone, the held copy goes. A load with no fader (none seen) hands
  over in half a second.
- **The first boot** has no place to hold: the runtime's room (nothing new is submitted while the
  main menu waits for its place, nor for a loading screen's first two pictures, drawn before its UI
  goes in with its alpha), the loading bar alone floating in it (an alpha layout of pieces is
  floated over the room too), then the menu world, faded in from black as the game's fader says.
- **Its UI with alpha while loading.** After each Present with the loading screen open, the game's
  next UI is sent into the HUD copy as it is over the world (`ui_pass`), and a clear of the target
  while it is (the loading screen begins its own frame, with its background colour) clears to
  nothing (a `Clear` hook).
- **Fader 2 is the cell and save loads'** (and one other caller's); sleeping and waiting
  use fader 1, which isn't touched. A fade of fader 2 without a load would show the held place
  instead of black.

### What's where

| | Where |
|---|---|
| `SLOT_HELD`, `SlotInfo::held_over`, `GS_HANDS_ONLY`, `ANCHOR_VISTA` | `framework/crates/protocol` (and the C header) |
| `Adapter::heartbeat` | `framework/crates/adapter` |
| The held picture, its handover, floating pieces over the void, the vista's placing, hands only | `framework/crates/runtime` (`held.rs`, `session.rs`, `warp.rs`, `hud.rs`, `body`) |
| The menu world, its spots, New's reset, the sky's update | `crates/oblivion/src/worldmenu.rs` |
| The world rendered under the main menu | `worldmenu.rs` |
| Held frames, `held_over`, the loading heartbeat | `crates/oblivion/src/render.rs` |
| The main menu's and the loading bar's pieces | `crates/oblivion/src/hud.rs` |

### Settled since, and still open

- Settled: `coc` from the boot's main menu brings the default player into the cell once the
  console's manager is made (above); a New from the boot's main menu reaches the prison cell with
  the menu world on.
- Open (the playtest's, `docs/playtest/worldmenu.md`): that the font change of a button's text
  (`font` 2) re-lays it (if not, the colour alone changes); how the held picture's edges look
  turning far during a long load (the cube, not built, is the answer if they show).
