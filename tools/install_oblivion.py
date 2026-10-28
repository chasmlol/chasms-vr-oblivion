"""Installs RustVR's Oblivion adapter into the game, or takes it out again.

    python tools/install_oblivion.py [--game <Oblivion folder>] [--uninstall]

Installing copies the adapter (`target/i686-pc-windows-msvc/release/dinput8.dll`, built with
`cargo build --release -p rustvr-oblivion --target i686-pc-windows-msvc`) next to `Oblivion.exe` and writes `rustvr-runtime.txt` there,
pointing at `target/release/rustvr-play.exe` (the framework's runtime, see README.md): starting Oblivion then starts the RustVR runtime in the
background, which takes the headset. Anything already in the way (another `dinput8.dll`) is moved into `rustvr_backup/` first; uninstalling removes RustVR's files and
puts the backup back.
"""
import argparse
import os
import shutil
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion"
OURS = ["dinput8.dll", "dinput8.pdb", "rustvr-runtime.txt"]
IN_THE_WAY = ["dinput8.dll", "dinput8.pdb"]
BACKUP = "rustvr_backup"



def copy(src, dst):
    """Copies, waiting up to 15 s for a game that's closing to let go of the file."""
    for attempt in range(60):
        try:
            shutil.copy2(src, dst)
            return
        except PermissionError:
            if attempt == 59:
                raise
            time.sleep(0.25)

def is_ours(path):
    try:
        with open(path, "rb") as f:
            return b"rustvr-oblivion" in f.read()
    except OSError:
        return False


def install(game):
    dll = os.path.join(REPO, "target", "i686-pc-windows-msvc", "release", "dinput8.dll")
    play = os.path.join(REPO, "target", "release", "rustvr-play.exe")
    for p in (dll, play):
        if not os.path.exists(p):
            sys.exit(f"missing {p}: cargo build --release -p rustvr-oblivion --target i686-pc-windows-msvc, and cargo build --release "
                     "--manifest-path framework/Cargo.toml --target-dir target -p rustvr-runtime -p rustvr-range")
    backup = os.path.join(game, BACKUP)
    for name in IN_THE_WAY:
        p = os.path.join(game, name)
        if not os.path.exists(p) or (name == "dinput8.dll" and is_ours(p)):
            continue
        if name == "dinput8.pdb" and is_ours(os.path.join(game, "dinput8.dll")):
            continue
        os.makedirs(backup, exist_ok=True)
        dest = os.path.join(backup, name)
        if os.path.exists(dest):
            sys.exit(f"{dest} already exists: look at {backup} before installing again")
        shutil.move(p, dest)
        print(f"moved {name} -> {BACKUP}\\{name}")
    copy(dll, os.path.join(game, "dinput8.dll"))
    pdb = os.path.join(REPO, "target", "i686-pc-windows-msvc", "release", "dinput8.pdb")
    if os.path.exists(pdb):
        copy(pdb, os.path.join(game, "dinput8.pdb"))
    with open(os.path.join(game, "rustvr-runtime.txt"), "w", encoding="utf-8") as f:
        f.write("# The RustVR runtime Oblivion starts in the background (tools/install_oblivion.py).\n")
        f.write(f'"{play}"\n')
    print(f"installed the RustVR adapter into {game}")
    print(f"  runtime: {play}")
    print("  start Oblivion (with the headset on): RustVR starts with it.")


def uninstall(game):
    for name in OURS:
        p = os.path.join(game, name)
        if name.startswith("dinput8") and os.path.exists(os.path.join(game, "dinput8.dll")) and not is_ours(os.path.join(game, "dinput8.dll")):
            continue
        if os.path.exists(p):
            os.remove(p)
            print(f"removed {name}")
    backup = os.path.join(game, BACKUP)
    if os.path.isdir(backup):
        for name in os.listdir(backup):
            dest = os.path.join(game, name)
            if os.path.exists(dest):
                print(f"{name} is in the way: left in {BACKUP}")
                continue
            shutil.move(os.path.join(backup, name), dest)
            print(f"restored {name}")
        if not os.listdir(backup):
            os.rmdir(backup)


def restore_ini(game):
    """Puts the player's Oblivion.ini values back (the adapter's backup) when the game was closed
    without the adapter doing it (killed after a test)."""
    backup = os.path.join(game, "rustvr-oblivion-ini.txt")
    ini = os.path.join(os.path.expanduser("~"), "Documents", "My Games", "Oblivion", "Oblivion.ini")
    if not os.path.exists(backup) or not os.path.exists(ini):
        return
    lines = open(ini, encoding="latin-1").read().splitlines()
    for entry in open(backup, encoding="latin-1").read().splitlines():
        parts = entry.split("\t", 2)
        if len(parts) != 3:
            continue
        section, key, value = parts
        current = None
        for i, line in enumerate(lines):
            t = line.strip()
            if t.startswith("["):
                current = t[1:-1].lower()
            elif current == section.lower() and "=" in t and t.split("=", 1)[0].strip().lower() == key.lower():
                lines[i] = f"{key}={value}"
    open(ini, "w", encoding="latin-1", newline="").write("\r\n".join(lines) + "\r\n")
    os.remove(backup)
    print("Oblivion.ini is the player's again")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--game", default=DEFAULT_GAME)
    ap.add_argument("--uninstall", action="store_true")
    ap.add_argument("--restore-ini", action="store_true")
    a = ap.parse_args()
    if not os.path.exists(os.path.join(a.game, "Oblivion.exe")):
        sys.exit(f"no Oblivion.exe in {a.game}")
    if a.restore_ini:
        restore_ini(a.game)
        return
    (uninstall if a.uninstall else install)(a.game)


if __name__ == "__main__":
    main()
