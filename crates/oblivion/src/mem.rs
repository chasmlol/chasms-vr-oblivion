//! Reading the game's memory safely: every pointer walk checks the page first, so a stale pointer
//! can't fault the game.
//!
//! Asking Windows about a page (`VirtualQuery`) costs tens of microseconds in the game's process
//! (its address space is full of small regions), more than the game's whole frame when the
//! adapter reads thousands of values. So the readable regions found are remembered ([`Regions`])
//! and asked about again only after [`REMEMBER`]: long enough for the reads of a frame, short
//! enough that memory the game frees (it frees between frames, on its own thread) is soon
//! forgotten.

/// How long a region found readable is taken to stay so.
#[cfg(windows)]
const REMEMBER: std::time::Duration = std::time::Duration::from_millis(50);
/// At most this many regions are remembered at once (the reads of a frame touch a few heaps).
#[cfg(windows)]
const MAX_REGIONS: usize = 64;

/// The readable regions found lately: (start, end), and since when.
#[cfg(windows)]
struct Regions {
    since: Option<std::time::Instant>,
    spans: Vec<(usize, usize)>,
}

#[cfg(windows)]
static REGIONS: std::sync::Mutex<Regions> = std::sync::Mutex::new(Regions { since: None, spans: Vec::new() });

/// Forgets every region (each frame, and in tests).
#[cfg(windows)]
pub fn forget() {
    if let Ok(mut r) = REGIONS.lock() {
        r.spans.clear();
        r.since = None;
    }
}

/// The bytes at `p` can be read (committed, readable, not a guard page).
#[cfg(windows)]
pub fn is_readable(p: *const u8, n: usize) -> bool {
    use windows::Win32::System::Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery};
    if p.is_null() || (p as usize) < 0x10000 {
        return false;
    }
    let mut at = p as usize;
    // (A pointer near the top of the address space wraps: never readable.)
    let Some(end) = at.checked_add(n.max(1)) else { return false };
    let mut regions = REGIONS.lock().unwrap_or_else(|e| e.into_inner());
    if regions.since.is_some_and(|t| t.elapsed() > REMEMBER) {
        regions.spans.clear();
        regions.since = None;
    }
    if regions.spans.iter().any(|&(s, e)| s <= at && end <= e) {
        return true;
    }
    while at < end {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        if unsafe { VirtualQuery(Some(at as *const _), &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) } == 0 {
            return false;
        }
        if mbi.State != MEM_COMMIT || mbi.Protect.0 & (PAGE_GUARD.0 | PAGE_NOACCESS.0) != 0 || mbi.Protect.0 == 0 {
            return false;
        }
        let base = mbi.BaseAddress as usize;
        let Some(next) = base.checked_add(mbi.RegionSize) else { return false };
        if regions.spans.len() >= MAX_REGIONS {
            regions.spans.remove(0);
        }
        regions.spans.push((base, next));
        regions.since.get_or_insert_with(std::time::Instant::now);
        at = next;
    }
    true
}

#[cfg(not(windows))]
pub fn forget() {}

#[cfg(not(windows))]
pub fn is_readable(_p: *const u8, _n: usize) -> bool {
    false
}

/// Reads a `T` at `p` if it's readable.
pub fn read<T: Copy>(p: *const u8) -> Option<T> {
    is_readable(p, std::mem::size_of::<T>()).then(|| unsafe { (p as *const T).read_unaligned() })
}

/// Reads a `T` at an absolute address in the game.
pub fn at<T: Copy>(addr: usize) -> Option<T> {
    read(addr as *const u8)
}

/// Follows a pointer stored at `addr` (0 when null or unreadable).
pub fn deref(addr: usize) -> usize {
    at::<u32>(addr).unwrap_or(0) as usize
}

/// Writes `bytes` over the game's code at `addr`, if what's there is `expect` (the build the
/// patch was made for). True if it's patched.
#[cfg(windows)]
pub fn patch_code(addr: usize, expect: &[u8], bytes: &[u8]) -> bool {
    use windows::Win32::System::Diagnostics::Debug::FlushInstructionCache;
    use windows::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect};
    use windows::Win32::System::Threading::GetCurrentProcess;
    if expect.len() != bytes.len() || !is_readable(addr as *const u8, expect.len()) {
        return false;
    }
    let p = addr as *mut u8;
    if unsafe { std::slice::from_raw_parts(p, expect.len()) } != expect {
        return false;
    }
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if unsafe { VirtualProtect(p as *const _, bytes.len(), PAGE_EXECUTE_READWRITE, &mut old) }.is_err() {
        return false;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len()) };
    unsafe {
        let _ = VirtualProtect(p as *const _, bytes.len(), old, &mut old);
        let _ = FlushInstructionCache(GetCurrentProcess(), Some(p as *const _), bytes.len());
    }
    true
}

#[cfg(not(windows))]
pub fn patch_code(_addr: usize, _expect: &[u8], _bytes: &[u8]) -> bool {
    false
}
