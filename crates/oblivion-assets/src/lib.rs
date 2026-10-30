//! Oblivion's game files, read for the RustVR runtime: its archives ([`bsa`]), models ([`nif`]) and
//! faces ([`facegen`]).
//! Nothing here is ever written back into the game; what's extracted goes to `local_assets/`
//! (git-ignored: the files are the game's).

pub mod bsa;
pub mod dds;
pub mod export;
pub mod facegen;
pub mod horse;
pub mod nif;
pub mod preview;
pub mod sounds;
