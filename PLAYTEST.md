# Playtest: the main menu in Cyrodiil, and loading without leaving the world

Save: none

`playtest.bat` boots the game with no save loaded, so it stays at its main menu, and the main menu
takes you to its place: the Imperial City across Lake Rumare at sunset, from the slope above Weye
(`set RUSTVR_OBLIVION_MENU_SPOT=...` at the top of the `.bat` picks another). From there, every
load you make (Continue, Load, a door, fast travel, `coc`, quitting to the main menu) keeps the place
you were in round you, with only the game's loading bar floating in it.

The research and the design are in `docs/research/worldmenu.md`.

## The place

Picked: spot 1, the Imperial City across Lake Rumare at sunset (the default). The six `.bat` files at
the worktree's root still each boot `rustvr-weye` and move you to a candidate spot at its hour and
weather (the game's own `coc`/`cow`, `setpos`, `setangle`, `set gamehour`, `fw`), and any of them
can be the main menu's place with `menu_spot` or `RUSTVR_OBLIVION_MENU_SPOT`:

| `.bat` | The spot |
|---|---|
| `spot-1-imperial-city.bat` | The Imperial City across Lake Rumare at sunset, from the slope above Weye: east down the great bridge to the White-Gold Tower, the sun behind you (the default) |
| `spot-2-cloud-ruler.bat` | Cloud Ruler Temple in the Jerall Mountains, morning: south over Bruma and the Great Forest |
| `spot-3-dive-rock.bat` | Dive Rock, the highest lookout in Cyrodiil, early morning: south-west over the Nibenay |
| `spot-4-anvil.bat` | Anvil's lighthouse on the Gold Coast at sunset, over the Abecean Sea |
| `spot-5-rumare-night.bat` | Lake Rumare by moonlight, from Vilverin's ruin, the city across the water |
| `spot-6-first-view.bat` | The first view: Lake Rumare from the prison sewer's exit, facing Vilverin, morning |

## What to try, in order

1. **The start.** Run `playtest.bat` and put the headset on. As the game starts you're in the
   runtime's room. When the game's main menu comes up it takes you to the place at once: the game's
   loading bar floats alone in the room (no loading picture), then the place fades in from black.
   - Expect the Imperial City at sunset, the logo and the menu's words (Continue, New, Load, Options,
     Credits, Exit) in a column floating out over the lake, 5 m away, to the left of the White-Gold
     Tower so they never cover it, a little under the horizon, fixed there in the world.
   - You face the view whichever way you're standing in the room.
   - Your hands are there, but no body (the game has no character for you yet).
   - The main menu's music should play (tell me if the place's load stops it).
2. **The place lives.** Look round, lean, crouch: it's the game's world, drawn in stereo every frame.
   The clouds drift, the water moves, the grass and trees sway; the hour and anything living in it
   stay still. The menu stays where it floats while you look round.
3. **The words.** Point the laser at each word: the game's brush-stroke comes up under it, with the
   game's own focus sound and a tick in the hand. The words are the main menu's own Kingthings, in the
   HUD's cream with a shadow (the main menu's brown is for the film that's behind it in the flat game).
   - **Credits**: the game's credits float ahead; B (or its own way out) brings you back.
   - **Options**: the game's options float ahead, as from the pause menu.
   - **Load**: the game's list of saves floats ahead.
   - The runtime's own settings aren't there (they're beside the pause menu in a game).
4. **Continue** (or Load a save): the game's question pops up where you look; answer Yes. The view of
   the city stays round you, still moving with your head (lean and turn: it isn't stuck to your face),
   and only the loading bar floats ahead, filling. When the save's place is loaded it takes over: the
   view of the city fades out as the new place fades in, as fast as the game's own fade from black.
5. **A door, fast travel, `coc`.** In the game, go through a door into a building and out again, and
   fast travel once. Each time: the place you were in stays round you with the bar, then the new place
   fades in from it.
6. **Quit to the main menu** (the pause menu's Quit, then Main Menu): the last view of your game stays
   while the game resets, the bar comes, and you're back over the Imperial City with the menu.
7. **New**: answer Yes. The view stays while the game resets itself (as a quit to the main menu
   does, so the new game is a clean one), then the game's intro film plays as before (flat, in the
   runtime's room: films aren't held over), then the prison and character creation.
8. **Exit**: the game's question, then it quits.

What to tell me: anything that drops you into the room or the headset's own grid; the bar's size and
place; the held view's edges when you turn far during a long load; the fade at the end of a load;
whether the words are easy to read and to point at; the place's look (and which place you like).

## What I checked, and what I couldn't

Checked:
- It builds (adapter, runtime, range, xrsim), and the unit tests pass: protocol 11, adapter SDK 18,
  runtime 219 (a new one: the title over a view floats beside its subject and stays while the head
  turns), Oblivion adapter 39 (three new: the spots by name and in turn, the game's lines that go
  there). The C header is regenerated.
- What it relies on: how the main menu comes up and what each of its buttons does; the
  frame not drawing the world under the main menu; menu mode freezing the sky, water and
  wind; quitting to the main menu resetting the game; the loading screen's tiles and that it
  presents from inside the load; the game's fader 2 after a load (its black, its ten frames, its fade).
- The spots: positions, heights and headings from `Oblivion.esm`'s own map markers and land, and
  nothing standing where you'd stand.

Couldn't (the game wasn't run, by the rules):
- That `coc` from the boot's main menu brings the default player into the cell cleanly, and that the
  frame then draws the world under the menu (it does only for a player with 3D: the log says
  `the player's 3D there: true` or `false` as the place comes up).
- That the font change of the words takes (if not, they're cream in the plain Kingthings).
- How the held view looks at its edges when you turn far round during a long load.
- That New goes through the game's reset into character creation.
- The log lines to look for: `worldmenu:` (the place, its lines, New), `render: the game can't draw
  its world` (holding), `render: the game is busy loading` (the heartbeat), and in the runtime's log
  `the place the player was in is held` and `the new place has taken over`.

If the place doesn't come, the main menu stays the flat one (after 45 s at most), and
`menu_world = false` in `rustvr-oblivion.toml` turns it off.
