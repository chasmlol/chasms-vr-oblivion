//! The game's own sounds as the runtime's sound pack (`rustvr_audio::pack`): every sound the
//! runtime makes for the hands (a grab, a holster, the gravity gloves, a thing knocking into the
//! world, a slot found) is one of Oblivion's, as the game itself sounds it.
//!
//! Each is one of the game's sound records (`SOUN`, read from `Oblivion.esm`). A record names a
//! file, or a folder whose files it picks among, and how much quieter than its file it plays (its
//! static attenuation); a menu or 2D sound plays in the head, not where it happens. Which record
//! each of the runtime's events plays:
//! - a thing the hands take, put away, drop or catch: the inventory's sounds of a thing of its kind
//!   (`ITM<kind>Up`, `ITM<kind>Down`), as the game's item-sound choice picks them (0x5E96E0), a set
//!   per kind named by it ([`kind_of`]: `potion`, `weaponblade` ...);
//! - a thing knocking into the world: the game's collision sound for what it's made of and how
//!   heavy it is, as the game's collision-sound choice picks them (0x6B0350): a set per kind
//!   and collision sound ([`with_collision`]: `potion.cglasssmall`), and the kind's own set for
//!   the one the kind is usually made of;
//! - the weapon on the body drawn and sheathed: the game's draw and sheathe of its type
//!   (`WPN<type>Equip`, `WPN<type>Unequip`), an enchanted one's draw with its enchanted ring over
//!   it, as the game layers it ([`weapon_set`]);
//! - the gravity gloves: a big weapon's swish, quiet, as a thing is flicked to the hand (the
//!   player's pick of the game's whooshes); Telekinesis's travel as it flies (a Mysticism effect:
//!   its bolt's, from its `MGEF` record); landing in the hand, a thing is taken as any is (its
//!   kind's sound of being taken); a thing locked onto has no sound, only its pulse in the hand;
//! - a bite or a gulp of what's held at the mouth: the game's chewing and swallowing;
//! - a hand finding a slot, and the runtime's own menu: the menus' focus and OK.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};

use crate::bsa::{Archives, normalise};

/// One of the game's sound records.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundRecord {
    pub editor_id: String,
    /// Under `Data\Sound`: a `.wav`, or a folder (ending in `\`) whose files it picks among.
    pub file: String,
    /// `TESSound::kFlags_*`: menu sound 0x20, 2D 0x40.
    pub flags: u32,
    /// How much quieter than its file it plays, in decibels.
    pub attenuation: f32,
}

impl SoundRecord {
    /// Its level against its file's.
    pub fn gain(&self) -> f32 {
        10f32.powf(-self.attenuation / 20.0)
    }

    /// Placed where it happens (not a menu's or a 2D sound, which play in the head).
    pub fn spatial(&self) -> bool {
        self.flags & (0x20 | 0x40) == 0
    }
}

/// The sound records of a plugin (`Oblivion.esm`), by editor id (lowercase).
pub fn read_records(esm: &[u8]) -> Result<HashMap<String, SoundRecord>> {
    let mut out = HashMap::new();
    ensure!(esm.len() > 20 && &esm[..4] == b"TES4", "not a plugin");
    let header = u32_at(esm, 4)? as usize;
    walk(esm, 20 + header, esm.len(), &mut out)?;
    Ok(out)
}

fn u32_at(b: &[u8], at: usize) -> Result<u32> {
    let s = b.get(at..at + 4).context("cut short")?;
    Ok(u32::from_le_bytes(s.try_into().unwrap()))
}

/// Records and groups from `at` to `end`: into the sounds' top group (and any group inside one).
fn walk(b: &[u8], mut at: usize, end: usize, out: &mut HashMap<String, SoundRecord>) -> Result<()> {
    while at + 20 <= end {
        let kind = &b[at..at + 4];
        let size = u32_at(b, at + 4)? as usize;
        if kind == b"GRUP" {
            let label = &b[at + 8..at + 12];
            let top = u32_at(b, at + 12)? == 0;
            if !top || label == b"SOUN" {
                walk(b, at + 20, (at + size).min(end), out)?;
            }
            at += size.max(20);
            continue;
        }
        let flags = u32_at(b, at + 8)?;
        let body = b.get(at + 20..at + 20 + size).context("record cut short")?;
        if kind == b"SOUN" {
            let body = if flags & 0x0004_0000 != 0 {
                let mut v = Vec::new();
                flate2::read::ZlibDecoder::new(body.get(4..).context("compressed record")?).read_to_end(&mut v)?;
                v
            } else {
                body.to_vec()
            };
            if let Some(r) = sound_record(&body) {
                out.insert(r.editor_id.to_ascii_lowercase(), r);
            }
        }
        at += 20 + size;
    }
    Ok(())
}

/// A `SOUN` record's fields: `EDID`, `FNAM`, and `SNDX` (or the older `SNDD`): flags at +4, the
/// static attenuation (hundredths of a decibel) at +8.
fn sound_record(body: &[u8]) -> Option<SoundRecord> {
    let text = |d: &[u8]| String::from_utf8_lossy(d.split(|&c| c == 0).next().unwrap_or_default()).into_owned();
    let (mut editor_id, mut file, mut flags, mut attenuation) = (String::new(), String::new(), 0, 0.0);
    let mut at = 0;
    while at + 6 <= body.len() {
        let kind = &body[at..at + 4];
        let size = u16::from_le_bytes([body[at + 4], body[at + 5]]) as usize;
        let d = body.get(at + 6..at + 6 + size)?;
        match kind {
            b"EDID" => editor_id = text(d),
            b"FNAM" => file = text(d),
            b"SNDX" | b"SNDD" if d.len() >= 8 => {
                flags = u32::from_le_bytes(d[4..8].try_into().unwrap());
                if kind == b"SNDX" && d.len() >= 10 {
                    attenuation = i16::from_le_bytes([d[8], d[9]]) as f32 / 100.0;
                }
            }
            _ => {}
        }
        at += 6 + size;
    }
    (!editor_id.is_empty() && !file.is_empty()).then_some(SoundRecord { editor_id, file, flags, attenuation })
}

/// The sound set of things of a kind, from the inventory's sound of one (`ITMPotionUp`, as the
/// game's item-sound choice names it): `potion`. None for a sound that isn't one of the inventory's.
pub fn kind_of(item_sound: &str) -> Option<String> {
    let s = item_sound.strip_prefix("ITM").or_else(|| item_sound.strip_prefix("itm"))?;
    let s = s.strip_suffix("Up").or_else(|| s.strip_suffix("Down"))?;
    (!s.is_empty()).then(|| s.to_ascii_lowercase())
}

/// The sound set of things of `kind` that knock into the world with the collision sound
/// `collision` (one of [`COLLISIONS`]).
pub fn with_collision(kind: &str, collision: &str) -> String {
    format!("{kind}.{}", collision.to_ascii_lowercase())
}

/// The collision sounds the game's `ImpactMixer` picks among (by the thing's
/// material, then by its mass; "Static" for no mass), by editor id.
pub const COLLISIONS: &[&str] = &[
    "CStoneSmall", "CStoneMedium", "CStoneLarge", "CStoneStatic",
    "CClothSmall", "CClothMedium", "CClothLarge",
    "CEarthSmall", "CEarthMedium", "CEarthLarge",
    "CGlassSmall", "CGlassMedium", "CGlassLarge",
    "CGrassSmall", "CGrassMedium", "CGrassLarge",
    "CMetalSmall", "CMetalMedium", "CMetalLarge", "CMetalStatic",
    "COrganicSmall",
    "CSkinSmall", "CSkinMedium", "CSkinLarge",
    "CWoodSmall", "CWoodMedium", "CWoodLarge", "CWoodStatic",
    "CSpecialHeavyStone", "CSpecialHeavyMetal", "CSpecialHeavyWood",
    "CChainSmall", "CChainMedium", "CChainLarge",
];

/// The inventory's kinds of thing (the item-sound choice's), each with the collision sound of what a thing
/// of the kind is usually made of: what one drawn from a slot (whose own the game doesn't say)
/// knocks into the world with.
const KINDS: &[(&str, &str)] = &[
    ("ITMGeneric", "CWoodSmall"),
    ("ITMLockpick", "CMetalSmall"),
    ("ITMGold", "CMetalSmall"),
    ("ITMApparatus", "CGlassSmall"),
    ("ITMArmorHeavy", "CMetalMedium"),
    ("ITMArmorLight", "CClothMedium"),
    ("ITMScroll", "CClothSmall"),
    ("ITMBook", "CClothMedium"),
    ("ITMClothing", "CClothSmall"),
    ("ITMIngredient", "COrganicSmall"),
    ("ITMWeaponBlade", "CMetalSmall"),
    ("ITMWeaponBlunt", "CMetalMedium"),
    ("ITMWeaponStaff", "CWoodMedium"),
    ("ITMWeaponBow", "CWoodSmall"),
    ("ITMAmmo", "CWoodSmall"),
    ("ITMSoulGem", "CGlassSmall"),
    ("ITMKey", "CMetalSmall"),
    ("ITMPotion", "CGlassSmall"),
];

/// The weapon types (`TESObjectWEAP::type`, 0..5): the game's draw and sheathe sounds' stem, the
/// enchanted draw ring it layers over an enchanted one's (the blades and blunts
/// only), its hit, and the inventory's kind of it.
const WEAPONS: &[(&str, bool, &str, &str)] = &[
    ("WPNBlade1Hand", true, "WPNHitBlade", "weaponblade"),
    ("WPNBlade2Hand", true, "WPNHitBlade", "weaponblade"),
    ("WPNBlunt1Hand", true, "WPNHitBlunt", "weaponblunt"),
    ("WPNBlunt2Hand", true, "WPNHitBlunt", "weaponblunt"),
    ("WPNStaff", false, "WPNHitBlunt", "weaponstaff"),
    ("WPNBow", false, "WPNHitBlunt", "weaponbow"),
];

/// The sound set of the weapon on the body, of `TESObjectWEAP::type` `kind`, enchanted or not.
pub fn weapon_set(kind: u32, enchanted: bool) -> Option<String> {
    let (stem, ring, ..) = WEAPONS.get(kind as usize)?;
    let set = stem.to_ascii_lowercase();
    Some(if enchanted && *ring { format!("{set}.enchanted") } else { set })
}

/// What each of the runtime's events plays with no set of its own (`rustvr_audio::Event::name`:
/// the record and a gain on top of its own). Oblivion has no guns or grenades: theirs are the
/// nearest of its own (a bow's draw for a bolt, a lock's tumblers for its clicks, a mine going off).
const DEFAULTS: &[(&str, &[(&str, f32)])] = &[
    ("grab", &[("ITMGenericUp", 1.0)]),
    // (Let go of, a thing is set down more softly than it's put away; it knocks into the world
    // with its own sound as it lands.)
    ("release", &[("ITMGenericDown", 0.6)]),
    ("holster_in", &[("ITMGenericDown", 1.0)]),
    ("holster_out", &[("ITMGenericUp", 1.0)]),
    ("fail", &[("ITMIngredientNothing", 1.0)]),
    ("zone_tick", &[("UIMenuFocus", 1.0)]),
    ("pouch", &[("ITMAmmoUp", 1.0)]),
    ("glove_hover", &[("UIMenuFocus", 1.0)]),
    // (Locking onto a thing is felt only: no `glove_lock`, so the pack plays nothing for it.)
    ("glove_flick", &[("WPNSwishLarge", 0.175)]),
    ("glove_travel", &[("SPLMysticismTravelLP", 1.0)]),
    ("glove_catch", &[("ITMGenericUp", 1.0)]),
    ("impact", &[("CWoodSmall", 1.0)]),
    ("ui_hover", &[("UIMenuFocus", 1.0)]),
    ("ui_click", &[("UIMenuOK", 1.0)]),
    ("melee", &[("WPNHitHand", 1.0)]),
    // A bite of what's held at the mouth, a gulp of a drink: the game's own chewing and swallowing
    // (its `use` sounds of food and potions, the item-sound choice's).
    ("bite", &[("NPCHumanChew", 1.0)]),
    ("gulp", &[("NPCHumanSwallow", 1.0)]),
    ("knife_throw", &[("WPNSwishSmall", 1.0)]),
    ("knife_stick", &[("WPNHitArrow", 1.0)]),
    ("mag_out", &[("ITMAmmoUp", 1.0)]),
    ("mag_in", &[("ITMAmmoDown", 1.0)]),
    ("bolt_back", &[("WPNBowDraw", 1.0)]),
    ("bolt_forward", &[("UILockTumblerLock", 1.0)]),
    ("last_round", &[("UILockClickNow", 1.0)]),
    ("dry_fire", &[("UILockPickAttempt", 1.0)]),
    ("fire", &[("bowShoot", 1.0)]),
    ("grenade_pin", &[("UILockTumblerLock", 1.0)]),
    ("grenade_arm", &[("ITMTorchHeldEquip", 1.0)]),
    ("grenade_tick", &[("UILockTumblerNudge", 1.0)]),
    ("grenade_throw", &[("WPNSwishMedium", 1.0)]),
    ("explosion", &[("TRPMineExplode", 1.0)]),
];

/// The pack, as `sounds.toml` and the files it names (pack-relative path -> the archive's path).
pub struct Pack {
    pub toml: String,
    pub files: BTreeMap<String, String>,
    /// Records the pack wanted that the game hasn't got.
    pub missing: Vec<String>,
}

/// Builds the pack from the game's records (`records`, [`read_records`]) and its files (`names`:
/// every file the archives have).
pub fn build(records: &HashMap<String, SoundRecord>, names: &[&str]) -> Pack {
    let mut p = PackWriter { records, names, files: BTreeMap::new(), missing: Vec::new() };
    let mut t = String::new();
    t.push_str("# Oblivion's own sounds for the runtime, from its sound records (oblivion-extract sounds).\n");
    t.push_str("format = 1\nname = \"The Elder Scrolls IV: Oblivion\"\nsource = \"Oblivion.esm and Oblivion - Sounds.bsa\"\n\n[defaults]\n");
    for (event, layers) in DEFAULTS {
        p.event(&mut t, event, layers);
    }
    for (stem, collision) in KINDS {
        let kind = kind_of(&format!("{stem}Up")).unwrap();
        let (up, down) = (format!("{stem}Up"), format!("{stem}Down"));
        let _ = write!(t, "\n[sets.{kind}]\n");
        p.event(&mut t, "grab", &[(up.as_str(), 1.0)]);
        p.event(&mut t, "release", &[(down.as_str(), 0.6)]);
        p.event(&mut t, "holster_out", &[(up.as_str(), 1.0)]);
        p.event(&mut t, "holster_in", &[(down.as_str(), 1.0)]);
        p.event(&mut t, "glove_catch", &[(up.as_str(), 1.0)]);
        p.event(&mut t, "impact", &[(*collision, 1.0)]);
        for c in COLLISIONS.iter().filter(|c| records.contains_key(&c.to_ascii_lowercase())) {
            let _ = write!(t, "\n[sets.\"{}\"]\nextends = \"{kind}\"\n", with_collision(&kind, c));
            p.event(&mut t, "impact", &[(*c, 1.0)]);
        }
    }
    for (k, (stem, ring, hit, kind)) in WEAPONS.iter().enumerate() {
        let set = weapon_set(k as u32, false).unwrap();
        let (equip, unequip) = (format!("{stem}Equip"), format!("{stem}Unequip"));
        let _ = write!(t, "\n[sets.{set}]\nextends = \"{kind}\"\n");
        p.event(&mut t, "holster_out", &[(equip.as_str(), 1.0)]);
        p.event(&mut t, "holster_in", &[(unequip.as_str(), 1.0)]);
        p.event(&mut t, "melee", &[(*hit, 1.0)]);
        if *ring {
            let _ = write!(t, "\n[sets.\"{}\"]\nextends = \"{set}\"\n", weapon_set(k as u32, true).unwrap());
            p.event(&mut t, "holster_out", &[(equip.as_str(), 1.0), (format!("{stem}EquipEnchanted").as_str(), 1.0)]);
        }
    }
    p.missing.sort();
    p.missing.dedup();
    Pack { toml: t, files: p.files, missing: p.missing }
}

struct PackWriter<'a> {
    records: &'a HashMap<String, SoundRecord>,
    names: &'a [&'a str],
    files: BTreeMap<String, String>,
    missing: Vec<String>,
}

impl PackWriter<'_> {
    /// `event = [{ files = [...], gain = .., spatial = .. }, ...]`: a layer per record (a file, or
    /// every file of its folder), those the game hasn't got left out.
    fn event(&mut self, t: &mut String, event: &str, layers: &[(&str, f32)]) {
        let mut out = Vec::new();
        for (name, gain) in layers {
            let Some(r) = self.records.get(&name.to_ascii_lowercase()) else {
                self.missing.push((*name).to_owned());
                continue;
            };
            let files = self.files_of(r);
            if files.is_empty() {
                self.missing.push(format!("{name} ({})", r.file));
                continue;
            }
            let list = files.iter().map(|f| format!("\"{f}\"")).collect::<Vec<_>>().join(", ");
            let spatial = if r.spatial() { String::new() } else { ", spatial = false".into() };
            out.push(format!("{{ files = [{list}], gain = {:.3}{spatial} }}", r.gain() * gain));
        }
        if !out.is_empty() {
            let _ = writeln!(t, "{event} = [{}]", out.join(", "));
        }
    }

    /// A record's files (pack-relative), noted to be copied.
    fn files_of(&mut self, r: &SoundRecord) -> Vec<String> {
        let path = normalise(&format!("sound\\{}", r.file));
        let found: Vec<String> = if path.ends_with('\\') {
            let mut v: Vec<String> = self.names.iter().filter(|n| n.strip_prefix(path.as_str()).is_some_and(|rest| !rest.contains('\\') && rest.ends_with(".wav"))).map(|n| (*n).to_owned()).collect();
            v.sort();
            v
        } else if self.names.contains(&path.as_str()) {
            vec![path]
        } else {
            Vec::new()
        };
        found
            .into_iter()
            .map(|n| {
                let rel = n.strip_prefix("sound\\").unwrap_or(&n).replace('\\', "/");
                self.files.insert(rel.clone(), n);
                rel
            })
            .collect()
    }
}

/// Writes the pack into `out` (`<local_assets>/oblivion/sounds`): `sounds.toml` and every file it
/// names, read from the game's archives.
pub fn write(game: &Path, archives: &Archives, out: &Path) -> Result<Pack> {
    let esm = std::fs::read(game.join("Data").join("Oblivion.esm")).context("Oblivion.esm")?;
    let records = read_records(&esm)?;
    if records.is_empty() {
        bail!("Oblivion.esm has no sound records");
    }
    let names: Vec<&str> = archives.names().collect();
    let pack = build(&records, &names);
    for (rel, name) in &pack.files {
        let to = out.join(rel);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::write(&to, archives.read(name)?).with_context(|| to.display().to_string())?;
    }
    std::fs::write(out.join("sounds.toml"), &pack.toml)?;
    Ok(pack)
}

