//! Reading the game's memory safely: every pointer walk checks the page first, so a stale pointer
//! can't fault the game.

/// The bytes at `p` can be read (committed, readable, not a guard page).
#[cfg(windows)]
pub fn is_readable(p: *const u8, n: usize) -> bool {
    use windows::Win32::System::Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery};
    if p.is_null() || (p as usize) < 0x10000 {
        return false;
    }
    let mut at = p as usize;
    let end = at + n.max(1);
    while at < end {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        if unsafe { VirtualQuery(Some(at as *const _), &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) } == 0 {
            return false;
        }
        if mbi.State != MEM_COMMIT || mbi.Protect.0 & (PAGE_GUARD.0 | PAGE_NOACCESS.0) != 0 || mbi.Protect.0 == 0 {
            return false;
        }
        at = mbi.BaseAddress as usize + mbi.RegionSize;
    }
    true
}

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
