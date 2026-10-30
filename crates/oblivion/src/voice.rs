//! Talking to Cyrodiil's people out loud, through the framework's voice plugin (`rustvr-voice`,
//! the user's m8; `rustvr_protocol::voice`). See `docs/research/voice.md`.
//!
//! * **Who's listening**: the person the head looks at most directly, within talking distance,
//!   when the player starts talking; looking at no one keeps whoever was listening while they're
//!   near. They turn to the player and look at them, as the game's own speakers do (the say-to
//!   look target and the game's own turn, as `SayTopic` has them), until a while after the last
//!   word. Activating them still opens the game's dialogue, untouched.
//! * **Who they are**, from the game: their name, race, sex, class, factions and ranks, their
//!   disposition towards the player, where they are, what they're doing, the time, and what the
//!   game's dialogue has them say (their greeting, rumours, idle chatter: the game's own pick of
//!   their lines, by its own conditions, for this speaker and the player).
//! * **Their voice**: those same lines' voice files. Lines only this person says (their own
//!   actor's) make a voice of their own; otherwise their race and sex's voice, as in the game.
//!   The files are copied out of the game's archives for the plugin to clone.
//! * **A line**: the plugin writes `<n>.wav` and the mouth's shapes; here they become the game's
//!   own lip file, and the person speaks the WAV with the game's own `Actor::Speak`, into their
//!   process's voice slot, with its talking state, its subtitle, their expression from how the
//!   line is said: the face moves with the game's lip sync and the voice comes from them in 3D.
//! * **Memory** goes with the game's saves: every save and load is told to the plugin.

/// Talking distance: the game starts dialogue by activation, at arm's reach, and has no distance
/// for talking across a room; this is a conversation's, a few steps (metres).
const TALKING_DISTANCE: f32 = 5.0;
/// Someone listening is let go past this (metres), and who's around is sent within it.
const AROUND: f32 = 8.0;
/// The widest angle off where the head looks that someone is being talked to (degrees).
const LOOK_CONE: f32 = 25.0;
/// Someone stays listening this long after the last word, theirs or the player's (seconds).
const ENGAGED: f64 = 12.0;

/// Oblivion's emotions, as its dialogue responses and `Actor::Speak` have them.
pub const EMOTIONS: [&str; 7] = ["neutral", "anger", "disgust", "fear", "sad", "happy", "surprise"];

/// The game's emotion for how a line is said (its own words for it: "angrily", "with a laugh"),
/// and how strongly (0 to 100). Neutral for anything else.
pub fn emotion(how: &str) -> (u32, u32) {
    let how = how.to_lowercase();
    let strong = ["very", "furious", "terrified", "sobbing", "shout", "yell", "scream", "roar"].iter().any(|w| how.contains(w));
    let kinds: [(u32, &[&str]); 6] = [
        (1, &["ang", "furious", "annoy", "irrit", "snap", "shout", "yell", "growl", "stern", "cold", "harsh"]),
        (2, &["disgust", "revolt", "sneer", "contempt", "scorn"]),
        (3, &["afraid", "fear", "scared", "nervous", "terrif", "worried", "anxious", "whisper"]),
        (4, &["sad", "sorrow", "mourn", "grief", "tired", "weary", "sigh", "sob", "regret"]),
        (5, &["laugh", "happy", "warm", "cheer", "smil", "amus", "glad", "friendly", "joy", "chuckl", "grin", "pleased"]),
        (6, &["surpris", "shock", "astonish", "startl", "amazed", "disbelief"]),
    ];
    for (kind, words) in kinds {
        if words.iter().any(|w| how.contains(w)) {
            return (kind, if strong { 80 } else { 50 });
        }
    }
    (0, 50)
}

/// Oblivion's lip file for the mouth's frames (FaceGen's 16 phonemes, 30 a second): the 12-byte
/// header (1, the decoded size and 16, 1), then the data run-length encoded (a 0 byte and a `u16`
/// count stand for that many zero bytes): the frames, the lead-in (none: frame 0 is the voice's
/// start) and each frame's 16 phonemes and 17 modifiers (the blinks, brows and looks, left to the
/// game's own face).
pub fn lip_file(frames: &[[f32; 16]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 + frames.len() * 132);
    data.extend((frames.len() as u32).to_le_bytes());
    data.extend(0i32.to_le_bytes());
    for frame in frames {
        for w in frame {
            data.extend(w.clamp(0.0, 1.0).to_le_bytes());
        }
        data.extend([0u8; 17 * 4]);
    }
    let mut out = Vec::with_capacity(12 + data.len() / 2);
    out.extend(1u32.to_le_bytes());
    out.extend((data.len() as u32 + 16).to_le_bytes());
    out.extend(1u32.to_le_bytes());
    let mut i = 0;
    while i < data.len() {
        if data[i] != 0 {
            out.push(data[i]);
            i += 1;
            continue;
        }
        let run = data[i..].iter().take(u16::MAX as usize).take_while(|&&b| b == 0).count();
        out.push(0);
        out.extend((run as u16).to_le_bytes());
        i += run;
    }
    out
}

/// How someone feels about the player, in words, from the game's disposition (0 to 100).
fn feeling(disposition: i32) -> &'static str {
    match disposition {
        ..20 => "hates",
        20..40 => "dislikes",
        40..60 => "has no strong feelings about",
        60..80 => "likes",
        _ => "is very fond of",
    }
}

/// What someone's doing, from the type of their AI package.
fn doing(package_type: u8) -> &'static str {
    match package_type {
        0 => "looking for something",
        1 => "following someone",
        2 => "escorting someone",
        3 => "having something to eat",
        4 => "getting ready to sleep",
        5 => "wandering about",
        6 => "on their way somewhere",
        7 => "keeping someone company",
        8 => "busy with something",
        9 => "lying in wait",
        10 => "getting away from something",
        11 => "casting a spell",
        _ => "going about their day",
    }
}

/// The time of day in words, from the game's hour.
fn time_of_day(hour: f32) -> &'static str {
    match hour as u32 {
        0..5 => "the dead of night",
        5..8 => "early morning",
        8..11 => "morning",
        11..14 => "around midday",
        14..18 => "afternoon",
        18..21 => "evening",
        _ => "night",
    }
}

fn article(word: &str) -> &'static str {
    if word.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']) { "an" } else { "a" }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::{combat, engine, mem};
    use glam::Vec3;
    use rustvr_adapter::View;
    use rustvr_protocol::*;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, mpsc};

    // ---- the game's own (Oblivion 1.2.0.416; docs/research/voice.md) ------------------------
    /// `Actor::Speak` (thiscall on the actor): the voice file, its sound's slot, the emotion and
    /// its value, the line's length in characters, set the emotion, flat (2D), load the lip in
    /// the background, use the lip file. Returns how long it talks (seconds; x87).
    const SPEAK: usize = 0x005F_7310;
    /// `iSpeakSoundLipDistance`'s value: the face moves only within it (`Say` and `SayTo` lift it
    /// while they speak; so does a line of ours).
    const LIP_DISTANCE: usize = 0x00B3_6798;
    /// The HUD's subtitle for a line (cdecl: the text, its sound, 0, -1), and the Gameplay
    /// setting that shows them (`bGeneralSubtitles`).
    const SUBTITLE: usize = 0x0057_ACC0;
    const SUBTITLES_ON: usize = 0x00B1_3208;
    /// The game's turn of an actor towards a heading, a step a frame (cdecl: the actor, the
    /// heading, force), and the angles within which a speaker only looks (degrees).
    const TURN_TO: usize = 0x0068_5530;
    const TALK_TURN_ANGLE: usize = 0x00B3_6C10;
    /// `TESTopic` -> the line it has `speaker` say to `target` (thiscall: speaker, target, 0, 0):
    /// a `DialogueItem` (its responses' list at +0, the info at +0xC), freed with 0x6B81D0 and
    /// the game's free. A response: its text at +0, its emotion +8 and value +0xC, its voice file
    /// +0x10.
    const DIALOGUE_ITEM: usize = 0x0052_F790;
    const ITEM_CLEAR: usize = 0x006B_81D0;
    const FREE: usize = 0x0040_1F20;
    /// `TESSaveLoadGame::SaveGame` and `LoadGame` (thiscall: 0, the save's name, 0).
    const SAVE_GAME: usize = 0x0046_5130;
    const LOAD_GAME: usize = 0x0046_5860;
    /// `LookupFormByID` (cdecl), `__RTDynamicCast`, and the type descriptors to find a form's
    /// name with (as `TESObjectREFR::GetName`, 0x4DA2A0).
    const LOOKUP_FORM: usize = 0x0046_B250;
    const DYNAMIC_CAST: usize = 0x0098_32E6;
    const RTTI_FORM: usize = 0x00B0_2F9C;
    const RTTI_FULL_NAME: usize = 0x00B0_3298;

    /// Where the game's vtables are (its read-only data): a pointer whose first word isn't in
    /// here isn't one of its objects.
    const GAME_VTABLES: std::ops::Range<usize> = 0x00A3_0000..0x00B4_0000;
    /// `Actor`: its process (`HighProcess` near the player), its say-to target.
    const PROCESS: usize = 0x58;
    const HIGH_PROCESS_VTABLE: usize = 0x00A7_1814;
    const VT_DISPOSITION: usize = 0x224;
    /// `HighProcess`: the line's time left, its voice's sound slot, talking; and its virtuals:
    /// talking on, the time, stop a sound (its slot), the say-to look target (set, clear), the
    /// current package, two that keep a speaker from turning (as `SayTopic` asks them).
    const TALK_TIME: usize = 0x21C;
    const VOICE_SLOT: usize = 0x220;
    const VT_SET_TALKING: usize = 0x204;
    const VT_SET_TALK_TIME: usize = 0x20C;
    const VT_STOP_SOUND: usize = 0x344;
    const VT_SAY_TO_LOOK: usize = 0x484;
    const VT_CLEAR_SAY_TO_LOOK: usize = 0x49C;
    const VT_PACKAGE: usize = 0x184;
    const VT_NO_TURN_A: usize = 0x36C;
    const VT_NO_TURN_B: usize = 0x4DC;
    const NO_TURN_FIELD: usize = 0x34;
    /// `TESPackage`'s type; Travel and UseItemAt don't turn to talk (`SayTopic`).
    const PACKAGE_TYPE: usize = 0x20;
    /// The vtables of a package from the game's data (`TESPackage`) and of the ones it makes as
    /// things happen: talking to someone, an alarm, fleeing, watching a fight, a trespasser, combat.
    const PACKAGE_VTABLE: usize = 0x00A6_743C;
    const DIALOGUE_PACKAGE_VTABLE: usize = 0x00A7_151C;
    const ALARM_PACKAGE_VTABLE: usize = 0x00A6_EF94;
    const FLEE_PACKAGE_VTABLE: usize = 0x00A7_1664;
    const SPECTATOR_PACKAGE_VTABLE: usize = 0x00A7_46AC;
    const TRESPASS_PACKAGE_VTABLE: usize = 0x00A7_47BC;
    const COMBAT_VTABLE: usize = 0x00A7_0A14;

    /// `TESNPC`: its flags (bit 0 female), its factions' list, its race, its class.
    const NPC_FLAGS: usize = 0x28;
    const NPC_FACTIONS: usize = 0x3C;
    const NPC_RACE: usize = 0xE8;
    const NPC_CLASS: usize = 0x104;
    /// `TESFaction`: its flags (bit 0 hidden from the player), its ranks' list (each a male and a
    /// female title, `BSStringT`s).
    const FACTION_FLAGS: usize = 0x34;
    const FACTION_RANKS: usize = 0x3C;
    /// `TESObjectREFR`'s cell; `TESObjectCELL`'s flags (bit 0 interior) and worldspace.
    const REF_CELL: usize = 0x40;
    const CELL_FLAGS: usize = 0x24;
    const CELL_WORLD: usize = 0x50;
    /// `TESTopicInfo`'s conditions (data, next): a condition's function at +8 (`GetIsID` 0x48)
    /// and its first parameter at +0xC.
    const INFO_CONDITIONS: usize = 0x18;
    const GET_IS_ID: u16 = 0x48;
    /// The game's own topics that every speaker has lines for (Oblivion.esm's form ids):
    /// GREETING, HELLO, Rumors (INFOGENERAL), Idle Chatter, GOODBYE.
    const GREETING: u32 = 0xC8;
    const HELLO: u32 = 0xD2;
    const RUMORS: u32 = 0xD7;
    const IDLE_CHATTER: u32 = 0xD5;
    const GOODBYE: u32 = 0xD4;
    /// The calendar's globals (their value at +0x24): GameYear, GameHour.
    const GAME_YEAR: u32 = 0x35;
    const GAME_HOUR: u32 = 0x38;

    static REAL_SAVE: AtomicUsize = AtomicUsize::new(0);
    static REAL_LOAD: AtomicUsize = AtomicUsize::new(0);
    type SaveLoadFn = unsafe extern "thiscall" fn(usize, u32, *const u8, u32) -> u8;

    /// A line of a person's, from the game's dialogue: what's said, its voice file, whether only
    /// they say it.
    struct Said {
        topic: u32,
        text: String,
        file: String,
        own: bool,
    }

    /// A sentence of the plugin's, waiting to be said after the one before.
    struct Pending {
        speaker: u32,
        line: u32,
        how: String,
        text: String,
    }

    #[derive(Default)]
    struct Voice {
        read: u64,
        connected: bool,
        /// Who's listening, and until when (after the last word).
        listener: Option<(u32, f64)>,
        /// What the game's dialogue has each person say, and their voice's key (once each).
        known: HashMap<u32, (Vec<Said>, String)>,
        queue: VecDeque<Pending>,
        /// The one talking now, until when.
        talking: Option<(u32, f64)>,
        scene_at: f64,
        /// The voices whose clips have gone to the plugin's folder.
        voiced: HashSet<String>,
        clips: Option<mpsc::Sender<(PathBuf, Vec<(String, String)>)>>,
        events: Vec<VoiceEvent>,
    }

    static VOICE: Mutex<Option<Voice>> = Mutex::new(None);

    pub fn install() {
        crate::hooks::install_into(SAVE_GAME as *const (), save_game as *const (), "TESSaveLoadGame::SaveGame", &REAL_SAVE);
        crate::hooks::install_into(LOAD_GAME as *const (), load_game as *const (), "TESSaveLoadGame::LoadGame", &REAL_LOAD);
    }

    /// A save's name as the characters' memory keeps it (the file's name, no folder or `.ess`).
    fn save_name(name: *const u8) -> String {
        let name = combat::read_str(name as usize);
        let name = name.rsplit(['\\', '/']).next().unwrap_or(&name);
        name.strip_suffix(".ess").unwrap_or(name).to_string()
    }

    unsafe extern "thiscall" fn save_game(this: usize, a: u32, name: *const u8, b: u32) -> u8 {
        let real: SaveLoadFn = unsafe { std::mem::transmute(REAL_SAVE.load(Ordering::Acquire)) };
        let ok = unsafe { real(this, a, name, b) };
        crate::hooks::guard_void("voice save", || {
            if ok != 0 {
                tell(VoiceEvent::new(VE_SAVED, 0, 0, &save_name(name)));
            }
        });
        ok
    }

    unsafe extern "thiscall" fn load_game(this: usize, a: u32, name: *const u8, b: u32) -> u8 {
        let saved = crate::hooks::guard("voice load", String::new(), || save_name(name));
        let real: SaveLoadFn = unsafe { std::mem::transmute(REAL_LOAD.load(Ordering::Acquire)) };
        let ok = unsafe { real(this, a, name, b) };
        crate::hooks::guard_void("voice load", || {
            if ok != 0 {
                tell(VoiceEvent::new(VE_LOADED, 0, 0, &saved));
                if let Some(v) = VOICE.lock().unwrap().as_mut() {
                    v.listener = None;
                    v.talking = None;
                    v.queue.clear();
                    v.known.clear();
                }
            }
        });
        ok
    }

    /// An event for the plugin, sent on the next frame it runs in (saves and loads happen outside
    /// one); kept a ring's worth while it isn't running.
    fn tell(event: VoiceEvent) {
        let mut guard = VOICE.lock().unwrap();
        let events = &mut guard.get_or_insert_with(Voice::default).events;
        if events.len() >= VOICE_EVENT_SLOTS {
            events.remove(0);
        }
        events.push(event);
    }

    // ---- small helpers on the game's memory -------------------------------------------------

    fn vcall0(obj: usize, slot: usize) -> usize {
        let f = mem::deref(mem::deref(obj) + slot);
        if !engine::CODE.contains(&f) {
            return 0;
        }
        type F = unsafe extern "thiscall" fn(usize) -> usize;
        unsafe { std::mem::transmute::<usize, F>(f)(obj) }
    }

    fn vcall1(obj: usize, slot: usize, a: usize) -> usize {
        let f = mem::deref(mem::deref(obj) + slot);
        if !engine::CODE.contains(&f) {
            return 0;
        }
        type F = unsafe extern "thiscall" fn(usize, usize) -> usize;
        unsafe { std::mem::transmute::<usize, F>(f)(obj, a) }
    }

    /// A `BSStringT`'s text (its characters, and their count at +4), whole: a line of dialogue can
    /// run to hundreds.
    fn read_text(string: usize) -> String {
        let chars = mem::deref(string);
        let count = mem::at::<u16>(string + 4).unwrap_or(0) as usize;
        if chars == 0 {
            return String::new();
        }
        let bytes = mem::bytes(chars as *const u8, count.clamp(1, 2048)).unwrap_or_default();
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }

    /// A form's name (its `TESFullName`), or "" (also for a name with no letters in it: the
    /// placeholder of a class made up in the race menu).
    fn full_name(form: usize) -> String {
        // (only one of the game's own objects: the cast reads its vtable's type information)
        if form == 0 || !GAME_VTABLES.contains(&mem::deref(form)) {
            return String::new();
        }
        type Cast = unsafe extern "C" fn(usize, i32, usize, usize, i32) -> usize;
        let named = unsafe { std::mem::transmute::<usize, Cast>(DYNAMIC_CAST)(form, 0, RTTI_FORM, RTTI_FULL_NAME, 0) };
        let name = if named == 0 { String::new() } else { read_text(named + 4).trim().to_string() };
        if name.chars().any(char::is_alphanumeric) { name } else { String::new() }
    }

    fn lookup(id: u32) -> usize {
        type F = unsafe extern "C" fn(u32) -> usize;
        unsafe { std::mem::transmute::<usize, F>(LOOKUP_FORM)(id) }
    }

    fn global(id: u32) -> Option<f32> {
        let g = lookup(id);
        (g != 0).then(|| mem::at::<f32>(g + 0x24)).flatten()
    }

    fn base(actor: usize) -> usize {
        mem::deref(actor + 0x1C)
    }

    fn high_process(actor: usize) -> usize {
        let p = mem::deref(actor + PROCESS);
        if p != 0 && mem::deref(p) == HIGH_PROCESS_VTABLE { p } else { 0 }
    }

    fn female(actor: usize) -> bool {
        mem::at::<u32>(base(actor) + NPC_FLAGS).is_some_and(|f| f & 1 != 0)
    }

    fn id_of(actor: usize) -> u32 {
        mem::at::<u32>(actor + 0x0C).unwrap_or(0)
    }

    /// Where an actor's head is (the protocol's space, metres), or their feet and a head's height.
    fn head(actor: usize) -> Option<Vec3> {
        let root = mem::deref(actor + 0x3C);
        if root != 0 {
            if let Some(h) = crate::nodes::find(root, "Bip01 Head").and_then(combat::node_pos) {
                return Some(h);
            }
        }
        let feet = mem::at::<[f32; 3]>(actor + engine::refr::POS)?;
        Some(crate::camera::basis().point_from_game(Vec3::from(feet)) + Vec3::Y * 1.6)
    }

    /// People near the player who can talk (the game's NPCs, not creatures; alive), with where
    /// each one's head is this frame.
    fn people() -> Vec<(usize, Vec3)> {
        combat::high_actors()
            .into_iter()
            .filter(|&a| mem::deref(a) == 0x00A6_FC9C && combat::actor_value(a, 8) > 0)
            .filter_map(|a| Some((a, head(a)?)))
            .collect()
    }

    // ---- who they are --------------------------------------------------------------------

    /// The lines the game's dialogue has `speaker` say to the player for `topic` (its own pick,
    /// by its conditions; a random topic picks again each time), each response's text and file.
    fn lines(topic_id: u32, speaker: usize, player: usize) -> Vec<Said> {
        let topic = lookup(topic_id);
        if topic == 0 {
            return Vec::new();
        }
        type Make = unsafe extern "thiscall" fn(usize, usize, usize, u32, u32) -> usize;
        type Clear = unsafe extern "thiscall" fn(usize);
        type Free = unsafe extern "C" fn(usize);
        let item = unsafe { std::mem::transmute::<usize, Make>(DIALOGUE_ITEM)(topic, speaker, player, 0, 0) };
        if item == 0 {
            return Vec::new();
        }
        // (a line only this person says: a condition of its info that they're who they are)
        let npc = base(speaker);
        let info = mem::deref(item + 0x0C);
        let mut own = false;
        let mut node = info + INFO_CONDITIONS;
        for _ in 0..32 {
            let data = mem::deref(node);
            if data != 0 && mem::at::<u16>(data + 8).map(|f| f & 0x0FFF) == Some(GET_IS_ID) && mem::deref(data + 0x0C) == npc {
                own = true;
            }
            node = mem::deref(node + 4);
            if node == 0 {
                break;
            }
        }
        let mut out = Vec::new();
        let mut node = item;
        for _ in 0..16 {
            let response = mem::deref(node);
            if response == 0 {
                break;
            }
            let text = read_text(response);
            let file = read_text(response + 0x10);
            if !text.trim().is_empty() {
                out.push(Said { topic: topic_id, text: text.trim().to_string(), file, own });
            }
            node = mem::deref(node + 4);
            if node == 0 {
                break;
            }
        }
        unsafe {
            std::mem::transmute::<usize, Clear>(ITEM_CLEAR)(item);
            std::mem::transmute::<usize, Free>(FREE)(item);
        }
        out
    }

    /// Everything the game's dialogue has them say, and their voice's key: their own lines' (a
    /// voice of their own) or their race and sex's, where the game files those lines.
    fn dialogue(speaker: usize, player: usize) -> (Vec<Said>, String) {
        let mut said: Vec<Said> = Vec::new();
        for topic in [GREETING, HELLO, RUMORS, RUMORS, RUMORS, IDLE_CHATTER, GOODBYE] {
            for line in lines(topic, speaker, player) {
                if !said.iter().any(|s| s.text == line.text) {
                    said.push(line);
                }
            }
        }
        let own = said.iter().any(|s| s.own && !s.file.is_empty());
        let key = if own {
            format!("npc\\{:08X}", id_of(base(speaker)))
        } else {
            // Data\Sound\Voice\<plugin>\<voice race>\<sex>\<file>: the voice is the folder.
            said.iter()
                .filter(|s| !s.file.is_empty())
                .find_map(|s| {
                    let lower = s.file.to_lowercase().replace('/', "\\");
                    let folder = lower.split_once("sound\\voice\\")?.1.rsplit_once('\\')?.0.to_string();
                    Some(folder)
                })
                .unwrap_or_else(|| format!("npc\\{:08X}", id_of(base(speaker))))
        };
        (said, key)
    }

    /// Who they are, in words, from the game's data.
    fn persona(actor: usize, player: usize, said: &[Said]) -> String {
        let npc = base(actor);
        let name = full_name(npc);
        let race = full_name(mem::deref(npc + NPC_RACE));
        let sex = if female(actor) { "woman" } else { "man" };
        let class = full_name(mem::deref(npc + NPC_CLASS));
        let player_name = player_name(player);
        let mut out = format!("{name} is {} {race} {sex}", article(&race));
        if !class.is_empty() {
            out += &format!(", {} {class}", article(&class));
        }
        out += ".";
        let factions = factions(actor);
        if !factions.is_empty() {
            out += &format!(" {name} belongs to {}.", factions.join(", "));
        }
        let disposition = vcall1(actor, VT_DISPOSITION, player) as i32;
        out += &format!(" {name} {} {player_name} ({disposition} out of 100).", feeling(disposition));
        let (p_race, p_class) = (full_name(mem::deref(base(player) + NPC_RACE)), full_name(mem::deref(base(player) + NPC_CLASS)));
        let p_sex = if female(player) { "woman" } else { "man" };
        out += &format!(" {player_name} is {} {p_race} {p_sex}{}.", article(&p_race), if p_class.is_empty() { String::new() } else { format!(", {} {p_class}", article(&p_class)) });
        let say = |topic: u32| said.iter().filter(move |s| s.topic == topic).map(|s| format!("\"{}\"", s.text));
        let greeting: Vec<String> = say(GREETING).chain(say(HELLO)).take(2).collect();
        if !greeting.is_empty() {
            out += &format!("\n\nHow {name} greets {player_name}: {}", greeting.join(" "));
        }
        let rumours: Vec<String> = say(RUMORS).chain(say(IDLE_CHATTER)).take(5).collect();
        if !rumours.is_empty() {
            out += &format!("\n\nWhat {name} has heard and might talk about: {}", rumours.join(" "));
        }
        out += &format!("\n\nThese are {name}'s own words in their own way of speaking: talk like that.");
        out
    }

    fn player_name(player: usize) -> String {
        let name = full_name(base(player));
        if name.is_empty() { "the player".to_string() } else { name }
    }

    /// Their factions (the ones the player can know of), with their rank's title.
    fn factions(actor: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut node = base(actor) + NPC_FACTIONS;
        for _ in 0..32 {
            let data = mem::deref(node);
            if data != 0 {
                let faction = mem::deref(data);
                let rank = mem::at::<i8>(data + 4).unwrap_or(-1);
                let hidden = mem::at::<u8>(faction + FACTION_FLAGS).is_none_or(|f| f & 1 != 0);
                let name = full_name(faction);
                if faction != 0 && rank >= 0 && !hidden && !name.is_empty() {
                    let title = rank_title(faction, rank as usize, female(actor));
                    out.push(if title.is_empty() { format!("the {name}") } else { format!("the {name} (as {} {title})", article(&title)) });
                }
            }
            node = mem::deref(node + 4);
            if node == 0 {
                break;
            }
        }
        out
    }

    fn rank_title(faction: usize, rank: usize, female: bool) -> String {
        let mut node = faction + FACTION_RANKS;
        for _ in 0..rank {
            node = mem::deref(node + 4);
            if node == 0 {
                return String::new();
            }
        }
        let data = mem::deref(node);
        if data == 0 {
            return String::new();
        }
        let female_title = combat::read_str(mem::deref(data + 8));
        if female && !female_title.is_empty() { female_title } else { combat::read_str(mem::deref(data)) }
    }

    /// What holds for them now, in words: where, doing what, when, who else is about.
    fn situation(actor: usize, others: &[String]) -> String {
        let name = full_name(base(actor));
        let cell = mem::deref(actor + REF_CELL);
        let mut place = full_name(cell);
        let interior = mem::at::<u8>(cell + CELL_FLAGS).is_some_and(|f| f & 1 != 0);
        if !interior {
            let world = full_name(mem::deref(cell + CELL_WORLD));
            place = match (place.is_empty(), world.is_empty()) {
                (true, false) => world,
                (false, false) if !place.contains(&world) => format!("{place}, {world}"),
                _ => place,
            };
        }
        let mut out = String::new();
        if !place.is_empty() {
            out += &format!("{name} is {} {place}. ", if interior { "inside" } else { "in" });
        }
        let process = high_process(actor);
        let package = if process != 0 { vcall0(process, VT_PACKAGE) } else { 0 };
        // (the game's own packages say what kind they are; the ones it makes up as things happen
        // are kinds of their own, their type no guide)
        let activity = match mem::deref(package) {
            0 => None,
            PACKAGE_VTABLE => Some(doing(mem::at::<u8>(package + PACKAGE_TYPE).unwrap_or(255))),
            DIALOGUE_PACKAGE_VTABLE => Some("about to talk to someone"),
            ALARM_PACKAGE_VTABLE => Some("raising the alarm"),
            FLEE_PACKAGE_VTABLE => Some("running from danger"),
            SPECTATOR_PACKAGE_VTABLE => Some("watching a fight"),
            TRESPASS_PACKAGE_VTABLE => Some("seeing off a trespasser"),
            COMBAT_VTABLE => Some("fighting"),
            _ => None,
        };
        if let Some(activity) = activity {
            out += &format!("{name} is {activity}. ");
        }
        if let Some(hour) = global(GAME_HOUR) {
            out += &format!("It's {}", time_of_day(hour));
            if let Some(year) = global(GAME_YEAR) {
                out += &format!(", in the year 3E {}", year as u32);
            }
            out += ". ";
        }
        if !others.is_empty() {
            out += &format!("Also nearby: {}.", others.join(", "));
        }
        out.trim().to_string()
    }

    // ---- voices ---------------------------------------------------------------------------

    /// The plugin's folders: where it writes lines (under the game's own voice folder, where the
    /// game plays them from) and where each voice's clips go.
    fn folders() -> (PathBuf, PathBuf) {
        let game = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
        let voices = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("RustVR").join("voice").join("oblivion").join("voices");
        (game.join("Data").join("Sound").join("Voice").join("RustVR"), voices)
    }

    /// Copies a voice's lines out of the game's archives (or loose, as the game reads them) into
    /// its folder, on a thread of its own.
    fn clip_writer() -> mpsc::Sender<(PathBuf, Vec<(String, String)>)> {
        let (tx, rx) = mpsc::channel::<(PathBuf, Vec<(String, String)>)>();
        std::thread::spawn(move || {
            let game = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
            let data = game.join("Data");
            let archives: Vec<oblivion_assets::bsa::Bsa> = std::fs::read_dir(&data)
                .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.to_string_lossy().to_lowercase().contains("voices") && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("bsa"))).collect::<Vec<_>>())
                .unwrap_or_default()
                .iter()
                .filter_map(|p| oblivion_assets::bsa::Bsa::open(p).map_err(|e| tracing::warn!("voice: {e:#}")).ok())
                .collect();
            for (dir, lines) in rx {
                // (written beside it, then put in place whole: the plugin may be reading the folder)
                let part = dir.with_extension("part");
                let _ = std::fs::remove_dir_all(&part);
                let _ = std::fs::create_dir_all(&part);
                let mut k = 0;
                for (file, text) in lines {
                    let inside = file.to_lowercase().replace('/', "\\");
                    let inside = inside.strip_prefix("data\\").unwrap_or(&inside).to_string();
                    let bytes = std::fs::read(data.join(&inside)).ok().or_else(|| archives.iter().find(|a| a.contains(&inside)).and_then(|a| a.read(&inside).ok()));
                    let Some(bytes) = bytes else { continue };
                    k += 1;
                    let _ = std::fs::write(part.join(format!("{k}.mp3")), bytes);
                    let _ = std::fs::write(part.join(format!("{k}.txt")), text);
                }
                let _ = std::fs::remove_dir_all(&dir);
                match std::fs::rename(&part, &dir) {
                    Ok(()) => tracing::info!("voice: {} of the game's lines for {}", k, dir.display()),
                    Err(e) => tracing::warn!("voice: {}: {e}", dir.display()),
                }
            }
        });
        tx
    }

    // ---- saying a line --------------------------------------------------------------------

    /// `<n>.lips` (the plugin's) as the game's `<n>.lip`; false if there's none.
    fn write_lip(lines_dir: &Path, line: u32) -> bool {
        let Some((_, frames)) = std::fs::read(lines_dir.join(format!("{line}.lips"))).ok().and_then(|b| read_lips(&b)) else { return false };
        std::fs::write(lines_dir.join(format!("{line}.lip")), lip_file(&frames)).is_ok()
    }

    /// The actor speaks line `n`'s WAV as the game's speakers do (`SayTopic`): its sound in their
    /// voice slot, their lip file, expression, talking state and time, the subtitle, looking at
    /// the player. How long it takes (seconds).
    fn speak(actor: usize, player: usize, line: u32, how: &str, text: &str) -> f64 {
        let process = high_process(actor);
        if process == 0 {
            return 0.0;
        }
        let path = format!("Data\\Sound\\Voice\\RustVR\\{line}.wav");
        let Ok(c_path) = std::ffi::CString::new(path) else { return 0.0 };
        let (kind, value) = emotion(how);
        type Speak = unsafe extern "thiscall" fn(usize, *const u8, usize, u32, u32, u32, u32, u32, u32, u32) -> f64;
        type Subtitle = unsafe extern "C" fn(*const u8, usize, u32, f32) -> u8;
        type SetF = unsafe extern "thiscall" fn(usize, f32);
        vcall1(process, VT_STOP_SOUND, 0);
        let lip_distance = mem::at::<i32>(LIP_DISTANCE);
        unsafe { *(LIP_DISTANCE as *mut i32) = i32::MAX };
        let seconds = unsafe {
            std::mem::transmute::<usize, Speak>(SPEAK)(actor, c_path.as_ptr() as *const u8, process + VOICE_SLOT, kind, value, text.len() as u32, 1, 0, 0, 1)
        };
        if let Some(d) = lip_distance {
            unsafe { *(LIP_DISTANCE as *mut i32) = d };
        }
        vcall1(process, VT_SET_TALKING, 1);
        let set_time = mem::deref(mem::deref(process) + VT_SET_TALK_TIME);
        if engine::CODE.contains(&set_time) {
            unsafe { std::mem::transmute::<usize, SetF>(set_time)(process, seconds as f32) };
        }
        vcall1(process, VT_SAY_TO_LOOK, player);
        if mem::at::<u8>(SUBTITLES_ON).is_some_and(|on| on != 0)
            && !text.trim().is_empty()
            && let Ok(c_text) = std::ffi::CString::new(text)
        {
            unsafe { std::mem::transmute::<usize, Subtitle>(SUBTITLE)(c_text.as_ptr() as *const u8, mem::deref(process + VOICE_SLOT), 0, -1.0) };
        }
        tracing::info!("voice: {} says line {line} ({}, {kind}/{value}, {seconds:.1} s): \"{text}\"", full_name(base(actor)), EMOTIONS[kind as usize]);
        seconds
    }

    /// Stops what they're saying (the player talked over them).
    fn hush(actor: usize) {
        let process = high_process(actor);
        if process != 0 {
            vcall1(process, VT_STOP_SOUND, 0);
            vcall1(process, VT_SET_TALKING, 0);
            if let Some(t) = mem::at::<f32>(process + TALK_TIME)
                && t > 0.0
            {
                unsafe { *((process + TALK_TIME) as *mut f32) = 0.0 };
            }
        }
    }

    /// A listener turns to the player and looks at them, as the game's speakers turn to whom
    /// they talk to (`SayTopic`): not while on their way somewhere or using something, and only
    /// past the game's talking turn angle.
    fn face(actor: usize, player: usize) {
        let process = high_process(actor);
        if process == 0 {
            return;
        }
        vcall1(process, VT_SAY_TO_LOOK, player);
        let package = vcall0(process, VT_PACKAGE);
        let busy = package != 0 && matches!(mem::at::<u8>(package + PACKAGE_TYPE), Some(6 | 8));
        if busy || mem::deref(process + NO_TURN_FIELD) != 0 || vcall0(process, VT_NO_TURN_A) & 0xFF != 0 || vcall0(process, VT_NO_TURN_B) & 0xFF != 0 {
            return;
        }
        let (Some(me), Some(them), Some(rot)) = (mem::at::<[f32; 3]>(player + engine::refr::POS), mem::at::<[f32; 3]>(actor + engine::refr::POS), mem::at::<[f32; 3]>(actor + engine::refr::ROT)) else { return };
        // Headings: 0 north (+y), turning east (+x).
        let heading = (me[0] - them[0]).atan2(me[1] - them[1]).rem_euclid(std::f32::consts::TAU);
        let off = (heading - rot[2] + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        let within = mem::at::<i32>(TALK_TURN_ANGLE).unwrap_or(30) as f32;
        if off.abs().to_degrees() > within {
            type Turn = unsafe extern "C" fn(usize, f32, u32);
            unsafe { std::mem::transmute::<usize, Turn>(TURN_TO)(actor, heading, 1) };
        }
    }

    /// Lets a listener go back to what they were doing.
    fn release(actor: usize) {
        let process = high_process(actor);
        if process != 0 {
            vcall0(process, VT_CLEAR_SAY_TO_LOOK);
        }
    }

    // ---- each frame -----------------------------------------------------------------------

    /// The one the head looks at most directly within talking distance (their id and actor).
    fn looked_at(view: &View, people: &[(usize, Vec3)]) -> Option<(u32, usize)> {
        let (eye, look) = (view.head_pos, view.look_dir.normalize_or_zero());
        people
            .iter()
            .filter_map(|&(a, head)| {
                let to = head - eye;
                let d = to.length();
                let angle = look.angle_between(to / d.max(1e-3)).to_degrees();
                (d <= TALKING_DISTANCE && angle <= LOOK_CONE).then_some((angle + d * 2.0, id_of(a), a))
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .map(|(_, id, a)| (id, a))
    }

    /// The voice's part of a game frame (the main thread, after the link's): who's listening, the
    /// plugin's lines said, the scene for the plugin.
    pub fn frame(view: &View, now: f64) {
        crate::hooks::guard_void("voice", || run(view, now));
    }

    fn run(view: &View, now: f64) {
        let alive = crate::link::with_adapter(|a| a.voice_alive()).unwrap_or(false);
        let mut guard = VOICE.lock().unwrap();
        let v = guard.get_or_insert_with(Voice::default);
        if !alive {
            if v.connected {
                tracing::info!("voice: the plugin is gone");
                if let Some(actor) = v.listener.and_then(|(id, _)| find(id)) {
                    release(actor);
                }
                v.connected = false;
                v.listener = None;
                v.queue.clear();
            }
            return;
        }
        if !v.connected {
            v.connected = true;
            v.read = crate::link::with_adapter(|a| a.voice_lines_written()).unwrap_or(0);
            tracing::info!("voice: the plugin is running: talk to whoever you look at");
        }
        let events: Vec<VoiceEvent> = std::mem::take(&mut v.events);
        for e in &events {
            crate::link::with_adapter(|a| a.push_voice_event(e));
        }
        let player = engine::player();
        let (lines_dir, voices_dir) = folders();
        let people = people();
        let near = |id: u32, within: f32| people.iter().any(|&(a, h)| id_of(a) == id && h.distance(view.head_pos) <= within);
        let lines = crate::link::with_adapter(|a| a.read_voice_lines(&mut v.read)).unwrap_or_default();
        for line in lines {
            match line.kind {
                VL_LISTEN => {
                    // Whoever's looked at now is being talked to; looking at no one keeps whoever
                    // was listening, while they're near.
                    let looked = looked_at(view, &people);
                    let kept = v.listener.filter(|&(id, _)| near(id, AROUND));
                    let next = looked.map(|(id, _)| id).or(kept.map(|(id, _)| id));
                    if let Some((old, _)) = v.listener
                        && Some(old) != next
                        && let Some(actor) = find(old)
                    {
                        release(actor);
                    }
                    v.listener = next.map(|id| (id, now + ENGAGED));
                    if let Some(id) = next {
                        tracing::info!("voice: talking to {}", find(id).map(|a| full_name(base(a))).unwrap_or_default());
                        v.scene_at = 0.0;
                    }
                }
                VL_HEARD => {
                    tracing::info!("voice: heard \"{}\"", line.text());
                    if let Some((id, _)) = v.listener {
                        v.listener = Some((id, now + ENGAGED));
                    }
                }
                VL_SAY => v.queue.push_back(Pending { speaker: line.speaker, line: line.line, how: line.how().to_string(), text: line.text().to_string() }),
                VL_STOP => {
                    v.queue.clear();
                    if let Some(actor) = v.talking.take().and_then(|(id, _)| find(id)) {
                        hush(actor);
                    }
                }
                _ => {}
            }
        }

        // The plugin's sentences, one after another as each ends.
        if v.talking.is_none_or(|(_, until)| now >= until - 0.05)
            && let Some(next) = v.queue.pop_front()
        {
            v.talking = None;
            if let Some(actor) = find(next.speaker) {
                if !write_lip(&lines_dir, next.line) {
                    tracing::warn!("voice: no mouth shapes for line {}: the game's own flapping mouth", next.line);
                }
                let seconds = speak(actor, player, next.line, &next.how, &next.text);
                v.talking = Some((next.speaker, now + seconds.max(0.2)));
                v.listener = Some((next.speaker, now + seconds + ENGAGED));
            }
        }

        // The listener faces the player while engaged, and is let go after.
        if let Some((id, until)) = v.listener {
            match find(id) {
                Some(actor) if now < until && near(id, AROUND) => face(actor, player),
                Some(actor) => {
                    release(actor);
                    v.listener = None;
                    v.scene_at = 0.0;
                }
                None => v.listener = None,
            }
        }

        // What the plugin knows of the scene: every half second, or when the listener changes.
        if now - v.scene_at >= 0.5 {
            v.scene_at = now;
            if v.clips.is_none() {
                v.clips = Some(clip_writer());
            }
            let around: Vec<usize> = people.iter().filter(|&&(_, h)| h.distance(view.head_pos) <= AROUND).map(|&(a, _)| a).collect();
            let mut scene: Box<VoiceScene> = bytemuck::zeroed_box();
            scene.flags = VS_READY | if crate::menus::menu_mode() { VS_DEAF } else { 0 };
            scene.listener = v.listener.map_or(0, |(id, _)| id);
            set_name(&mut scene.game, "The Elder Scrolls IV: Oblivion");
            set_name(&mut scene.player, &player_name(player));
            set_name(&mut scene.lines_dir, &lines_dir.to_string_lossy());
            set_name(&mut scene.voices_dir, &voices_dir.to_string_lossy());
            for &actor in &around {
                let id = id_of(actor);
                if !v.known.contains_key(&id) {
                    let known = dialogue(actor, player);
                    v.known.insert(id, known);
                }
                let (said, key) = &v.known[&id];
                if v.voiced.insert(key.clone()) && !voices_dir.join(key).join("reference.wav").exists() {
                    let clips: Vec<(String, String)> = said.iter().filter(|s| !s.file.is_empty() && (s.own || !key.starts_with("npc\\"))).map(|s| (s.file.clone(), s.text.clone())).collect();
                    if let Some(tx) = &v.clips {
                        let _ = tx.send((voices_dir.join(key), clips));
                    }
                }
                scene.add(id, &full_name(base(actor)), key);
            }
            if let Some(actor) = v.listener.and_then(|(id, _)| find(id)) {
                let id = id_of(actor);
                if !v.known.contains_key(&id) {
                    let known = dialogue(actor, player);
                    v.known.insert(id, known);
                }
                if scene.speaker(id).is_none() {
                    scene.add(id, &full_name(base(actor)), &v.known[&id].1);
                }
                set_name(&mut scene.persona, &persona(actor, player, &v.known[&id].0));
                let others: Vec<String> = around.iter().filter(|&&a| a != actor).map(|&a| full_name(base(a))).filter(|n| !n.is_empty()).take(6).collect();
                set_name(&mut scene.situation, &situation(actor, &others));
            }
            crate::link::with_adapter(|a| a.write_voice_scene(&scene));
        }
    }

    /// An actor near the player by their id.
    fn find(id: u32) -> Option<usize> {
        combat::high_actors().into_iter().find(|&a| id_of(a) == id)
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn frame(_view: &rustvr_adapter::View, _now: f64) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn how_a_line_is_said_is_its_emotion() {
        assert_eq!(emotion("angrily"), (1, 50));
        assert_eq!(emotion("shouting furiously"), (1, 80));
        assert_eq!(emotion("with a laugh"), (5, 50));
        assert_eq!(emotion("Sadly"), (4, 50));
        assert_eq!(emotion("surprised"), (6, 50));
        assert_eq!(emotion(""), (0, 50));
    }

    #[test]
    fn lip_files_are_the_games_own() {
        let mut open = [0.0; 16];
        open[0] = 1.0;
        let file = lip_file(&[open, [0.0; 16], open]);
        let word = |at: usize| u32::from_le_bytes(file[at..at + 4].try_into().unwrap());
        assert_eq!((word(0), word(8)), (1, 1));
        // (decoded: the frames and lead-in, then 3 frames of 33 floats; the header counts 16 more)
        assert_eq!(word(4) as usize, 8 + 3 * 132 + 16);
        let mut decoded = Vec::new();
        let mut i = 12;
        while i < file.len() {
            if file[i] != 0 {
                decoded.push(file[i]);
                i += 1;
            } else {
                decoded.extend(std::iter::repeat_n(0u8, u16::from_le_bytes([file[i + 1], file[i + 2]]) as usize));
                i += 3;
            }
        }
        assert_eq!(decoded.len(), 8 + 3 * 132);
        assert_eq!(u32::from_le_bytes(decoded[0..4].try_into().unwrap()), 3);
        assert_eq!(f32::from_le_bytes(decoded[8..12].try_into().unwrap()), 1.0, "frame 0's Aah");
        assert_eq!(f32::from_le_bytes(decoded[8 + 2 * 132..8 + 2 * 132 + 4].try_into().unwrap()), 1.0, "frame 2's Aah");
    }
}
