//! Alchemy by hand: Oblivion's own apparatus set out on a table and used with the hands. The
//! game's mortar and pestle becomes two pieces, the mortar a bowl ([`rustvr_protocol::Receptacle`])
//! that takes up to four ingredients dropped into it (the game's menu has four slots, each kind
//! once), the pestle the runtime's own piece that grinds them. The alembic, calcinator and retort
//! count when they stand within reach of the mortar, the best of each kind, as the game's menu takes
//! the best the player carries. What the bowl would make shows on a card beside it, made of the
//! game's alchemy menu's own art, icons and fonts. With everything ground, the game makes the
//! potion and its bottle rises out of the mortar.
//!
//! Everything the game decides is the game's, through its own alchemy menu ([`menu`]): made for a
//! moment whenever what's in the bowl changes (its ingredient slots filled with what's there, its
//! effects worked out by its own code, what the player's skill shows), read, and gone again in the
//! same frame, without the menu mode it would bring (the world never stops). Making the potion
//! presses its Create: the effects, magnitudes, durations, name, value and weight, the skill used,
//! the ingredients used up (taken from the bowl first, the game's own pick-up), its sound and
//! message. The bottle is the game's own drop of it from the inventory, then put in the mortar.
//! A mix with no shared effect has nothing to make: the game's own failure message and sound, and
//! the ingredients are lost, as the message says. See `docs/research/alchemy.md`.
//!
//! Forms: `TESObjectAPPA` model path +0x34, icon +0x4C, weight +0x6C, quality +0x74, kind +0x78
//! (0 mortar and pestle, 1 alembic, 2 calcinator, 3 retort); `IngredientItem` icon +0x5C.

use glam::{Quat, Vec3};
use rustvr_protocol::*;

/// Apparatus within this far of the mortar (metres) count at its bench, and ingredients within
/// twice it may go into it.
pub const REACH: f32 = 0.75;
/// The game's alchemy menu has four ingredient slots.
pub const MAX_INGREDIENTS: u32 = 4;
/// Metres of grinding each ingredient takes: about three turns of a hand's circle in the bowl.
pub const GRIND_WORK: f32 = 0.45;
/// A pestle's id: its mortar's, in a range of its own (`0xFE`, a plugin slot nothing loads).
pub const PESTLE: u32 = 0xFE00_0000;
/// How long the bottle takes to rise out of the mortar (seconds), and how far over the rim its
/// middle stops (metres: the game's bottle is about 23 cm tall, its middle its origin).
pub const RISE_TIME: f64 = 0.9;
pub const RISE_ABOVE: f32 = 0.12;
/// How long a new bottle may take to be among the things to take (seconds) before it's left be.
pub const FIND_TIME: f64 = 2.0;
/// Seconds the bottle waits over the mortar for a hand before it's the game's again (it settles
/// on the table).
pub const WAIT: f64 = 45.0;

/// The pestle that goes with the mortar `id`.
pub fn pestle_id(mortar: u32) -> u32 {
    PESTLE | (mortar & 0x00FF_FFFF)
}

/// Of the apparatus found near a mortar (kind, quality, form), the best of each kind: alembic,
/// calcinator and retort (the mortar is the one in use).
pub fn best(found: &[(u8, f32, usize)]) -> [usize; 3] {
    let mut out = [(f32::MIN, 0usize); 3];
    for &(kind, quality, form) in found {
        if (1..=3).contains(&kind) && quality > out[kind as usize - 1].0 {
            out[kind as usize - 1] = (quality, form);
        }
    }
    out.map(|(_, f)| f)
}

/// What the game's menu shows for what's in the bowl.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shown {
    /// The potion's name (the game's default: its first effect's), when it has effects.
    pub name: String,
    /// Each effect's icon (a texture path under `textures\`) and the game's words for it.
    pub effects: Vec<(String, String)>,
    /// What's in the bowl: each ingredient's icon (under `textures\`) and name.
    pub ingredients: Vec<(String, String)>,
    /// The apparatus it's made with (the mortar, and the alembic, calcinator and retort that count):
    /// each one's icon.
    pub apparatus: Vec<String>,
}

/// The card's fonts: the potion's name (the menu's font 1) and the effects (its font 3).
pub struct Fonts {
    pub title: String,
    pub rows: String,
}

/// The menu's brown (its text's red, green and blue: 117, 59, 33).
pub const INK: [f32; 4] = [117.0 / 255.0, 59.0 / 255.0, 33.0 / 255.0, 1.0];
/// The part of the alchemy menu's art (`menus\alchemy\alchemy.dds`) the card is cut from: its
/// effects box, a ruled panel of parchment; and how big it is in the menu's units (the art is
/// stretched over 1150 by 840).
pub const ART: &str = "textures\\menus\\alchemy\\alchemy.dds";
pub const ART_UV: [f32; 4] = [0.535, 0.409, 0.823, 0.771];
pub const CARD_SIZE: [f32; 2] = [331.0, 304.0];

/// A cache name for a texture under `textures\`.
pub fn texture_name(path: &str) -> String {
    format!("oblivion/{}", oblivion_assets::export::texture_key(path))
}

/// How wide the card is in the world (metres).
pub const CARD_WIDTH: f32 = 0.2;

/// The card beside the mortar: the menu's effects box, the potion's name at its top, each effect
/// as the menu lists it (its icon, the game's words), and along its foot what's in the bowl (each
/// ingredient's icon) and what it's made with (each apparatus's icon, smaller, at the right).
pub fn card(anchor: u32, revision: u32, shown: &Shown, fonts: &Fonts) -> Card {
    let mut c = Card::new(anchor, CARD_SIZE);
    c.revision = revision;
    c.world_width = CARD_WIDTH;
    c.push(CardPiece::image(&texture_name(ART), [0.0, 0.0, CARD_SIZE[0], CARD_SIZE[1]], ART_UV));
    let mut y = 14.0;
    if !shown.name.is_empty() {
        c.push(CardPiece::text(&fonts.title, &shown.name, [16.0, y, CARD_SIZE[0] - 32.0, 34.0], INK, PIECEF_CENTRE));
        y += 44.0;
    }
    for (icon, words) in shown.effects.iter().take(6) {
        c.push(CardPiece::image(&texture_name(icon), [14.0, y + 4.0, 32.0, 32.0], [0.0, 0.0, 1.0, 1.0]));
        c.push(CardPiece::text(&fonts.rows, words, [54.0, y, CARD_SIZE[0] - 68.0, 40.0], INK, 0));
        y += 42.0;
    }
    for (k, (icon, _)) in shown.ingredients.iter().take(MAX_INGREDIENTS as usize).enumerate() {
        c.push(CardPiece::image(&texture_name(icon), [14.0 + 40.0 * k as f32, CARD_SIZE[1] - 52.0, 36.0, 36.0], [0.0, 0.0, 1.0, 1.0]));
    }
    for (k, icon) in shown.apparatus.iter().take(4).enumerate() {
        c.push(CardPiece::image(&texture_name(icon), [CARD_SIZE[0] - 42.0 - 32.0 * k as f32, CARD_SIZE[1] - 46.0, 28.0, 28.0], [0.0, 0.0, 1.0, 1.0]));
    }
    c
}

/// A bowl as the runtime takes it, from the mortar's measurements (`oblivion_assets::bench`).
pub fn receptacle(mortar: u32, b: &oblivion_assets::bench::Bench) -> Receptacle {
    let mut r: Receptacle = bytemuck::Zeroable::zeroed();
    r.id = mortar;
    r.flags = REC_GRIND;
    r.capacity = MAX_INGREDIENTS;
    r.grinder = pestle_id(mortar);
    r.floor = b.floor;
    r.radius = b.radius;
    r.depth = b.depth;
    r.wall = b.wall;
    r.grind_work = GRIND_WORK;
    r.tip = b.tip;
    r.tip_radius = b.tip_radius;
    r.rest_pos = b.pestle_pos;
    r.rest_rot = b.pestle_rot;
    r
}

/// The pestle as the runtime's own piece: where it rests in the mortar at `at` (the mortar's pose),
/// drawn from its own model (in the mortar's model space, so placed back by its rest pose), a
/// capsule along its length. `mass`: its share of the mortar and pestle's.
pub fn pestle_desc(mortar: u32, at: (Vec3, Quat), b: &oblivion_assets::bench::Bench, mass: f32, seq: u32) -> ObjectDesc {
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    let rest = (Vec3::from_array(b.pestle_pos), Quat::from_array(b.pestle_rot).normalize());
    let pose = (at.0 + at.1 * rest.0, (at.1 * rest.1).normalize());
    d.id = pestle_id(mortar);
    d.kind = OBJ_PROP;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_RUNTIME_DRAWS | OBJF_DRAWN_ALWAYS;
    d.spawn_seq = seq;
    d.spawn_pose = Pose { pos: pose.0.to_array(), valid: 1, rot: pose.1.to_array() };
    d.shape = SHAPE_CAPSULE;
    d.radius = b.pestle_radius;
    d.half_extents = [b.pestle_radius, (b.pestle_half - b.pestle_radius).max(0.005), b.pestle_radius];
    d.mass = mass;
    let inv = rest.1.inverse();
    d.model_pos = (inv * -rest.0).to_array();
    d.model_rot = inv.to_array();
    set_name(&mut d.name, &format!("oblivion/{}", b.pestle));
    d
}

/// The pestle's share of the apparatus's weight: as its size is of the two pieces'.
pub fn pestle_share(b: &oblivion_assets::bench::Bench) -> f32 {
    let pestle = std::f32::consts::PI * b.pestle_radius * b.pestle_radius * b.pestle_half * 2.0;
    let outer = b.radius + b.wall;
    let mortar = std::f32::consts::PI * ((outer * outer - b.radius * b.radius) * b.depth + outer * outer * (b.floor[1] - b.bottom).max(0.005));
    (pestle / (pestle + mortar).max(1e-6)).clamp(0.05, 0.6)
}

/// A note's id: its mortar's, in a range of its own (`0xFD`, a plugin slot as unlikely as the
/// pestle's).
pub const NOTE: u32 = 0xFD00_0000;

/// The note that goes with the mortar `id`.
pub fn note_id(mortar: u32) -> u32 {
    NOTE | (mortar & 0x00FF_FFFF)
}

/// The note's paper: the game's own for a note, the scroll its book menu opens a note on
/// (`menus\book\scroll_background.dds`), all of it that's paper (its top 990 by 780 of 1024: the
/// scroll's head and ribbon, and the framed page under them).
pub const NOTE_ART: &str = "textures\\menus\\book\\scroll_background.dds";
pub const NOTE_UV: [f32; 4] = [0.0, 0.0, 990.0 / 1024.0, 780.0 / 1024.0];
/// The note in its own units: the scroll's, a quarter as large again, so the game's handwriting
/// (its line 57 high) is as big on it as a large hand writes.
pub const NOTE_SIZE: [f32; 2] = [1237.5, 975.0];
/// How wide the note is in the world (metres), and half how thick (its box).
pub const NOTE_WIDTH: f32 = 0.26;
pub const NOTE_HALF_THICK: f32 = 0.002;
/// A sheet of paper's weight (kilograms), as near as the hands' physics takes one.
pub const NOTE_MASS: f32 = 0.05;
/// Where the note is put: this far from the mortar towards the player (metres), its page towards
/// them.
pub const NOTE_OUT: f32 = 0.2;
/// When it's put there: once the player is at the bench, this near its mortar (metres, along the
/// floor) and facing it, the mortar within this angle (radians) of the way they look. So with
/// benches in a row, each one's note faces the player where they stand at it, not where they were
/// when they first came in sight of it.
pub const NOTE_NEAR: f32 = 2.0;
pub const NOTE_FACING: f32 = 0.6;
/// Where the writing goes: inside the scroll's frame (the book menu's text starts at 115, 156 of
/// the scroll), down to its foot.
const WRITE_X: f32 = 160.0;
const WRITE_W: f32 = 920.0;
const WRITE_TOP: f32 = 205.0;
const FOOT_Y: f32 = 880.0;
/// A step done, faded on the page.
const DONE_ALPHA: f32 = 0.35;

/// The note as the runtime takes it: a sheet ([`OBJF_SHEET`]) at `at`, its card its face.
pub fn note_desc(mortar: u32, at: (Vec3, Quat), seq: u32) -> ObjectDesc {
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = note_id(mortar);
    d.kind = OBJ_PROP;
    d.flags = OBJF_GRABBABLE | OBJF_SHEET;
    d.spawn_seq = seq;
    d.spawn_pose = Pose { pos: at.0.to_array(), valid: 1, rot: at.1.to_array() };
    d.shape = SHAPE_BOX;
    d.half_extents = [NOTE_WIDTH * 0.5, NOTE_HALF_THICK, NOTE_WIDTH * 0.5 * NOTE_SIZE[1] / NOTE_SIZE[0]];
    d.mass = NOTE_MASS;
    set_name(&mut d.name, "alchemy note");
    d
}

/// A potion the bench can make: the game's own (what its menu makes of these ingredients).
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub name: String,
    /// Its first effect's icon (under `textures\`), and each effect in the game's words.
    pub icon: String,
    pub effects: Vec<String>,
    pub value: i32,
    pub ingredients: Vec<Ingredient>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ingredient {
    /// Its form's id.
    pub form: u32,
    pub name: String,
    /// Its icon (under `textures\`).
    pub icon: String,
    /// One lies at the bench (else it's in the player's pack).
    pub near: bool,
}

/// The page the note is on: the list (which page of it), or the recipe chosen from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    List(usize),
    Recipe,
}

/// How far the bench has got with the recipe the note is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    /// What's in the bowl that isn't in it (names).
    pub extra: Vec<String>,
    /// Each of its ingredients is in the bowl.
    pub put: Vec<bool>,
    /// It's all ground: the game's making it, or has.
    pub ground: bool,
}

/// What touching a part of the note does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Touch {
    Choose(usize),
    Back,
    Turn,
}

/// A font as the note's layout needs it: a line's height, and how far each character takes the
/// pen (as the runtime lays the game's text out).
pub struct Measure {
    pub height: f32,
    pub advance: [f32; 256],
}

impl Measure {
    pub fn of(font: &rustvr_asset::Font) -> Measure {
        Measure { height: font.height, advance: std::array::from_fn(|c| font.advance(c as u8)) }
    }

    /// For a font that won't load: about as big as `height`'s handwriting.
    pub fn rough(height: f32) -> Measure {
        Measure { height, advance: [height * 0.3; 256] }
    }

    fn bytes_width(&self, words: &[u8]) -> f32 {
        words.iter().map(|&c| self.advance[c as usize]).sum()
    }

    /// How wide `text` is on one line.
    pub fn width(&self, text: &str) -> f32 {
        self.bytes_width(&latin1(text))
    }

    /// How many lines `text` takes at `width`, broken at spaces as the runtime breaks it.
    pub fn lines(&self, text: &str, width: f32) -> usize {
        let bytes = latin1(text);
        let mut lines = 1;
        let mut line: Vec<u8> = Vec::new();
        for word in bytes.split(|&c| c == b' ') {
            let mut next = line.clone();
            if !next.is_empty() {
                next.push(b' ');
            }
            next.extend_from_slice(word);
            if !line.is_empty() && self.bytes_width(&next) > width {
                lines += 1;
                line = word.to_vec();
            } else {
                line = next;
            }
        }
        lines
    }
}

/// Text as the game's 8-bit characters (Latin-1): what isn't one is left out, as the runtime
/// leaves it out.
fn latin1(text: &str) -> Vec<u8> {
    text.chars().filter_map(|c| u8::try_from(c as u32).ok()).collect()
}

/// The note's fonts: the game's handwriting (its font 5, as its notes are written) for what's
/// written on it, and the scroll's own print (font 1) for the game's words for the effects.
pub struct NoteFonts {
    pub hand: (String, Measure),
    pub print: (String, Measure),
}

/// A list row's words: the potion, and what it's made of.
fn row_words(r: &Recipe) -> String {
    format!("{}: {}", r.name, r.ingredients.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(", "))
}

/// A row of the note: its icon's size, where its words start, and the gap under it.
const ROW_ICON: f32 = 40.0;
const ROW_TEXT_X: f32 = WRITE_X + ROW_ICON + 12.0;
const ROW_GAP: f32 = 8.0;

/// The list's pages: the recipes each one shows (the rows that fit above the foot, at most what a
/// card's pieces hold).
pub fn list_pages(recipes: &[Recipe], fonts: &NoteFonts) -> Vec<std::ops::Range<usize>> {
    let hand = &fonts.hand.1;
    let first = WRITE_TOP + hand.height + 24.0;
    let most = (MAX_CARD_PIECES - 3) / 2;
    let mut pages = Vec::new();
    let (mut start, mut y) = (0, first);
    for (i, r) in recipes.iter().enumerate() {
        let h = hand.lines(&row_words(r), WRITE_X + WRITE_W - ROW_TEXT_X) as f32 * hand.height + ROW_GAP;
        if i > start && (y + h > FOOT_Y || i - start >= most) {
            pages.push(start..i);
            (start, y) = (i, first);
        }
        y += h;
    }
    pages.push(start..recipes.len());
    pages
}

/// The note: on its list page, the potions the bench can make (the game's own, the best first),
/// each with what it's made of, and a foot to turn over to more; on a recipe's page, the potion,
/// the game's words for its effects, and what to do to make it, each step faded once it's done.
/// `complete`: every recipe has been worked out (else it's still being written); `chosen`: the
/// recipe a recipe's page shows. Also where on it each touch does something (card units).
pub fn note_card(anchor: u32, revision: u32, page: Page, recipes: &[Recipe], chosen: Option<&Recipe>, complete: bool, progress: &Progress, fonts: &NoteFonts) -> (Card, Vec<([f32; 4], Touch)>) {
    let mut c = Card::new(anchor, NOTE_SIZE);
    c.revision = revision;
    c.world_width = NOTE_WIDTH;
    c.push(CardPiece::image(&texture_name(NOTE_ART), [0.0, 0.0, NOTE_SIZE[0], NOTE_SIZE[1]], NOTE_UV));
    let mut targets = Vec::new();
    let ((hand_font, hand), (print_font, print)) = (&fonts.hand, &fonts.print);
    let faded = [INK[0], INK[1], INK[2], DONE_ALPHA];
    match (page, chosen) {
        (Page::Recipe, Some(r)) => {
            // The potion: its icon and name, centred; touched, back to the list.
            let w = hand.width(&r.name).min(WRITE_W - 60.0);
            let x = WRITE_X + (WRITE_W - w - 60.0) * 0.5;
            c.push(CardPiece::image(&texture_name(&r.icon), [x, WRITE_TOP + (hand.height - 48.0) * 0.5, 48.0, 48.0], [0.0, 0.0, 1.0, 1.0]));
            c.push(CardPiece::text(hand_font, &r.name, [x + 60.0, WRITE_TOP, w + 8.0, hand.height], INK, 0));
            targets.push(([WRITE_X, WRITE_TOP, WRITE_W, hand.height], Touch::Back));
            let mut y = WRITE_TOP + hand.height + 8.0;
            for words in r.effects.iter().take(4) {
                let h = print.lines(words, WRITE_W - 80.0) as f32 * print.height;
                c.push(CardPiece::text(print_font, words, [WRITE_X + 40.0, y, WRITE_W - 80.0, h], INK, PIECEF_CENTRE));
                y += h + 4.0;
            }
            y += 24.0;
            let step = |c: &mut Card, icon: Option<&str>, words: &str, done: bool, y: &mut f32| {
                let x = WRITE_X + ROW_ICON + 12.0;
                let h = hand.lines(words, WRITE_X + WRITE_W - x) as f32 * hand.height;
                if let Some(icon) = icon {
                    c.push(CardPiece::image(&texture_name(icon), [WRITE_X, *y + (hand.height - ROW_ICON) * 0.5, ROW_ICON, ROW_ICON], [0.0, 0.0, 1.0, 1.0]));
                }
                c.push(CardPiece::text(hand_font, words, [x, *y, WRITE_X + WRITE_W - x, h], if done { faded } else { INK }, 0));
                *y += h + ROW_GAP;
            };
            if !progress.extra.is_empty() {
                let names = match progress.extra.as_slice() {
                    [one] => one.clone(),
                    [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
                    [] => String::new(),
                };
                step(&mut c, None, &format!("Take the {names} out of the mortar"), false, &mut y);
            }
            for (k, ing) in r.ingredients.iter().enumerate() {
                let put = progress.put.get(k).copied().unwrap_or(false);
                let words = if ing.near || put { format!("Put in the {}", ing.name) } else { format!("Put in the {}, from your pack", ing.name) };
                step(&mut c, Some(&ing.icon), &words, put, &mut y);
            }
            let grind = if r.ingredients.len() == 1 { "Grind it with the pestle" } else { "Grind them with the pestle" };
            step(&mut c, None, grind, progress.ground, &mut y);
            step(&mut c, None, "Take your potion", false, &mut y);
            let back = "Other potions";
            c.push(CardPiece::text(hand_font, back, [WRITE_X, FOOT_Y, WRITE_W, hand.height], INK, PIECEF_RIGHT));
            targets.push(([WRITE_X + WRITE_W * 0.5, FOOT_Y - 10.0, WRITE_W * 0.5, hand.height + 20.0], Touch::Back));
        }
        _ => {
            let shown = if let Page::List(p) = page { p } else { 0 };
            c.push(CardPiece::text(hand_font, "Potions you can make", [WRITE_X, WRITE_TOP, WRITE_W, hand.height], INK, PIECEF_CENTRE));
            let mut y = WRITE_TOP + hand.height + 24.0;
            if recipes.is_empty() {
                let words = if complete { "Nothing you have makes a potion you know." } else { "..." };
                c.push(CardPiece::text(hand_font, words, [WRITE_X, y, WRITE_W, hand.height * 2.0], INK, PIECEF_CENTRE));
            }
            let pages = list_pages(recipes, fonts);
            let p = shown.min(pages.len() - 1);
            for i in pages[p].clone() {
                let r = &recipes[i];
                let words = row_words(r);
                let w = WRITE_X + WRITE_W - ROW_TEXT_X;
                let h = hand.lines(&words, w) as f32 * hand.height;
                c.push(CardPiece::image(&texture_name(&r.icon), [WRITE_X, y + (hand.height - ROW_ICON) * 0.5, ROW_ICON, ROW_ICON], [0.0, 0.0, 1.0, 1.0]));
                c.push(CardPiece::text(hand_font, &words, [ROW_TEXT_X, y, w, h], INK, 0));
                targets.push(([WRITE_X, y - ROW_GAP * 0.5, WRITE_W, h + ROW_GAP], Touch::Choose(i)));
                y += h + ROW_GAP;
            }
            if pages.len() > 1 {
                let words = format!("Turn over ({} of {})", p + 1, pages.len());
                c.push(CardPiece::text(hand_font, &words, [WRITE_X, FOOT_Y, WRITE_W, hand.height], INK, PIECEF_RIGHT));
                targets.push(([WRITE_X + WRITE_W * 0.5, FOOT_Y - 10.0, WRITE_W * 0.5, hand.height + 20.0], Touch::Turn));
            }
        }
    }
    (c, targets)
}

/// What a touch at `at` (card units) does, of `targets`.
pub fn touched(targets: &[([f32; 4], Touch)], at: [f32; 2]) -> Option<Touch> {
    targets.iter().find(|(r, _)| at[0] >= r[0] && at[0] <= r[0] + r[2] && at[1] >= r[1] && at[1] <= r[1] + r[3]).map(|(_, t)| *t)
}

/// The recipes found, as the note lists them: of potions with the same effects (whatever their
/// strength), the most valuable; the most valuable first.
pub fn best_recipes(found: Vec<Recipe>) -> Vec<Recipe> {
    let kind = |r: &Recipe| r.effects.iter().map(|e| e.chars().filter(|c| !c.is_ascii_digit()).collect::<String>()).collect::<Vec<_>>();
    let mut out: Vec<Recipe> = Vec::new();
    for r in found {
        match out.iter_mut().find(|o| kind(o) == kind(&r)) {
            Some(o) if r.value > o.value => *o = r,
            Some(_) => {}
            None => out.push(r),
        }
    }
    out.sort_by(|a, b| b.value.cmp(&a.value).then_with(|| a.name.cmp(&b.name)));
    out
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

/// The game's alchemy menu, made for a moment to work a potion out (or make it) and gone again.
#[cfg(all(windows, target_arch = "x86"))]
pub mod menu {
    use crate::mem;

    /// `kMenuType_Alchemy`.
    const ALCHEMY: u32 = 1040;
    /// The alchemy menu made with its four apparatus (cdecl: their 12-byte records, mortar,
    /// alembic, calcinator, retort): its `TileMenu`.
    const OPEN: usize = 0x0059_3810;
    /// A tile's menu (thiscall), and the open menu of a type (cdecl).
    const MENU_OF: usize = 0x0058_98F0;
    const MENU_BY_TYPE: usize = 0x0058_9B70;
    /// The game's heap (cdecl: size; pointer).
    const ALLOC: usize = 0x0040_1F00;
    const FREE: usize = 0x0040_1F20;
    /// The ingredients set: the weight and the potion worked out again (thiscall: menu, 1); the
    /// slot it shows is `SLOT`'s.
    const SET_INGREDIENTS: usize = 0x0059_4F00;
    const SLOT: usize = 0x00B3_B270;
    /// Create (thiscall: menu), and the potion made before that's the same (thiscall: the data
    /// handler, the potion).
    const CREATE: usize = 0x0059_4CA0;
    const SAME_AS: usize = 0x0044_A750;
    const DATA_HANDLER: usize = 0x00B3_3A98;
    /// A menu's fade taken off the interface's list (cdecl: menu).
    const UNFADE: usize = 0x0058_3CF0;
    /// `InterfaceManager*`: its mode (+8, a byte: 1 the game, 3 a menu), the tile and menu under
    /// the cursor (+0x88, +0x98, +0x9C) and its stack of open menus (+0xE0, ten ids); and the menu
    /// background's flag, which a menu opening sets.
    const INTERFACE: usize = 0x00B3_A6E0;
    const MODE: usize = 0x8;
    const STACK: usize = 0xE0;
    const UNDER: [usize; 3] = [0x88, 0x98, 0x9C];
    const MENU_BACKGROUND: usize = 0x00B4_2D54;
    /// `AlchemyMenu`: its id once registered (+0x14), the effect list's tile (+0x50), the potion
    /// (+0x94: name +0x28, value +0x78, its effects +0x30), the ingredients' weight (+0x98), the
    /// ingredients (+0xB0, four).
    const MENU_ID: usize = 0x14;
    const EFFECT_LIST: usize = 0x50;
    const POTION: usize = 0x94;
    const WEIGHT: usize = 0x98;
    const INGREDIENTS: usize = 0xB0;
    const NAME: usize = 0x28;
    const VALUE: usize = 0x78;
    /// Tiles: their values (+0x14) and children (+0x30), lists of `{next, previous, data}` from
    /// +4; a value's string +8 and its id +0x18 (u16): `user1` an effect's icon, `user3` its words.
    const VALUES: usize = 0x14;
    const CHILDREN: usize = 0x30;
    const USER1: u16 = 4015;
    const USER3: u16 = 4017;

    /// What the game's menu made of the bowl's ingredients.
    #[derive(Debug, Default)]
    pub struct Brewed {
        pub name: String,
        /// Each effect's icon (its path as the menu names it, `Icons\...`) and words.
        pub effects: Vec<(String, String)>,
        pub weight: f32,
        pub value: i32,
        /// With `create`: the potion made (its form).
        pub potion: usize,
    }

    fn list(at: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = mem::deref(at + 4);
        while node != 0 && out.len() < 64 {
            let data = mem::deref(node + 8);
            if data != 0 {
                out.push(data);
            }
            node = mem::deref(node);
        }
        out
    }

    fn string(tile: usize, id: u16) -> Option<String> {
        let v = list(tile + VALUES).into_iter().find(|&v| mem::at::<u16>(v + 0x18) == Some(id))?;
        let s = mem::deref(v + 8);
        (s != 0).then(|| crate::combat::read_str(s))
    }

    fn alloc_record(form: usize) -> usize {
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        let r = unsafe { std::mem::transmute::<usize, Alloc>(ALLOC)(12) };
        if r != 0 {
            unsafe { (r as *mut [usize; 3]).write([0, 1, form]) };
        }
        r
    }

    fn free(p: usize) {
        type Free = unsafe extern "C" fn(usize);
        if p != 0 {
            unsafe { std::mem::transmute::<usize, Free>(FREE)(p) };
        }
    }

    /// The game's alchemy menu with these apparatus (mortar, alembic, calcinator, retort; 0: none)
    /// and ingredients (at most four, each once), worked out and, with `create`, pressed Create;
    /// then gone. None when it can't be made now (the player's own alchemy menu is open, or there's
    /// no mortar).
    pub fn brew(apparatus: [usize; 4], ingredients: &[usize], create: bool) -> Option<Brewed> {
        type Open = unsafe extern "C" fn(usize, usize, usize, usize) -> usize;
        type This = unsafe extern "thiscall" fn(usize) -> usize;
        type ByType = unsafe extern "C" fn(u32) -> usize;
        type Set = unsafe extern "thiscall" fn(usize, u8) -> u32;
        type Same = unsafe extern "thiscall" fn(usize, usize) -> usize;
        type Unfade = unsafe extern "C" fn(usize);
        type Destroy = unsafe extern "thiscall" fn(usize, u8) -> usize;
        let im = mem::deref(INTERFACE);
        if im == 0 || apparatus[0] == 0 || unsafe { std::mem::transmute::<usize, ByType>(MENU_BY_TYPE)(ALCHEMY) } != 0 {
            return None;
        }
        // The interface as it is: making the menu pushes it on the stack of open menus (menu mode,
        // the world stopped) and takes the cursor's tiles; it's all put back at once.
        let stack: [u32; 10] = mem::at(im + STACK)?;
        let mode: u8 = mem::at(im + MODE)?;
        let under: Vec<usize> = UNDER.iter().map(|o| mem::deref(im + o)).collect();
        let background: u8 = mem::at(MENU_BACKGROUND)?;
        let records: Vec<usize> = apparatus.iter().map(|&f| if f != 0 { alloc_record(f) } else { 0 }).collect();
        let tile = unsafe { std::mem::transmute::<usize, Open>(OPEN)(records[0], records[1], records[2], records[3]) };
        unsafe {
            ((im + STACK) as *mut [u32; 10]).write(stack);
            ((im + MODE) as *mut u8).write(mode);
            for (o, v) in UNDER.iter().zip(&under) {
                ((im + o) as *mut usize).write(*v);
            }
            (MENU_BACKGROUND as *mut u8).write(background);
        }
        let menu = if tile != 0 { unsafe { std::mem::transmute::<usize, This>(MENU_OF)(tile) } } else { 0 };
        // (Its id: the menu's `GetID`, vtable +0x34.)
        let id = if menu != 0 { unsafe { std::mem::transmute::<usize, This>(mem::deref(mem::deref(menu) + 0x34))(menu) } } else { 0 };
        if menu == 0 || id != ALCHEMY as usize {
            records.into_iter().for_each(free);
            tracing::warn!("alchemy: the game's alchemy menu didn't open");
            return None;
        }
        // Its ingredients, then the game works the potion out. (None: only made and gone, the
        // game's reading of its menu's file done once, before it's wanted.)
        for (k, &form) in ingredients.iter().take(4).enumerate() {
            unsafe { ((menu + INGREDIENTS + 4 * k) as *mut usize).write(alloc_record(form)) };
        }
        if !ingredients.is_empty() {
            unsafe {
                (SLOT as *mut u32).write(0);
                std::mem::transmute::<usize, Set>(SET_INGREDIENTS)(menu, 1);
            }
        }
        let potion = mem::deref(menu + POTION);
        let rows = list(mem::deref(menu + EFFECT_LIST) + CHILDREN);
        let mut b = Brewed {
            name: if potion != 0 { crate::combat::read_str(mem::deref(potion + NAME)) } else { String::new() },
            effects: rows.iter().filter_map(|&t| Some((string(t, USER1).unwrap_or_default(), string(t, USER3)?))).collect(),
            weight: mem::at::<f32>(menu + WEIGHT).unwrap_or(0.0),
            value: if potion != 0 { mem::at::<i32>(potion + VALUE).unwrap_or(0) } else { 0 },
            potion: 0,
        };
        if create && !b.effects.is_empty() && potion != 0 {
            let same = unsafe { std::mem::transmute::<usize, Same>(SAME_AS)(mem::deref(DATA_HANDLER), potion) };
            unsafe { std::mem::transmute::<usize, This>(CREATE)(menu) };
            b.potion = if same != 0 { same } else { potion };
        }
        // Gone: its apparatus records are the closer's to free (the menu's own end frees the
        // rest); not on the stack, so its end takes nothing off it; its fade off the list.
        for (k, r) in records.into_iter().enumerate() {
            if mem::deref(menu + 0x78 + 4 * k) == r {
                unsafe { ((menu + 0x78 + 4 * k) as *mut usize).write(0) };
            }
            free(r);
        }
        unsafe {
            ((menu + MENU_ID) as *mut u32).write(0);
            std::mem::transmute::<usize, Unfade>(UNFADE)(menu);
            let destroy = mem::deref(mem::deref(menu));
            std::mem::transmute::<usize, Destroy>(destroy)(menu, 1);
        }
        Some(b)
    }
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av, refr};
    use crate::{camera, items, mem};
    use oblivion_assets::bench::{Bench, Benches};
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    const FORM_FLAGS: usize = 0x8;
    const DELETED: u32 = 0x20;
    const FORM_ID: usize = 0xC;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const APPA_ICON: usize = 0x4C;
    const APPA_MODEL: usize = 0x34;
    const APPA_WEIGHT: usize = 0x6C;
    const APPA_QUALITY: usize = 0x74;
    const APPA_KIND: usize = 0x78;
    const INGR_ICON: usize = 0x5C;
    /// The player's `RemoveItem` (vtable +0x100) and pick-up (+0x2CC, `Actor` 0xB3), as the game's
    /// Create and activation call them.
    const VT_REMOVE: usize = 0x100;
    /// The game's message on the screen (cdecl: text, 0, 1, -1), and its interface sounds (cdecl:
    /// which; 30 `ITMIngredientNothing`).
    const MESSAGE: usize = 0x0057_ACC0;
    const UI_SOUND: usize = 0x0057_DE50;
    const SOUND_NOTHING: u32 = 30;
    /// `sPotionFailed`'s text.
    const POTION_FAILED: usize = 0x00B3_88E0;
    /// Whether the player is in combat (thiscall: player, 0), the setting that allows alchemy
    /// then (`iAllowAlchemyDuringCombat`) and the game's words when it doesn't, as using an
    /// apparatus checks them.
    const IN_COMBAT: usize = 0x0066_05A0;
    const ALCHEMY_IN_COMBAT: usize = 0x00B3_8A98;
    const NO_ALCHEMY_IN_COMBAT: usize = 0x00B3_8A60;
    /// Seconds before a bench refused in combat tries again.
    const RETRY: f64 = 3.0;
    /// The game's sounds (`SOUN` form ids): an ingredient put down; organic matter knocked about
    /// (a crunch).
    const SND_INGREDIENT_DOWN: u32 = 0x0008_9772;
    const SND_CRUNCH: u32 = 0x0009_7C61;
    /// A scroll picked up and put down (`ITMScrollUp`, `ITMScrollDown`), and a page turned
    /// (`ITMBookPageTurn`): the note's.
    const SND_SCROLL_UP: u32 = 0x0008_B099;
    const SND_SCROLL_DOWN: u32 = 0x0008_B09A;
    const SND_PAGE_TURN: u32 = 0x0005_947D;
    /// `IngredientItem`: its name (its `MagicItem`'s, +0x24, name +4) and its effects (that
    /// `MagicItem`'s list, from +0x34: `{effect, next}`, an effect's code at +0), as the game's
    /// menu reads them (0x593CD0).
    const INGR_NAME: usize = 0x28;
    const INGR_EFFECTS: usize = 0x34;
    const FORM_TYPE: usize = 0x04;
    /// How many of each ingredient's effects the player knows, for an Alchemy skill (cdecl: the
    /// skill; its mastery's setting), a skill's mastery (cdecl: the skill; 4 a master's), and the
    /// player's base skill (thiscall: actor, 19), as the menu takes them.
    const KNOWN_EFFECTS: usize = 0x0041_BA00;
    const MASTERY: usize = 0x0056_A300;
    const BASE_AV: usize = 0x005F_1910;
    const AV_ALCHEMY: u32 = 19;
    /// The game's frame time the note's recipes may take a frame (seconds), and how often a bench
    /// looks again at what it could make.
    const RECIPE_BUDGET: f64 = 0.002;
    const RECIPE_LOOK: f64 = 1.0;

    fn benches() -> &'static Benches {
        static B: OnceLock<Benches> = OnceLock::new();
        B.get_or_init(|| {
            let b = Benches::load(&rustvr_asset::default_root().join("oblivion")).unwrap_or_default();
            tracing::info!("alchemy: {} mortars measured for the hands (oblivion-extract bench)", b.benches.len());
            b
        })
    }

    /// The bench for an apparatus form: its mortar and pestle's measurements, if it's one.
    fn bench_of(base: usize) -> Option<&'static Bench> {
        if mem::at::<u8>(base + 4) != Some(items::APPARATUS) || mem::at::<u8>(base + APPA_KIND) != Some(0) {
            return None;
        }
        let model = crate::combat::read_str(mem::deref(base + APPA_MODEL));
        benches().of(&model)
    }

    /// A mortar and pestle among the things to take: drawn by the runtime as its mortar alone (the
    /// pestle is the runtime's own piece), its game model hidden ([`frame`]).
    pub fn dress(refr: usize, d: &mut ObjectDesc) {
        let Some(b) = bench_of(mem::deref(refr + REFR_BASE)) else { return };
        d.flags |= OBJF_RUNTIME_DRAWS | OBJF_DRAWN_ALWAYS;
        d.model_rot = Quat::IDENTITY.to_array();
        set_name(&mut d.name, &format!("oblivion/{}", b.mortar));
    }

    struct Station {
        refr: usize,
        node: usize,
        bench: &'static Bench,
        /// The pestle's spawn number (it's offered again, afresh, when the mortar is).
        seq: u32,
        /// What's in the bowl (item ids), as last seen; what the card shows of it.
        contents: Vec<u32>,
        shown: Option<Shown>,
        revision: u32,
        strokes: u32,
        /// It's been made (or failed): until the bowl is empty again.
        done: bool,
        /// Refused (in combat): not again until then.
        retry_at: f64,
        /// Its game model's flags before it was hidden.
        flags: Option<u16>,
        /// How much of what's in the bowl is ground (0 .. 1).
        ground: f32,
        note: Note,
    }

    /// A bench's note ([`note_card`]): where it was put, what it lists, the page it's on, and what
    /// the hands have done with it.
    struct Note {
        /// Where it's put when the bench appears (the runtime's world).
        at: Option<(Vec3, Quat)>,
        recipes: Vec<Recipe>,
        /// Every recipe of `recipes` is worked out; what they were worked out of (or are being),
        /// when the bench last looked, and the work still to do.
        complete: bool,
        key: Option<RecipeKey>,
        looked: f64,
        work: Option<Work>,
        page: Page,
        /// The recipe chosen (its page shows it, as it was when it was chosen or the list last
        /// had it), and the list's page last on (its page goes back there).
        chosen: Option<Recipe>,
        list_page: usize,
        /// The potion of the recipe it's on has been made: until its bottle is taken.
        made: bool,
        /// The touches read so far (None: not yet seen), and a hand has it.
        touches: Option<u32>,
        held: bool,
        revision: u32,
        /// What it shows (its page, the bench's progress, its recipes' generation), the card, and
        /// where on it touches do something.
        shown: Option<(Page, Progress, u32, bool)>,
        recipes_gen: u32,
        card: Option<Card>,
        targets: Vec<([f32; 4], Touch)>,
    }

    impl Note {
        /// Back to the list's page it was last on.
        fn back_to_list(&mut self) {
            self.page = Page::List(self.list_page);
            self.chosen = None;
            self.made = false;
        }

        fn new() -> Note {
            Note {
                at: None,
                recipes: Vec::new(),
                complete: false,
                key: None,
                looked: f64::MIN,
                work: None,
                page: Page::List(0),
                chosen: None,
                list_page: 0,
                made: false,
                touches: None,
                held: false,
                revision: 1,
                shown: None,
                recipes_gen: 0,
                card: None,
                targets: Vec::new(),
            }
        }
    }

    /// What a bench's recipes are worked out of: the ingredients the player has (each form's id,
    /// and whether one lies at the bench), the apparatus, and how many effects the player knows.
    #[derive(Clone, Debug, PartialEq)]
    struct RecipeKey {
        ingredients: Vec<(u32, bool)>,
        apparatus: [u32; 4],
        known: i32,
        master: bool,
    }

    /// Recipes being worked out: the apparatus, the ingredients at the bench, the mixes still to
    /// try (ingredient forms), and what they've made so far.
    struct Work {
        apparatus: [usize; 4],
        near: Vec<usize>,
        tries: Vec<Vec<usize>>,
        found: Vec<Recipe>,
    }

    /// A bottle rising out of a mortar.
    struct Rising {
        mortar: u32,
        refr: usize,
        from: Vec3,
        to: Vec3,
        t0: f64,
    }

    /// Work for the game's frame (where its scripts run): what to do there.
    enum Job {
        /// The game's alchemy menu made and gone once, so the first ingredient in doesn't wait on
        /// its reading of the menu's file.
        Warm { mortar: u32 },
        /// Work out what the bowl would make, for its card.
        Preview { mortar: u32 },
        /// Everything's ground: make it.
        Make { mortar: u32 },
        /// A game sound from the mortar.
        Sound { mortar: u32, sound: u32 },
        /// A game sound of the note's (as the game plays its interface's: not from anywhere).
        NoteSound { sound: u32 },
    }

    struct State {
        benches: HashMap<u32, Station>,
        rising: Option<Rising>,
        jobs: Vec<Job>,
        states: Box<ObjectStates>,
        receptacles: Box<Receptacles>,
        sent_receptacles: Vec<Receptacle>,
        cards: Box<Cards>,
        sent_cards: Vec<(u32, u32)>,
        /// The pestles and notes sent (their ids and spawn numbers).
        sent_pieces: Vec<(u32, u32)>,
        next_seq: u32,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State {
            benches: HashMap::new(),
            rising: None,
            jobs: Vec::new(),
            states: bytemuck::zeroed_box(),
            receptacles: bytemuck::zeroed_box(),
            sent_receptacles: Vec::new(),
            cards: bytemuck::zeroed_box(),
            sent_cards: Vec::new(),
            sent_pieces: Vec::new(),
            next_seq: 1,
        }))
    }

    /// The font the game's interface uses as its `n`th (Oblivion.ini's `SFontFile_<n>`), as a cache
    /// name.
    fn font(n: u32) -> String {
        let default = ["", "Data\\Fonts\\Kingthings_Regular.fnt", "Data\\Fonts\\Kingthings_Shadowed.fnt", "Data\\Fonts\\Tahoma_Bold_Small.fnt", "Data\\Fonts\\Daedric_Font.fnt", "Data\\Fonts\\Handwritten.fnt"];
        let path = crate::game_settings::ini_value("Fonts", &format!("SFontFile_{n}")).unwrap_or_else(|| default.get(n as usize).copied().unwrap_or(default[1]).to_owned());
        format!("oblivion/{}", oblivion_assets::export::font_key(&path))
    }

    fn fonts() -> &'static Fonts {
        static F: OnceLock<Fonts> = OnceLock::new();
        F.get_or_init(|| Fonts { title: font(1), rows: font(3) })
    }

    /// The note's fonts, measured from the cache as the runtime will draw them.
    fn note_fonts() -> &'static NoteFonts {
        static F: OnceLock<NoteFonts> = OnceLock::new();
        F.get_or_init(|| {
            let measured = |n: u32, rough: f32| {
                let name = font(n);
                let key = name.trim_start_matches("oblivion/");
                let m = match rustvr_asset::load_font(&rustvr_asset::default_root().join("oblivion"), key) {
                    Ok(f) => Measure::of(&f),
                    Err(e) => {
                        tracing::warn!("alchemy: the note's font {n} ({name}) won't load, so it's laid out roughly: {e:#}");
                        Measure::rough(rough)
                    }
                };
                (name, m)
            };
            NoteFonts { hand: measured(5, 57.0), print: measured(1, 28.0) }
        })
    }

    /// Where a reference is (game units).
    fn game_pos(r: usize) -> Option<Vec3> {
        mem::at::<[f32; 3]>(r + refr::POS).map(Vec3::from)
    }

    fn form_id(f: usize) -> u32 {
        mem::at::<u32>(f + FORM_ID).unwrap_or(0)
    }

    /// The node's world pose in the runtime's world.
    fn node_pose(node: usize) -> Option<(Vec3, Quat)> {
        let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
        let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
        let b = camera::basis();
        Some((b.point_from_game(Vec3::from(t)), b.rot_from_game(items::ni_matrix(r)).normalize()))
    }

    /// The mortar's model is still the one the station knows (not taken, its 3D not made again).
    fn alive(b: &Station) -> bool {
        mem::deref(b.refr + REFR_NODE) == b.node && b.node != 0 && mem::at::<u32>(b.refr + FORM_FLAGS).is_some_and(|f| f & DELETED == 0)
    }

    /// Hides (or shows again) a mortar's game model: the runtime draws its pieces.
    fn hide(b: &mut Station, hidden: bool) {
        let at = b.node + av::FLAGS;
        // (Only while the reference still has that model: taken, or its 3D made again, it's gone.)
        if !alive(b) || !mem::is_readable(at as *const u8, 2) {
            b.flags = None;
            return;
        }
        match (hidden, b.flags) {
            (true, None) => {
                let f = mem::at::<u16>(at).unwrap_or(0);
                b.flags = Some(f);
                unsafe { (at as *mut u16).write(f | av::APP_CULLED) };
            }
            (false, Some(f)) => {
                b.flags = None;
                unsafe { (at as *mut u16).write(f) };
            }
            _ => {}
        }
    }

    /// The game's frame, where its scripts run: work out what bowls would make, make potions,
    /// play sounds.
    pub fn frame(now: f64) {
        let jobs = with_state(|s| std::mem::take(&mut s.jobs));
        for job in jobs {
            match job {
                Job::Warm { mortar } => {
                    let Some(refr) = with_state(|s| s.benches.get(&mortar).map(|b| b.refr)) else { continue };
                    let (apparatus, _) = bench_forms(refr, &[], &items::offered());
                    crate::hooks::guard("alchemy menu", None, || menu::brew(apparatus, &[], false));
                }
                Job::Preview { mortar } => preview(mortar),
                Job::Make { mortar } => make(mortar),
                Job::Sound { mortar, sound } => {
                    crate::console::run_script(&format!("{mortar:08X}.PlaySound3D {sound:08X}"));
                }
                Job::NoteSound { sound } => {
                    crate::console::run_script(&format!("PlaySound {sound:08X}"));
                }
            }
        }
        recipes(now);
    }

    /// The ingredient forms the player has: in the pack, and lying at the bench at `mortar` (where
    /// they may go into it); and those at the bench.
    fn ingredients_at(mortar: usize, offered: &[items::Offered]) -> (Vec<usize>, Vec<usize>) {
        let at = game_pos(mortar).unwrap_or_default();
        let mut near: Vec<usize> = offered
            .iter()
            .filter(|o| o.kind == items::INGREDIENT)
            .filter(|o| game_pos(o.refr).is_some_and(|p| p.distance(at) <= REACH * 2.0 * camera::UNITS_PER_METER))
            .map(|o| mem::deref(o.refr + REFR_BASE))
            .filter(|&f| f != 0)
            .collect();
        near.sort();
        near.dedup();
        let mut all = near.clone();
        for (form, count) in crate::body::carried() {
            if count > 0 && mem::at::<u8>(form + FORM_TYPE) == Some(items::INGREDIENT) && !all.contains(&form) {
                all.push(form);
            }
        }
        (all, near)
    }

    /// The codes of an ingredient's first `known` effects.
    fn effect_codes(form: usize, known: i32) -> Vec<u32> {
        let mut out = Vec::new();
        let mut node = form + INGR_EFFECTS;
        while node != 0 && out.len() < known.clamp(0, 8) as usize {
            let effect = mem::deref(node);
            if effect != 0 {
                out.push(mem::at::<u32>(effect).unwrap_or(0));
            }
            node = mem::deref(node + 4);
        }
        out
    }

    /// How many of each ingredient's effects the player knows, and whether they're a master (whose
    /// ingredients make potions alone): the game's own rules.
    fn alchemist() -> (i32, bool) {
        type BaseAv = unsafe extern "thiscall" fn(usize, u32) -> u32;
        type OfSkill = unsafe extern "C" fn(i32) -> i32;
        let player = engine::player();
        if player == 0 {
            return (0, false);
        }
        let skill = unsafe { std::mem::transmute::<usize, BaseAv>(BASE_AV)(player, AV_ALCHEMY) } as i32;
        unsafe { (std::mem::transmute::<usize, OfSkill>(KNOWN_EFFECTS)(skill), std::mem::transmute::<usize, OfSkill>(MASTERY)(skill) >= 4) }
    }

    /// The benches' recipes: each looks now and then at what the player has, and when that's
    /// changed works out again what it can make. Every mix that could make something (each two
    /// ingredients that share an effect the player knows, and a master's each alone) goes through
    /// the game's own alchemy menu, as the card's does, a few a frame.
    fn recipes(now: f64) {
        let offered = items::offered();
        let looks: Vec<(u32, usize)> = with_state(|s| {
            s.benches
                .iter_mut()
                .filter(|(_, b)| now - b.note.looked >= RECIPE_LOOK)
                .map(|(id, b)| {
                    b.note.looked = now;
                    (*id, b.refr)
                })
                .collect()
        });
        let (known, master) = if looks.is_empty() { (0, false) } else { alchemist() };
        for (mortar, refr) in looks {
            let (apparatus, _) = bench_forms(refr, &[], &offered);
            let (all, near) = ingredients_at(refr, &offered);
            let key = RecipeKey { ingredients: all.iter().map(|&f| (form_id(f), near.contains(&f))).collect(), apparatus: apparatus.map(form_id), known, master };
            if with_state(|s| s.benches.get(&mortar).is_none_or(|b| b.note.key.as_ref() == Some(&key))) {
                continue;
            }
            let codes: Vec<Vec<u32>> = all.iter().map(|&f| effect_codes(f, known)).collect();
            let mut tries: Vec<Vec<usize>> = if master { all.iter().map(|&f| vec![f]).collect() } else { Vec::new() };
            for i in 0..all.len() {
                for j in i + 1..all.len() {
                    if codes[i].iter().any(|c| codes[j].contains(c)) {
                        tries.push(vec![all[i], all[j]]);
                    }
                }
            }
            tracing::info!(
                "alchemy: the note at {mortar:#x} works out what {} ingredients make ({} at the bench; {} mixes to try, {known} effects known{})",
                all.len(),
                near.len(),
                tries.len(),
                if master { ", a master" } else { "" }
            );
            // (Taken from the end: in order.)
            tries.reverse();
            with_state(|s| {
                if let Some(b) = s.benches.get_mut(&mortar) {
                    b.note.key = Some(key);
                    b.note.work = Some(Work { apparatus, near, tries, found: Vec::new() });
                    b.note.complete = false;
                }
            });
        }
        // A few mixes a frame, through the game's menu.
        let t0 = std::time::Instant::now();
        let mortars: Vec<u32> = with_state(|s| s.benches.iter().filter(|(_, b)| b.note.work.is_some()).map(|(id, _)| *id).collect());
        for mortar in mortars {
            let Some(mut work) = with_state(|s| s.benches.get_mut(&mortar).and_then(|b| b.note.work.take())) else { continue };
            while t0.elapsed().as_secs_f64() < RECIPE_BUDGET {
                let Some(mix) = work.tries.pop() else { break };
                let Some(brewed) = crate::hooks::guard("alchemy menu", None, || menu::brew(work.apparatus, &mix, false)) else {
                    // (The player's own alchemy menu is open: later.)
                    work.tries.push(mix);
                    break;
                };
                let Some((icon, _)) = brewed.effects.first() else { continue };
                let icon = format!("textures\\menus\\{icon}");
                let ingredients = mix
                    .iter()
                    .map(|&f| Ingredient {
                        form: form_id(f),
                        name: crate::combat::read_str(mem::deref(f + INGR_NAME)),
                        icon: format!("textures\\menus\\icons\\{}", crate::combat::read_str(mem::deref(f + INGR_ICON))),
                        near: work.near.contains(&f),
                    })
                    .collect();
                work.found.push(Recipe { name: brewed.name, icon, effects: brewed.effects.into_iter().map(|e| e.1).collect(), value: brewed.value, ingredients });
            }
            with_state(|s| {
                let Some(b) = s.benches.get_mut(&mortar) else { return };
                if !work.tries.is_empty() {
                    b.note.work = Some(work);
                    return;
                }
                b.note.recipes = best_recipes(std::mem::take(&mut work.found));
                b.note.complete = true;
                b.note.recipes_gen += 1;
                // The recipe chosen as the list has it now (where its ingredients are); gone from it
                // (used up), as it was.
                if let Some(c) = &mut b.note.chosen {
                    let same = |n: &&Recipe| n.name == c.name && n.ingredients.iter().map(|i| i.form).eq(c.ingredients.iter().map(|i| i.form));
                    if let Some(n) = b.note.recipes.iter().find(same) {
                        *c = n.clone();
                    }
                }
                tracing::info!(
                    "alchemy: the note at {mortar:#x} lists {} potions: {}",
                    b.note.recipes.len(),
                    b.note.recipes.iter().take(8).map(|r| format!("{} ({}, value {})", r.name, r.ingredients.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(" + "), r.value)).collect::<Vec<_>>().join("; ")
                );
            });
        }
    }

    /// The apparatus the bench at `mortar` has (its mortar, and the best alembic, calcinator and
    /// retort within reach), and the forms of its ingredients.
    fn bench_forms(mortar: usize, contents: &[u32], offered: &[items::Offered]) -> ([usize; 4], Vec<usize>) {
        let base = mem::deref(mortar + REFR_BASE);
        let at = game_pos(mortar).unwrap_or_default();
        let near: Vec<(u8, f32, usize)> = offered
            .iter()
            .filter(|o| o.kind == items::APPARATUS && o.refr != mortar)
            .filter(|o| game_pos(o.refr).is_some_and(|p| p.distance(at) <= REACH * camera::UNITS_PER_METER))
            .filter_map(|o| {
                let f = mem::deref(o.refr + REFR_BASE);
                Some((mem::at::<u8>(f + APPA_KIND)?, mem::at::<f32>(f + APPA_QUALITY)?, f))
            })
            .collect();
        let [alembic, calcinator, retort] = best(&near);
        let forms = contents.iter().filter_map(|id| offered.iter().find(|o| o.id == *id)).map(|o| mem::deref(o.refr + REFR_BASE)).collect();
        ([base, alembic, calcinator, retort], forms)
    }

    /// What the bench at `mortar` would make, on its card.
    fn preview(mortar: u32) {
        let Some((refr, contents)) = with_state(|s| s.benches.get(&mortar).map(|b| (b.refr, b.contents.clone()))) else { return };
        let offered = items::offered();
        let (apparatus, forms) = bench_forms(refr, &contents, &offered);
        let ingredients: Vec<(String, String)> = contents
            .iter()
            .filter_map(|id| offered.iter().find(|o| o.id == *id))
            .map(|o| {
                let f = mem::deref(o.refr + REFR_BASE);
                let icon = crate::combat::read_str(mem::deref(f + INGR_ICON));
                (format!("textures\\menus\\icons\\{icon}"), items::facts(o.refr).name)
            })
            .collect();
        let t0 = std::time::Instant::now();
        let brewed = if forms.is_empty() { None } else { crate::hooks::guard("alchemy menu", None, || menu::brew(apparatus, &forms, false)) };
        let took = t0.elapsed().as_secs_f32() * 1000.0;
        let shown = Shown {
            name: brewed.as_ref().filter(|b| !b.effects.is_empty()).map(|b| b.name.clone()).unwrap_or_default(),
            effects: brewed.as_ref().map(|b| b.effects.iter().map(|(icon, words)| (format!("textures\\menus\\{icon}"), words.clone())).collect()).unwrap_or_default(),
            ingredients,
            apparatus: apparatus.iter().filter(|&&f| f != 0).map(|&f| format!("textures\\menus\\icons\\{}", crate::combat::read_str(mem::deref(f + APPA_ICON)))).collect(),
        };
        tracing::info!(
            "alchemy: {} in the mortar ({:#x}; alembic {:#x}, calcinator {:#x}, retort {:#x}): the game would make {:?}, {:?}, weight {:.2}, value {} ({took:.1} ms)",
            forms.len(),
            form_id(apparatus[0]),
            form_id(apparatus[1]),
            form_id(apparatus[2]),
            form_id(apparatus[3]),
            shown.name,
            shown.effects.iter().map(|e| e.1.as_str()).collect::<Vec<_>>(),
            brewed.as_ref().map_or(0.0, |b| b.weight),
            brewed.as_ref().map_or(0, |b| b.value)
        );
        with_state(|s| {
            if let Some(b) = s.benches.get_mut(&mortar) {
                b.shown = Some(shown);
                b.revision = b.revision.wrapping_add(1);
            }
        });
    }

    /// Everything in the bowl is ground: the game makes the potion (or there's nothing to make).
    fn make(mortar: u32) {
        let Some((refr, contents)) = with_state(|s| s.benches.get(&mortar).map(|b| (b.refr, b.contents.clone()))) else { return };
        let offered = items::offered();
        let (apparatus, forms) = bench_forms(refr, &contents, &offered);
        if forms.is_empty() {
            return;
        }
        let player = engine::player();
        type Message = unsafe extern "C" fn(usize, u32, u32, f32);
        type InCombat = unsafe extern "thiscall" fn(usize, u8) -> u8;
        // No alchemy in combat, unless the game's setting allows it (its own words): ground, it
        // waits.
        if unsafe { std::mem::transmute::<usize, InCombat>(IN_COMBAT)(player, 0) } != 0 && mem::at::<u32>(ALCHEMY_IN_COMBAT) == Some(0) {
            unsafe { std::mem::transmute::<usize, Message>(MESSAGE)(mem::deref(NO_ALCHEMY_IN_COMBAT), 0, 1, -1.0) };
            with_state(|s| {
                if let Some(b) = s.benches.get_mut(&mortar) {
                    b.done = false;
                    b.retry_at = crate::link::now() + RETRY;
                }
            });
            tracing::info!("alchemy: no potions in combat (the game's rule): it waits");
            return;
        }
        // The ingredients come out of the bowl into the player's things (the game's own pick-up,
        // stealing and all): the game's Create uses them up from there. (No card of anything held
        // meanwhile: the game's card would read what's being taken.)
        crate::hud_info::set_held(0);
        let taken: Vec<usize> = contents.iter().filter_map(|id| offered.iter().find(|o| o.id == *id)).map(|o| o.refr).collect();
        for &r in &taken {
            items::pick_up(r);
        }
        let brewed = crate::hooks::guard("alchemy create", None, || menu::brew(apparatus, &forms, true));
        match brewed {
            Some(b) if b.potion != 0 => {
                let id = form_id(b.potion);
                tracing::info!("alchemy: the game made {:?} ({id:#010x}): {:?}, weight {:.2}, value {}", b.name, b.effects.iter().map(|e| e.1.as_str()).collect::<Vec<_>>(), b.weight, b.value);
                // Its bottle, out of the player's things into the world (the game's own drop),
                // rises out of the mortar.
                crate::console::run_script(&format!("player.drop {id:08X} 1"));
                if let Some(bottle) = items::newest_of(b.potion) {
                    let Some(m) = with_state(|s| s.benches.get(&mortar).and_then(|b| node_pose(b.node).map(|p| (p, b.bench.floor, b.bench.depth)))) else { return };
                    let ((mp, mq), floor, depth) = m;
                    let from = mp + mq * Vec3::from_array(floor);
                    let to = from + mq * Vec3::Y * (depth + RISE_ABOVE);
                    tracing::info!("alchemy: its bottle {:#010x} rises out of the mortar", form_id(bottle));
                    with_state(|s| s.rising = Some(Rising { mortar, refr: bottle, from, to, t0: crate::link::now() }));
                }
                // The note on its recipe: made, until the bottle's taken.
                with_state(|s| {
                    if let Some(b) = s.benches.get_mut(&mortar).filter(|b| b.note.page == Page::Recipe) {
                        b.note.made = true;
                    }
                });
            }
            Some(_) => {
                // No effect shared: nothing to make. The game's own words for a failed potion, and
                // its sound for an ingredient that does nothing; the ingredients are lost.
                type Remove = unsafe extern "thiscall" fn(usize, usize, usize, i32, u8, usize, usize, usize, usize, u8, u8) -> usize;
                let remove = mem::deref(mem::deref(player) + VT_REMOVE);
                for &f in &forms {
                    unsafe { std::mem::transmute::<usize, Remove>(remove)(player, f, 0, 1, 0, 0, 0, 0, 0, 1, 1) };
                }
                type Sound = unsafe extern "C" fn(u32);
                unsafe {
                    std::mem::transmute::<usize, Message>(MESSAGE)(mem::deref(POTION_FAILED), 0, 1, -1.0);
                    std::mem::transmute::<usize, Sound>(UI_SOUND)(SOUND_NOTHING);
                }
                tracing::info!("alchemy: no effect shared by the {} ingredients: the potion failed", forms.len());
                with_state(|s| {
                    if let Some(b) = s.benches.get_mut(&mortar) {
                        b.note.back_to_list();
                    }
                });
            }
            None => tracing::warn!("alchemy: the game's alchemy menu couldn't be made"),
        }
    }

    /// Before the eyes are drawn: the benches near the player (their bowls, pestles and cards for
    /// the runtime), what's happened in them, and a bottle rising.
    pub fn place(view: &rustvr_adapter::View, now: f64) {
        let offered = items::offered();
        with_state(|s| {
            if !crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false) {
                s.states.count = 0;
            }
            let states = &s.states.states[..(s.states.count as usize).min(MAX_OBJECTS)];
            // The mortars among the things to take.
            let mut seen = Vec::new();
            for o in offered.iter().filter(|o| o.kind == items::APPARATUS) {
                let Some(bench) = bench_of(mem::deref(o.refr + REFR_BASE)) else { continue };
                seen.push(o.id);
                if s.benches.get(&o.id).is_none_or(|b| b.refr != o.refr) {
                    let seq = s.next_seq;
                    s.next_seq += 1;
                    let node = mem::deref(o.refr + REFR_NODE);
                    let mut b = Station {
                        refr: o.refr,
                        node,
                        bench,
                        seq,
                        contents: Vec::new(),
                        shown: None,
                        revision: 1,
                        strokes: 0,
                        done: false,
                        retry_at: 0.0,
                        flags: None,
                        ground: 0.0,
                        note: Note::new(),
                    };
                    hide(&mut b, true);
                    let f = items::facts(o.refr);
                    tracing::info!("alchemy: {} ({:#x}{}) is a bench: its pestle out of it, its bowl the hands'", f.name, o.id, if f.stolen { ", someone else's" } else { "" });
                    s.benches.insert(o.id, b);
                    s.jobs.push(Job::Warm { mortar: o.id });
                }
                // Its 3D made again (the cell reloaded): the new one hidden.
                let node = mem::deref(o.refr + REFR_NODE);
                if let Some(b) = s.benches.get_mut(&o.id).filter(|b| b.node != node && node != 0) {
                    b.node = node;
                    b.flags = None;
                    hide(b, true);
                }
            }
            // Gone (taken, out of reach): the game's own model of it again.
            s.benches.retain(|id, b| {
                let keep = seen.contains(id);
                if !keep {
                    hide(b, false);
                }
                keep
            });
            // What's in each bowl, its grinding, and whether it's done.
            for (id, b) in s.benches.iter_mut() {
                let mut contents: Vec<u32> = states.iter().filter(|st| st.flags & OS_IN_RECEPTACLE != 0 && st.parent == *id).map(|st| st.id).collect();
                contents.sort();
                if contents != b.contents {
                    let more = contents.iter().any(|c| !b.contents.contains(c));
                    b.contents = contents;
                    if more {
                        s.jobs.push(Job::Sound { mortar: *id, sound: SND_INGREDIENT_DOWN });
                    }
                    if b.contents.is_empty() {
                        b.done = false;
                        b.shown = None;
                    } else {
                        s.jobs.push(Job::Preview { mortar: *id });
                    }
                }
                if let Some(st) = states.iter().find(|st| st.id == *id) {
                    if st.shots > b.strokes && !b.contents.is_empty() {
                        s.jobs.push(Job::Sound { mortar: *id, sound: SND_CRUNCH });
                    }
                    b.strokes = st.shots;
                    b.ground = st.slide;
                    if st.slide >= 0.999 && !b.contents.is_empty() && !b.done && now >= b.retry_at {
                        b.done = true;
                        tracing::info!("alchemy: everything in the bowl is ground");
                        s.jobs.push(Job::Make { mortar: *id });
                    }
                }
            }
            // (The notes' states apart: the notes are the state's to change.)
            let note_states: Vec<ObjectState> = states.iter().filter(|st| st.id & 0xFF00_0000 == NOTE).copied().collect();
            notes(s, view, &note_states, &offered);
            publish(s, &offered);
            // A bottle rising out of a mortar: where it is now, still, until a hand has it.
            if let Some(r) = &s.rising {
                let held = items::in_hand(r.refr);
                let t = ((now - r.t0) / RISE_TIME).clamp(0.0, 1.0) as f32;
                let k = t * t * (3.0 - 2.0 * t);
                let lost = now - r.t0 > FIND_TIME && !items::offered().iter().any(|o| o.refr == r.refr);
                if held || now - r.t0 > WAIT || lost {
                    if !held {
                        items::release(r.refr);
                    }
                    // Its note, the potion made, to the list again.
                    if let Some(b) = s.benches.get_mut(&r.mortar).filter(|b| b.note.made) {
                        b.note.back_to_list();
                    }
                    s.rising = None;
                } else {
                    items::hold_at(r.refr, r.from.lerp(r.to, k), Quat::IDENTITY);
                }
            }
            let _ = view;
        });
    }

    /// Whether the player (their head) is at the bench whose mortar is at `mp`: [`NOTE_NEAR`] it
    /// and facing it.
    fn at_bench(view: &rustvr_adapter::View, mp: Vec3) -> bool {
        let to = Vec3::new(mp.x - view.head_pos.x, 0.0, mp.z - view.head_pos.z);
        let look = Vec3::new(view.look_dir.x, 0.0, view.look_dir.z);
        to.length() <= NOTE_NEAR && to.try_normalize().zip(look.try_normalize()).is_some_and(|(t, l)| t.dot(l) >= NOTE_FACING.cos())
    }

    /// Where a bench's note is put: on what the mortar stands on, [`NOTE_OUT`] from it towards the
    /// player's head at `head`, its page the right way up to them.
    fn note_place(mortar: (Vec3, Quat), bench: &Bench, head: Vec3) -> (Vec3, Quat) {
        let (mp, mq) = mortar;
        let d = Vec3::new(head.x - mp.x, 0.0, head.z - mp.z).try_normalize().unwrap_or(Vec3::Z);
        let foot = (mp + mq * Vec3::new(0.0, bench.bottom, 0.0)).y;
        let at = Vec3::new(mp.x, foot + NOTE_HALF_THICK + 0.01, mp.z) + d * NOTE_OUT;
        // Its top (+Z) away from the player.
        (at, Quat::from_rotation_y((-d.x).atan2(-d.z)))
    }

    /// The notes: a hand taking one up or putting it down (the game's scroll sounds), a fingertip
    /// on one (a page turned: a recipe chosen, back to the list, the list turned over), and each
    /// one's page as the bench gets on with its recipe.
    fn notes(s: &mut State, view: &rustvr_adapter::View, states: &[ObjectState], offered: &[items::Offered]) {
        let fonts = note_fonts();
        for (id, b) in s.benches.iter_mut() {
            if b.note.at.is_none() {
                b.note.at = node_pose(b.node).filter(|(mp, _)| alive(b) && at_bench(view, *mp)).map(|p| note_place(p, b.bench, view.head_pos));
            }
            let n = &mut b.note;
            if let Some(st) = states.iter().find(|st| st.id == note_id(*id)) {
                let held = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0;
                if held != n.held {
                    n.held = held;
                    s.jobs.push(Job::NoteSound { sound: if held { SND_SCROLL_UP } else { SND_SCROLL_DOWN } });
                }
                let seen = n.touches.replace(st.shots);
                if seen.is_some_and(|t| st.shots > t) {
                    let at = [st.slide * NOTE_SIZE[0], st.fuse * NOTE_SIZE[1]];
                    let touch = touched(&n.targets, at);
                    tracing::info!("alchemy: the note at {id:#x} touched at {at:.0?}: {touch:?}");
                    match touch {
                        Some(Touch::Choose(i)) => {
                            n.chosen = n.recipes.get(i).cloned();
                            n.page = Page::Recipe;
                        }
                        Some(Touch::Back) => n.back_to_list(),
                        Some(Touch::Turn) => {
                            n.list_page = (n.list_page + 1) % list_pages(&n.recipes, fonts).len().max(1);
                            n.page = Page::List(n.list_page);
                        }
                        None => {}
                    }
                    if touch.is_some() {
                        s.jobs.push(Job::NoteSound { sound: SND_PAGE_TURN });
                    }
                }
            }
            // How far the bench has got with the recipe chosen: what's in the bowl of it (and
            // what isn't), and whether it's ground.
            let progress = match (&n.chosen, n.page) {
                (Some(r), Page::Recipe) if n.made => Progress { extra: Vec::new(), put: vec![true; r.ingredients.len()], ground: true },
                (Some(r), Page::Recipe) => {
                    let inside: Vec<(u32, usize)> = b.contents.iter().filter_map(|c| offered.iter().find(|o| o.id == *c)).map(|o| (form_id(mem::deref(o.refr + REFR_BASE)), o.refr)).collect();
                    Progress {
                        extra: inside.iter().filter(|(f, _)| !r.ingredients.iter().any(|i| i.form == *f)).map(|(_, refr)| items::facts(*refr).name).collect(),
                        put: r.ingredients.iter().map(|i| inside.iter().any(|(f, _)| *f == i.form)).collect(),
                        ground: !inside.is_empty() && b.ground >= 0.999,
                    }
                }
                _ => Progress::default(),
            };
            let shown = (n.page, progress, n.recipes_gen, n.complete);
            if n.shown.as_ref() != Some(&shown) {
                n.revision = n.revision.wrapping_add(1);
                let (card, targets) = note_card(note_id(*id), n.revision, n.page, &n.recipes, n.chosen.as_ref(), n.complete, &shown.1, fonts);
                n.card = Some(card);
                n.targets = targets;
                n.shown = Some(shown);
            }
        }
    }

    /// The runtime's part: each bench's bowl (what may go in: the ingredients near it, never a kind
    /// already in), its pestle, its note and its cards.
    fn publish(s: &mut State, offered: &[items::Offered]) {
        let mut recs: Vec<Receptacle> = Vec::new();
        let mut pieces: Vec<ObjectDesc> = Vec::new();
        let mut cards: Vec<Card> = Vec::new();
        for (id, b) in &s.benches {
            let mut r = receptacle(*id, b.bench);
            let at = game_pos(b.refr).unwrap_or_default();
            let kinds: Vec<usize> = b.contents.iter().filter_map(|c| offered.iter().find(|o| o.id == *c)).map(|o| mem::deref(o.refr + REFR_BASE)).collect();
            for c in &b.contents {
                r.push_accept(*c);
            }
            for o in offered.iter().filter(|o| o.kind == items::INGREDIENT && !b.contents.contains(&o.id)) {
                let base = mem::deref(o.refr + REFR_BASE);
                if !kinds.contains(&base) && game_pos(o.refr).is_some_and(|p| p.distance(at) <= REACH * 2.0 * camera::UNITS_PER_METER) {
                    r.push_accept(o.id);
                }
            }
            recs.push(r);
            if let Some(pose) = node_pose(b.node).filter(|_| alive(b)) {
                // (The game's weight is about pounds: a kilogram is two.)
                let weight = mem::at::<f32>(mem::deref(b.refr + REFR_BASE) + APPA_WEIGHT).filter(|w| w.is_finite() && *w > 0.0).unwrap_or(1.0);
                let mass = weight * 0.45 * pestle_share(b.bench);
                pieces.push(pestle_desc(*id, pose, b.bench, mass, b.seq));
            }
            if let Some(at) = b.note.at {
                pieces.push(note_desc(*id, at, b.seq));
            }
            if let Some(shown) = b.shown.as_ref().filter(|_| !b.contents.is_empty()) {
                cards.push(card(*id, b.revision, shown, fonts()));
            }
            // (A note's page only once it's down: with nothing to lie on it would float.)
            if let Some(c) = b.note.card.filter(|_| b.note.at.is_some()) {
                cards.push(c);
            }
        }
        recs.sort_by_key(|r| r.id);
        if recs != s.sent_receptacles {
            s.receptacles.count = 0;
            for r in recs.iter().take(MAX_RECEPTACLES) {
                s.receptacles.entries[s.receptacles.count as usize] = *r;
                s.receptacles.count += 1;
            }
            if crate::link::with_adapter(|a| a.write_receptacles(&s.receptacles)).unwrap_or(false) {
                // What may go into each bowl, and where it is from the mortar (metres east, up,
                // north).
                for r in &recs {
                    let Some(m) = s.benches.get(&r.id).and_then(|b| game_pos(b.refr)) else { continue };
                    let near: Vec<String> = r
                        .accepts()
                        .iter()
                        .filter_map(|id| offered.iter().find(|o| o.id == *id))
                        .filter_map(|o| Some((items::facts(o.refr).name, (game_pos(o.refr)? - m) / camera::UNITS_PER_METER)))
                        .map(|(n, d)| format!("{n} {:.2?}", [d.x, d.z, d.y]))
                        .collect();
                    tracing::info!("alchemy: into the mortar {:#x} may go: {}", r.id, near.join(", "));
                }
                s.sent_receptacles = recs;
            }
        }
        let mut ids: Vec<(u32, u32)> = pieces.iter().map(|p| (p.id, p.spawn_seq)).collect();
        ids.sort();
        if ids != s.sent_pieces && crate::link::with_adapter(|a| crate::objects::publish(crate::objects::Owner::Alchemy, &pieces, a)).unwrap_or(false) {
            for p in &pieces {
                let (at, q) = (Vec3::from_array(p.spawn_pose.pos), Quat::from_array(p.spawn_pose.rot));
                if p.flags & OBJF_SHEET != 0 {
                    tracing::info!("alchemy: a note the hands' own ({:#x}) at {at:.3?}, its top towards {:.2?}", p.id, q * Vec3::Z);
                } else {
                    let half = p.half_extents[1] + p.radius;
                    tracing::info!("alchemy: a pestle the hands' own ({:#x}): its handle's end at {:.3?}, its tip at {:.3?}", p.id, at + q * Vec3::Y * half, at - q * Vec3::Y * half);
                }
            }
            s.sent_pieces = ids;
        }
        let mut keys: Vec<(u32, u32)> = cards.iter().map(|c| (c.anchor, c.revision)).collect();
        keys.sort();
        if keys != s.sent_cards {
            s.cards.count = 0;
            for c in cards.iter().take(MAX_CARDS) {
                s.cards.cards[s.cards.count as usize] = *c;
                s.cards.count += 1;
            }
            if crate::link::with_adapter(|a| a.write_cards(&s.cards)).unwrap_or(false) {
                s.sent_cards = keys;
            }
        }
    }
}

