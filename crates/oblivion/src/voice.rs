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
//! * **A line**, streamed (RustVR's Voice setting, on by default): the person speaks a long silent
//!   sound with the game's own `Actor::Speak` (into their process's voice slot, with its talking
//!   state: the voice from them in 3D, at the game's voice volume), and the plugin's voice is
//!   written into that sound's buffer just ahead of where it plays, as it comes; each frame their
//!   face's mouth is set to the plugin's mouth shapes at the voice where it plays, each sentence
//!   brings its subtitle and their expression from how it's said. Otherwise, a sentence at a
//!   time: the plugin writes `<n>.wav` and the mouth's shapes; here they become the game's own lip
//!   file, and the person speaks the WAV, the face moving with the game's lip sync.
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

/// The game's emotion for how a line is said (its own words for it: "angrily", "hurt, but still
/// beaming"), the first of its words that says one, and how strongly (0 to 100: plainly enough to
/// see on a face, more for "very" and the like, less for "mildly"). Neutral for anything else.
pub fn emotion(how: &str) -> (u32, u32) {
    let how = how.to_lowercase();
    let strong = ["very", "furious", "terrified", "sobbing", "shout", "yell", "scream", "roar", "livid", "ecstatic", "beside"].iter().any(|w| how.contains(w));
    let mild = ["mild", "slight", "a little", "a bit", "faint", "soft", "quiet", "gentl"].iter().any(|w| how.contains(w));
    let kinds: [(u32, &[&str]); 6] = [
        (
            1,
            &[
                "angr", "anger", "furious", "fury", "annoy", "irrit", "snap", "shout", "yell", "growl", "stern", "cold", "harsh", "scowl", "glar", "frown", "bitter", "sharp",
                "huff", "seeth", "livid", "irate", "indignant", "outrag", "menac", "threat", "snarl", "grumbl", "grump", "exasperat", "impatien", "curt", "gruff", "terse",
            ],
        ),
        (2, &["disgust", "revolt", "sneer", "contempt", "scorn", "disdain", "sniff", "repuls", "distaste", "grimac", "appal", "nause", "loath", "sour"]),
        (
            3,
            &[
                "afraid", "fear", "scared", "nervous", "terrif", "worried", "anxious", "whisper", "trembl", "shak", "stammer", "hesitant", "uneas", "panic",
                "frighten", "timid", "wary", "flinch", "jumpy",
            ],
        ),
        (
            4,
            &[
                "sad", "sorrow", "mourn", "grief", "tired", "weary", "sigh", "sob", "regret", "hurt", "wound", "disappoint", "deflat", "crestfallen", "glum", "gloom",
                "melanchol", "wistful", "somber", "sombre", "downcast", "tear", "cry", "lonely", "ashamed", "sheepish", "apolog", "sulk", "pout",
            ],
        ),
        (
            5,
            &[
                "laugh", "happy", "warm", "cheer", "smil", "amus", "glad", "friendly", "joy", "chuckl", "grin", "pleased", "beam", "delight", "eager", "thrill",
                "excit", "enthus", "bright", "gush", "giggl", "proud", "adoring", "fond", "playful", "grateful", "relie", "merry", "jovial", "wink", "teas", "chirp",
                "bubbl", "hearty", "ecstatic", "elated", "gleeful", "glee",
            ],
        ),
        (
            6,
            &[
                "surpris", "shock", "astonish", "startl", "amazed", "disbelief", "gasp", "aghast", "stunned", "wide-eyed", "incredul", "taken aback", "baffl",
                "confus", "bewilder", "puzzl",
            ],
        ),
    ];
    let first = kinds.iter().filter_map(|(kind, words)| words.iter().filter_map(|w| how.find(w)).min().map(|at| (at, *kind))).min();
    match first {
        Some((_, kind)) => (kind, if strong { 100 } else if mild { 40 } else { 70 }),
        None => (0, 50),
    }
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

/// Who someone is, a paragraph from `data/bios.toml` (written from UESP), by the name the game
/// gives them; none for a guard or a bandit, nobody in particular.
pub fn bio(name: &str) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static BIOS: OnceLock<HashMap<String, String>> = OnceLock::new();
    let bios = BIOS.get_or_init(|| {
        #[derive(serde::Deserialize)]
        struct Entry {
            bio: String,
            #[serde(default)]
            names: Vec<String>,
        }
        let entries: HashMap<String, Entry> = toml::from_str(include_str!("../data/bios.toml")).unwrap_or_else(|e| {
            tracing::warn!("voice: data/bios.toml: {e}");
            HashMap::new()
        });
        let mut bios = HashMap::new();
        for (name, entry) in entries {
            // (one paragraph: the file's lines are only its wrapping)
            let bio = entry.bio.split_whitespace().collect::<Vec<_>>().join(" ");
            for name in entry.names.iter().chain(std::iter::once(&name)) {
                bios.insert(name.to_lowercase(), bio.clone());
            }
        }
        bios
    });
    bios.get(&name.trim().to_lowercase()).map(String::as_str)
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
    /// `HighProcess`'s talking flag.
    const TALKING: usize = 0x228;
    /// The audio manager (its global) and its sound by id (thiscall: the manager, the id; 0 once
    /// it's gone). A sound (`TESGameSound`): its flags (0x200: waiting to play, or paused for a
    /// menu) and its `IDirectSoundBuffer8`, the whole sound in one buffer the game fills once and
    /// never again. The buffer's GetCurrentPosition, Lock and Unlock.
    const AUDIO_MANAGER: usize = 0x00B3_C214;
    const FIND_SOUND: usize = 0x006A_B130;
    const SOUND_WAITING: u32 = 0x200;
    const SOUND_BUFFER: usize = 0x50;
    const DSB_POSITION: usize = 0x10;
    const DSB_LOCK: usize = 0x2C;
    const DSB_UNLOCK: usize = 0x4C;
    /// The head's face (`BSFaceGenAnimationData`, the process's vtable +0xE8, thiscall: the actor)
    /// and its virtuals: drop its keys to come (the expressions', the mouth's, the modifiers', the
    /// custom one's), a mouth's keyframe (the 16 phonemes, its time: 0, at once), a blend to rest
    /// (the channel, 0 the mouth; its time; 1), an expression (its index, 0 to 1); and the game's
    /// setting that shows expressions.
    const VT_FACE: usize = 0xE8;
    const FACE_VTABLE: usize = 0x00A6_424C;
    const VT_FACE_CLEAR: usize = 0xB0;
    const VT_FACE_MOUTH: usize = 0xA0;
    const VT_FACE_REST: usize = 0xA8;
    const VT_FACE_EXPRESSION: usize = 0xC8;
    const EXPRESSIONS_ON: usize = 0x00B1_206C;
    /// A streamed line's silent sound (seconds: a longer line goes on in another).
    const STREAM_SECONDS: u32 = 30;
    /// The voice is written this far past where the sound card reads the buffer (seconds).
    const WRITE_MARGIN: f32 = 0.02;
    /// The mouth leads where the buffer plays by the sound's way to the ears (seconds).
    const MOUTH_LEAD: f32 = 0.03;
    /// A streamed line nothing's come for this long, with no end said, is given up (seconds).
    const QUIET: f64 = 10.0;
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

    /// A line streamed as it's made (`VoiceStream`): it plays in a long silent sound of the game's,
    /// spoken as any line, its voice written into the sound's buffer just ahead of where it plays,
    /// the mouth set each frame to the voice's where it plays.
    struct Streamed {
        speaker: u32,
        line: u32,
        rate: u32,
        how: String,
        /// The voice and its mouth so far, from the line's start.
        samples: Vec<i16>,
        frames: Vec<[f32; 16]>,
        /// Its sentences to come: where each starts (a sample), how it's said, its words.
        sentences: VecDeque<(u32, String, String)>,
        /// The line's length (samples), once the plugin's said it.
        end: Option<u32>,
        /// The game's sound (its id; 0: none yet), its buffer's bytes, the samples written into it,
        /// where the next go (a byte), and each run of samples (where in the buffer it starts, the
        /// sample there): the voice running out, the next goes on past where it plays, after a gap.
        sound: u32,
        bytes: u32,
        written: usize,
        at: u32,
        runs: Vec<(u32, usize)>,
        /// When anything last came.
        heard: f64,
    }

    /// A sentence of the plugin's, waiting to be said after the one before.
    struct Pending {
        speaker: u32,
        line: u32,
        /// Its voice's length.
        ms: u32,
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
        /// The line being streamed, and one just ended (its speaker, when: the game ends it its own
        /// way a frame later, or it's ended here).
        stream: Option<Streamed>,
        ending: Option<(u32, f64)>,
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
                    v.stream = None;
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
        if let Some(bio) = bio(&name) {
            out += &format!("\n\n{bio}");
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
    fn speak(actor: usize, player: usize, file: &str, how: &str, text: &str) -> f64 {
        let process = high_process(actor);
        if process == 0 {
            return 0.0;
        }
        let path = format!("Data\\Sound\\Voice\\RustVR\\{file}");
        let Ok(c_path) = std::ffi::CString::new(path) else { return 0.0 };
        let (kind, value) = emotion(how);
        type Speak = unsafe extern "thiscall" fn(usize, *const u8, usize, u32, u32, u32, u32, u32, u32, u32) -> f64;
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
        subtitle(process, text);
        tracing::info!("voice: {} says {file} ({}, {kind}/{value}, {seconds:.1} s): \"{text}\"", full_name(base(actor)), EMOTIONS[kind as usize]);
        seconds
    }

    /// The subtitle for a line said in the process's voice slot (if the game shows them).
    fn subtitle(process: usize, text: &str) {
        type Subtitle = unsafe extern "C" fn(*const u8, usize, u32, f32) -> u8;
        if mem::at::<u8>(SUBTITLES_ON).is_some_and(|on| on != 0)
            && !text.trim().is_empty()
            && let Ok(c_text) = std::ffi::CString::new(text)
        {
            unsafe { std::mem::transmute::<usize, Subtitle>(SUBTITLE)(c_text.as_ptr() as *const u8, mem::deref(process + VOICE_SLOT), 0, -1.0) };
        }
    }

    /// The face of an actor's head (None without one).
    fn face_of(actor: usize) -> Option<usize> {
        let process = high_process(actor);
        let f = mem::deref(mem::deref(process) + VT_FACE);
        if process == 0 || !engine::CODE.contains(&f) {
            return None;
        }
        type F = unsafe extern "thiscall" fn(usize, usize) -> usize;
        let face = unsafe { std::mem::transmute::<usize, F>(f)(process, actor) };
        (face != 0 && mem::deref(face) == FACE_VTABLE).then_some(face)
    }

    fn face_call(face: usize, slot: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(face) + slot);
        engine::CODE.contains(&f).then_some(f)
    }

    /// The mouth at once in these shapes (FaceGen's 16 phonemes, 0 to 1), its keys to come dropped.
    fn set_mouth(face: usize, shapes: &[f32; 16]) {
        let (Some(clear), Some(mouth)) = (face_call(face, VT_FACE_CLEAR), face_call(face, VT_FACE_MOUTH)) else { return };
        let shapes = shapes.map(|w| if w.is_finite() { w.clamp(0.0, 1.0) } else { 0.0 });
        type Clear = unsafe extern "thiscall" fn(usize, u32, u32, u32, u32);
        type Mouth = unsafe extern "thiscall" fn(usize, *const f32, f32);
        unsafe {
            std::mem::transmute::<usize, Clear>(clear)(face, 0, 1, 0, 0);
            std::mem::transmute::<usize, Mouth>(mouth)(face, shapes.as_ptr(), 0.0);
        }
    }

    /// The mouth shut, a blend to rest as the game ends its lip sync.
    fn mouth_to_rest(face: usize) {
        let Some(rest) = face_call(face, VT_FACE_REST) else { return };
        type Rest = unsafe extern "thiscall" fn(usize, u32, f32, u32);
        unsafe { std::mem::transmute::<usize, Rest>(rest)(face, 0, 0.2, 1) };
    }

    /// Their expression from how a sentence is said (as `Actor::Speak` sets it, if the game shows
    /// expressions): the emotion's expression (0x54F590's table) at its value.
    fn set_expression(face: usize, how: &str) {
        let (kind, value) = emotion(how);
        let index = match kind {
            1 | 2 => 0,
            3 => 1,
            5 => 2,
            4 => 3,
            6 => 4,
            _ => return,
        };
        if mem::at::<u8>(EXPRESSIONS_ON).is_some_and(|on| on != 0)
            && let Some(f) = face_call(face, VT_FACE_EXPRESSION)
        {
            type Expression = unsafe extern "thiscall" fn(usize, u32, f32);
            unsafe { std::mem::transmute::<usize, Expression>(f)(face, index, value as f32 / 100.0) };
        }
    }

    /// The long silent sound a streamed line plays in (`stream-<rate>.wav`, written once, with no
    /// lip file: the mouth is set here).
    fn silent_wav(lines_dir: &Path, rate: u32) -> Option<String> {
        let name = format!("stream-{rate}.wav");
        let path = lines_dir.join(&name);
        if !path.exists() {
            let data = (rate * STREAM_SECONDS * 2) as usize;
            let mut wav = Vec::with_capacity(44 + data);
            wav.extend(b"RIFF");
            wav.extend((36 + data as u32).to_le_bytes());
            wav.extend(b"WAVEfmt ");
            wav.extend(16u32.to_le_bytes());
            wav.extend(1u16.to_le_bytes());
            wav.extend(1u16.to_le_bytes());
            wav.extend(rate.to_le_bytes());
            wav.extend((rate * 2).to_le_bytes());
            wav.extend(2u16.to_le_bytes());
            wav.extend(16u16.to_le_bytes());
            wav.extend(b"data");
            wav.extend((data as u32).to_le_bytes());
            wav.resize(44 + data, 0);
            std::fs::create_dir_all(lines_dir).ok()?;
            std::fs::write(&path, wav).ok()?;
        }
        let _ = std::fs::remove_file(path.with_extension("lip"));
        Some(name)
    }

    /// A sound of the game's by its id (0: gone).
    fn sound(id: u32) -> usize {
        let manager = mem::deref(AUDIO_MANAGER);
        if manager == 0 || id == 0 {
            return 0;
        }
        type F = unsafe extern "thiscall" fn(usize, u32) -> usize;
        unsafe { std::mem::transmute::<usize, F>(FIND_SOUND)(manager, id) }
    }

    /// Where a DirectSound buffer plays and where it may be written from (bytes).
    fn buffer_position(dsb: usize) -> Option<(u32, u32)> {
        let f = mem::deref(mem::deref(dsb) + DSB_POSITION);
        if dsb == 0 || f == 0 {
            return None;
        }
        type F = unsafe extern "system" fn(usize, *mut u32, *mut u32) -> i32;
        let (mut play, mut write) = (0u32, 0u32);
        (unsafe { std::mem::transmute::<usize, F>(f)(dsb, &mut play, &mut write) } >= 0).then_some((play, write))
    }

    /// Writes 16-bit samples into a DirectSound buffer from a byte.
    fn buffer_write(dsb: usize, at: u32, pcm: &[i16]) -> bool {
        let (lock, unlock) = (mem::deref(mem::deref(dsb) + DSB_LOCK), mem::deref(mem::deref(dsb) + DSB_UNLOCK));
        if lock == 0 || unlock == 0 || pcm.is_empty() {
            return false;
        }
        type Lock = unsafe extern "system" fn(usize, u32, u32, *mut *mut u8, *mut u32, *mut *mut u8, *mut u32, u32) -> i32;
        type Unlock = unsafe extern "system" fn(usize, *mut u8, u32, *mut u8, u32) -> i32;
        let bytes: &[u8] = bytemuck::cast_slice(pcm);
        let (mut p1, mut l1, mut p2, mut l2) = (std::ptr::null_mut(), 0u32, std::ptr::null_mut(), 0u32);
        if unsafe { std::mem::transmute::<usize, Lock>(lock)(dsb, at, bytes.len() as u32, &mut p1, &mut l1, &mut p2, &mut l2, 0) } < 0 {
            return false;
        }
        unsafe {
            let first = (l1 as usize).min(bytes.len());
            if !p1.is_null() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), p1, first);
            }
            if !p2.is_null() {
                std::ptr::copy_nonoverlapping(bytes[first..].as_ptr(), p2, (l2 as usize).min(bytes.len() - first));
            }
            std::mem::transmute::<usize, Unlock>(unlock)(dsb, p1, l1, p2, l2);
        }
        true
    }

    /// How a streamed line's frame went.
    #[derive(PartialEq)]
    enum Flow {
        Going,
        /// Said to its end (or given up), and ended here.
        Ended,
        /// Over without a word more: the game stopped it, or they're gone.
        Gone,
    }

    /// The streamed line's frame: what's come since, into the game's sound; the mouth where it
    /// plays; each sentence's subtitle and expression as it's reached; its end.
    fn stream_frame(s: &mut Streamed, now: f64, player: usize, lines_dir: &Path) -> Flow {
        let before = (s.samples.len(), s.frames.len());
        crate::link::with_adapter(|a| a.take_voice_stream(s.line, &mut s.samples, &mut s.frames));
        if (s.samples.len(), s.frames.len()) != before {
            s.heard = now;
        }
        let Some(actor) = find(s.speaker) else { return Flow::Gone };
        let process = high_process(actor);
        if process == 0 {
            return Flow::Gone;
        }
        if s.sound == 0 {
            if s.samples.len() <= s.written {
                let waits = now - s.heard < QUIET && s.end.is_none_or(|e| s.written < e as usize);
                return if waits { Flow::Going } else { Flow::Gone };
            }
            let Some(file) = silent_wav(lines_dir, s.rate) else { return Flow::Gone };
            speak(actor, player, &file, &s.how, "");
            let handle = mem::deref(process + VOICE_SLOT);
            s.sound = if handle == 0 { 0 } else { mem::at::<u32>(handle).unwrap_or(0) };
            if sound(s.sound) == 0 {
                tracing::warn!("voice: line {}'s sound didn't start", s.line);
                return Flow::Gone;
            }
            s.bytes = s.rate * STREAM_SECONDS * 2;
            s.at = 0;
            s.runs.clear();
            s.runs.push((0, s.written));
        }
        let snd = sound(s.sound);
        if snd == 0 {
            // (its sound played to its end with more to say: the rest in another; else the game
            // stopped it: a menu, the player far off)
            if s.at + 2 > s.bytes && s.written < s.samples.len() {
                s.sound = 0;
                return Flow::Going;
            }
            end_line(actor);
            return Flow::Ended;
        }
        let dsb = mem::deref(snd + SOUND_BUFFER);
        let Some((play, write)) = buffer_position(dsb) else { return Flow::Gone };
        let waiting = mem::at::<u32>(snd).is_some_and(|f| f & SOUND_WAITING != 0);
        // (what's written played already, the voice ran out: what's come goes on past where it
        // plays)
        if s.written < s.samples.len() && s.at < write && !(waiting && play == 0) {
            s.at = ((write as f32 + WRITE_MARGIN * s.rate as f32 * 2.0) as u32 & !1).min(s.bytes);
            s.runs.push((s.at, s.written));
        }
        let n = (s.samples.len() - s.written).min(((s.bytes - s.at) / 2) as usize);
        if n > 0 && buffer_write(dsb, s.at, &s.samples[s.written..s.written + n]) {
            s.written += n;
            s.at += n as u32 * 2;
        }
        // Where it plays, in the line's samples (and whether that's voice, not a gap).
        let i = s.runs.iter().rposition(|&(b, _)| b <= play).unwrap_or(0);
        let (start, first) = s.runs[i];
        let run_end = s.runs.get(i + 1).map_or(s.written, |r| r.1);
        let at = first + ((play.saturating_sub(start)) / 2) as usize;
        let voice = !waiting && at < run_end;
        // (for the plugin: how much is said, so it knows when the game is about to wait)
        crate::link::with_adapter(|a| a.set_voice_stream_played(s.line, at.min(run_end) as u64));
        if let Some(face) = face_of(actor) {
            let hop = (s.rate / VOICE_STREAM_FPS).max(1) as f32;
            let t = (at as f32 + MOUTH_LEAD * s.rate as f32) / hop;
            let (f, into) = (t as usize, t.fract());
            let shapes = match (voice, s.frames.get(f), s.frames.get(f + 1)) {
                (true, Some(a), Some(b)) => std::array::from_fn(|p| a[p] + (b[p] - a[p]) * into),
                (true, Some(a), None) => *a,
                _ => [0.0; 16],
            };
            set_mouth(face, &shapes);
            // (a sentence starts as its voice plays; not in the wait before it comes)
            let reached = if voice { at } else { at.min(run_end.saturating_sub(1)).max(first) };
            while let Some((from, how, text)) = s.sentences.front()
                && (*from as usize) <= reached
            {
                set_expression(face, how);
                subtitle(process, text);
                s.sentences.pop_front();
            }
        }
        // (talking while there's voice to come; the game's dialogue lets a line end at 0)
        let left = s.at.saturating_sub(play) as f32 / (s.rate * 2) as f32;
        let set_time = mem::deref(mem::deref(process) + VT_SET_TALK_TIME);
        if engine::CODE.contains(&set_time) {
            type SetF = unsafe extern "thiscall" fn(usize, f32);
            unsafe { std::mem::transmute::<usize, SetF>(set_time)(process, (left + 0.3).max(0.5)) };
        }
        let done = s.end.is_some_and(|e| s.written >= e as usize) || (s.end.is_none() && now - s.heard > QUIET);
        if done && !waiting && play >= s.at {
            tracing::info!("voice: line {} said ({:.1} s)", s.line, s.written as f32 / s.rate as f32);
            end_line(actor);
            return Flow::Ended;
        }
        Flow::Going
    }

    /// A streamed line's end: its sound stopped (its subtitle with it), no talk time left, the
    /// mouth shut; the game's own dialogue ends their talking (a frame later).
    fn end_line(actor: usize) {
        let process = high_process(actor);
        if process == 0 {
            return;
        }
        vcall1(process, VT_STOP_SOUND, 0);
        let set_time = mem::deref(mem::deref(process) + VT_SET_TALK_TIME);
        if engine::CODE.contains(&set_time) {
            type SetF = unsafe extern "thiscall" fn(usize, f32);
            unsafe { std::mem::transmute::<usize, SetF>(set_time)(process, 0.0) };
        }
        if let Some(face) = face_of(actor) {
            mouth_to_rest(face);
        }
    }

    /// Stops what they're saying (the player talked over them), the mouth shut.
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
            if let Some(face) = face_of(actor) {
                mouth_to_rest(face);
            }
        }
    }

    /// A listener turns to the player and looks at them, as the game's speakers turn to whom
    /// they talk to (`SayTopic`): not while on their way somewhere or using something, and only
    /// past the game's turn angle (0.8 of `iActorTurnDegree`), to the headset.
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
        // (to the headset, where the room puts the player, else their feet)
        let me = crate::look::head().or_else(|| mem::at::<[f32; 3]>(player + engine::refr::POS));
        let (Some(me), Some(them), Some(rot)) = (me, mem::at::<[f32; 3]>(actor + engine::refr::POS), mem::at::<[f32; 3]>(actor + engine::refr::ROT)) else { return };
        // Headings: 0 north (+y), turning east (+x).
        let heading = (me[0] - them[0]).atan2(me[1] - them[1]).rem_euclid(std::f32::consts::TAU);
        let off = (heading - rot[2] + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        let within = 0.8 * mem::at::<i32>(TALK_TURN_ANGLE).unwrap_or(90) as f32;
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
                if let Some(actor) = v.stream.take().and_then(|s| find(s.speaker)) {
                    hush(actor);
                }
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
                VL_SAY => v.queue.push_back(Pending { speaker: line.speaker, line: line.line, ms: line.duration_ms, how: line.how().to_string(), text: line.text().to_string() }),
                VL_STREAM => {
                    if let Some(actor) = v.stream.take().filter(|s| s.sound != 0).and_then(|s| find(s.speaker)) {
                        hush(actor);
                    }
                    v.stream = Some(Streamed {
                        speaker: line.speaker,
                        line: line.line,
                        rate: line.at.clamp(8000, 48000),
                        how: line.how().to_string(),
                        samples: Vec::new(),
                        frames: Vec::new(),
                        sentences: VecDeque::new(),
                        end: None,
                        sound: 0,
                        bytes: 0,
                        written: 0,
                        at: 0,
                        runs: Vec::new(),
                        heard: now,
                    });
                }
                VL_SUBTITLE => {
                    if let Some(s) = v.stream.as_mut().filter(|s| s.line == line.line) {
                        s.sentences.push_back((line.at, line.how().to_string(), line.text().to_string()));
                    }
                }
                VL_END => {
                    if let Some(s) = v.stream.as_mut().filter(|s| s.line == line.line) {
                        s.end = Some(line.at);
                    }
                }
                VL_STOP => {
                    v.queue.clear();
                    if let Some(actor) = v.talking.take().and_then(|(id, _)| find(id)) {
                        hush(actor);
                    }
                    if let Some(actor) = v.stream.take().and_then(|s| find(s.speaker)) {
                        hush(actor);
                    }
                }
                _ => {}
            }
        }

        // The streamed line, as it comes; one just ended, its talking off if the game hasn't.
        if let Some((id, at)) = v.ending
            && now - at > 0.1
        {
            v.ending = None;
            // (not if they've a line in their voice slot again: the game's own, or ours)
            if let Some(process) = find(id).map(high_process).filter(|&p| p != 0)
                && mem::at::<u8>(process + TALKING).is_some_and(|t| t != 0)
                && mem::deref(process + VOICE_SLOT) == 0
                && v.stream.as_ref().is_none_or(|s| s.speaker != id)
            {
                vcall1(process, VT_SET_TALKING, 0);
            }
        }
        if let Some(s) = v.stream.as_mut() {
            let speaker = s.speaker;
            match stream_frame(s, now, player, &lines_dir) {
                Flow::Going => {
                    v.listener = Some((speaker, now + ENGAGED));
                    v.talking = Some((speaker, now + 0.1));
                }
                Flow::Ended => {
                    v.stream = None;
                    v.ending = Some((speaker, now));
                }
                Flow::Gone => v.stream = None,
            }
        }

        // The plugin's sentences, one after another as each ends: by its voice's own length (the
        // game's is a tenth of a second longer, its lip file's end padded), so a sentence cut in
        // two at a pause joins up again with that pause as it was.
        if v.talking.is_none_or(|(_, until)| now >= until)
            && let Some(next) = v.queue.pop_front()
        {
            v.talking = None;
            if let Some(actor) = find(next.speaker) {
                if !write_lip(&lines_dir, next.line) {
                    tracing::warn!("voice: no mouth shapes for line {}: the game's own flapping mouth", next.line);
                }
                let seconds = speak(actor, player, &format!("{}.wav", next.line), &next.how, &next.text);
                let length = if seconds <= 0.0 { 0.2 } else if next.ms > 0 { next.ms as f64 / 1000.0 } else { seconds };
                v.talking = Some((next.speaker, now + length));
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
        assert_eq!(emotion("angrily"), (1, 70));
        assert_eq!(emotion("shouting furiously"), (1, 100));
        assert_eq!(emotion("with a laugh"), (5, 70));
        assert_eq!(emotion("Sadly"), (4, 70));
        assert_eq!(emotion("surprised"), (6, 70));
        assert_eq!(emotion("mildly amused"), (5, 40));
        assert_eq!(emotion(""), (0, 50));
        // (the Adoring Fan's own: the first of its words that says one)
        assert_eq!(emotion("hurt, but still beaming").0, 4);
        assert_eq!(emotion("confused, then delighted").0, 6);
        assert_eq!(emotion("trying to scowl, but it comes out wobbly").0, 1);
        assert_eq!(emotion("eagerly").0, 5);
        assert_eq!(emotion("evenly").0, 0);
        assert_eq!(emotion("in a strange, changing voice").0, 0);
    }

    #[test]
    fn people_with_a_bio_have_one_paragraph_of_it() {
        let gin = bio("Gin-Wulm").expect("Gin-Wulm's bio");
        assert!(gin.starts_with("Gin-Wulm is the Argonian master smith") && !gin.contains('\n'));
        assert_eq!(bio("sevarius atius"), bio("Severius Atius"), "by any of their names, any case");
        // (and the tool's, everyone else with a page: the Isles' people too)
        assert!(bio("Hannibal Traven").is_some_and(|b| b.starts_with("Hannibal Traven is")));
        assert!(bio("Haskill").is_some());
        assert!(bio("Imperial Watch").is_none());
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
