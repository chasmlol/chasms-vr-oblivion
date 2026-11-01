//! Grass all round the player. The game grows grass only in a cone in front of its camera: once a
//! frame the player's camera update (0x66B710) asks the grass build (0x4EBDA0) to build it again
//! once the camera has moved 64 units or turned past a threshold, and for each loaded cell the
//! build (0x4EB3F0) keeps a 512-unit grass patch near the camera, or one with a corner within twice
//! the cone's half-angle of where the camera looks. The cone is sized from the flat screen's field
//! of view (46 degrees either side at 75), rebuilt after a 19-degree turn. A headset sees far wider
//! and turns all the time: the grass came and went at the edges of the view, and patches grew as
//! the head turned to them.
//!
//! Two of the game's own branches are made unconditional (`docs/design/visuals.md`):
//! * the build keeps every patch as it keeps a near one (0x4EB632, `jnz` to the near path, made a
//!   `jmp`), so the grass stands all round, to the game's own grass distance (its cells and
//!   quadrants past `fGrassEndDistance` are dropped before this, as the game drops them);
//! * the update no longer builds again for a turn of the head (0x66C10E, the `jz` to the build on a
//!   turn, made two `nop`s), which with no cone only cost it time; it still builds again when the
//!   player has moved 64 units, and when the game forces it.
//!
//! The patches near the player are about three times as many: the build costs more, the grass is
//! drawn in the same groups (each eye culls them).

/// The build's test of a patch's distance: `jnz` (near: keep it) to 0x4EB82B.
const KEEP_AT: usize = 0x004E_B632;
const KEEP_GAME: [u8; 6] = [0x0F, 0x85, 0xF3, 0x01, 0x00, 0x00];
/// `jmp 0x4EB82B`, `nop`.
const KEEP_ALL: [u8; 6] = [0xE9, 0xF4, 0x01, 0x00, 0x00, 0x90];
/// The update's test of the turn since the last build: `jz` to the build.
const TURN_AT: usize = 0x0066_C10E;
const TURN_GAME: [u8; 2] = [0x74, 0x0E];
const TURN_NONE: [u8; 2] = [0x90, 0x90];

/// Testing, `RUSTVR_OBLIVION_GAME_GRASS=1`: the game's own cone.
fn enabled() -> bool {
    !std::env::var("RUSTVR_OBLIVION_GAME_GRASS").is_ok_and(|v| v == "1")
}

pub fn install() {
    if !enabled() {
        tracing::info!("grass: testing: grown in the game's own cone in front (RUSTVR_OBLIVION_GAME_GRASS)");
        return;
    }
    // (The turn's build is only let go once the cone is: without it the grass would stay where the
    // last build's cone was.)
    if !crate::mem::patch_code(KEEP_AT, &KEEP_GAME, &KEEP_ALL) {
        tracing::error!("grass: the build's cone isn't the one known: grown in the game's own cone in front");
        return;
    }
    let turn = crate::mem::patch_code(TURN_AT, &TURN_GAME, &TURN_NONE);
    tracing::info!("grass: grown all round the player{}", if turn { "" } else { " (built again on turning, as the game does)" });
}
