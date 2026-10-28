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
| The HUD | After the eyes, the back buffer is cleared, so what the game draws before it presents is its HUD and menus alone. The HUD is placed piece by piece and added to the view; outside the world, the whole picture is a menu panel. | `render.rs`, `hud.rs` |
| The player | Before each game frame, the player faces where the head looks (rotation and pitch). A third-person view goes back to first person. | `player.rs` |
| Input | The game's DirectInput keyboard and mouse read the runtime's actions, through the player's own `Oblivion.ini` bindings. While in VR the real keyboard and mouse don't reach the game. | `input.rs` |
| The window | It's always "active", so the game never pauses for focus. In the test mode it's off-screen and never activated. | `window.rs` |
| Settings | `Oblivion.ini` is windowed and skips the intro films while in VR. The player's values are kept and put back. | `game_settings.rs` |
| Tests | `rustvr-console-test` runs timed commands from the game's frame: `LoadGame <save>`, or any line of script. | `console.rs` |

## Controls (so far)

| VR | Oblivion |
|---|---|
| Left stick | Forward / Back / Slide Left / Slide Right, where the head looks |
| Jump | Jump |
| Use | Activate |
| Crouch | Crouch/Sneak |
| Inventory | Menu Mode |
| Menu button | Escape |

## Settings (`rustvr-oblivion.toml` beside the DLL)

| Key | Default | What |
|---|---|---|
| `d3d9on12` | true | Make the device on D3D9On12 (off: plain Direct3D 9, no VR) |
| `background` | false | Tests: the window is off-screen and never activated, and message boxes are answered |
| `eye_width`, `eye_height` | 0 | The per-eye size to render at (0: the game's resolution) |
