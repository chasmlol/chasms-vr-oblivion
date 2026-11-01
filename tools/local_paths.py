"""Where this machine keeps the game, for the Python tools (as tools/paths.ps1 does for the scripts).

Oblivion's folder is RUSTVR_OBLIVION_DIR, else `game` in %LOCALAPPDATA%\\RustVR\\dev.toml
(tools/dev.example.toml), else where the game's installer left it in the registry, else the first
of Steam's libraries that has it.
"""
import os
import re
import tomllib

DEV_CONFIG = os.path.join(os.environ.get("LOCALAPPDATA", ""), "RustVR", "dev.toml")


def dev_setting(key, section=None):
    """A value from dev.toml, or None."""
    try:
        with open(DEV_CONFIG, "rb") as f:
            cfg = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError):
        return None
    if section:
        cfg = cfg.get(section) or {}
    return cfg.get(key)


def _registry(root, key, value):
    try:
        import winreg
        with winreg.OpenKey(root, key) as k:
            return winreg.QueryValueEx(k, value)[0]
    except OSError:
        return None


def find_oblivion():
    """Oblivion's folder from the registry (the game's installer, then Steam's libraries), or None."""
    import winreg
    p = _registry(winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Bethesda Softworks\Oblivion", "Installed Path")
    if p and os.path.exists(os.path.join(p, "Oblivion.exe")):
        return os.path.normpath(p)
    steam = _registry(winreg.HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath")
    if not steam:
        return None
    libraries = [steam]
    try:
        with open(os.path.join(steam, "steamapps", "libraryfolders.vdf"), encoding="utf-8") as f:
            libraries += [m.replace("\\\\", "\\") for m in re.findall(r'"path"\s+"([^"]+)"', f.read())]
    except OSError:
        pass
    for lib in libraries:
        g = os.path.join(lib, "steamapps", "common", "Oblivion")
        if os.path.exists(os.path.join(g, "Oblivion.exe")):
            return os.path.normpath(g)
    return None


def oblivion():
    """Oblivion's folder (see the module's doc), or None."""
    return os.environ.get("RUSTVR_OBLIVION_DIR") or dev_setting("game") or find_oblivion()
