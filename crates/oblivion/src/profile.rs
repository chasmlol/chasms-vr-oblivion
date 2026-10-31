//! Testing, `RUSTVR_OBLIVION_PROFILE=1`: what the game's thread does in a long frame. A thread of
//! the adapter's stops it about once a millisecond, notes where it is (its instruction and the game's
//! code on its stack: the calls it's in) and lets it go on; a frame (render to render) longer than
//! 30 ms is then summed up in the log: which module it was in (the game's, D3D9On12, the driver,
//! Windows) and the game's addresses most often on its stack.
//!
//! While the game's thread is stopped nothing is locked or allocated (it may hold the lock or the
//! heap); the stack is read from its pointer to the end of its committed region.

#[cfg(all(windows, target_arch = "x86"))]
pub use imp::frame_start;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn frame_start(_thread: u32) {}

pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTVR_OBLIVION_PROFILE").is_ok_and(|v| v == "1"))
}

#[cfg(all(windows, target_arch = "x86"))]
mod imp {
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// The game's code.
    const CODE: std::ops::Range<u32> = 0x0040_1000..0x00A2_8000;
    /// A frame longer than this is looked into.
    const LONG: Duration = Duration::from_millis(30);
    /// The samples kept (a few seconds of them).
    const KEEP: usize = 8192;
    const CALLS: usize = 16;
    /// At most this many long frames are summed up in a run.
    const REPORTS: u32 = 400;

    #[derive(Clone, Copy)]
    struct Sample {
        at: Instant,
        eip: u32,
        calls: [u32; CALLS],
        n: u8,
    }

    static SAMPLES: Mutex<Option<VecDeque<Sample>>> = Mutex::new(None);
    static LONG_FRAMES: Mutex<Vec<(Instant, Instant)>> = Mutex::new(Vec::new());
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);

    /// The game's thread (`thread`) starts a render: a frame since the last one longer than
    /// [`LONG`] is to be summed up.
    pub fn frame_start(thread: u32) {
        if !super::enabled() {
            return;
        }
        static STARTED: std::sync::Once = std::sync::Once::new();
        STARTED.call_once(|| {
            tracing::info!("profile: testing: the game's thread is sampled, frames over 30 ms summed up (RUSTVR_OBLIVION_PROFILE)");
            std::thread::spawn(move || run(thread));
        });
        let now = Instant::now();
        let mut last = LAST.lock().unwrap();
        if let Some(l) = *last
            && now - l > LONG
        {
            LONG_FRAMES.lock().unwrap().push((l, now));
        }
        *last = Some(now);
    }

    fn run(thread: u32) {
        use windows::Win32::System::Diagnostics::Debug::{CONTEXT, CONTEXT_FLAGS, GetThreadContext};
        use windows::Win32::System::Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, VirtualQuery};
        use windows::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_GET_CONTEXT, THREAD_QUERY_INFORMATION, THREAD_SUSPEND_RESUME};
        let Ok(h) = (unsafe { OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION, false, thread) }) else {
            tracing::warn!("profile: the game's thread can't be opened");
            return;
        };
        *SAMPLES.lock().unwrap() = Some(VecDeque::with_capacity(KEEP));
        let mut modules: HashMap<usize, String> = HashMap::new();
        let mut reports = 0;
        loop {
            std::thread::sleep(Duration::from_millis(1));
            let mut s = Sample { at: Instant::now(), eip: 0, calls: [0; CALLS], n: 0 };
            if unsafe { SuspendThread(h) } == u32::MAX {
                continue;
            }
            // (CONTEXT_i386 | CONTEXT_CONTROL: the instruction and stack pointers.)
            let mut ctx = CONTEXT { ContextFlags: CONTEXT_FLAGS(0x0001_0001), ..Default::default() };
            if unsafe { GetThreadContext(h, &mut ctx) }.is_ok() {
                s.eip = ctx.Eip;
                let mut mbi = MEMORY_BASIC_INFORMATION::default();
                let esp = ctx.Esp as usize;
                if unsafe { VirtualQuery(Some(esp as *const _), &mut mbi, size_of::<MEMORY_BASIC_INFORMATION>()) } != 0 && mbi.State == MEM_COMMIT {
                    let end = (mbi.BaseAddress as usize + mbi.RegionSize).min(esp + 8192);
                    let mut p = esp;
                    while p + 4 <= end && (s.n as usize) < CALLS {
                        let v = unsafe { (p as *const u32).read_volatile() };
                        if CODE.contains(&v) {
                            s.calls[s.n as usize] = v;
                            s.n += 1;
                        }
                        p += 4;
                    }
                }
            }
            unsafe { ResumeThread(h) };
            // (Only from here on: locks and allocation.)
            let long: Vec<(Instant, Instant)> = std::mem::take(&mut *LONG_FRAMES.lock().unwrap());
            let mut samples = SAMPLES.lock().unwrap();
            let Some(samples) = samples.as_mut() else { continue };
            if samples.len() == KEEP {
                samples.pop_front();
            }
            samples.push_back(s);
            for (from, to) in long {
                if reports < REPORTS {
                    reports += 1;
                    report(samples, from, to, &mut modules);
                }
            }
        }
    }

    /// The module an address is in (its file's name).
    fn module(addr: u32, cache: &mut HashMap<usize, String>) -> String {
        use windows::Win32::Foundation::HMODULE;
        use windows::Win32::System::LibraryLoader::{GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, GetModuleFileNameW, GetModuleHandleExW};
        let mut m = HMODULE::default();
        let flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        if unsafe { GetModuleHandleExW(flags, windows::core::PCWSTR(addr as usize as *const u16), &mut m) }.is_err() {
            return "?".into();
        }
        cache
            .entry(m.0 as usize)
            .or_insert_with(|| {
                let mut buf = [0u16; 260];
                let n = unsafe { GetModuleFileNameW(Some(m), &mut buf) } as usize;
                let path = String::from_utf16_lossy(&buf[..n]);
                path.rsplit('\\').next().unwrap_or("?").to_owned()
            })
            .clone()
    }

    fn report(samples: &VecDeque<Sample>, from: Instant, to: Instant, cache: &mut HashMap<usize, String>) {
        let these: Vec<&Sample> = samples.iter().filter(|s| s.at >= from && s.at < to).collect();
        if these.is_empty() {
            return;
        }
        let mut modules: HashMap<String, u32> = HashMap::new();
        let mut addresses: HashMap<u32, u32> = HashMap::new();
        for s in &these {
            *modules.entry(module(s.eip, cache)).or_default() += 1;
            let mut seen: Vec<u32> = s.calls[..s.n as usize].to_vec();
            if CODE.contains(&s.eip) {
                seen.push(s.eip);
            }
            seen.sort_unstable();
            seen.dedup();
            for a in seen {
                *addresses.entry(a).or_default() += 1;
            }
        }
        let mut modules: Vec<(String, u32)> = modules.into_iter().collect();
        modules.sort_by(|a, b| b.1.cmp(&a.1));
        let mut addresses: Vec<(u32, u32)> = addresses.into_iter().collect();
        addresses.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let modules: Vec<String> = modules.iter().map(|(m, n)| format!("{m} {n}")).collect();
        let addresses: Vec<String> = addresses.iter().take(24).map(|(a, n)| format!("{a:#x}:{n}")).collect();
        tracing::info!(
            "profile: a {:.0} ms frame, {} samples; in {}; the game's code on the stack (address:samples) {}",
            (to - from).as_secs_f64() * 1000.0,
            these.len(),
            modules.join(", "),
            addresses.join(" ")
        );
    }
}
