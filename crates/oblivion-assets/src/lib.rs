//! Oblivion's game files, read for the RustVR runtime: its archives ([`bsa`]), models ([`nif`]),
//! faces ([`facegen`]) and the shapes the hands meet ([`hitbox`]).
//! Nothing here is ever written back into the game; what's extracted goes to `local_assets/`
//! (git-ignored: the files are the game's).

pub mod bench;
pub mod bsa;
pub mod dds;
pub mod export;
pub mod facegen;
pub mod hitbox;
pub mod horse;
pub mod install;
pub mod nif;
pub mod preview;
pub mod sounds;
pub mod ui;
