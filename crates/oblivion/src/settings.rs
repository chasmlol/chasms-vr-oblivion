//! The adapter's settings: `rustvr-oblivion.toml` beside the DLL (the game folder). Every key is
//! optional.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Oblivion's device on D3D9On12 (needed for VR; off: plain Direct3D 9, no VR).
    pub d3d9on12: bool,
    /// Tests: the game never takes focus or comes to the front.
    pub background: bool,
    /// The size Oblivion renders each eye at (`Oblivion.ini`'s iSize W/H while in VR); 0: the
    /// headset's own, as the runtime last reported it.
    pub eye_width: u32,
    pub eye_height: u32,
    /// How opaque the menus' backgrounds (their parchment) are as they float in the world, of what
    /// the game makes them (1: solid, as the game draws them); under 1 the world shows through
    /// them, the text and icons on them staying as the game has them (`crate::hud`).
    pub menu_background: f32,
    /// The body slot the map is kept in, drawn from by hand (`crate::map`): "hip_right", "hip_left",
    /// "belt" (the belt pouch on the weapon hand's side), "chest_left" or "chest_right"; empty (the
    /// default for now): it isn't on the body.
    pub map_slot: String,
    /// The main menu in a real place in Cyrodiil (`crate::worldmenu`); off: the flat main menu.
    pub menu_world: bool,
    /// Which place (`crate::worldmenu::SPOTS`: "imperial-city", "cloud-ruler", "dive-rock",
    /// "anvil", "rumare-night", "first-view"), or several separated by commas, one of them each time
    /// the game starts; empty: the Imperial City across Lake Rumare.
    pub menu_spot: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { d3d9on12: true, background: false, eye_width: 0, eye_height: 0, menu_background: 1.0, map_slot: String::new(), menu_world: true, menu_spot: String::new() }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();
static DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

pub fn parse(text: &str) -> Settings {
    toml::from_str(text).unwrap_or_else(|e| {
        tracing::warn!("settings: rustvr-oblivion.toml: {e}; using defaults");
        Settings::default()
    })
}

pub fn load_from(dir: &std::path::Path) {
    let _ = DIR.set(dir.to_path_buf());
    let s = std::fs::read_to_string(dir.join("rustvr-oblivion.toml")).map(|t| parse(&t)).unwrap_or_default();
    tracing::info!("settings: {s:?}");
    let _ = SETTINGS.set(s);
}

/// The game folder.
pub fn dir() -> Option<&'static std::path::Path> {
    DIR.get().map(|d| d.as_path())
}

pub fn get() -> &'static Settings {
    SETTINGS.get_or_init(Settings::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_keys_are_defaults() {
        let s = parse("background = true\n");
        assert!(s.background && s.d3d9on12 && s.menu_world);
        assert_eq!(s.eye_width, 0);
    }
}
