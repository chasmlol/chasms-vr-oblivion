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
| The HUD | After the eyes, the back buffer is cleared, so what the game draws before it presents is its HUD and menus alone. The HUD is placed piece by piece and added to the view; outside the world, the whole picture is a menu panel. | `render.rs`, `hud.rs` |
| The player | Before each game frame, the player faces where the head looks (rotation and pitch). A third-person view goes back to first person. | `player.rs` |
| Input | The game's DirectInput keyboard and mouse read the runtime's actions, through the player's own `Oblivion.ini` bindings. While in VR the real keyboard and mouse don't reach the game. | `input.rs` |
| The window | It's always "active", so the game never pauses for focus. In the test mode it's off-screen and never activated. | `window.rs` |
| Settings | `Oblivion.ini` is windowed and skips the intro films while in VR. The player's values are kept and put back. | `game_settings.rs` |
| The body | The runtime draws the player's body from the game's own models: the race's skin where nothing is worn, the armour and clothes worn over it, on the game's skeleton. | `body.rs` |
| Lighting | The body and what the hands hold are lit as Oblivion's own shaders light them: the scene's sun or moon, the point lights near the player (torches, candles, spells), the ambient (Night-Eye's too), normal maps, specular, the skin's rim, glow maps and the fog. | `lighting.rs`, `docs/research/oblivion-lighting.md` |
| Weapons | The equipped weapon (the game's model) hangs at the left hip, or on the back if two-handed, and the hand draws it. The equipped shield is taken from over the left shoulder. Edges, points, heads and guards are measured on each model. | `weapons.rs` |
| Combat | Hits are the game's own hit with the equipped weapon (damage, skill, enchantments). A blow met by the weapon or shield is the game's block. Spells charge in the palm and go off as the game's own cast: target spells from the palm along the aim, touch spells into what the hand touches, self spells on the player. | `combat.rs` |
| Things in the world | Everything the player can take near them (weapons, armour, clothes, books, potions, ingredients, keys, soul gems, sigil stones, apparatus, carriable lights, clutter) is the hands' to take or pull with the gloves: its collision its own Havok shape, its weight its Havok mass, drawn by the game where the hand has it (a candle still burns), and let go, the game's again, flying on with the throw. A card beside it says what it is: its name, damage or armour, weight, value, condition, and whether it's stolen. Let go in any body slot (a hip, a shoulder, the chest, the belt), the player takes it: the game's own pick-up, with its stealing and its message. | `items.rs`, `objects.rs` |
| Alchemy | Every mortar and pestle near the player is a bench: the runtime draws it as two pieces from the game's model (the mortar, a bowl the hands use; the pestle, the hands' own), the game's own model hidden. Up to four ingredients go into the bowl, each kind once (the game's menu's rules), and lie in it as themselves, shrinking as the pestle grinds them; the alembic, calcinator and retort within 75 cm count, the best of each kind. Whenever what's in the bowl changes, the game's own alchemy menu is made for a moment and gone again (no menu mode) to work out what it would make, shown on a card of its own art, icons and fonts over the mortar. All ground, its Create makes the potion (the ingredients picked up and used, the skill, the message, the sound) and the bottle rises out of the mortar to be taken; with no effect shared, the game's own failure words and sound, the ingredients lost. Not in combat (the game's rule). See [research/alchemy.md](research/alchemy.md). | `alchemy.rs` |
| Menus | While a menu is open it's a panel; the game's cursor follows the laser and the trigger clicks. | `menus.rs` |
| Tests | `rustvr-console-test` runs timed commands from the game's frame: `LoadGame <save>`, `SelectSpell <id>`, `PlaceInFront <ref> <metres>`, `PlaceAt <base> <ahead> <right> <up> [<turn>]` (a new one of a base form there, the player's own, settled where it falls: a bench set out), or any line of script (base forms by hex id). Compile errors are logged. | `console.rs` |

## Setting up

The game's models are converted once into `local_assets/oblivion` (never committed):

```
cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons meshes/characters/_male meshes/characters/khajiit meshes/characters/argonian meshes/armor meshes/clothes textures/characters
```

For alchemy, the mortars and pestles split and measured, and the game's fonts and menu art for its
cards and its note (the book menu's scroll):

```
cargo run --release -p oblivion-assets --bin oblivion-extract -- bench meshes/clutter/magesguild/mortarpestle
cargo run --release -p oblivion-assets --bin oblivion-extract -- export-ui fonts textures/menus/alchemy textures/menus/book textures/menus/icons/magic textures/menus/icons/clutter
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
| B on a menu, or the menu button | Escape (closes the menu; in the world, the pause menu) |
| Laser + trigger on a menu | The game's cursor and click |
| Grip at the left hip / over a shoulder | Draw the weapon / take the shield |
| Grip on a thing in the world, or point and flick (the gloves) | Take it in the hand; let go in a body slot to take it into the inventory |
| An ingredient let go over a mortar; the pestle worked round in it | Into the bowl; ground, the game makes the potion and its bottle rises out of the mortar |
| Swing | Attack (the game's hit; a fast committed swing is a power attack) |
| Weapon or shield across a blow | Block |

## Settings (`rustvr-oblivion.toml` beside the DLL)

| Key | Default | What |
|---|---|---|
| `d3d9on12` | true | Make the device on D3D9On12 (off: plain Direct3D 9, no VR) |
| `background` | false | Tests: the window is off-screen and never activated, and message boxes are answered |
| `eye_width`, `eye_height` | 0 | The per-eye size to render at (0: the game's resolution) |
