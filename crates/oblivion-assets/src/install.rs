//! Where the user's Oblivion is, found as the repository's scripts find it (`tools/paths.ps1`):
//! `RUSTVR_OBLIVION_DIR`, else `game` in `%LOCALAPPDATA%\RustVR\dev.toml`, else where the game's
//! installer left it in the registry, else the first of Steam's libraries that has it.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Oblivion's folder (the one with `Oblivion.exe`), if it can be found.
pub fn game_folder() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("RUSTVR_OBLIVION_DIR") {
        return Some(PathBuf::from(p));
    }
    if let Some(p) = dev_setting("game") {
        return Some(PathBuf::from(p));
    }
    let has_game = |p: &Path| p.join("Oblivion.exe").is_file();
    if let Some(p) = registry(r"HKLM\SOFTWARE\Bethesda Softworks\Oblivion", "Installed Path").map(PathBuf::from)
        && has_game(&p)
    {
        return Some(p);
    }
    let steam = PathBuf::from(registry(r"HKCU\Software\Valve\Steam", "SteamPath")?);
    let mut libraries = vec![steam.clone()];
    if let Ok(vdf) = std::fs::read_to_string(steam.join(r"steamapps\libraryfolders.vdf")) {
        // (Lines `"path"  "D:\\SteamLibrary"`.)
        libraries.extend(vdf.lines().filter_map(|l| {
            let mut quoted = l.split('"').skip(1).step_by(2);
            (quoted.next()? == "path").then(|| quoted.next().map(|p| PathBuf::from(p.replace(r"\\", r"\")))).flatten()
        }));
    }
    libraries.into_iter().map(|l| l.join(r"steamapps\common\Oblivion")).find(|g| has_game(g))
}

/// A string from `%LOCALAPPDATA%\RustVR\dev.toml` (`tools/dev.example.toml`).
fn dev_setting(key: &str) -> Option<String> {
    let path = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"RustVR\dev.toml");
    let table: toml::Table = std::fs::read_to_string(path).ok()?.parse().ok()?;
    table.get(key)?.as_str().map(str::to_owned)
}

/// A string value from the registry's 32-bit view (where the game's installer writes), by `reg`.
fn registry(key: &str, value: &str) -> Option<String> {
    let out = Command::new("reg").args(["query", key, "/v", value, "/reg:32"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (_, data) = text.lines().find_map(|l| l.split_once("REG_SZ"))?;
    let data = data.trim();
    (!data.is_empty()).then(|| data.to_owned())
}
