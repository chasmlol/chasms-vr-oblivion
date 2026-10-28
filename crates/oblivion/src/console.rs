//! Running console commands: a line of script compiled and run as the console does (a temporary
//! `Script`, as xOBSE's `RunScriptLine`). For tests, `rustvr-console-test` in the game folder lists
//! commands to run at set times (`<seconds> <command>` per line, seconds since the game's first
//! frame), e.g. `8 LoadGame "Skip Tutorial"`; the file is read once and deleted.

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

/// Runs a console line: `LoadGame <save>` loads a save, anything else is compiled and run as a
/// line of script. True if it ran.
pub fn run(line: &str) -> bool {
    if let Some(name) = line.strip_prefix("LoadGame ") {
        return load_game(name.trim().trim_matches('"'));
    }
    run_script(line)
}

/// Compiles and runs one line of script. True if it ran.
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
        let ok = std::mem::transmute::<usize, RunFn>(COMPILE_AND_RUN)(script, *state, 1, std::ptr::null_mut()) != 0;
        std::mem::transmute::<usize, ThisFn>(DESTRUCTOR)(script);
        ok
    }
}

#[cfg(not(target_arch = "x86"))]
pub fn run_script(_line: &str) -> bool {
    false
}

struct Test {
    start: std::time::Instant,
    lines: Vec<(f32, String)>,
}
static TEST: Mutex<Option<Test>> = Mutex::new(None);

pub fn parse(text: &str) -> Vec<(f32, String)> {
    let mut out: Vec<(f32, String)> = text
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let (t, cmd) = l.split_once(' ')?;
            Some((t.parse().ok()?, cmd.trim().to_owned()))
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// Reads the test file (once).
pub fn load_test(dir: &std::path::Path) {
    let path = dir.join("rustvr-console-test");
    let Ok(text) = std::fs::read_to_string(&path) else { return };
    let _ = std::fs::remove_file(&path);
    let lines = parse(&text);
    tracing::info!("console: {} test commands", lines.len());
    *TEST.lock().unwrap() = Some(Test { start: std::time::Instant::now(), lines });
}

/// Once a frame, on the game's thread: the test commands that are due.
pub fn tick() {
    let due: Vec<String> = {
        let mut t = TEST.lock().unwrap();
        let Some(test) = t.as_mut() else { return };
        let now = test.start.elapsed().as_secs_f32();
        let n = test.lines.iter().take_while(|(at, _)| *at <= now).count();
        test.lines.drain(..n).map(|(_, c)| c).collect()
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
        let l = super::parse("8 LoadGame \"Skip Tutorial\"\n\n2.5 tgm\nnonsense\n");
        assert_eq!(l, vec![(2.5, "tgm".to_owned()), (8.0, "LoadGame \"Skip Tutorial\"".to_owned())]);
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
