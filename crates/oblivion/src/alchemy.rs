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
//! Forms: `TESObjectAPPA` model path +0x34, quality +0x74, kind +0x78 (0 mortar and pestle,
//! 1 alembic, 2 calcinator, 3 retort); `IngredientItem` icon +0x5C.

use glam::{Quat, Vec3};
use rustvr_protocol::*;

/// Apparatus (and ingredients) within this far of the mortar (metres) are the bench's.
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
        // Its ingredients, then the game works the potion out.
        for (k, &form) in ingredients.iter().take(4).enumerate() {
            unsafe { ((menu + INGREDIENTS + 4 * k) as *mut usize).write(alloc_record(form)) };
        }
        unsafe {
            (SLOT as *mut u32).write(0);
            std::mem::transmute::<usize, Set>(SET_INGREDIENTS)(menu, 1);
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
    /// The game's sounds (`SOUN` form ids): an ingredient put down; organic matter knocked about
    /// (a crunch).
    const SND_INGREDIENT_DOWN: u32 = 0x0008_9772;
    const SND_CRUNCH: u32 = 0x0009_7C61;

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
        /// Its game model's flags before it was hidden.
        flags: Option<u16>,
    }

    /// A bottle rising out of a mortar.
    struct Rising {
        refr: usize,
        from: Vec3,
        to: Vec3,
        t0: f64,
    }

    /// Work for the game's frame (where its scripts run): what to do there.
    enum Job {
        /// Work out what the bowl would make, for its card.
        Preview { mortar: u32 },
        /// Everything's ground: make it.
        Make { mortar: u32 },
        /// A game sound from the mortar.
        Sound { mortar: u32, sound: u32 },
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
        sent_pestles: Vec<(u32, u32)>,
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
            sent_pestles: Vec::new(),
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

    /// Hides (or shows again) a mortar's game model: the runtime draws its pieces.
    fn hide(b: &mut Station, hidden: bool) {
        let at = b.node + av::FLAGS;
        if !mem::is_readable(at as *const u8, 2) {
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
    pub fn frame(_now: f64) {
        let jobs = with_state(|s| std::mem::take(&mut s.jobs));
        for job in jobs {
            match job {
                Job::Preview { mortar } => preview(mortar),
                Job::Make { mortar } => make(mortar),
                Job::Sound { mortar, sound } => {
                    crate::console::run_script(&format!("{mortar:08X}.PlaySound3D {sound:08X}"));
                }
            }
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
        let brewed = if forms.is_empty() { None } else { crate::hooks::guard("alchemy menu", None, || menu::brew(apparatus, &forms, false)) };
        let shown = Shown {
            name: brewed.as_ref().filter(|b| !b.effects.is_empty()).map(|b| b.name.clone()).unwrap_or_default(),
            effects: brewed.as_ref().map(|b| b.effects.iter().map(|(icon, words)| (format!("textures\\menus\\{icon}"), words.clone())).collect()).unwrap_or_default(),
            ingredients,
            apparatus: apparatus.iter().filter(|&&f| f != 0).map(|&f| format!("textures\\menus\\icons\\{}", crate::combat::read_str(mem::deref(f + APPA_ICON)))).collect(),
        };
        tracing::info!(
            "alchemy: {} in the mortar ({:#x}; alembic {:#x}, calcinator {:#x}, retort {:#x}): the game would make {:?}, {:?}, weight {:.2}, value {}",
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
        // The ingredients come out of the bowl into the player's things (the game's own pick-up,
        // stealing and all): the game's Create uses them up from there. (No card of anything held
        // meanwhile: the game's card would read what's being taken.)
        crate::hud_info::set_held(0);
        let taken: Vec<usize> = contents.iter().filter_map(|id| offered.iter().find(|o| o.id == *id)).map(|o| o.refr).collect();
        for &r in &taken {
            items::pick_up(r);
        }
        let player = engine::player();
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
                    with_state(|s| s.rising = Some(Rising { refr: bottle, from, to, t0: crate::link::now() }));
                }
            }
            Some(_) => {
                // No effect shared: nothing to make. The game's own words for a failed potion, and
                // its sound for an ingredient that does nothing; the ingredients are lost.
                type Remove = unsafe extern "thiscall" fn(usize, usize, usize, i32, u8, usize, usize, usize, usize, u8, u8) -> usize;
                let remove = mem::deref(mem::deref(player) + VT_REMOVE);
                for &f in &forms {
                    unsafe { std::mem::transmute::<usize, Remove>(remove)(player, f, 0, 1, 0, 0, 0, 0, 0, 1, 1) };
                }
                type Message = unsafe extern "C" fn(usize, u32, u32, f32);
                type Sound = unsafe extern "C" fn(u32);
                unsafe {
                    std::mem::transmute::<usize, Message>(MESSAGE)(mem::deref(POTION_FAILED), 0, 1, -1.0);
                    std::mem::transmute::<usize, Sound>(UI_SOUND)(SOUND_NOTHING);
                }
                tracing::info!("alchemy: no effect shared by the {} ingredients: the potion failed", forms.len());
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
                    let mut b = Station { refr: o.refr, node, bench, seq, contents: Vec::new(), shown: None, revision: 1, strokes: 0, done: false, flags: None };
                    hide(&mut b, true);
                    let f = items::facts(o.refr);
                    tracing::info!("alchemy: {} ({:#x}{}) is a bench: its pestle out of it, its bowl the hands'", f.name, o.id, if f.stolen { ", someone else's" } else { "" });
                    s.benches.insert(o.id, b);
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
                    if st.slide >= 0.999 && !b.contents.is_empty() && !b.done {
                        b.done = true;
                        tracing::info!("alchemy: everything in the bowl is ground");
                        s.jobs.push(Job::Make { mortar: *id });
                    }
                }
            }
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
                    s.rising = None;
                } else {
                    items::hold_at(r.refr, r.from.lerp(r.to, k), Quat::IDENTITY);
                }
            }
            let _ = view;
        });
    }

    /// The runtime's part: each bench's bowl (what may go in: the ingredients near it, never a kind
    /// already in), its pestle and its card.
    fn publish(s: &mut State, offered: &[items::Offered]) {
        let mut recs: Vec<Receptacle> = Vec::new();
        let mut pestles: Vec<ObjectDesc> = Vec::new();
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
            if let Some(pose) = node_pose(b.node) {
                // (The game's weight is about pounds: a kilogram is two.)
                let weight = mem::at::<f32>(mem::deref(b.refr + REFR_BASE) + APPA_WEIGHT).filter(|w| w.is_finite() && *w > 0.0).unwrap_or(1.0);
                let mass = weight * 0.45 * pestle_share(b.bench);
                pestles.push(pestle_desc(*id, pose, b.bench, mass, b.seq));
            }
            if let Some(shown) = b.shown.as_ref().filter(|_| !b.contents.is_empty()) {
                cards.push(card(*id, b.revision, shown, fonts()));
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
        let mut ids: Vec<(u32, u32)> = pestles.iter().map(|p| (p.id, p.spawn_seq)).collect();
        ids.sort();
        if ids != s.sent_pestles && crate::link::with_adapter(|a| crate::objects::publish(crate::objects::Owner::Alchemy, &pestles, a)).unwrap_or(false) {
            for p in &pestles {
                let (at, q) = (Vec3::from_array(p.spawn_pose.pos), Quat::from_array(p.spawn_pose.rot));
                let half = p.half_extents[1] + p.radius;
                tracing::info!("alchemy: a pestle the hands' own ({:#x}): its handle's end at {:.3?}, its tip at {:.3?}", p.id, at + q * Vec3::Y * half, at - q * Vec3::Y * half);
            }
            s.sent_pestles = ids;
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

