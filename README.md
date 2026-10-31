# Chasms VR: Oblivion

The Elder Scrolls IV: Oblivion in VR, played with your hands and your whole body.

It runs on RustVR (the [Chasms VR Framework](framework/README.md)), a VR runtime written in Rust
that any flat game can connect to. The runtime is a 64-bit program of its own: it owns the headset
and the player's body, hands, holsters and menus. The game connects to it through an adapter, here
a 32-bit `dinput8.dll` that sits beside `Oblivion.exe`. The adapter renders Oblivion's world once
for each eye, tells the runtime what the game is doing, and turns what your hands and body do into
the game's own actions. The two talk over shared memory (`Local\RustVR_v17`, the framework's
`rustvr-protocol`).

The game is the source of truth: the hits, the spells, the blood, the sounds and the models are
Oblivion's own. Starting Oblivion starts the runtime with it.

## What's in it

- **Seeing the world.** Each eye is rendered by the game at the headset's own field of view and
  resolution (with an overscan for the desktop view), through Direct3D 9 on 12. The fog is by
  distance, the same in both eyes; the distant trees, the leaves, the grass and the lighting are
  fixed for stereo; textures get anisotropic filtering. Loading screens keep the last picture of
  the world round you.
- **Hands and body.** Physics hands, and a full body solved from the head and hands (IK), drawn as
  your character: their race, skin and what they wear, armour moving as rigid plates. Your
  character's lighting falls on them. People look at your real head.
- **Fighting.** Swing the game's own weapons: hits are Oblivion's hits, a weapon or shield across a
  blow is a block, and cuts bring Oblivion's blood and dismemberment. Archery: the bow from your
  back, an arrow from the quiver, nocked, drawn and loosed. Spells are readied, charged and cast
  from the hand, with the game's casting art; staffs cast from their tip. Grab people and creatures.
- **Moving.** Walk with the stick where you look, snap or smooth turn, jump. Crouch for real to
  sneak. Swim with your arms. Ride horses by the reins.
- **The world by hand.** Pick things up, throw them, put them in holsters on your body (your
  inventory, by category, with the holsters' editor to set them up). Open doors and work levers by
  hand. Pick locks with the pick in your hand, and pockets by reaching into them. Read books and
  notes held in the hands. Alchemy at a table with the game's own mortar, pestle and apparatus. Eat
  and drink by holding food to your mouth, read a scroll by holding it up. A burning torch goes in
  any holster it fits, like a weapon.
- **Menus.** The game's menus and HUD float in the world and are pointed at with a laser. The main
  menu stands in a real place in Cyrodiil. In the race menu you shape your face by hand, with a big
  copy of your head and a mirror. Every character has a Mirror power that holds up a mirror.
- **Simple voice commands.** Say a menu's words to pick them, or ask someone what they have for
  sale, with Windows' own offline speech recognition.
- **The desktop view.** A window on the monitor, for recording and streaming, shows a steadied
  camera with looks to choose (motion blur, depth of field, lens and film effects, colour grades), a
  zoom while you aim and a walking bob that never reaches the headset. OBS can be seen in the
  headset on the back of your left wrist.
- **Talking to people out loud** (an optional plugin, not in the default build): see
  [The voice plugin](#the-voice-plugin).

[docs/OBLIVION.md](docs/OBLIVION.md) has every feature, how it's done and where in the code.
`docs/playtest/` has how to try each one, and `docs/research/` what each is built on.

## What you need

- Windows 10 or 11.
- Oblivion's Steam release (1.2.0.416, Game of the Year Deluxe or the base game; other releases
  aren't supported), signed in to Steam. xOBSE and its plugins (such as Oblivion Stutter Remover)
  work beside it.
- A PC VR headset with an OpenXR runtime: SteamVR, Virtual Desktop, or Meta's Quest Link.
- To build: [rustup](https://rustup.rs) (the repository pins Rust 1.99 in `rust-toolchain.toml`)
  with the 32-bit target, Visual Studio's C++ build tools and CMake (the OpenXR loader is built from
  source), and Python 3.11 or newer for the tools.

## Getting it

The framework is a git submodule in `framework/`:

```bat
git clone --recurse-submodules https://github.com/chasmlol/chasms-vr-oblivion.git
cd chasms-vr-oblivion
rustup target add i686-pc-windows-msvc
```

## Building

The adapter is 32-bit like the game; the runtime is 64-bit, built from the framework into the same
`target` folder:

```bat
cargo build --release -p rustvr-oblivion --target i686-pc-windows-msvc
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-runtime -p rustvr-range
```

If `cargo` on your PATH isn't rustup's (a standalone Rust install can shadow it), put
`%USERPROFILE%\.cargo\bin` first.

## The game's models and sounds

The runtime draws the game's own models (your weapons, body, armour, the horse's saddle, what's in
people's pockets) and plays its sounds. They are read once from your install into
`local_assets/oblivion`, which is git-ignored: they're the game's files, never to be committed or
shared. `oblivion-extract` finds the game through its registry key or Steam (or `--game <folder>`):

```bat
cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons meshes/staffs meshes/lights meshes/characters meshes/armor meshes/clothes meshes/creatures/horse textures/characters textures/menus/misc/hud_reticle.dds meshes/clutter meshes/oblivion/sigil meshes/dungeons/misc
cargo run --release -p oblivion-assets --bin oblivion-extract -- sounds
cargo run --release -p oblivion-assets --bin oblivion-extract -- bench meshes/clutter/magesguild/mortarpestle
cargo run --release -p oblivion-assets --bin oblivion-extract -- export-ui fonts textures/menus/alchemy textures/menus/book textures/menus/icons/magic textures/menus/icons/clutter
```

[docs/OBLIVION.md](docs/OBLIVION.md), "Setting up", says what each brings.

## Installing

```bat
python tools\install_oblivion.py
```

This copies the adapter (`dinput8.dll`) beside `Oblivion.exe` and writes `rustvr-runtime.txt` there,
pointing at `target\release\rustvr-play.exe`. A `dinput8.dll` already there is moved into
`rustvr_backup\` first. `python tools\install_oblivion.py --uninstall` takes it all out again and
puts the backup back. The script finds the game as `oblivion-extract` does; `--game <folder>` names
it.

## Playing

Put the headset on (or connect it in Virtual Desktop) and start Oblivion from Steam. The adapter
starts the runtime in the background, which takes the headset. While the game runs in VR, the
adapter sets a few of its `Oblivion.ini` values for the headset (each eye's size, in a window, and
some settings raised to what a headset needs) and puts your own back when it closes.

The menu button opens the game's pause menu, and RustVR's own menu floats beside it, starting on a
Help page that lists the controls. [docs/OBLIVION.md](docs/OBLIVION.md), "Controls", has the whole
table.

## Settings

RustVR's settings are in its in-headset menu, and saved in `%LOCALAPPDATA%\RustVR\settings.toml`
(the same file for every game; per-game overrides go in `profiles\<adapter>.toml` there). The tabs:

| Tab | What's in it |
|---|---|
| Help | The controls, by topic |
| Comfort | Turning (snap or smooth), the walking direction (head or off hand), roomscale, ducking to crouch and the stealth crouch amount, camera bob, vignettes and the fade in walls, riding (the view's turn, its vignette, the reins) |
| Hands | Physics and ghost hands, grip toggles, the gravity gloves, grab reach, weight, throwing, eating, drinking and reading by hand, the holsters' editor, the quiver and the bow, interaction sounds |
| Voice | Simple voice commands, and the voice plugin's settings (its microphone, the voices' volume, acting out, streaming, talking over them) |
| Body | Seated mode, your real height or the character's, your height and arm span (or measure them), how much body shows, the game's light on it, body tracking, hand size, the mirror |
| Controls | The weapon hand, two-handed aiming, gestures, ability flicks, melee by swinging, the stick's dead zone, swimming with your arms |
| Weapon | How a held gun sits in the hand, picture-in-picture scopes |
| Display | Supersampling, world scale, the picture's timing and smoothing, the refresh rate, the overscan, the HUD's, subtitles' and menus' place and size, the desktop view |
| Studio | The desktop view's looks and effects, its field of view, the zoom while aiming, its bob |

The adapter has a few settings of its own in `rustvr-oblivion.toml` beside `Oblivion.exe` (every
key optional: the main menu's place in Cyrodiil, the menus' background, anisotropic filtering,
each eye's size); [docs/OBLIVION.md](docs/OBLIVION.md) lists them. Logs go to
`%LOCALAPPDATA%\RustVR\rustvr.log` (the runtime) and `rustvr-oblivion.log` in the game's folder.

## The voice plugin

Talking to Cyrodiil's people out loud, and hearing them answer in their own voices with their lips
moving, is an optional plugin for those who want to run the servers it needs. It isn't in the
default build: the commands above, and `tools\oblivion.ps1`, leave it out.

It needs three servers: speech to text (m8's `stt-server` with the Parakeet model), the cloned
voices (Breeze TTS 2's server), and an LLM with an OpenAI-style API (hosted, or a local one). m8 is
[chasmlol/m8](https://github.com/chasmlol/m8), branch `rustvr`; the plugin builds it in.

1. Build the plugin into the same folder as the runtime, so `rustvr-voice.exe` sits beside
   `rustvr-play.exe`:

   ```bat
   cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-voice
   ```

2. Say where the servers and their models are: copy `tools\dev.example.toml` to
   `%LOCALAPPDATA%\RustVR\dev.toml` and fill in its `[voice]` lines.
3. Start them: `powershell -ExecutionPolicy Bypass -File clips\voice-servers.ps1`. It starts what
   isn't up, waits for it, and turns RustVR's Voice setting on.
4. The plugin's first start writes `%LOCALAPPDATA%\RustVR\voice.toml`: set its `[llm]` (the URL and
   model), and its servers' URLs if yours differ. Your API key goes in its `api_key`, or in the
   environment variable its `api_key_env` names. It stays on your machine: never put a key in a
   repository.

The Voice tab has the plugin's settings. [docs/playtest/voice.md](docs/playtest/voice.md) has the
rest.

## How the repositories fit together

| Repository | What |
|---|---|
| chasms-vr-oblivion (this one) | The Oblivion adapter (`crates/oblivion`), `oblivion-extract` (`crates/oblivion-assets`), the armour marks (`content/armour`), the tools and the docs |
| [chasms-vr-framework](framework/README.md), in `framework/` | The runtime, the protocol, the adapter SDK, the test range, the emulated headset, the voice plugin. Shared by every game's adapter |
| [chasmlol/m8](https://github.com/chasmlol/m8) (public, branch `rustvr`) | The voice plugin's speech pipeline, a git dependency of `framework/crates/voice` |

Metro 2033 Redux has its own adapter repository on the same framework.

| Path | What |
|---|---|
| `crates/oblivion` | The adapter (`dinput8.dll`) |
| `crates/oblivion-assets` | `oblivion-extract`: reads Oblivion's archives, models, textures and sounds into `local_assets/` |
| `crates/oblivion/data/bios.toml` | A paragraph on each of the game's people, for the voice plugin (written from UESP's pages, CC BY-SA 2.5) |
| `content/armour/oblivion.toml` | Which parts of the game's armour models are rigid plates ([framework/docs/ARMOUR.md](framework/docs/ARMOUR.md)) |
| `docs/` | [OBLIVION.md](docs/OBLIVION.md) (how the adapter works), [DEVELOPING.md](docs/DEVELOPING.md) (working on it), `playtest/`, `research/` |
| `tools/` | Installing, the fast boot for checks and playtests (`oblivion.ps1`, `play.ps1`), emulated-headset scripts (`xrsim/`) |
| `clips/` | Batch files that boot the trailer's shots ([clips/CLIPS.md](clips/CLIPS.md)) |

## Working on it

[docs/DEVELOPING.md](docs/DEVELOPING.md): the one-command boot into a test save, checks on the
emulated headset, feature worktrees and playtests, and the rules the project keeps (the game's own
data first, none of the game's code or files committed).

## License

[MIT](LICENSE). Oblivion and its files are Bethesda's: nothing of the game is in this repository.
