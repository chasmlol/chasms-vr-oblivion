# Oblivion in the Chasms VR Framework

The adapter (`crates/oblivion`, built as a 32-bit `dinput8.dll` beside `Oblivion.exe`) puts
Oblivion's world in the headset and the framework's player in it. Oblivion runs its world, and the
runtime runs the player: the head, the hands, the body and what they hold. The adapter only
translates. Where the adapter hooks the game is at the end, under "Hooks".
[research/oblivion-engine.md](research/oblivion-engine.md).

## How it works

| Part | What it does | Where |
|---|---|---|
| The picture | Oblivion's Direct3D 9 device is made on D3D9On12. After each eye's render, the back buffer goes through a shared Direct3D 12 texture and fence into the runtime's eye texture. | `d3d9.rs` |
| The eyes | The world render runs once per eye each frame, with the camera at that eye's pose and the eye's own asymmetric frustum. The game's first-person arms aren't drawn. | `render.rs`, `camera.rs` |
| Resolution | The game renders each eye at the headset's eye size as the runtime asks for it (its recommended size times the runtime's Resolution scale): the window, back buffer and every render target are made at that size. | `resolution.rs` |
| Depth | Each eye's depth buffer (resolved from its samples under the game's antialiasing) goes to the runtime as linear depth, so the hands, the body and what they hold are hidden behind the world. | `d3d9.rs`, `depth.rs` |
| Collision | The runtime's physics gets the world Oblivion's own Havok has: every fixed body's shapes near the player (the land, buildings, rocks, trees, furniture) as triangles, so what the hands hold, drop and throw stops at it and lands on it. | `collision.rs`, `docs/research/oblivion-havok.md` |
| The HUD | After the eyes, the game draws its HUD and menus into an alpha render target of ours (not its back buffer), alpha blended as coverage: the HUD texture is the UI premultiplied, over nothing. The HUD is cut from its own menus' tiles: the bars, the weapon and the spell on the back of the off-hand wrist, the active effects up the forearm, the compass low in front, the reticle and the card of what it's on where the head looks, a foe's health over the foe, subtitles and notices on the caption strip. Outside the world, the whole picture is a menu panel. | `render.rs`, `d3d9.rs`, `hud.rs`, `tiles.rs` |
| The player | Before each game frame, the player faces where the head looks (rotation and pitch). A third-person view goes back to first person. | `player.rs` |
| Input | The game's DirectInput keyboard and mouse read the runtime's actions, through the player's own `Oblivion.ini` bindings. While in VR the real keyboard and mouse don't reach the game. | `input.rs` |
| The window | It's always "active", so the game never pauses for focus. In the test mode it's off-screen and never activated. | `window.rs` |
| Settings | `Oblivion.ini` is windowed and skips the intro films while in VR. The player's values are kept and put back. | `game_settings.rs` |
| The body | The runtime draws the player's body from the game's own models: the race's skin where nothing is worn, the armour and clothes worn over it, on the game's skeleton. | `body.rs` |
| Lighting | The body and what the hands hold are lit as Oblivion's own shaders light them: the scene's sun or moon, the point lights near the player (torches, candles, spells), the ambient (Night-Eye's too), normal maps, specular, the skin's rim, glow maps and the fog. | `lighting.rs`, `docs/research/oblivion-lighting.md` |
| Weapons | The equipped weapon (the game's model) hangs at the left hip, or on the back if two-handed, and the hand draws it. The equipped shield is taken from over the left shoulder. Edges, points, heads and guards are measured on each model. | `weapons.rs` |
| Combat | Hits are the game's own hit with the equipped weapon (damage, skill, enchantments). A blow met by the weapon or shield is the game's block. Spells charge in the palm and go off as the game's own cast: target spells from the palm along the aim, touch spells into what the hand touches, self spells on the player. | `combat.rs` |
| Things in the world | Everything the player can take near them (weapons, armour, clothes, books, potions, ingredients, keys, soul gems, sigil stones, apparatus, carriable lights, clutter) is the hands' to take or pull with the gloves: its collision its own Havok shape, its weight its Havok mass, drawn by the game where the hand has it (a candle still burns), and let go, the game's again, flying on with the throw. A card beside it says what it is: its name, damage or armour, weight, value, condition, and whether it's stolen. Let go in any body slot (a hip, a shoulder, the chest, the belt), the player takes it: the game's own pick-up, with its stealing and its message. | `items.rs`, `objects.rs` |
| Menus | The world goes on being rendered under menus (the game's frozen picture of it is skipped), and every menu floats in it in pieces of its own UI, read from its tiles: a container's list, tabs and buttons beside the container; barter beside the merchant; dialogue at the speaker (the name over their head, the line under their face, the topics beside them); the big four ahead of the player with the HUD's switcher; questions and counts where the eyes look; services from dialogue beside the speaker; anything else ahead, cut out of the screen. The parchment is a little see-through. The game's cursor follows the laser (with a tick in the hand as the game's hover moves onto another thing), the trigger clicks, the right stick turns its mouse wheel. See `docs/research/menus.md`. | `hud.rs`, `tiles.rs`, `menus.rs`, `render.rs` |
| Books and notes | A book or note lying about, taken in a hand, is read at once: the game reads it as activating it does (its book menu, a skill book's skill rise, a note's script, its Take with the stealing), once while it's held (a magic scroll isn't: the game would take it). The book menu floats over the book in the hand, turned to the eyes however the hand turns it, steadied (no shake with the hand), the game's two pages tipped a little about the spine (a note's scroll in one piece), drawn with the hands in their depth; the free hand's grip near the pages, dragged across them, turns them (a page every 5 cm; the game's own page step and sound), and its fingertip presses the buttons (Take, Exit, the page arrows), no laser. Let go of, or Exit: shut (it opens again once it's let go of and taken again). It isn't a menu: opened with no menu up it's kept off the interface's menu stack, so the game plays on (walking, fighting, the HUD but what the book covers), and its pages and buttons are worked on the menu itself (its own page step and click), the game's mouse being its play. A book activated on a shelf by looking at it comes to the off hand and opens there; one read from the inventory is held in the off hand like a sheet. See `docs/design/books.md`. | `books.rs`, `items.rs`, `hud.rs` |
| The map | A folded map is kept at the right hip (`map_slot`; the game's sword is on the left). Drawn from it, the game's map opens (the local map inside, the world map outside) and its page and tabs are held in the hand like paper, turned to the eyes; the other hand's grip on its far side spreads it between the hands (bigger as they part), its grip on the face drags the map (the game's own pan), and its fingertip presses the tabs and places, no laser (fast travel, with the game's own question and rules). Put back or let go, it goes back to the hip and the map shuts. | `map.rs`, `hud.rs` |
| Tests | `rustvr-console-test` runs timed commands from the game's frame: `LoadGame <save>`, `SelectSpell <id>`, `PlaceInFront <ref> <metres>`, or any line of script (base forms by hex id). Compile errors are logged. | `console.rs` |

## Setting up

The game's models are converted once into `local_assets/oblivion` (never committed):

```
cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons meshes/characters/_male meshes/characters/khajiit meshes/characters/argonian meshes/armor meshes/clothes textures/characters
```

Then `python tools/install_oblivion.py` builds and installs the adapter beside `Oblivion.exe`.

## Controls

| VR | Oblivion |
|---|---|
| Left stick | Walking, where the head looks, as the game's own analog joystick walking: slow to full speed over the stick's travel, at the pace Always Run sets (pushing it clears Auto Move) |
| Left stick click (sprint) | Run, when Always Run is off |
| Right stick click, or its down flick, or ducking | Crouch/Sneak (kept in step with the game's sneaking) |
| A (either hand) | Jump |
| Trigger, empty hand | Activate (what the head looks at); held: Rest/Wait. The off hand, with a spell ready: charge it, let go (or touch, for a touch spell) to cast |
| Off hand's B | Menu Mode (inventory, magic, map, journal); held: the Quick Menu (hotkeys) |
| B on a menu | Menu Mode, which closes it (on the pause menu and its options, load and save: Escape) |
| The menu button | Escape (in the world, the pause menu) |
| Laser + trigger on a menu | The game's cursor and click |
| Right stick on a menu | The game's mouse wheel (scrolls the list under the laser) |
| Grip at the left hip / over a shoulder | Draw the weapon / take the shield |
| Grip on a thing in the world, or point and flick (the gloves) | Take it in the hand; let go in a body slot to take it into the inventory |
| A book or note taken in a hand | Read it (the game's book menu over the book) |
| The free hand's grip near an open book's pages, dragged across them | Turn the pages (leftwards: on; a page every 5 cm) |
| Grip at the right hip | Draw the map (the game's map in the hand); the other hand: spread it (either side), drag it (its face), touch it with a fingertip (tabs, places: fast travel) |
| Swing | Attack (the game's hit; a fast committed swing is a power attack) |
| Weapon or shield across a blow | Block |

## Settings (`rustvr-oblivion.toml` beside the DLL)

| Key | Default | What |
|---|---|---|
| `d3d9on12` | true | Make the device on D3D9On12 (off: plain Direct3D 9, no VR) |
| `background` | false | Tests: the window is off-screen and never activated, and message boxes are answered |
| `eye_width`, `eye_height` | 0 | The per-eye size to render at (0: the game's resolution) |
| `menu_background` | 0.8 | How opaque the menus' parchment is, of what the game makes it (1: as the game draws it) |
| `map_slot` | "hip_right" | The body slot the map is kept in: "hip_right", "hip_left", "belt" (the belt pouch on the weapon hand's side), "chest_left", "chest_right" |
