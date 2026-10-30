//! Running console commands: a line of script compiled and run as the console does (a temporary
//! `Script`, as xOBSE's `RunScriptLine`). Base forms go by their hex form ids (the game keeps no
//! editor ids for most of them: `player.placeatme 00025e6e 1 150 0`). For tests,
//! `rustvr-console-test` in the game folder lists commands to run at set times, one a line:
//! `<seconds> <command>` (since the game's first frame), `menu[+<seconds>] <command>` (after the main
//! menu is up) or `game[+<seconds>] <command>` (after a game is loaded and the player is in it),
//! e.g. `menu LoadGame "rustvr-city"`, then `game+1 player.additem 0000000A 5`. The file is read
//! once and deleted (`tools/oblivion.ps1` writes it).

use std::sync::Mutex;

const CONSTRUCTOR: usize = 0x004F_BAA0;
const MARK_TEMPORARY: usize = 0x0046_B590;
const SET_TEXT: usize = 0x004F_9E50;
const COMPILE_AND_RUN: usize = 0x004F_BF00;
const DESTRUCTOR: usize = 0x004F_C980;
const SCRIPT_STATE: usize = 0x0058_5C10;
/// `sizeof(Script)` in the game.
const SCRIPT_SIZE: usize = 0x50;

/// `TESSaveLoadGame*` and its `LoadGame(this, 0, name, 0)` (thiscall), as xOBSE's `LoadGameEx`.
const SAVE_LOAD_GAME: usize = 0x00B3_3B00;
const LOAD_GAME: usize = 0x0046_5860;

/// Loads a save by name (its file name without `.ess`). `LoadGame` is a console-only command, so
/// a line of script can't do it.
#[cfg(target_arch = "x86")]
pub fn load_game(name: &str) -> bool {
    type LoadFn = unsafe extern "thiscall" fn(*mut std::ffi::c_void, u32, *const u8, u32) -> u8;
    let Ok(name) = std::ffi::CString::new(name) else { return false };
    let this = crate::mem::deref(SAVE_LOAD_GAME);
    if this == 0 {
        return false;
    }
    unsafe { std::mem::transmute::<usize, LoadFn>(LOAD_GAME)(this as *mut _, 0, name.as_ptr() as *const u8, 0) != 0 }
}

#[cfg(not(target_arch = "x86"))]
pub fn load_game(_name: &str) -> bool {
    false
}

/// `LookupFormByID` (cdecl) and `PlayerCharacter::SetActiveSpell` (thiscall, a `MagicItem*`).
const LOOKUP_FORM: usize = 0x0046_B250;
const SET_ACTIVE_SPELL: usize = 0x0066_4700;

/// Selects the spell with form id `id` as the player's (the game's own selection).
#[cfg(target_arch = "x86")]
pub fn select_spell(id: u32) -> bool {
    type Lookup = unsafe extern "C" fn(u32) -> usize;
    type Select = unsafe extern "thiscall" fn(usize, usize) -> u8;
    let player = crate::engine::player();
    let form = unsafe { std::mem::transmute::<usize, Lookup>(LOOKUP_FORM)(id) };
    if player == 0 || form == 0 {
        return false;
    }
    unsafe { std::mem::transmute::<usize, Select>(SET_ACTIVE_SPELL)(player, form + 0x18) != 0 }
}

#[cfg(not(target_arch = "x86"))]
pub fn select_spell(_id: u32) -> bool {
    false
}

/// The form id of the player's spell called `name` (as the magic menu lists their spells and
/// powers), if they have one.
fn spell_named(name: &str) -> Option<u32> {
    let (table, _) = crate::combat::spells();
    table.spells[..table.count as usize].iter().find(|s| rustvr_protocol::name_str(&s.name).eq_ignore_ascii_case(name)).map(|s| s.id)
}

/// Runs a console line: `LoadGame <save>` loads a save, `SelectSpell <name or hex id>` selects one
/// of the player's spells by its name, or any spell by its id (a spell the game makes, as the mirror
/// spell, has no id to know beforehand), `PlaceInFront <ref hex id> <metres> [<metres up>]` puts a
/// reference that far in front of the player (and that far over their feet), facing them (tests),
/// anything else is compiled and run as a line of script. True if it ran.
pub fn run(line: &str) -> bool {
    if let Some(name) = line.strip_prefix("LoadGame ") {
        return load_game(name.trim().trim_matches('"'));
    }
    if let Some(spell) = line.strip_prefix("SelectSpell ") {
        let spell = spell.trim().trim_matches('"');
        let id = spell_named(spell).or_else(|| u32::from_str_radix(spell.trim_start_matches("0x"), 16).ok());
        return id.is_some_and(select_spell);
    }
    if line.trim() == "InspectTorch" {
        return crate::torch::inspect();
    }
    if let Some(rest) = line.strip_prefix("PlaceInFront ") {
        let mut it = rest.split_whitespace();
        let (Some(r), Some(m)) = (it.next(), it.next().and_then(|m| m.parse::<f32>().ok())) else { return false };
        let up = it.next().and_then(|u| u.parse::<f32>().ok()).unwrap_or(0.0);
        return place_in_front(r, m, up);
    }
    run_script(line)
}

/// Puts the reference `r` (its hex form id) `metres` in front of the player, `up` over their feet
/// and facing them: the game's own `setpos` and `setangle`.
fn place_in_front(r: &str, metres: f32, up: f32) -> bool {
    use crate::engine::{self, refr};
    let p = engine::player();
    let (Some(pos), Some(rot)) = (crate::mem::at::<[f32; 3]>(p + refr::POS), crate::mem::at::<[f32; 3]>(p + refr::ROT)) else { return false };
    // Heading: 0 north (+y), turning east (+x).
    let h = rot[2];
    let d = metres * crate::camera::UNITS_PER_METER;
    let (x, y) = (pos[0] + d * h.sin(), pos[1] + d * h.cos());
    let facing = (h + std::f32::consts::PI).to_degrees().rem_euclid(360.0);
    [format!("{r}.setpos x {x:.1}"), format!("{r}.setpos y {y:.1}"), format!("{r}.setpos z {:.1}", pos[2] + up * crate::camera::UNITS_PER_METER), format!("{r}.setangle z {facing:.1}")].iter().all(|l| run_script(l))
}

/// Compile errors so far (a line ran if it compiled: what the game's run returns is the script's
/// own result).
static COMPILE_ERRORS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Compiles and runs one line of script. True if it compiled (and so ran).
#[cfg(target_arch = "x86")]
pub fn run_script(line: &str) -> bool {
    use std::ffi::c_void;
    type ThisFn = unsafe extern "thiscall" fn(*mut c_void);
    type SetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const u8);
    type RunFn = unsafe extern "thiscall" fn(*mut c_void, *mut c_void, u32, *mut c_void) -> u8;
    type StateFn = unsafe extern "C" fn() -> *mut *mut c_void;
    let Ok(text) = std::ffi::CString::new(line) else { return false };
    let mut obj = [0u8; SCRIPT_SIZE + 16];
    let script = obj.as_mut_ptr() as *mut c_void;
    unsafe {
        let state = std::mem::transmute::<usize, StateFn>(SCRIPT_STATE)();
        if state.is_null() || (*state).is_null() {
            return false;
        }
        std::mem::transmute::<usize, ThisFn>(CONSTRUCTOR)(script);
        std::mem::transmute::<usize, ThisFn>(MARK_TEMPORARY)(script);
        std::mem::transmute::<usize, SetTextFn>(SET_TEXT)(script, text.as_ptr() as *const u8);
        let errors = COMPILE_ERRORS.load(std::sync::atomic::Ordering::Acquire);
        std::mem::transmute::<usize, RunFn>(COMPILE_AND_RUN)(script, *state, 1, std::ptr::null_mut());
        std::mem::transmute::<usize, ThisFn>(DESTRUCTOR)(script);
        COMPILE_ERRORS.load(std::sync::atomic::Ordering::Acquire) == errors
    }
}

#[cfg(not(target_arch = "x86"))]
pub fn run_script(_line: &str) -> bool {
    false
}

/// The console's print (`ConsoleManager::Print(this, fmt, va_list)`, thiscall; xOBSE's
/// `Console_Print` calls it), the script compiler's error report (cdecl `(buffer, fmt, ...)`: it
/// prints only while the console is open) and the game's own `vsprintf`.
const CONSOLE_PRINT: usize = 0x0058_5C90;
const COMPILE_ERROR: usize = 0x004F_CE30;
const VSPRINTF: usize = 0x0098_2820;

/// Everything the game prints to its console goes to the log as well, and why a line of script
/// didn't compile.
#[cfg(target_arch = "x86")]
mod print {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REAL: AtomicUsize = AtomicUsize::new(0);
    static REAL_ERROR: AtomicUsize = AtomicUsize::new(0);

    pub fn install() {
        crate::hooks::install_into(super::CONSOLE_PRINT as *const (), print as *const (), "Console print", &REAL);
        crate::hooks::install_into(super::COMPILE_ERROR as *const (), compile_error as *const (), "Script compile error", &REAL_ERROR);
    }

    type Vsprintf = unsafe extern "C" fn(*mut u8, *const u8, *const c_void) -> i32;

    fn format(fmt: *const u8, args: *const c_void) -> String {
        // (The game formats into 1 KiB itself.)
        let mut buf = [0u8; 4096];
        let n = unsafe { std::mem::transmute::<usize, Vsprintf>(super::VSPRINTF)(buf.as_mut_ptr(), fmt, args) };
        String::from_utf8_lossy(&buf[..(n.max(0) as usize).min(buf.len())]).trim_end().to_owned()
    }

    /// The compiler's messages take a few word-sized arguments (names, line numbers): passed on
    /// as they came.
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn compile_error(buffer: usize, fmt: *const u8, a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> i32 {
        type Report = unsafe extern "C" fn(usize, *const u8, usize, usize, usize, usize, usize, usize) -> i32;
        if !fmt.is_null() {
            let args = [a, b, c, d, e, f];
            tracing::warn!("script compile: {}", format(fmt, args.as_ptr() as *const c_void));
            super::COMPILE_ERRORS.fetch_add(1, Ordering::AcqRel);
        }
        unsafe { std::mem::transmute::<usize, Report>(REAL_ERROR.load(Ordering::Acquire))(buffer, fmt, a, b, c, d, e, f) }
    }

    unsafe extern "thiscall" fn print(this: *mut c_void, fmt: *const u8, args: *mut c_void) -> i32 {
        type Print = unsafe extern "thiscall" fn(*mut c_void, *const u8, *mut c_void) -> i32;
        if !fmt.is_null() {
            tracing::info!("game console: {}", format(fmt, args));
        }
        unsafe { std::mem::transmute::<usize, Print>(REAL.load(Ordering::Acquire))(this, fmt, args) }
    }
}

#[cfg(target_arch = "x86")]
pub use print::install;

/// What a test line's time counts from: the game's first frame, the main menu coming up, or a game
/// being loaded and the player in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum From {
    Start,
    Menu,
    Game,
}

struct Test {
    start: std::time::Instant,
    menu: Option<std::time::Instant>,
    game: Option<std::time::Instant>,
    lines: Vec<(From, f32, String)>,
}
static TEST: Mutex<Option<Test>> = Mutex::new(None);

pub fn parse(text: &str) -> Vec<(From, f32, String)> {
    let mut out: Vec<(From, f32, String)> = text
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let (when, cmd) = l.split_once(' ')?;
            let after = |rest: &str| if rest.is_empty() { Some(0.0) } else { rest.strip_prefix('+')?.parse().ok() };
            let (from, t) = if let Some(rest) = when.strip_prefix("menu") {
                (From::Menu, after(rest)?)
            } else if let Some(rest) = when.strip_prefix("game") {
                (From::Game, after(rest)?)
            } else {
                (From::Start, when.parse().ok()?)
            };
            Some((from, t, cmd.trim().to_owned()))
        })
        .collect();
    // (Stable: lines due together run in the file's order.)
    out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    out
}

/// Reads the test file (once).
pub fn load_test(dir: &std::path::Path) {
    let path = dir.join("rustvr-console-test");
    let Ok(text) = std::fs::read_to_string(&path) else { return };
    let _ = std::fs::remove_file(&path);
    let lines = parse(&text);
    tracing::info!("console: {} test commands", lines.len());
    *TEST.lock().unwrap() = Some(Test { start: std::time::Instant::now(), menu: None, game: None, lines });
}

/// Once a frame, on the game's thread: the test commands that are due.
pub fn tick() {
    let due: Vec<String> = {
        let mut t = TEST.lock().unwrap();
        let Some(test) = t.as_mut() else { return };
        let now = std::time::Instant::now();
        if test.menu.is_none() && crate::menus::menu_open(crate::menus::MENU_MAIN) {
            tracing::info!("console: the main menu is up after {:.1} s", test.start.elapsed().as_secs_f32());
            test.menu = Some(now);
        }
        if test.game.is_none() && crate::menus::in_world() {
            tracing::info!("console: in the game after {:.1} s", test.start.elapsed().as_secs_f32());
            test.game = Some(now);
        }
        let since = |from: From| match from {
            From::Start => Some(test.start),
            From::Menu => test.menu,
            From::Game => test.game,
        };
        let mut due = Vec::new();
        test.lines.retain(|(from, at, cmd)| {
            let ready = since(*from).is_some_and(|s| now.duration_since(s).as_secs_f32() >= *at);
            if ready {
                due.push(cmd.clone());
            }
            !ready
        });
        due
    };
    for cmd in due {
        tracing::info!("console: {cmd}");
        let watch = watchdog(&cmd);
        let ok = run(&cmd);
        watch.store(true, std::sync::atomic::Ordering::Release);
        tracing::info!("console: {cmd} -> {}", if ok { "ran" } else { "failed" });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_timed_commands() {
        use super::From;
        let l = super::parse("8 LoadGame \"Skip Tutorial\"\n\n2.5 tgm\nnonsense\ngame+1 coc Weye\nmenu LoadGame x\n");
        assert_eq!(
            l,
            vec![
                (From::Start, 2.5, "tgm".to_owned()),
                (From::Start, 8.0, "LoadGame \"Skip Tutorial\"".to_owned()),
                (From::Menu, 0.0, "LoadGame x".to_owned()),
                (From::Game, 1.0, "coc Weye".to_owned()),
            ]
        );
    }
}

/// If the command hasn't returned in 5 s, logs where the game's thread is: its instruction pointer
/// and the game-code return addresses on its stack.
#[cfg(target_arch = "x86")]
fn watchdog(cmd: &str) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows::Win32::System::Diagnostics::Debug::{CONTEXT, CONTEXT_FLAGS, GetThreadContext};
    use windows::Win32::System::Threading::{GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS};
    let done = std::sync::Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let tid = unsafe { GetCurrentThreadId() };
    let cmd = cmd.to_owned();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        if flag.load(Ordering::Acquire) {
            return;
        }
        let Ok(t) = (unsafe { OpenThread(THREAD_ALL_ACCESS, false, tid) }) else { return };
        unsafe { SuspendThread(t) };
        let mut ctx = CONTEXT { ContextFlags: CONTEXT_FLAGS(0x0001_0007), ..Default::default() };
        let ok = unsafe { GetThreadContext(t, &mut ctx) }.is_ok();
        let mut calls = Vec::new();
        if ok {
            let mut sp = ctx.Esp as usize;
            for _ in 0..4096 {
                let Some(v) = crate::mem::at::<u32>(sp) else { break };
                if (0x0040_1000..0x00A2_8000).contains(&v) {
                    calls.push(format!("{v:#x}"));
                    if calls.len() >= 24 {
                        break;
                    }
                }
                sp += 4;
            }
        }
        unsafe { ResumeThread(t) };
        tracing::warn!("console: '{cmd}' hasn't returned after 5 s: eip {:#x}, game addresses on the stack {}", ctx.Eip, calls.join(" "));
    });
    done
}

#[cfg(not(target_arch = "x86"))]
fn watchdog(_cmd: &str) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
}
