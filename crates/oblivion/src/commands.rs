//! Simple voice commands, with Windows' own speech recognition (SAPI's recognizer, offline, on the
//! headset's mic), no servers and no AI: RustVR's Voice setting "Simple voice commands"
//! (`RT_VOICE_COMMANDS`).
//!
//! * **In a menu**, its words: every short text of the menu on top that shows (a dialogue's
//!   topics, "Goodbye", "Barter") is a phrase, and said, the game's cursor goes to it and clicks,
//!   as the laser does.
//! * **Looking at someone**, out of menus: "what have you got for sale" (and the like) activates
//!   them and, once their dialogue is up, clicks its "Barter"; "can we talk" activates them.
//!
//! Said as people say them: each phrase also with the words people put round it ("alright mate,
//! what have you got?", "tell me about the Imperial City"), not with anything at all (a wildcard
//! hears "wares" in "the weather's nice").
//!
//! The recognizer runs on a thread of its own (its own COM apartment) with one rule of the phrases
//! now, set from the game's frame as what shows changes; what it hears comes back to the frame.

/// The phrases that ask someone what they sell, and that ask to talk.
const BARTER: [&str; 13] = [
    "what have you got for sale",
    "what do you have for sale",
    "what are you selling",
    "what do you sell",
    "what have you got",
    "show me what you've got",
    "can i buy something",
    "show me your wares",
    "let me see your wares",
    "let's trade",
    "let's barter",
    "i'd like to buy something",
    "i want to buy something",
];
const TALK: [&str; 4] = ["can we talk", "let's talk", "i'd like to talk", "hello there"];
/// The words people put before and after asking someone something, and before a menu's word.
const BEFORE: [&str; 6] = ["", "alright", "alright mate", "hey", "so", "okay"];
const AFTER: [&str; 5] = ["", "mate", "then", "friend", "there"];
const ASKING: [&str; 5] = ["", "tell me about", "what about", "what do you know about", "any"];
/// A menu's text longer than this (characters, words) is what's said or written in it, not a
/// thing to pick.
const LONGEST: usize = 48;
const MOST_WORDS: usize = 7;
/// What's heard counts at this much of the recognizer's confidence (0 to 1): what's said is about
/// 0.95; "what have you been up to, mate" heard as "what have you got, mate", 0.68.
const SURE: f32 = 0.8;

/// A phrase as the recognizer has it: lower case, its letters, digits and apostrophes, one space
/// between words.
pub fn phrase(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '\'' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `phrase` with each of the words before and after it.
pub fn said(phrase: &str, before: &[&str], after: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for b in before {
        for a in after {
            out.push([*b, phrase, *a].iter().filter(|w| !w.is_empty()).copied().collect::<Vec<_>>().join(" "));
        }
    }
    out
}

/// Whether a menu's text is something to pick (a topic, a button), not what's said or written.
pub fn pickable(text: &str) -> bool {
    let p = phrase(text);
    !p.is_empty() && text.chars().count() <= LONGEST && p.split(' ').count() <= MOST_WORDS && p.chars().any(char::is_alphabetic)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::{hud_info, link, menus, tiles};
    use std::sync::Mutex;
    use std::sync::mpsc::{self, Receiver, Sender};
    use windows::Win32::Media::Speech::*;
    use windows::Win32::System::Com::*;
    use windows::core::{HSTRING, Interface, PCWSTR, PWSTR, w};

    /// What a heard phrase does.
    #[derive(Clone, Debug, PartialEq)]
    enum Does {
        /// Click the menu's text that says it.
        Pick(String),
        Barter,
        Talk,
    }

    /// Something being done over the next frames.
    enum Doing {
        /// The cursor on the menu's text `text` and a click: frames since it began.
        Click { text: String, frame: u32 },
        /// Someone activated, their dialogue's "Barter" clicked once it's up (until when).
        Barter { until: f64 },
    }

    #[derive(Default)]
    struct State {
        started: bool,
        phrases: Vec<String>,
        does: Vec<(String, Does)>,
        set: Option<Sender<Vec<String>>>,
        heard: Option<Receiver<(String, f32)>>,
        doing: Option<Doing>,
        /// When the phrases were last worked out.
        looked: f64,
        /// An activate to tap (for the input bridge).
        tap: bool,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    /// A menu's texts that show, with where they are (the menu on top's).
    fn texts() -> Vec<(String, [f32; 4])> {
        let kind = tiles::top_menu();
        if kind == 0 {
            return Vec::new();
        }
        let tree = tiles::Tree::read(tiles::menu_tile(kind));
        tree.0
            .iter()
            .filter(|p| p.shown && p.kind == tiles::KIND_TEXT && p.rect[2] > p.rect[0] && p.rect[3] > p.rect[1])
            .filter_map(|p| tiles::text(p.tile).filter(|t| pickable(t)).map(|t| (t, p.rect)))
            .collect()
    }

    fn is_person(r: usize) -> bool {
        r != 0 && crate::combat::high_actors().contains(&r)
    }

    /// The game's frame (after the world's, before the next input): the phrases for what shows
    /// now, and what was heard.
    pub fn frame(now: f64) {
        crate::hooks::guard_void("voice commands", || run(now));
    }

    fn run(now: f64) {
        let on = link::last().is_some_and(|s| s.frame.flags & rustvr_protocol::RT_VOICE_COMMANDS != 0);
        let mut guard = STATE.lock().unwrap();
        let s = guard.get_or_insert_with(State::default);
        if on && !s.started {
            s.started = true;
            let (set, phrases) = mpsc::channel();
            let (heard_tx, heard) = mpsc::channel();
            std::thread::spawn(move || {
                if let Err(e) = listen(phrases, heard_tx) {
                    tracing::warn!("voice commands: Windows' speech recognition didn't start: {e}");
                }
            });
            s.set = Some(set);
            s.heard = Some(heard);
            tracing::info!("voice commands: listening with Windows' speech recognition");
        }
        if !s.started {
            return;
        }
        // The phrases for what shows, a few times a second.
        if now - s.looked > 0.3 {
            s.looked = now;
            let mut does: Vec<(String, Does)> = Vec::new();
            if on && menus::menu_mode() {
                for (text, _) in texts() {
                    for p in said(&phrase(&text), &ASKING, &[""]) {
                        if !does.iter().any(|(q, _)| *q == p) {
                            does.push((p, Does::Pick(text.clone())));
                        }
                    }
                }
            } else if on && is_person(hud_info::looked_at()) {
                for (phrases, what) in [(&BARTER[..], Does::Barter), (&TALK[..], Does::Talk)] {
                    for base in phrases {
                        does.extend(said(base, &BEFORE, &AFTER).into_iter().map(|p| (p, what.clone())));
                    }
                }
            }
            let phrases: Vec<String> = does.iter().map(|(p, _)| p.clone()).collect();
            if phrases != s.phrases {
                s.phrases = phrases.clone();
                if let Some(set) = &s.set {
                    let _ = set.send(phrases);
                }
            }
            s.does = does;
        }
        // What was heard.
        let heard: Vec<(String, f32)> = s.heard.as_ref().map(|h| h.try_iter().collect()).unwrap_or_default();
        for (text, sure) in heard {
            let p = phrase(&text);
            let does = s.does.iter().find(|(q, _)| *q == p).map(|(_, d)| d.clone());
            tracing::info!("voice commands: heard \"{text}\" ({sure:.2}){}", if does.is_some() && sure >= SURE { "" } else { ": not acted on" });
            if sure < SURE || s.doing.is_some() {
                continue;
            }
            match does {
                Some(Does::Pick(text)) => s.doing = Some(Doing::Click { text, frame: 0 }),
                Some(Does::Barter) => {
                    s.tap = true;
                    s.doing = Some(Doing::Barter { until: now + 3.0 });
                }
                Some(Does::Talk) => s.tap = true,
                None => {}
            }
        }
        // Someone asked what they sell: their dialogue's "Barter", once it's up.
        if let Some(Doing::Barter { until }) = s.doing {
            if now > until {
                s.doing = None;
            } else if tiles::top_menu() == menus::MENU_DIALOG
                && let Some((text, _)) = texts().into_iter().find(|(t, _)| phrase(t) == "barter")
            {
                s.doing = Some(Doing::Click { text, frame: 0 });
            }
        }
    }

    /// Before the game reads its input: a click on a menu's text (the cursor on it a few frames,
    /// then the left button down and up), or an activate to tap (true: the input bridge taps it).
    pub fn before_update(v: &mut crate::input::Virtual) -> bool {
        crate::hooks::guard("voice commands input", false, || {
            let mut guard = STATE.lock().unwrap();
            let Some(s) = guard.as_mut() else { return false };
            let tap = std::mem::take(&mut s.tap);
            if let Some(Doing::Click { text, frame }) = &mut s.doing {
                let Some(screen) = tiles::screen() else {
                    s.doing = None;
                    return tap;
                };
                match texts().into_iter().find(|(t, _)| t == text) {
                    Some((_, rect)) if *frame < 8 => {
                        if let Some(uv) = tiles::uv_of(rect, screen, 0.0) {
                            menus::point([(uv[0] + uv[2]) * 0.5, (uv[1] + uv[3]) * 0.5]);
                        }
                        // (the cursor on it first: the game's hover moves there; then pressed, then let go)
                        if (3..6).contains(frame) {
                            v.buttons |= 1;
                        }
                        *frame += 1;
                    }
                    _ => s.doing = None,
                }
            }
            tap
        })
    }

    /// The headset's mic for the recognizer: the audio input whose name says Virtual Desktop's,
    /// else the system's default.
    fn microphone() -> windows::core::Result<windows::core::IUnknown> {
        unsafe {
            let category: ISpObjectTokenCategory = CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL)?;
            category.SetId(SPCAT_AUDIOIN, false)?;
            let tokens = category.EnumTokens(PCWSTR::null(), PCWSTR::null())?;
            loop {
                let mut token = [None];
                tokens.Next(1, token.as_mut_ptr(), None)?;
                let Some(token) = token[0].take() else { break };
                let name = token.GetStringValue(PCWSTR::null()).map(|n| {
                    let s = n.to_string().unwrap_or_default();
                    CoTaskMemFree(Some(n.0 as *const _));
                    s
                });
                if let Ok(name) = name
                    && name.to_lowercase().contains("virtual desktop")
                {
                    tracing::info!("voice commands: the mic is \"{name}\"");
                    return token.cast();
                }
            }
            tracing::info!("voice commands: the mic is the system's default");
            let default: ISpAudio = CoCreateInstance(&SpMMAudioIn, None, CLSCTX_ALL)?;
            default.cast()
        }
    }

    /// The recognizer's thread: its rule set to each new list of phrases, what it's sure it heard
    /// sent back.
    fn listen(phrases: Receiver<Vec<String>>, heard: Sender<(String, f32)>) -> windows::core::Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            let recognizer: ISpRecognizer = CoCreateInstance(&SpInprocRecognizer, None, CLSCTX_ALL)?;
            recognizer.SetInput(&microphone()?, true)?;
            let context = recognizer.CreateRecoContext()?;
            context.SetNotifyWin32Event()?;
            // (SAPI's event interest: the event's bit and its two check bits)
            let interest = (1u64 << SPEI_RECOGNITION.0) | (1u64 << SPEI_RESERVED1.0) | (1u64 << SPEI_RESERVED2.0);
            context.SetInterest(interest, interest)?;
            let grammar = context.CreateGrammar(1)?;
            let mut rule = SPSTATEHANDLE::default();
            grammar.GetRule(w!("phrases"), 1, (SPRAF_TopLevel.0 | SPRAF_Active.0) as u32, true, &mut rule)?;
            loop {
                let mut latest = None;
                loop {
                    match phrases.try_recv() {
                        Ok(p) => latest = Some(p),
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
                    }
                }
                // (nothing to listen for: the rule off, as it was; a rule of no words can't be
                // committed, SPERR_NO_TERMINATING_RULE_PATH)
                if let Some(list) = latest {
                    let set = || -> windows::core::Result<()> {
                        grammar.SetRuleState(PCWSTR::null(), std::ptr::null_mut(), SPRS_INACTIVE)?;
                        if list.is_empty() {
                            return Ok(());
                        }
                        grammar.ClearRule(rule)?;
                        for p in &list {
                            grammar.AddWordTransition(rule, SPSTATEHANDLE::default(), &HSTRING::from(p.as_str()), w!(" "), SPWT_LEXICAL, 1.0, std::ptr::null())?;
                        }
                        grammar.Commit(0)?;
                        grammar.SetRuleState(PCWSTR::null(), std::ptr::null_mut(), SPRS_ACTIVE)
                    };
                    if let Err(e) = set() {
                        tracing::warn!("voice commands: the {} phrases weren't taken: {e}", list.len());
                    }
                }
                let _ = context.WaitForNotifyEvent(100);
                loop {
                    let mut event = SPEVENT::default();
                    let mut fetched = 0u32;
                    if context.GetEvents(1, &mut event, &mut fetched).is_err() || fetched == 0 {
                        break;
                    }
                    // (the event's id is its first 16 bits; a recognition's lParam is its result,
                    // ours to release)
                    if (event._bitfield & 0xFFFF) != SPEI_RECOGNITION.0 || event.lParam.0 == 0 {
                        continue;
                    }
                    let result = ISpRecoResult::from_raw(event.lParam.0 as *mut _);
                    let sure = result.GetPhrase().map(|p| {
                        let sure = (*p).Base.Rule.SREngineConfidence;
                        CoTaskMemFree(Some(p as *const _));
                        sure
                    });
                    let mut text = PWSTR::null();
                    if result.GetText(u32::MAX, u32::MAX, true, &mut text, None).is_ok() && !text.is_null() {
                        let said = text.to_string().unwrap_or_default();
                        CoTaskMemFree(Some(text.0 as *const _));
                        if heard.send((said, sure.unwrap_or(0.0))).is_err() {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn frame(_now: f64) {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn before_update(_v: &mut crate::input::Virtual) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_menus_words_are_phrases_and_only_short_ones_are_picked() {
        assert_eq!(phrase("Rumors"), "rumors");
        assert_eq!(phrase("  The Imperial  City! "), "the imperial city");
        assert_eq!(phrase("Let's trade."), "let's trade");
        assert!(pickable("Goodbye") && pickable("Shadowmere's Stable"));
        // (what they say, and a number, aren't things to pick)
        assert!(!pickable("Well met, traveller. The city has been quiet since the Emperor's death, but the guards still talk."));
        assert!(!pickable("250") && !pickable(""));
    }

    #[test]
    fn a_phrase_is_also_said_with_the_words_round_it() {
        let all = said("what have you got", &BEFORE, &AFTER);
        assert_eq!(all.len(), BEFORE.len() * AFTER.len());
        assert!(all.contains(&"what have you got".to_string()) && all.contains(&"alright mate what have you got then".to_string()));
        assert!(said("rumors", &ASKING, &[""]).contains(&"any rumors".to_string()));
    }
}
