//! Oblivion's HUD in VR: which part of its 2D HUD is what, so the runtime places each piece
//! (`rustvr_protocol::hud`). The HUD has no alpha of its own (it's drawn over a cleared back
//! buffer, [`crate::render`]), so it's added to the view.
//!
//! Oblivion's main HUD (`menus\main\hud_main_menu.xml`) keeps its pieces anchored to the screen's
//! corners and centre at any aspect: the health, magicka and fatigue bars and the equipped weapon
//! and spell icons in the lower left, the compass at the bottom middle, the crosshair (and what it
//! can activate: "Open Door", a name) at the centre, the sneak eye above it; the card of what it's
//! on (its name and numbers) at the bottom right ([`crate::hud_info`]).

use rustvr_protocol::{HUD_CENTER, HUD_HELD, HUD_HINTS, HUD_STATUS, HUD_SUBTITLES, HUDL_ADDITIVE, HUDL_REST_HIDDEN, HudElement, HudLayout};

fn element(role: u32, uv: [f32; 4]) -> HudElement {
    HudElement { role, uv, ..Default::default() }
}

/// The layout; `card`: where the game's card of a thing is on the HUD (`crate::hud_info`), for the
/// thing in a hand.
pub fn layout(card: Option<[f32; 4]>) -> HudLayout {
    let mut elements = vec![
        // The bars, the equipped weapon and spell, the compass: on the off-hand wrist.
        element(HUD_STATUS, [0.0, 0.86, 0.5, 1.0]),
        // The crosshair and what it can activate: at gaze.
        element(HUD_CENTER, [0.3, 0.38, 0.7, 0.62]),
        // Spoken lines.
        element(HUD_SUBTITLES, [0.1, 0.62, 0.9, 0.86]),
        // Messages ("You have learned..."), top left.
        element(HUD_HINTS, [0.0, 0.0, 0.5, 0.16]),
    ];
    // The card of what the crosshair's on (or what a hand has): over the thing in a hand.
    if let Some(uv) = card {
        elements.push(element(HUD_HELD, uv));
    }
    HudLayout::new(HUDL_ADDITIVE | HUDL_REST_HIDDEN, &elements)
}
