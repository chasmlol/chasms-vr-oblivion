# Chasms VR: Oblivion

The Elder Scrolls IV: Oblivion in VR, on the [Chasms VR Framework](framework/README.md).

The game is the world and the framework's runtime is the player. Oblivion renders its world for
both eyes, and the runtime draws your hands, body and what they hold in it. The adapter
(`dinput8.dll`, beside `Oblivion.exe`) translates between the two. See
[docs/OBLIVION.md](docs/OBLIVION.md).

## Getting it

The framework is a git submodule in `framework/`:

```bat
git clone --recurse-submodules https://github.com/chasmlol/chasms-vr-oblivion.git
```

## Layout

| Path | What |
|---|---|
| `crates/oblivion` | The adapter (`dinput8.dll`, 32-bit like the game) |
| `crates/oblivion-assets` | `oblivion-extract`: reads Oblivion's archives and models (`.bsa`, `.nif`, `.dds`) |
| `framework/` | The Chasms VR Framework: the runtime, the protocol, the adapter SDK, the emulated headset |
| `docs/OBLIVION.md` | How the adapter works |
| `docs/research/oblivion-engine.md` | Oblivion's engine: addresses, structures, what starts it and what doesn't |
| `tools/` | Installing the adapter, and runs on the emulated headset |

## Building and playing

Oblivion is a 32-bit game, so the adapter is built for 32-bit Windows (`rustup target add
i686-pc-windows-msvc`). The runtime is built from the framework:

```bat
cargo build --release -p rustvr-oblivion --target i686-pc-windows-msvc
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-runtime
python tools\install_oblivion.py
```

Then start Oblivion from Steam. `python tools\install_oblivion.py --uninstall` takes the adapter
out again.

## Testing without a headset

```bat
powershell -ExecutionPolicy Bypass -File tools\xrsim_oblivion.ps1 -Script tools\xrsim\oblivion_walk.toml -Console tools\xrsim\oblivion_load.txt
```

This builds everything and installs the adapter. It runs the emulated headset, starts Oblivion with
its window off-screen and loads a save. Run it from an ordinary shell: in a sandbox, Steam's DRM
can't reach Steam and restarts the game through its launcher.

## License

[MIT](LICENSE)
