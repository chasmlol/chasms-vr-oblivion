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
| The eyes | The world render runs once per eye each frame, with the camera at that eye's pose and the eye's own asymmetric frustum. The game's first-person arms aren't drawn. The sky is centred on each eye (at infinity for both) and the weather on the head; each eye is a frame of the renderer's own, so the actors' skinned bodies are made for its camera. What else the game makes once a frame: `docs/research/visuals.md`. | `render.rs`, `camera.rs` |
| Resolution | The game renders each eye at the headset's eye size as the runtime asks for it (its recommended size times the runtime's Resolution scale): the window, back buffer and every render target are made at that size. | `resolution.rs` |
| Depth | Each eye's depth buffer (resolved from its samples under the game's antialiasing) goes to the runtime as linear depth, so the hands, the body and what they hold are hidden behind the world. | `d3d9.rs`, `depth.rs` |
| Collision | The runtime's physics gets the world Oblivion's own Havok has: every fixed body's shapes near the player (the land, buildings, rocks, trees, furniture) as triangles, so what the hands hold, drop and throw stops at it and lands on it. | `collision.rs`, `docs/research/oblivion-havok.md` |
| The HUD | After the eyes, the back buffer is cleared, so what the game draws before it presents is its HUD and menus alone. The HUD is placed piece by piece and added to the view; outside the world, the whole picture is a menu panel. | `render.rs`, `hud.rs` |
| The player | Before each game frame, the player faces where the head looks (rotation and pitch). A third-person view goes back to first person. | `player.rs` |
| Input | The game's DirectInput keyboard and mouse read the runtime's actions, through the player's own `Oblivion.ini` bindings. While in VR the real keyboard and mouse don't reach the game. | `input.rs` |
| The window | It's always "active", so the game never pauses for focus. In the test mode it's off-screen and never activated. | `window.rs` |
| Settings | `Oblivion.ini` is windowed and skips the intro films while in VR. The player's values are kept and put back. | `game_settings.rs` |
| The body | The runtime draws the player's body from the game's own models: the race's skin where nothing is worn, the armour and clothes worn over it, on the game's skeleton. The head: the race's head and the parts made to it (ears, mouth, teeth, tongue, eyes), the hair (unless a helmet or hood covers its slot) and what's worn on the head, each with the race's, the eyes' or the hair's own texture. The face: FaceGen's coordinates (the race's plus the NPC's), the age map FaceGen's age picks, the hair's colour; the runtime morphs the head's parts and makes its texture as the game does. In first person the head stays out of the player's eyes. | `body.rs`, `docs/research/mirror.md` |
| The mirror | A standing mirror the player calls up (the runtime's Body tab, or pulling the gripped hands apart in front of the chest). The game's world in it is one more render a frame from the head reflected in the glass, its near plane on the glass, copied with its depth; the runtime draws each eye's reflection with the player in it. | `render.rs`, `d3d9.rs` |
| The race menu | Opened by a new game or `showracemenu`: the world stays round the player, a mirror stands in front of them and the menu (the part of the picture its tiles take) beside it, worked by the laser. The game's own third-person player is hidden; every change in the menu (race, sex, face, hair, eyes) shows on the mirrored body as it's made. | `menus.rs`, `hud_info.rs`, `link.rs` |
| Lighting | The body and what the hands hold are lit as Oblivion's own shaders light them: the scene's sun or moon, the point lights near the player (torches, candles, spells), the ambient (Night-Eye's too), normal maps, specular, the skin's rim, glow maps and the fog. | `lighting.rs`, `docs/research/oblivion-lighting.md` |
| Weapons | The equipped weapon (the game's model) hangs at the left hip, or on the back if two-handed, and the hand draws it. The equipped shield is taken from over the left shoulder. Edges, points, heads and guards are measured on each model. | `weapons.rs` |
| Combat | Hits are the game's own hit with the equipped weapon (damage, skill, enchantments). A blow met by the weapon or shield is the game's block. Spells charge in the palm and go off as the game's own cast: target spells from the palm along the aim, touch spells into what the hand touches, self spells on the player. | `combat.rs` |
| Things in the world | Everything the player can take near them (weapons, armour, clothes, books, potions, ingredients, keys, soul gems, sigil stones, apparatus, carriable lights, clutter) is the hands' to take or pull with the gloves: its collision its own Havok shape, its weight its Havok mass, drawn by the game where the hand has it (a candle still burns), and let go, the game's again, flying on with the throw. A card beside it says what it is: its name, damage or armour, weight, value, condition, and whether it's stolen. Let go in any body slot (a hip, a shoulder, the chest, the belt), the player takes it: the game's own pick-up, with its stealing and its message. | `items.rs`, `objects.rs` |
| Menus | While a menu is open it's a panel; the game's cursor follows the laser and the trigger clicks. | `menus.rs` |
| Tests | `rustvr-console-test` runs timed commands from the game's frame: `LoadGame <save>`, `SelectSpell <name or id>`, `PlaceInFront <ref> <metres>`, or any line of script (base forms by hex id). Compile errors are logged. | `console.rs` |

## Setting up

The game's models are converted once into `local_assets/oblivion` (never committed):

```
cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons meshes/characters meshes/armor meshes/clothes textures/characters
```

(`meshes/characters` brings the races' heads and the parts made to them, the hair and their FaceGen
morphs, `morphs/*.rvf`, and FaceGen's age control, `facegen.toml`.)

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
| Swing | Attack (the game's hit; a fast committed swing is a power attack) |
| Weapon or shield across a blow | Block |

## Settings (`rustvr-oblivion.toml` beside the DLL)

| Key | Default | What |
|---|---|---|
| `d3d9on12` | true | Make the device on D3D9On12 (off: plain Direct3D 9, no VR) |
| `background` | false | Tests: the window is off-screen and never activated, and message boxes are answered |
| `eye_width`, `eye_height` | 0 | The per-eye size to render at (0: the game's resolution) |
| `anisotropy` | 16 | The anisotropic filtering of the game's textures (1: none, as the game has it). Oblivion has none of its own, and a driver's override doesn't reach D3D9On12 |
