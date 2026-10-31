//! RustVR adapter for The Elder Scrolls IV: Oblivion (Steam, 1.2.0.416), loaded into the game as
//! `dinput8.dll`. Oblivion is 32-bit: build with `--target i686-pc-windows-msvc`.
//!
//! The game is the world and the runtime is the player. The adapter only translates:
//!
//! * **The picture** ([`d3d9`]): Oblivion's Direct3D 9 device is made on D3D9On12, so every frame it
//!   renders is a Direct3D 12 resource underneath; each eye is copied from there into the runtime's
//!   eye textures (through a shared Direct3D 12 texture and fence).
//! * **The eyes** ([`render`]): Oblivion's world render runs once per eye, with its camera put where
//!   the headset's eye is and given that eye's field of view ([`camera`]).
//! * **The link** ([`link`]): the runtime's frame, where tracking space is in Oblivion's world, and
//!   what the game is doing.
//! * **The player's body** ([`body`]): the race's skin and what's worn, as the parts of the
//!   runtime's avatar (the game's own models).
//! * **Combat** ([`combat`], [`weapons`], [`archery`]): the game's creatures and spells to the
//!   runtime, the equipped weapon as the object in the hand (the game's own model; a bow and its
//!   arrows), and what the hands did (hits, casts, arrows loosed) done by the game.
//! * **The world** ([`items`], [`world`], [`lock`]): the things lying about, to take in the hands;
//!   the doors and levers, worked by hand with the game's own activation; the game's lock, its pick
//!   in the hand.
//! * **Pockets** ([`pockets`]): the people near a sneaking player wear their own things in pockets
//!   on their skeletons; a hand that takes one out, or puts one in, is the game's own pickpocketing.
//! * **Riding** ([`riding`]): the game's horses, got on by the saddle, ridden by the reins, the
//!   player's body in the game's own seat on them.
//!
//! The rest is plumbing: the DLL proxy ([`proxy`]), the game's addresses ([`engine`]), hooking
//! ([`hooks`], [`iat`]), the game window ([`window`]), the game's settings ([`game_settings`]) and
//! starting the runtime ([`launcher`]).

#![allow(clippy::missing_safety_doc)]
#![allow(linker_messages)]

pub mod alchemy;
pub mod archery;
pub mod body;
pub mod books;
pub mod camera;
pub mod clock;
pub mod collision;
pub mod combat;
pub mod console;
pub mod d3d9;
pub mod depth;
pub mod distant_trees;
pub mod engine;
pub mod families;
#[cfg(all(windows, target_arch = "x86"))]
pub mod entry;
pub mod game_settings;
pub mod gore;
pub mod grabs;
pub mod grass;
pub mod hooks;
pub mod hud;
pub mod hud_info;
pub mod iat;
pub mod input;
pub mod inventory;
pub mod items;
pub mod launcher;
pub mod leaves;
pub mod lighting;
pub mod link;
pub mod lock;
pub mod log;
pub mod magic;
pub mod map;
pub mod mem;
pub mod menus;
pub mod mirror_spell;
pub mod movement;
pub mod nodes;
pub mod objects;
pub mod player;
pub mod pockets;
pub mod profile;
pub mod props;
pub mod proxy;
pub mod render;
pub mod resolution;
pub mod reticle;
pub mod riding;
#[cfg(target_arch = "x86")]
pub mod scene;
pub mod settings;
pub mod shapes;
pub mod slot_menu;
pub mod sounds;
pub mod swim;
pub mod sysmem;
pub mod tiles;
pub mod timing;
pub mod torch;
pub mod weapons;
pub mod window;
pub mod world;
pub mod xobse;
